//! Application Domain objects for scripts

use crate::avm2::activation::Activation;
use crate::avm2::domain::Domain;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, TObject};
use crate::avm2::Error;
use crate::utils::HasPrefixField;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::{lock::Lock, Collect, Gc, GcWeak, Mutation};

/// A class instance allocator that allocates AppDomain objects.
pub fn application_domain_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let domain = activation.domain();
    let base = ScriptObjectData::new(class);

    Ok(DomainObject(Gc::new(
        activation.gc(),
        DomainObjectData {
            base,
            domain: Lock::new(domain),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct DomainObject<'gc>(pub Gc<'gc, DomainObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct DomainObjectWeak<'gc>(pub GcWeak<'gc, DomainObjectData<'gc>>);

impl fmt::Debug for DomainObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DomainObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Clone, Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct DomainObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// The domain this object holds
    domain: Lock<Domain<'gc>>,
}

impl<'gc> DomainObject<'gc> {
    /// Create a new object for a given domain.
    ///
    /// This function will call instance initializers. You do not need to do so
    /// yourself.
    pub fn from_domain(activation: &mut Activation<'_, 'gc>, domain: Domain<'gc>) -> Self {
        let class = activation.avm2().classes().application_domain;
        let base = ScriptObjectData::new(class);
        DomainObject(Gc::new(
            activation.gc(),
            DomainObjectData {
                base,
                domain: Lock::new(domain),
            },
        ))
    }

    pub fn domain(self) -> Domain<'gc> {
        self.0.domain.get()
    }

    pub fn init_domain(self, mc: &Mutation<'gc>, domain: Domain<'gc>) {
        unlock!(Gc::write(mc, self.0), DomainObjectData, domain).set(domain);
    }
}

impl<'gc> TObject<'gc> for DomainObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}
