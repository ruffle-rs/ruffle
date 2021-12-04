//! Boxed namespaces

use crate::avm2::activation::Activation;
use crate::avm2::names::Namespace;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates namespace objects.
pub fn namespace_allocator<'gc>(
    class: ClassObject<'gc>,
    proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let base = ScriptObjectData::base_new(Some(proto), Some(class));

    Ok(NamespaceObject(GcCell::allocate(
        activation.context.gc_context,
        NamespaceObjectData {
            base,
            namespace: Namespace::public(),
        },
    ))
    .into())
}

/// An Object which represents a boxed namespace name.
#[derive(Collect, Debug, Clone, Copy)]
#[collect(no_drop)]
pub struct NamespaceObject<'gc>(GcCell<'gc, NamespaceObjectData<'gc>>);

#[derive(Collect, Debug, Clone)]
#[collect(no_drop)]
pub struct NamespaceObjectData<'gc> {
    /// All normal script data.
    base: ScriptObjectData<'gc>,

    /// The namespace name this object is associated with.
    namespace: Namespace<'gc>,
}

impl<'gc> NamespaceObject<'gc> {
    /// Box a namespace into an object.
    pub fn from_namespace(
        activation: &mut Activation<'_, 'gc, '_>,
        namespace: Namespace<'gc>,
    ) -> Result<Object<'gc>, Error> {
        let class = activation.avm2().classes().namespace;
        let proto = activation.avm2().prototypes().namespace;
        let base = ScriptObjectData::base_new(Some(proto), Some(class));

        let mut this: Object<'gc> = NamespaceObject(GcCell::allocate(
            activation.context.gc_context,
            NamespaceObjectData { base, namespace },
        ))
        .into();
        this.install_instance_slots(activation);

        class.call_native_init(Some(this), &[], activation)?;

        Ok(this)
    }
}

impl<'gc> TObject<'gc> for NamespaceObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn to_string(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(self.0.read().namespace.as_uri().into())
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(self.0.read().namespace.as_uri().into())
    }

    fn as_namespace(&self) -> Option<Ref<Namespace<'gc>>> {
        Some(Ref::map(self.0.read(), |s| &s.namespace))
    }

    fn derive(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::NamespaceObject(*self);
        let base = ScriptObjectData::base_new(Some(this), None);

        Ok(NamespaceObject(GcCell::allocate(
            activation.context.gc_context,
            NamespaceObjectData {
                base,
                namespace: Namespace::public(),
            },
        ))
        .into())
    }
}
