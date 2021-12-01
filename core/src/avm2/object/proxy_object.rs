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

    fn get_property_local(
        self,
        receiver: Object<'gc>,
        multiname: &Multiname<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        // NOTE: This is incorrect behavior.
        // `QName` should instead store the whole multiname's namespace set,
        // so that it can be used to index other objects using the same
        // namespace set.
        if let Some(local_name) = multiname.local_name() {
            for namespace in multiname.namespace_set() {
                if namespace.is_any() || namespace.is_public() || namespace.is_namespace() {
                    let qname =
                        QNameObject::from_qname(activation, QName::new(*namespace, local_name))?;

                    return receiver.call_property_trait_only(
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

    fn set_property_local(
        self,
        receiver: Object<'gc>,
        multiname: &Multiname<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        println!("in proxy set_property_local");
        // NOTE: This is incorrect behavior.
        // `QName` should instead store the whole multiname's namespace set,
        // so that it can be used to index other objects using the same
        // namespace set.
        if let Some(local_name) = multiname.local_name() {
            for namespace in multiname.namespace_set() {
                if namespace.is_any() || namespace.is_public() || namespace.is_namespace() {
                    let qname =
                        QNameObject::from_qname(activation, QName::new(*namespace, local_name))?;

                    receiver.call_property_trait_only(
                        &QName::new(Namespace::Namespace(NS_FLASH_PROXY.into()), "setProperty")
                            .into(),
                        &[qname.into(), value],
                        activation,
                    )?;

                    return Ok(());
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
            let _ = local_name?;
            Ok(())
        } else {
            Err(format!("Cannot set undefined property {:?}", multiname.local_name()).into())
        }
    }

    fn call_property_local(
        self,
        _receiver: Object<'gc>,
        multiname: &Multiname<'gc>,
        arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        // NOTE: This is incorrect behavior.
        // `QName` should instead store the whole multiname's namespace set,
        // so that it can be used to index other objects using the same
        // namespace set.
        if let Some(local_name) = multiname.local_name() {
            for namespace in multiname.namespace_set() {
                if namespace.is_any() || namespace.is_public() || namespace.is_namespace() {
                    let qname =
                        QNameObject::from_qname(activation, QName::new(*namespace, local_name))?;

                    let mut args = vec![qname.into()];
                    args.extend_from_slice(arguments);

                    return self.call_property_trait_only(
                        &QName::new(Namespace::Namespace(NS_FLASH_PROXY.into()), "callProperty")
                            .into(),
                        &args[..],
                        activation,
                    );
                }
            }
        }

        Err(format!(
            "Attempted to call undefined property {:?}",
            multiname.local_name()
        )
        .into())
    }

    fn delete_property_local(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        multiname: &Multiname<'gc>,
    ) -> Result<bool, Error> {
        // NOTE: This is incorrect behavior.
        // `QName` should instead store the whole multiname's namespace set,
        // so that it can be used to index other objects using the same
        // namespace set.
        if let Some(local_name) = multiname.local_name() {
            for namespace in multiname.namespace_set() {
                if namespace.is_any() || namespace.is_public() || namespace.is_namespace() {
                    let qname =
                        QNameObject::from_qname(activation, QName::new(*namespace, local_name))?;

                    return Ok(self
                        .call_property_trait_only(
                            &QName::new(
                                Namespace::Namespace(NS_FLASH_PROXY.into()),
                                "deleteProperty",
                            )
                            .into(),
                            &[qname.into()],
                            activation,
                        )?
                        .coerce_to_boolean());
                }
            }
        }

        // Unknown properties on a dynamic class delete successfully.
        return Ok(!self
            .instance_of_class_definition()
            .map(|c| c.read().is_sealed())
            .unwrap_or(false));
    }

    fn has_property_via_in(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: &Multiname<'gc>,
    ) -> Result<bool, Error> {
        Ok(self
            .call_property_trait_only(
                &QName::new(Namespace::Namespace(NS_FLASH_PROXY.into()), "hasProperty").into(),
                // TODO: handle unwrap?
                // this should probably pass the multiname as-is? See above
                &[name.local_name().unwrap().into()],
                activation,
            )?
            .coerce_to_boolean())
    }

    fn get_next_enumerant(
        self,
        last_index: u32,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Option<u32>, Error> {
        Ok(Some(
            self.call_property_trait_only(
                &QName::new(Namespace::Namespace(NS_FLASH_PROXY.into()), "nextNameIndex").into(),
                &[last_index.into()],
                activation,
            )?
            .coerce_to_u32(activation)?,
        ))
    }

    fn get_enumerant_name(
        self,
        index: u32,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        self.call_property_trait_only(
            &QName::new(Namespace::Namespace(NS_FLASH_PROXY.into()), "nextName").into(),
            &[index.into()],
            activation,
        )
    }

    fn get_enumerant_value(
        self,
        index: u32,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        self.call_property_trait_only(
            &QName::new(Namespace::Namespace(NS_FLASH_PROXY.into()), "nextValue").into(),
            &[index.into()],
            activation,
        )
    }
}
