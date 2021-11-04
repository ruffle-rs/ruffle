//! Object representation for `Proxy`.

use crate::avm2::activation::Activation;
use crate::avm2::globals::NS_FLASH_PROXY;
use crate::avm2::names::{Multiname, Namespace, QName};
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, QNameObject, TObject};
use crate::avm2::value::Value;
use crate::avm2::{AvmString, Error};
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates Proxy objects.
pub fn proxy_allocator<'gc>(
    class: ClassObject<'gc>,
    proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let base = ScriptObjectData::base_new(Some(proto), Some(class));

    Ok(ProxyObject(GcCell::allocate(
        activation.context.gc_context,
        ProxyObjectData { base },
    ))
    .into())
}

#[derive(Clone, Collect, Debug, Copy)]
#[collect(no_drop)]
pub struct ProxyObject<'gc>(GcCell<'gc, ProxyObjectData<'gc>>);

#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct ProxyObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,
}

impl<'gc> TObject<'gc> for ProxyObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(Object::from(*self).into())
    }

    fn derive(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        let base = ScriptObjectData::base_new(Some((*self).into()), None);

        Ok(ProxyObject(GcCell::allocate(
            activation.context.gc_context,
            ProxyObjectData { base },
        ))
        .into())
    }

    fn get_property_undef(
        self,
        receiver: Object<'gc>,
        multiname: &Multiname<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        for namespace in multiname.namespace_set() {
            if let Some(local_name) = multiname.local_name() {
                if namespace.is_any() || namespace.is_public() || namespace.is_namespace() {
                    let qname = QNameObject::from_qname(
                        activation,
                        QName::new(namespace.clone(), local_name),
                    )?;

                    return receiver.call_property(
                        &QName::new(Namespace::Namespace(NS_FLASH_PROXY.into()), "getProperty")
                            .into(),
                        &[qname.into()],
                        activation,
                    );
                }
            }
        }

        if !self
            .instance_of_class_definition()
            .map(|c| c.read().is_sealed())
            .unwrap_or(false)
        {
            return Ok(Value::Undefined);
        }

        return Err(format!("Cannot get undefined property {:?}", multiname.local_name()).into());
    }

    fn set_property_undef(
        &mut self,
        receiver: Object<'gc>,
        multiname: &Multiname<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Option<QName<'gc>>, Error> {
        for namespace in multiname.namespace_set() {
            if let Some(local_name) = multiname.local_name() {
                if namespace.is_any() || namespace.is_public() || namespace.is_namespace() {
                    let qname = QNameObject::from_qname(
                        activation,
                        QName::new(namespace.clone(), local_name),
                    )?;

                    receiver.call_property(
                        &QName::new(Namespace::Namespace(NS_FLASH_PROXY.into()), "setProperty")
                            .into(),
                        &[qname.into(), value],
                        activation,
                    )?;

                    return Ok(None);
                }
            }
        }

        if !self
            .instance_of_class_definition()
            .map(|c| c.read().is_sealed())
            .unwrap_or(false)
        {
            let local_name: Result<AvmString<'gc>, Error> = multiname
                .local_name()
                .ok_or_else(|| "Cannot set undefined property using any name".into());
            Ok(Some(QName::dynamic_name(local_name?)))
        } else {
            Err(format!("Cannot set undefined property {:?}", multiname.local_name()).into())
        }
    }
}
