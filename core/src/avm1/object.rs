//! Object trait to expose objects to AVM

use crate::avm1::function::{ExecutionName, ExecutionReason, FunctionObject};
use crate::avm1::globals::bevel_filter::BevelFilter;
use crate::avm1::globals::blur_filter::BlurFilter;
use crate::avm1::globals::color_matrix_filter::ColorMatrixFilter;
use crate::avm1::globals::color_transform::ColorTransformObject;
use crate::avm1::globals::convolution_filter::ConvolutionFilter;
use crate::avm1::globals::date::Date;
use crate::avm1::globals::displacement_map_filter::DisplacementMapFilter;
use crate::avm1::globals::drop_shadow_filter::DropShadowFilter;
use crate::avm1::globals::file_reference::FileReferenceObject;
use crate::avm1::globals::glow_filter::GlowFilter;
use crate::avm1::globals::gradient_filter::GradientFilter;
use crate::avm1::globals::local_connection::LocalConnection;
use crate::avm1::globals::netconnection::NetConnection;
use crate::avm1::globals::shared_object::SharedObject;
use crate::avm1::globals::sound::Sound;
use crate::avm1::globals::style_sheet::StyleSheetObject;
use crate::avm1::globals::transform::TransformObject;
use crate::avm1::globals::xml::Xml;
use crate::avm1::globals::xml_socket::XmlSocket;
use crate::avm1::object::super_object::SuperObject;
use crate::avm1::xml::XmlNode;
use crate::avm1::{Activation, Error, Value};
use crate::bitmap::bitmap_data::BitmapData;
use crate::display_object::{
    Avm1Button, DisplayObject, EditText, MovieClip, TDisplayObject as _, Video,
};
use crate::html::TextFormat;
use crate::streams::NetStream;
use crate::string::AvmString;
use gc_arena::{Collect, Gc, Mutation};
use ruffle_macros::istr;
use std::cell::{Cell, RefCell};
use std::marker::PhantomData;

mod script_object;
pub mod stage_object;
pub mod super_object;

pub use script_object::{Object, ObjectHandle, ObjectWeak};

#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub enum NativeObject<'gc> {
    None,

    /// A `super` object, used to call superclass methods.
    ///
    /// `super` objects should never have any properties (including `__proto__`); instead,
    /// relevant operations are forwarded to the `SuperObject`'s target.
    Super(SuperObject<'gc>),
    /// A boxed boolean.
    Bool(bool),
    /// A boxed number.
    Number(BoxedF64<'gc>),
    /// A boxed string.
    String(AvmString<'gc>),
    /// Marker indicating that this object should behave like an array.
    ///
    /// This stores no data; all array properties are stored on the object's main `PropertyMap`.
    /// TODO(moulins): that doesn't seem entirely correct; in Flash Player, it is possible in
    /// certain circumstances (e.g. in a subclass constructor, before calling `super()`) to
    /// 'desynchronize' the "property view" and the "array view" (used by, e.g., `toString()`).
    Array(()),
    Function(Gc<'gc, FunctionObject<'gc>>),

    MovieClip(MovieClip<'gc>),
    Button(Avm1Button<'gc>),
    EditText(EditText<'gc>),
    Video(Video<'gc>),

    Date(Gc<'gc, Cell<Date>>),
    BlurFilter(BlurFilter<'gc>),
    BevelFilter(BevelFilter<'gc>),
    GlowFilter(GlowFilter<'gc>),
    DropShadowFilter(DropShadowFilter<'gc>),
    ColorMatrixFilter(ColorMatrixFilter<'gc>),
    DisplacementMapFilter(DisplacementMapFilter<'gc>),
    ConvolutionFilter(ConvolutionFilter<'gc>),
    GradientBevelFilter(GradientFilter<'gc>),
    GradientGlowFilter(GradientFilter<'gc>),
    ColorTransform(Gc<'gc, ColorTransformObject>),
    Transform(TransformObject<'gc>),
    TextFormat(Gc<'gc, RefCell<TextFormat>>),
    NetStream(NetStream<'gc>),
    BitmapData(BitmapData<'gc>),
    Xml(Xml<'gc>),
    XmlNode(XmlNode<'gc>),
    SharedObject(Gc<'gc, RefCell<SharedObject>>),
    XmlSocket(XmlSocket<'gc>),
    FileReference(FileReferenceObject<'gc>),
    NetConnection(NetConnection<'gc>),
    LocalConnection(LocalConnection<'gc>),
    Sound(Sound<'gc>),
    StyleSheet(StyleSheetObject<'gc>),
}

const _: () = assert!(size_of::<NativeObject<'_>>() <= size_of::<[usize; 2]>());

/// Small wrapper struct to keep boxed f64s word-sized on every architecture.
#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct BoxedF64<'gc> {
    #[cfg(target_pointer_width = "64")]
    value: f64,
    #[cfg(not(target_pointer_width = "64"))]
    value: Gc<'gc, f64>,
    _marker: PhantomData<Gc<'gc, ()>>,
}

impl<'gc> BoxedF64<'gc> {
    #[inline]
    pub fn new(#[allow(unused)] mc: &Mutation<'gc>, value: f64) -> Self {
        Self {
            #[cfg(target_pointer_width = "64")]
            value,
            #[cfg(not(target_pointer_width = "64"))]
            value: Gc::new(mc, value),
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn value(self) -> f64 {
        #[cfg(target_pointer_width = "64")]
        return self.value;
        #[cfg(not(target_pointer_width = "64"))]
        return *self.value;
    }
}

impl<'gc> NativeObject<'gc> {
    pub fn as_display_object(self) -> Option<DisplayObject<'gc>> {
        match self {
            Self::MovieClip(dobj) => Some(DisplayObject::MovieClip(dobj)),
            Self::Button(dobj) => Some(DisplayObject::Avm1Button(dobj)),
            Self::EditText(dobj) => Some(DisplayObject::EditText(dobj)),
            Self::Video(dobj) => Some(DisplayObject::Video(dobj)),
            _ => None,
        }
    }
}

impl<'gc> Object<'gc> {
    /// Retrieve a named property from the object, or its prototype.
    pub fn get(
        self,
        name: impl Into<AvmString<'gc>>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let (this, proto) = if let Some(super_object) = self.as_super_object() {
            (super_object.this(), super_object.proto(activation))
        } else {
            (self, Value::Object(self))
        };
        match search_prototype(proto, name.into(), activation, this, true)? {
            Some((value, _depth)) => Ok(value),
            None => Ok(Value::Undefined),
        }
    }

    /// Retrieve a non-virtual property from the object, or its prototype.
    pub fn get_stored(
        self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let mut depth = 0;
        let mut proto = Value::Object(self);

        while let Value::Object(p) = proto {
            if depth == 255 {
                return Err(Error::PrototypeRecursionLimit);
            }

            if let Some(value) = p.get_local_stored(name, activation) {
                return Ok(value);
            }

            proto = p.proto(activation);
            depth += 1;
        }

        Ok(Value::Undefined)
    }

    /// Set a named property on this object, or its prototype.
    pub fn set(
        self,
        name: impl Into<AvmString<'gc>>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        let name = name.into();
        if name.is_empty() {
            return Ok(());
        }

        let mut value = value;
        let (this, mut proto) = if let Some(super_object) = self.as_super_object() {
            (super_object.this(), super_object.proto(activation))
        } else {
            (self, Value::Object(self))
        };
        let watcher_result = self.call_watcher(activation, name, &mut value, this);

        if !self.has_own_property(activation, name) {
            // Before actually inserting a new property, we need to crawl the
            // prototype chain for virtual setters.
            while let Value::Object(this_proto) = proto {
                if this_proto.has_own_virtual(activation, name) {
                    if let Some(setter) = this_proto.setter(name, activation) {
                        if let Some(exec) = setter.as_function() {
                            let _ = exec.exec(
                                ExecutionName::Static("[Setter]"),
                                activation,
                                this.into(),
                                1,
                                &[value],
                                ExecutionReason::Special,
                                setter,
                            );
                        }
                    }
                    return Ok(());
                }

                proto = this_proto.proto(activation);
            }
        }

        let result = self.set_local(name, value, activation, this);
        watcher_result.and(result)
    }

    /// Call a method on the object.
    ///
    /// It is highly recommended to use this convenience method to perform
    /// method calls. It is morally equivalent to an AVM1 `ActionCallMethod`
    /// opcode. It will take care of retrieving the method, calculating its
    /// base prototype for `super` calls, and providing it with the correct
    /// `this` parameter.
    pub fn call_method(
        self,
        name: AvmString<'gc>,
        args: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc>,
        reason: ExecutionReason,
    ) -> Result<Value<'gc>, Error<'gc>> {
        match self.native_no_super() {
            NativeObject::Super(zuper) => return zuper.call_method(name, args, activation, reason),
            native => {
                if native
                    .as_display_object()
                    .is_some_and(|dobj| dobj.avm1_removed())
                {
                    return Ok(Value::Undefined);
                }
            }
        }

        let (method, depth) =
            match search_prototype(Value::Object(self), name, activation, self, true)? {
                Some((Value::Object(method), depth)) => (method, depth),
                _ => return Ok(Value::Undefined),
            };

        // If the method was found on the object itself, change `depth` as-if
        // the method was found on the object's prototype.
        let depth = depth.max(1);

        match method.as_function() {
            Some(exec) => exec.exec(
                ExecutionName::Dynamic(name),
                activation,
                self.into(),
                depth,
                args,
                reason,
                method,
            ),
            None => method.call(name, activation, self.into(), args),
        }
    }

    /// Determine if this object is an instance of a class.
    ///
    /// The class is provided in the form of its constructor function and the
    /// explicit prototype of that constructor function. It is assumed that
    /// they are already linked.
    ///
    /// Because ActionScript 2.0 added interfaces, this function cannot simply
    /// check the prototype chain and call it a day. Each interface represents
    /// a new, parallel prototype chain which also needs to be checked. You
    /// can't implement interfaces within interfaces (fortunately), but if you
    /// somehow could this would support that, too.
    pub fn is_instance_of(
        self,
        activation: &mut Activation<'_, 'gc>,
        constructor: Object<'gc>,
        prototype: Object<'gc>,
    ) -> Result<bool, Error<'gc>> {
        let mut proto_stack = vec![];
        if let Value::Object(p) = self.proto(activation) {
            proto_stack.push(p);
        }

        while let Some(this_proto) = proto_stack.pop() {
            if Object::ptr_eq(this_proto, prototype) {
                return Ok(true);
            }

            if let Value::Object(p) = this_proto.proto(activation) {
                proto_stack.push(p);
            }

            if activation.swf_version() >= 7 {
                for interface in this_proto.interfaces() {
                    if Object::ptr_eq(interface, constructor) {
                        return Ok(true);
                    }

                    if let Value::Object(o) = interface.get(istr!("prototype"), activation)? {
                        proto_stack.push(o);
                    }
                }
            }
        }

        Ok(false)
    }

    /// Get the underlying XML node for this object, if it exists.
    pub fn as_xml_node(self) -> Option<XmlNode<'gc>> {
        match self.native() {
            NativeObject::Xml(xml) => Some(xml.root()),
            NativeObject::XmlNode(xml_node) => Some(xml_node),
            _ => None,
        }
    }

    /// Check if this object is in the prototype chain of the specified test object.
    pub fn is_prototype_of(self, activation: &mut Activation<'_, 'gc>, other: Object<'gc>) -> bool {
        let mut proto = other.proto(activation);

        while let Value::Object(proto_ob) = proto {
            if std::ptr::eq(self.as_ptr(), proto_ob.as_ptr()) {
                return true;
            }

            proto = proto_ob.proto(activation);
        }

        false
    }

    pub fn ptr_eq(a: Object<'gc>, b: Object<'gc>) -> bool {
        std::ptr::eq(a.as_ptr(), b.as_ptr())
    }
}

pub enum ObjectPtr {}

/// Perform a prototype lookup of a given object.
///
/// This function returns both the `Value` and the prototype depth from which
/// it was grabbed from. If the property did not resolve, then it returns
/// `Ok(None)`.
///
/// The prototype depth can and should be used to populate the `depth`
/// parameter necessary to make `super` work.
pub fn search_prototype<'gc>(
    mut proto: Value<'gc>,
    name: AvmString<'gc>,
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    call_resolve_fn: bool,
) -> Result<Option<(Value<'gc>, u8)>, Error<'gc>> {
    let mut depth = 0;
    let orig_proto = proto;

    while let Value::Object(p) = proto {
        if depth == 255 {
            return Err(Error::PrototypeRecursionLimit);
        }

        if let Some(getter) = p.getter(name, activation) {
            if let Some(exec) = getter.as_function() {
                let result = exec.exec(
                    ExecutionName::Static("[Getter]"),
                    activation,
                    this.into(),
                    1,
                    &[],
                    ExecutionReason::Special,
                    getter,
                );
                let value = match result {
                    Ok(v) => v,
                    Err(Error::ThrownValue(e)) => return Err(Error::ThrownValue(e)),
                    Err(_) => Value::Undefined,
                };
                return Ok(Some((value, depth)));
            }
        }

        if let Some(value) = p.get_local_stored(name, activation) {
            return Ok(Some((value, depth)));
        }

        proto = p.proto(activation);
        depth += 1;
    }

    if call_resolve_fn {
        if let Some(resolve) = find_resolve_method(orig_proto, activation)? {
            let result =
                resolve.call(istr!("__resolve"), activation, this.into(), &[name.into()])?;
            return Ok(Some((result, 0)));
        }
    }

    Ok(None)
}

/// Finds the appropriate `__resolve` method for an object, searching its hierarchy too.
pub fn find_resolve_method<'gc>(
    mut proto: Value<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Option<Object<'gc>>, Error<'gc>> {
    let mut depth = 0;

    while let Value::Object(p) = proto {
        if depth == 255 {
            return Err(Error::PrototypeRecursionLimit);
        }

        let resolve = p.get_local_stored(istr!("__resolve"), activation);
        // FP completely skips over primitives (but not over non-function objects).
        if let Some(Value::Object(value)) = resolve {
            return Ok(Some(value));
        }

        proto = p.proto(activation);
        depth += 1;
    }

    Ok(None)
}
