//! Application Domain objects for scripts

use crate::avm2::activation::Activation;
use crate::avm2::domain::Domain;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use core::fmt;
use gc_arena::{Collect, GcCell, GcWeakCell, Mutation};
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates AppDomain objects.
pub fn application_domain_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let domain = activation.domain();
    let base = ScriptObjectData::new(class);

    Ok(DomainObject(GcCell::new(
        activation.context.gc_context,
        DomainObjectData { base, domain },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct DomainObject<'gc>(pub GcCell<'gc, DomainObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct DomainObjectWeak<'gc>(pub GcWeakCell<'gc, DomainObjectData<'gc>>);

impl fmt::Debug for DomainObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DomainObject")
            .field("ptr", &self.0.as_ptr())
            .finish()
    }
}

#[derive(Clone, Collect)]
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
        activation: &mut Activation<'_, 'gc>,
        domain: Domain<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let class = activation.avm2().classes().application_domain;
        let base = ScriptObjectData::new(class);
        let this: Object<'gc> = DomainObject(GcCell::new(
            activation.context.gc_context,
            DomainObjectData { base, domain },
        ))
        .into();
        this.install_instance_slots(activation.context.gc_context);

        // Note - we do *not* call the normal constructor, since that
        // creates a new domain using the system domain as a parent.
        class
            .superclass_object()
            .unwrap()
            .call_native_init(this.into(), &[], activation)?;
        Ok(this)
    }
}

impl<'gc> TObject<'gc> for DomainObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: &Mutation<'gc>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn as_application_domain(&self) -> Option<Domain<'gc>> {
        Some(self.0.read().domain)
    }

    fn init_application_domain(&self, mc: &Mutation<'gc>, domain: Domain<'gc>) {
        self.0.write(mc).domain = domain;
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        let this: Object<'gc> = Object::DomainObject(*self);

        Ok(this.into())
    }
}
