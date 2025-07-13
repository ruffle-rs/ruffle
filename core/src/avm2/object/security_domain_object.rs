use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::TObject;
use crate::utils::HasPrefixField;
use core::fmt;
use gc_arena::{Collect, Gc, GcWeak};

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct SecurityDomainObject<'gc>(pub Gc<'gc, SecurityDomainObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct SecurityDomainObjectWeak<'gc>(pub GcWeak<'gc, SecurityDomainObjectData<'gc>>);

impl fmt::Debug for SecurityDomainObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecurityDomainObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct SecurityDomainObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,
}

impl<'gc> TObject<'gc> for SecurityDomainObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}

impl<'gc> SecurityDomainObject<'gc> {
    pub fn new(activation: &mut Activation<'_, 'gc>) -> Self {
        let class = activation.avm2().classes().securitydomain;
        let base = ScriptObjectData::new(class);
        SecurityDomainObject(Gc::new(activation.gc(), SecurityDomainObjectData { base }))
    }
}
