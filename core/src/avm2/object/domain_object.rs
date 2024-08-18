//! Application Domain objects for scripts

use crate::avm2::activation::Activation;
use crate::avm2::domain::Domain;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
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
        activation.context.gc_context,
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

#[derive(Clone, Collect)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct DomainObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// The domain this object holds
    domain: Lock<Domain<'gc>>,
}

const _: () = assert!(std::mem::offset_of!(DomainObjectData, base) == 0);
const _: () =
    assert!(std::mem::align_of::<DomainObjectData>() == std::mem::align_of::<ScriptObjectData>());

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
        let this: Object<'gc> = DomainObject(Gc::new(
            activation.context.gc_context,
            DomainObjectData {
                base,
                domain: Lock::new(domain),
            },
        ))
        .into();

        // Note - we do *not* call the normal constructor, since that
        // creates a new domain using the system domain as a parent.
        class
            .superclass_object()
            .unwrap()
            .call_super_init(this.into(), &[], activation)?;
        Ok(this)
    }
}

impl<'gc> TObject<'gc> for DomainObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        // SAFETY: Object data is repr(C), and a compile-time assert ensures
        // that the ScriptObjectData stays at offset 0 of the struct- so the
        // layouts are compatible

        unsafe { Gc::cast(self.0) }
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn as_application_domain(&self) -> Option<Domain<'gc>> {
        Some(self.0.domain.get())
    }

    fn init_application_domain(&self, mc: &Mutation<'gc>, domain: Domain<'gc>) {
        unlock!(Gc::write(mc, self.0), DomainObjectData, domain).set(domain);
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        let this: Object<'gc> = Object::DomainObject(*self);

        Ok(this.into())
    }
}
