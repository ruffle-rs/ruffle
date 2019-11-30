//! Special object that implements `super`

use crate::avm1::function::Executable;
use crate::avm1::property::Attribute;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, ObjectPtr, ScriptObject, TObject, Value};
use crate::context::UpdateContext;
use crate::display_object::DisplayObject;
use enumset::EnumSet;
use gc_arena::{Collect, GcCell, MutationContext};
use std::collections::HashSet;

/// Implementation of the `super` object in AS2.
///
/// A `SuperObject` references all data from another object, but with one layer
/// of prototyping removed. It's as if the given object had been constructed
/// with it's parent class.
#[collect(no_drop)]
#[derive(Copy, Clone, Collect, Debug)]
pub struct SuperObject<'gc>(GcCell<'gc, SuperObjectData<'gc>>);

#[collect(no_drop)]
#[derive(Clone, Collect, Debug)]
pub struct SuperObjectData<'gc> {
    child: Object<'gc>,
    proto: Option<Object<'gc>>,
    constr: Option<Object<'gc>>,
    this: Option<Object<'gc>>,
}

impl<'gc> SuperObject<'gc> {
    pub fn from_child_object(
        child: Object<'gc>,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<Self, Error> {
        let child_proto = child.proto();
        let parent_proto = child_proto.and_then(|pr| pr.proto());
        let parent_constr = if let Some(child_proto) = child_proto {
            Some(
                child_proto
                    .get("constructor", avm, context)?
                    .resolve(avm, context)?
                    .as_object()?,
            )
        } else {
            None
        };

        Ok(Self(GcCell::allocate(
            context.gc_context,
            SuperObjectData {
                child,
                proto: parent_proto,
                constr: parent_constr,
                this: None,
            },
        )))
    }

    /// Set `this` to a particular value.
    ///
    /// This is intended to be called with a self-reference, so that future
    /// invocations of `super()` can get a `this` value one level up the chain.
    pub fn bind_this(&mut self, context: MutationContext<'gc, '_>, this: Object<'gc>) {
        self.0.write(context).this = Some(this);
    }
}

impl<'gc> TObject<'gc> for SuperObject<'gc> {
    fn get_local(
        &self,
        name: &str,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<ReturnValue<'gc>, Error> {
        self.0.read().child.get_local(name, avm, context, this)
    }

    fn set(
        &self,
        name: &str,
        value: Value<'gc>,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        //TODO: What happens if you set `super.__proto__`?
        self.0.read().child.set(name, value, avm, context)
    }

    fn call(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
        args: &[Value<'gc>],
    ) -> Result<ReturnValue<'gc>, Error> {
        if let Some(constr) = self.0.read().constr {
            constr.call(avm, context, self.0.read().this.unwrap_or(this), args)
        } else {
            Ok(Value::Undefined.into())
        }
    }

    #[allow(clippy::new_ret_no_self)]
    fn new(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
        args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
        if let Some(proto) = self.proto() {
            proto.new(avm, context, this, args)
        } else {
            // TODO: What happens when you `new super` but there's no
            // super? Is this code even reachable?!
            self.0.read().child.new(avm, context, this, args)
        }
    }

    fn delete(&self, _gc_context: MutationContext<'gc, '_>, _name: &str) -> bool {
        //We cannot delete through `super` because we don't have a GC context
        false
    }

    fn proto(&self) -> Option<Object<'gc>> {
        self.0.read().proto
    }

    fn define_value(
        &self,
        _gc_context: MutationContext<'gc, '_>,
        _name: &str,
        _value: Value<'gc>,
        _attributes: EnumSet<Attribute>,
    ) {
        //We cannot define through `super` because we don't have a GC context
    }

    fn set_attributes(
        &mut self,
        _gc_context: MutationContext<'gc, '_>,
        _name: Option<&str>,
        _set_attributes: EnumSet<Attribute>,
        _clear_attributes: EnumSet<Attribute>,
    ) {
        //We cannot set attributes through `super` because we don't have a GC context
    }

    fn add_property(
        &self,
        _gc_context: MutationContext<'gc, '_>,
        _name: &str,
        _get: Executable<'gc>,
        _set: Option<Executable<'gc>>,
        _attributes: EnumSet<Attribute>,
    ) {
        //We cannot add virtual properties through `super` because we don't have a GC context
    }

    fn has_property(&self, name: &str) -> bool {
        self.0.read().child.has_property(name)
    }

    fn has_own_property(&self, name: &str) -> bool {
        self.0.read().child.has_own_property(name)
    }

    fn is_property_enumerable(&self, name: &str) -> bool {
        self.0.read().child.is_property_enumerable(name)
    }

    fn is_property_overwritable(&self, name: &str) -> bool {
        self.0.read().child.is_property_overwritable(name)
    }

    fn get_keys(&self) -> HashSet<String> {
        //TODO: What's in `for var x in super`?
        self.0.read().child.get_keys()
    }

    fn as_string(&self) -> String {
        self.0.read().child.as_string()
    }

    fn type_of(&self) -> &'static str {
        self.0.read().child.type_of()
    }

    fn length(&self) -> usize {
        0
    }

    fn set_length(&self, _gc_context: MutationContext<'gc, '_>, _new_length: usize) {}

    fn array(&self) -> Vec<Value<'gc>> {
        vec![]
    }

    fn array_element(&self, _index: usize) -> Value<'gc> {
        Value::Undefined
    }

    fn set_array_element(
        &self,
        _index: usize,
        _value: Value<'gc>,
        _gc_context: MutationContext<'gc, '_>,
    ) -> usize {
        0
    }

    fn delete_array_element(&self, _index: usize, _gc_context: MutationContext<'gc, '_>) {}

    fn interfaces(&self) -> Vec<Object<'gc>> {
        if let Some(proto) = self.proto() {
            proto.interfaces()
        } else {
            vec![]
        }
    }

    fn set_interfaces(
        &mut self,
        _gc_context: MutationContext<'gc, '_>,
        _iface_list: Vec<Object<'gc>>,
    ) {
        //So we can't actually set interfaces through a `super`, because this
        //function doesn't get a `gc_context` to write with...
    }

    fn as_script_object(&self) -> Option<ScriptObject<'gc>> {
        None
    }

    fn as_super_object(&self) -> Option<SuperObject<'gc>> {
        Some(*self)
    }

    fn as_display_object(&self) -> Option<DisplayObject<'gc>> {
        self.0.read().child.as_display_object()
    }

    fn as_executable(&self) -> Option<Executable<'gc>> {
        self.0.read().child.as_executable()
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }
}
