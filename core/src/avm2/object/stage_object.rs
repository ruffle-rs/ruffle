//! AVM2 object impl for the display hierarchy.

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::function::Executable;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::scope::Scope;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::display_object::DisplayObject;
use crate::string::AvmString;
use gc_arena::{Collect, GcCell, MutationContext};

/// A class instance allocator that allocates Stage objects.
pub fn stage_allocator<'gc>(
    class: Object<'gc>,
    proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let base = ScriptObjectData::base_new(Some(proto), Some(class));

    Ok(StageObject(GcCell::allocate(
        activation.context.gc_context,
        StageObjectData {
            base,
            display_object: None,
        },
    ))
    .into())
}

#[derive(Clone, Collect, Debug, Copy)]
#[collect(no_drop)]
pub struct StageObject<'gc>(GcCell<'gc, StageObjectData<'gc>>);

#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct StageObjectData<'gc> {
    /// The base data common to all AVM2 objects.
    base: ScriptObjectData<'gc>,

    /// The associated display object, if one exists.
    display_object: Option<DisplayObject<'gc>>,
}

impl<'gc> StageObject<'gc> {
    /// Allocate the AVM2 side of a display object intended to be of a given
    /// class's type.
    ///
    /// This function makes no attempt to construct the returned object. You
    /// are responsible for calling the native initializer of the given
    /// class at a later time. Typically, a display object that can contain
    /// movie-constructed children must first allocate itself (using this
    /// function), construct it's children, and then finally initialize itself.
    /// Display objects that do not need to use this flow should use
    /// `for_display_object_childless`.
    pub fn for_display_object(
        activation: &mut Activation<'_, 'gc, '_>,
        display_object: DisplayObject<'gc>,
        class: Object<'gc>,
    ) -> Result<Self, Error> {
        let proto = class
            .get_property(
                class,
                &QName::new(Namespace::public(), "prototype"),
                activation,
            )?
            .coerce_to_object(activation)?;

        let mut instance = Self(GcCell::allocate(
            activation.context.gc_context,
            StageObjectData {
                base: ScriptObjectData::base_new(Some(proto), Some(class)),
                display_object: Some(display_object),
            },
        ));
        instance.install_instance_traits(activation, class)?;

        Ok(instance)
    }

    /// Allocate and construct the AVM2 side of a display object intended to be
    /// of a given class's type.
    ///
    /// This function is intended for display objects that do not have children
    /// and thus do not need to be allocated and initialized in separate phases.
    pub fn for_display_object_childless(
        activation: &mut Activation<'_, 'gc, '_>,
        display_object: DisplayObject<'gc>,
        class: Object<'gc>,
    ) -> Result<Self, Error> {
        let this = Self::for_display_object(activation, display_object, class)?;

        class.call_native_init(Some(this.into()), &[], activation, Some(class))?;

        Ok(this)
    }

    /// Create a `graphics` object for a given display object.
    pub fn graphics(
        activation: &mut Activation<'_, 'gc, '_>,
        display_object: DisplayObject<'gc>,
    ) -> Result<Self, Error> {
        let class = activation.avm2().classes().graphics;
        let proto = activation.avm2().prototypes().graphics;
        let mut this = Self(GcCell::allocate(
            activation.context.gc_context,
            StageObjectData {
                base: ScriptObjectData::base_new(Some(proto), Some(class)),
                display_object: Some(display_object),
            },
        ));
        this.install_instance_traits(activation, class)?;

        class.call_native_init(Some(this.into()), &[], activation, Some(class))?;

        Ok(this)
    }
}

impl<'gc> TObject<'gc> for StageObject<'gc> {
    fn get_property_local(
        self,
        reciever: Object<'gc>,
        name: &QName<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        let read = self.0.read();
        let rv = read.base.get_property_local(reciever, name, activation)?;

        drop(read);

        rv.resolve(activation)
    }

    fn set_property_local(
        self,
        reciever: Object<'gc>,
        name: &QName<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let mut write = self.0.write(activation.context.gc_context);
        let rv = write
            .base
            .set_property_local(reciever, name, value, activation)?;

        drop(write);

        rv.resolve(activation)?;

        Ok(())
    }

    fn init_property_local(
        self,
        reciever: Object<'gc>,
        name: &QName<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let mut write = self.0.write(activation.context.gc_context);
        let rv = write
            .base
            .init_property_local(reciever, name, value, activation)?;

        drop(write);

        rv.resolve(activation)?;

        Ok(())
    }

    fn is_property_overwritable(
        self,
        gc_context: MutationContext<'gc, '_>,
        name: &QName<'gc>,
    ) -> bool {
        self.0.write(gc_context).base.is_property_overwritable(name)
    }

    fn is_property_final(self, name: &QName<'gc>) -> bool {
        self.0.read().base.is_property_final(name)
    }

    fn delete_property(
        &self,
        gc_context: MutationContext<'gc, '_>,
        multiname: &QName<'gc>,
    ) -> bool {
        self.0.write(gc_context).base.delete_property(multiname)
    }

    fn get_slot(self, id: u32) -> Result<Value<'gc>, Error> {
        self.0.read().base.get_slot(id)
    }

    fn set_slot(
        self,
        id: u32,
        value: Value<'gc>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<(), Error> {
        self.0.write(mc).base.set_slot(id, value, mc)
    }

    fn init_slot(
        self,
        id: u32,
        value: Value<'gc>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<(), Error> {
        self.0.write(mc).base.init_slot(id, value, mc)
    }

    fn get_method(self, id: u32) -> Option<Object<'gc>> {
        self.0.read().base.get_method(id)
    }

    fn get_scope(self) -> Option<GcCell<'gc, Scope<'gc>>> {
        self.0.read().base.get_scope()
    }

    fn resolve_any(self, local_name: AvmString<'gc>) -> Result<Option<Namespace<'gc>>, Error> {
        self.0.read().base.resolve_any(local_name)
    }

    fn resolve_any_trait(
        self,
        local_name: AvmString<'gc>,
    ) -> Result<Option<Namespace<'gc>>, Error> {
        self.0.read().base.resolve_any_trait(local_name)
    }

    fn has_own_property(self, name: &QName<'gc>) -> Result<bool, Error> {
        self.0.read().base.has_own_property(name)
    }

    fn has_trait(self, name: &QName<'gc>) -> Result<bool, Error> {
        self.0.read().base.has_trait(name)
    }

    fn has_own_virtual_getter(self, name: &QName<'gc>) -> bool {
        self.0.read().base.has_own_virtual_getter(name)
    }

    fn has_own_virtual_setter(self, name: &QName<'gc>) -> bool {
        self.0.read().base.has_own_virtual_setter(name)
    }

    fn proto(&self) -> Option<Object<'gc>> {
        self.0.read().base.proto()
    }

    fn set_proto(self, mc: MutationContext<'gc, '_>, proto: Object<'gc>) {
        self.0.write(mc).base.set_proto(proto)
    }

    fn get_enumerant_name(&self, index: u32) -> Option<QName<'gc>> {
        self.0.read().base.get_enumerant_name(index)
    }

    fn property_is_enumerable(&self, name: &QName<'gc>) -> bool {
        self.0.read().base.property_is_enumerable(name)
    }

    fn set_local_property_is_enumerable(
        &self,
        mc: MutationContext<'gc, '_>,
        name: &QName<'gc>,
        is_enumerable: bool,
    ) -> Result<(), Error> {
        self.0
            .write(mc)
            .base
            .set_local_property_is_enumerable(name, is_enumerable)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn as_executable(&self) -> Option<Executable<'gc>> {
        None
    }

    fn as_class(&self) -> Option<GcCell<'gc, Class<'gc>>> {
        self.0.read().base.as_class()
    }

    fn as_class_object(&self) -> Option<Object<'gc>> {
        self.0.read().base.as_class_object()
    }

    fn set_class_object(self, mc: MutationContext<'gc, '_>, class_object: Object<'gc>) {
        self.0.write(mc).base.set_class_object(class_object);
    }

    fn as_display_object(&self) -> Option<DisplayObject<'gc>> {
        self.0.read().display_object
    }

    fn init_display_object(&self, mc: MutationContext<'gc, '_>, obj: DisplayObject<'gc>) {
        self.0.write(mc).display_object = Some(obj);
    }

    fn call(
        self,
        _reciever: Option<Object<'gc>>,
        _arguments: &[Value<'gc>],
        _activation: &mut Activation<'_, 'gc, '_>,
        _base_proto: Option<Object<'gc>>,
    ) -> Result<Value<'gc>, Error> {
        Err("Not a callable function!".into())
    }

    fn derive(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::StageObject(*self);
        let base = ScriptObjectData::base_new(Some(this), None);

        Ok(StageObject(GcCell::allocate(
            activation.context.gc_context,
            StageObjectData {
                base,
                display_object: None,
            },
        ))
        .into())
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn install_method(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName<'gc>,
        disp_id: u32,
        function: Object<'gc>,
        is_final: bool,
    ) {
        self.0
            .write(mc)
            .base
            .install_method(name, disp_id, function, is_final)
    }

    fn install_getter(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName<'gc>,
        disp_id: u32,
        function: Object<'gc>,
        is_final: bool,
    ) -> Result<(), Error> {
        self.0
            .write(mc)
            .base
            .install_getter(name, disp_id, function, is_final)
    }

    fn install_setter(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName<'gc>,
        disp_id: u32,
        function: Object<'gc>,
        is_final: bool,
    ) -> Result<(), Error> {
        self.0
            .write(mc)
            .base
            .install_setter(name, disp_id, function, is_final)
    }

    fn install_dynamic_property(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName<'gc>,
        value: Value<'gc>,
    ) -> Result<(), Error> {
        self.0.write(mc).base.install_dynamic_property(name, value)
    }

    fn install_slot(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName<'gc>,
        id: u32,
        value: Value<'gc>,
        is_final: bool,
    ) {
        self.0
            .write(mc)
            .base
            .install_slot(name, id, value, is_final)
    }

    fn install_const(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName<'gc>,
        id: u32,
        value: Value<'gc>,
        is_final: bool,
    ) {
        self.0
            .write(mc)
            .base
            .install_const(name, id, value, is_final)
    }
}
