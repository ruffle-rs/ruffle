//! AVM2 objects.

use crate::avm2::function::{
    Avm2ClassEntry, Avm2Function, Avm2MethodEntry, Executable, FunctionObject,
};
use crate::avm2::names::{Multiname, Namespace, QName};
use crate::avm2::property::Attribute;
use crate::avm2::return_value::ReturnValue;
use crate::avm2::scope::Scope;
use crate::avm2::script_object::ScriptObject;
use crate::avm2::value::Value;
use crate::avm2::{Avm2, Error};
use crate::context::UpdateContext;
use enumset::EnumSet;
use gc_arena::{Collect, GcCell, MutationContext};
use ruffle_macros::enum_trait_object;
use std::fmt::Debug;
use std::rc::Rc;
use swf::avm2::types::{AbcFile, Trait as AbcTrait, TraitKind as AbcTraitKind};

/// Represents an object that can be directly interacted with by the AVM2
/// runtime.
#[enum_trait_object(
    #[derive(Clone, Collect, Debug, Copy)]
    #[collect(no_drop)]
    pub enum Object<'gc> {
        ScriptObject(ScriptObject<'gc>),
        FunctionObject(FunctionObject<'gc>)
    }
)]
pub trait TObject<'gc>: 'gc + Collect + Debug + Into<Object<'gc>> + Clone + Copy {
    /// Retrieve a property by it's QName.
    fn get_property(
        self,
        name: &QName,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<ReturnValue<'gc>, Error>;

    /// Set a property by it's QName.
    fn set_property(
        self,
        name: &QName,
        value: Value<'gc>,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error>;

    /// Retrieve a slot by it's index.
    fn get_slot(self, id: u32) -> Result<Value<'gc>, Error>;

    /// Set a slot by it's index.
    fn set_slot(
        self,
        id: u32,
        value: Value<'gc>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<(), Error>;

    /// Resolve a multiname into a single QName, if any of the namespaces
    /// match.
    fn resolve_multiname(self, multiname: &Multiname) -> Option<QName> {
        for ns in multiname.namespace_set() {
            let qname = QName::qualified(ns, multiname.local_name());

            if self.has_property(&qname) {
                return Some(qname);
            }
        }

        None
    }

    /// Indicates whether or not a property exists on an object.
    fn has_property(self, _name: &QName) -> bool;

    /// Indicates whether or not a property exists on an object and is not part
    /// of the prototype chain.
    fn has_own_property(self, _name: &QName) -> bool {
        false
    }

    /// Indicates whether or not a property is overwritable.
    fn is_property_overwritable(self, _name: &QName) -> bool {
        false
    }

    /// Delete a named property from the object.
    ///
    /// Returns false if the property cannot be deleted.
    fn delete(&self, gc_context: MutationContext<'gc, '_>, multiname: &QName) -> bool {
        false
    }

    /// Retrieve the `__proto__` of a given object.
    ///
    /// The proto is another object used to resolve methods across a class of
    /// multiple objects. It should also be accessible as `__proto__` from
    /// `get`.
    fn proto(&self) -> Option<Object<'gc>>;

    /// Define a value on an object.
    ///
    /// Unlike setting a value, this function is intended to replace any
    /// existing virtual or built-in properties already installed on a given
    /// object. As such, this should not run any setters; the resulting name
    /// slot should either be completely replaced with the value or completely
    /// untouched.
    ///
    /// It is not guaranteed that all objects accept value definitions,
    /// especially if a property name conflicts with a built-in property, such
    /// as `__proto__`.
    fn define_value(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &QName,
        value: Value<'gc>,
        attributes: EnumSet<Attribute>,
    ) {
    }

    /// Install a method (or any other non-slot value) on an object.
    fn install_method(&mut self, mc: MutationContext<'gc, '_>, name: QName, function: Object<'gc>);

    /// Install a getter method on an object property.
    fn install_getter(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName,
        function: Executable<'gc>,
    ) -> Result<(), Error>;

    /// Install a setter method on an object property.
    fn install_setter(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName,
        function: Executable<'gc>,
    ) -> Result<(), Error>;

    /// Install a dynamic or built-in value property on an object.
    fn install_dynamic_property(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName,
        value: Value<'gc>,
    ) -> Result<(), Error>;

    /// Install a trait from an ABC file on an object.
    fn install_trait(
        &mut self,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        abc: Rc<AbcFile>,
        trait_entry: &AbcTrait,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
        fn_proto: Object<'gc>,
    ) -> Result<(), Error> {
        let trait_name = QName::from_abc_multiname(&abc, trait_entry.name.clone())?;
        avm_debug!(
            "Installing trait {:?} of kind {:?}",
            trait_name,
            trait_entry.kind
        );

        match &trait_entry.kind {
            AbcTraitKind::Method { method, .. } => {
                let method = Avm2MethodEntry::from_method_index(abc, method.clone()).unwrap();
                let function =
                    FunctionObject::from_abc_method(context.gc_context, method, scope, fn_proto);
                self.install_method(context.gc_context, trait_name, function);
            }
            AbcTraitKind::Getter { method, .. } => {
                let method = Avm2MethodEntry::from_method_index(abc, method.clone()).unwrap();
                let exec = Avm2Function::from_method(method, scope).into();
                self.install_getter(context.gc_context, trait_name, exec)?;
            }
            AbcTraitKind::Setter { method, .. } => {
                let method = Avm2MethodEntry::from_method_index(abc, method.clone()).unwrap();
                let exec = Avm2Function::from_method(method, scope).into();
                self.install_setter(context.gc_context, trait_name, exec)?;
            }
            AbcTraitKind::Class { class, .. } => {
                let type_entry = Avm2ClassEntry::from_class_index(abc, class.clone()).unwrap();
                let super_name = QName::from_abc_multiname(
                    &type_entry.abc(),
                    type_entry.instance().super_name.clone(),
                )?;
                let super_class = self
                    .get_property(&super_name, avm, context)?
                    .resolve(avm, context)?
                    .as_object()?;
                let super_proto = super_class
                    .get_property(
                        &QName::new(Namespace::public_namespace(), "prototype"),
                        avm,
                        context,
                    )?
                    .resolve(avm, context)?
                    .as_object()?;
                let mut class_proto = super_proto.construct(avm, context, &[])?;

                for trait_entry in type_entry.instance().traits.iter() {
                    class_proto.install_trait(
                        avm,
                        context,
                        type_entry.abc(),
                        trait_entry,
                        scope,
                        fn_proto,
                    )?;
                }

                let class = FunctionObject::from_abc_class(
                    avm,
                    context,
                    type_entry.clone(),
                    class_proto,
                    scope,
                    fn_proto,
                )?;
                let class_name = QName::from_abc_multiname(
                    &type_entry.abc(),
                    type_entry.instance().name.clone(),
                )?;
                self.install_method(context.gc_context, class_name, class);
            }
            _ => return Err("".into()),
        }

        Ok(())
    }

    /// Call the object.
    fn call(
        self,
        _reciever: Object<'gc>,
        _arguments: &[Value<'gc>],
        _avm: &mut Avm2<'gc>,
        _context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<ReturnValue<'gc>, Error> {
        Err("Object is not callable".into())
    }

    /// Construct a host object of some kind and return it's cell.
    ///
    /// As the first step in object construction, the `construct` method is
    /// called on the prototype to create a new object. The prototype may
    /// construct any object implementation it wants, however, it's expected
    /// to produce a like `TObject` implementor with itself as the new object's
    /// proto.
    ///
    /// After construction, the constructor function is `call`ed with the new
    /// object as `this` to initialize the object.
    ///
    /// The arguments passed to the constructor are provided here; however, all
    /// object construction should happen in `call`, not `new`. `new` exists
    /// purely so that host objects can be constructed by the VM.
    fn construct(
        &self,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error>;

    /// Get a raw pointer value for this object.
    fn as_ptr(&self) -> *const ObjectPtr;
}

pub enum ObjectPtr {}

impl<'gc> Object<'gc> {
    pub fn ptr_eq(a: Object<'gc>, b: Object<'gc>) -> bool {
        a.as_ptr() == b.as_ptr()
    }
}
