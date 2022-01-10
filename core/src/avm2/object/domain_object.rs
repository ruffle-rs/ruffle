//! Application Domain objects for scripts

use crate::avm2::activation::Activation;
use crate::avm2::domain::Domain;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates AppDomain objects.
pub fn appdomain_allocator<'gc>(
    class: ClassObject<'gc>,
    proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let domain = activation.domain();
    let base = ScriptObjectData::base_new(Some(proto), Some(class));

    Ok(DomainObject(GcCell::allocate(
        activation.context.gc_context,
        DomainObjectData { base, domain },
    ))
    .into())
}

#[derive(Clone, Collect, Debug, Copy)]
#[collect(no_drop)]
pub struct DomainObject<'gc>(GcCell<'gc, DomainObjectData<'gc>>);

#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct DomainObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// The domain this object holds
    domain: Domain<'gc>,
}

impl<'gc> DomainObject<'gc> {
    /// Create a new object for a given domain.
    ///
    /// This function will call instance initializers. You do not need to do so
    /// yourself.
    pub fn from_domain(
        activation: &mut Activation<'_, 'gc, '_>,
        domain: Domain<'gc>,
    ) -> Result<Object<'gc>, Error> {
        let class = activation.avm2().classes().application_domain;
        let proto = activation.avm2().prototypes().application_domain;
        let base = ScriptObjectData::base_new(Some(proto), Some(class));
        let mut this: Object<'gc> = DomainObject(GcCell::allocate(
            activation.context.gc_context,
            DomainObjectData { base, domain },
        ))
        .into();
        this.install_instance_slots(activation);

        class.call_init(Some(this), &[], activation)?;

        Ok(this)
    }
}

impl<'gc> TObject<'gc> for DomainObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn as_application_domain(&self) -> Option<Domain<'gc>> {
        Some(self.0.read().domain)
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        let this: Object<'gc> = Object::DomainObject(*self);

        Ok(this.into())
    }
}
