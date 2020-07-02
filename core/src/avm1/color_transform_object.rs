use crate::avm1::error::Error;
use crate::avm1::function::Executable;
use crate::avm1::property::Attribute;
use crate::avm1::{Object, ObjectPtr, ScriptObject, TObject, Value};
use crate::context::UpdateContext;
use enumset::EnumSet;
use gc_arena::{Collect, GcCell, MutationContext};

use crate::avm1::activation::Activation;
use std::borrow::Cow;
use std::fmt;

/// A ColorTransform
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct ColorTransformObject<'gc>(GcCell<'gc, ColorTransformData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct ColorTransformData<'gc> {
    /// The underlying script object.
    base: ScriptObject<'gc>,

    red_multiplier: f64,
    green_multiplier: f64,
    blue_multiplier: f64,
    alpha_multiplier: f64,
    red_offset: f64,
    green_offset: f64,
    blue_offset: f64,
    alpha_offset: f64,
}

macro_rules! add_object_accessors {
    ($([$set_ident: ident, $get_ident: ident, $var: ident],)*) => {
        $(
            pub fn $set_ident(&self, gc_context: MutationContext<'gc, '_>, v: f64) {
                self.0.write(gc_context).$var = v;
            }

            pub fn $get_ident(&self) -> f64 {
                self.0.read()
                    .$var
            }
        )*
    }
}

impl fmt::Debug for ColorTransformObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("ColorTransform")
            .field("redMultiplier", &this.red_multiplier)
            .field("greenMultiplier", &this.green_multiplier)
            .field("blueMultiplier", &this.blue_multiplier)
            .field("alphaMultiplier", &this.alpha_multiplier)
            .field("redOffset", &this.red_offset)
            .field("greenOffset", &this.green_offset)
            .field("blueOffset", &this.blue_offset)
            .field("alphaOffset", &this.alpha_offset)
            .finish()
    }
}

impl<'gc> ColorTransformObject<'gc> {
    pub fn empty_color_transform_object(
        gc_context: MutationContext<'gc, '_>,
        proto: Option<Object<'gc>>,
    ) -> Self {
        ColorTransformObject(GcCell::allocate(
            gc_context,
            ColorTransformData {
                base: ScriptObject::object(gc_context, proto),
                red_multiplier: 0.0,
                green_multiplier: 0.0,
                blue_multiplier: 0.0,
                alpha_multiplier: 0.0,
                red_offset: 0.0,
                green_offset: 0.0,
                blue_offset: 0.0,
                alpha_offset: 0.0,
            },
        ))
    }

    add_object_accessors!(
        [set_red_multiplier, get_red_multiplier, red_multiplier],
        [set_green_multiplier, get_green_multiplier, green_multiplier],
        [set_blue_multiplier, get_blue_multiplier, blue_multiplier],
        [set_alpha_multiplier, get_alpha_multiplier, alpha_multiplier],
        [set_red_offset, get_red_offset, red_offset],
        [set_green_offset, get_green_offset, green_offset],
        [set_blue_offset, get_blue_offset, blue_offset],
        [set_alpha_offset, get_alpha_offset, alpha_offset],
    );

    fn base(self) -> ScriptObject<'gc> {
        self.0.read().base
    }
}

impl<'gc> TObject<'gc> for ColorTransformObject<'gc> {
    fn get_local(
        &self,
        name: &str,
        activation: &mut Activation<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        self.base().get_local(name, activation, context, this)
    }

    fn set(
        &self,
        name: &str,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error<'gc>> {
        self.base().internal_set(
            name,
            value,
            activation,
            context,
            (*self).into(),
            Some(activation.avm.prototypes.color_transform),
        )
    }

    fn call(
        &self,
        name: &str,
        activation: &mut Activation<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
        base_proto: Option<Object<'gc>>,
        args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error<'gc>> {
        self.base()
            .call(name, activation, context, this, base_proto, args)
    }

    fn call_setter(
        &self,
        name: &str,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Option<Executable<'gc>> {
        self.base().call_setter(name, value, activation, context)
    }

    #[allow(clippy::new_ret_no_self)]
    fn new(
        &self,
        activation: &mut Activation<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        _this: Object<'gc>,
        _args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error<'gc>> {
        Ok(ColorTransformObject::empty_color_transform_object(
            context.gc_context,
            Some(activation.avm.prototypes.color_transform),
        )
        .into())
    }

    fn delete(
        &self,
        activation: &mut Activation<'_, 'gc>,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
    ) -> bool {
        self.base().delete(activation, gc_context, name)
    }

    fn proto(&self) -> Option<Object<'gc>> {
        self.base().proto()
    }

    fn set_proto(&self, gc_context: MutationContext<'gc, '_>, prototype: Option<Object<'gc>>) {
        self.base().set_proto(gc_context, prototype);
    }

    fn define_value(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        value: Value<'gc>,
        attributes: EnumSet<Attribute>,
    ) {
        self.base()
            .define_value(gc_context, name, value, attributes)
    }

    fn set_attributes(
        &mut self,
        gc_context: MutationContext<'gc, '_>,
        name: Option<&str>,
        set_attributes: EnumSet<Attribute>,
        clear_attributes: EnumSet<Attribute>,
    ) {
        self.base()
            .set_attributes(gc_context, name, set_attributes, clear_attributes)
    }

    fn add_property(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        get: Executable<'gc>,
        set: Option<Executable<'gc>>,
        attributes: EnumSet<Attribute>,
    ) {
        self.base()
            .add_property(gc_context, name, get, set, attributes)
    }

    fn add_property_with_case(
        &self,
        activation: &mut Activation<'_, 'gc>,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        get: Executable<'gc>,
        set: Option<Executable<'gc>>,
        attributes: EnumSet<Attribute>,
    ) {
        self.base()
            .add_property_with_case(activation, gc_context, name, get, set, attributes)
    }

    fn set_watcher(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: Cow<str>,
        callback: Executable<'gc>,
        user_data: Value<'gc>,
    ) {
        self.base()
            .set_watcher(gc_context, name, callback, user_data);
    }

    fn remove_watcher(&self, gc_context: MutationContext<'gc, '_>, name: Cow<str>) -> bool {
        self.base().remove_watcher(gc_context, name)
    }

    fn has_property(
        &self,
        activation: &mut Activation<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        name: &str,
    ) -> bool {
        self.base().has_property(activation, context, name)
    }

    fn has_own_property(
        &self,
        activation: &mut Activation<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        name: &str,
    ) -> bool {
        self.base().has_own_property(activation, context, name)
    }

    fn has_own_virtual(
        &self,
        activation: &mut Activation<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        name: &str,
    ) -> bool {
        self.base().has_own_virtual(activation, context, name)
    }

    fn is_property_enumerable(&self, activation: &mut Activation<'_, 'gc>, name: &str) -> bool {
        self.base().is_property_enumerable(activation, name)
    }

    fn get_keys(&self, activation: &mut Activation<'_, 'gc>) -> Vec<String> {
        self.base().get_keys(activation)
    }

    fn as_string(&self) -> Cow<str> {
        Cow::Owned(self.base().as_string().into_owned())
    }

    fn type_of(&self) -> &'static str {
        self.base().type_of()
    }

    fn interfaces(&self) -> Vec<Object<'gc>> {
        self.base().interfaces()
    }

    fn set_interfaces(
        &mut self,
        gc_context: MutationContext<'gc, '_>,
        iface_list: Vec<Object<'gc>>,
    ) {
        self.base().set_interfaces(gc_context, iface_list)
    }

    fn as_script_object(&self) -> Option<ScriptObject<'gc>> {
        Some(self.base())
    }

    fn as_color_transform_object(&self) -> Option<ColorTransformObject<'gc>> {
        Some(*self)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn length(&self) -> usize {
        self.base().length()
    }

    fn array(&self) -> Vec<Value<'gc>> {
        self.base().array()
    }

    fn set_length(&self, gc_context: MutationContext<'gc, '_>, length: usize) {
        self.base().set_length(gc_context, length)
    }

    fn array_element(&self, index: usize) -> Value<'gc> {
        self.base().array_element(index)
    }

    fn set_array_element(
        &self,
        index: usize,
        value: Value<'gc>,
        gc_context: MutationContext<'gc, '_>,
    ) -> usize {
        self.base().set_array_element(index, value, gc_context)
    }

    fn delete_array_element(&self, index: usize, gc_context: MutationContext<'gc, '_>) {
        self.base().delete_array_element(index, gc_context)
    }
}
