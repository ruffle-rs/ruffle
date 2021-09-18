//! Boxed namespaces

use crate::avm2::activation::Activation;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::scope::Scope;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::AvmString;
use crate::{
    impl_avm2_custom_object, impl_avm2_custom_object_instance, impl_avm2_custom_object_properties,
};
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::Ref;

/// A class instance allocator that allocates namespace objects.
pub fn namespace_allocator<'gc>(
    class: Object<'gc>,
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
        this.install_instance_traits(activation, class)?;

        class.call_native_init(Some(this), &[], activation, Some(class))?;

        Ok(this)
    }
}

impl<'gc> TObject<'gc> for NamespaceObject<'gc> {
    impl_avm2_custom_object!(base);
    impl_avm2_custom_object_properties!(base);
    impl_avm2_custom_object_instance!(base);

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
