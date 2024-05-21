//! Boxed namespaces

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Namespace;
use crate::string::AvmString;
use core::fmt;
use gc_arena::{Collect, GcCell, GcWeakCell, Mutation};
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates namespace objects.
pub fn namespace_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    let namespace = activation.context.avm2.public_namespace_base_version;
    Ok(NamespaceObject(GcCell::new(
        activation.context.gc_context,
        NamespaceObjectData {
            base,
            namespace,
            prefix: if namespace.as_uri().is_empty() {
                Some("".into())
            } else {
                None
            },
        },
    ))
    .into())
}

/// An Object which represents a boxed namespace name.
#[derive(Collect, Clone, Copy)]
#[collect(no_drop)]
pub struct NamespaceObject<'gc>(pub GcCell<'gc, NamespaceObjectData<'gc>>);

#[derive(Collect, Clone, Copy, Debug)]
#[collect(no_drop)]
pub struct NamespaceObjectWeak<'gc>(pub GcWeakCell<'gc, NamespaceObjectData<'gc>>);

impl fmt::Debug for NamespaceObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NamespaceObject")
            .field("ptr", &self.0.as_ptr())
            .finish()
    }
}

#[derive(Collect, Clone)]
#[collect(no_drop)]
pub struct NamespaceObjectData<'gc> {
    /// All normal script data.
    base: ScriptObjectData<'gc>,

    /// The namespace name this object is associated with.
    namespace: Namespace<'gc>,

    /// The prefix that this namespace has been given.
    prefix: Option<AvmString<'gc>>,
}

impl<'gc> NamespaceObject<'gc> {
    /// Box a namespace into an object.
    pub fn from_namespace(
        activation: &mut Activation<'_, 'gc>,
        namespace: Namespace<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let class = activation.avm2().classes().namespace;
        let base = ScriptObjectData::new(class);

        let this: Object<'gc> = NamespaceObject(GcCell::new(
            activation.context.gc_context,
            NamespaceObjectData {
                base,
                namespace,
                prefix: if namespace.as_uri().is_empty() {
                    Some("".into())
                } else {
                    None
                },
            },
        ))
        .into();
        this.install_instance_slots(activation.context.gc_context);

        class.call_native_init(this.into(), &[], activation)?;

        Ok(this)
    }

    pub fn init_namespace(&self, mc: &Mutation<'gc>, namespace: Namespace<'gc>) {
        self.0.write(mc).namespace = namespace;
    }

    pub fn namespace(self) -> Namespace<'gc> {
        return self.0.read().namespace;
    }

    pub fn prefix(&self) -> Option<AvmString<'gc>> {
        self.0.read().prefix
    }

    pub fn set_prefix(&self, mc: &Mutation<'gc>, prefix: Option<AvmString<'gc>>) {
        self.0.write(mc).prefix = prefix;
    }
}

impl<'gc> TObject<'gc> for NamespaceObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: &Mutation<'gc>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(self.0.read().namespace.as_uri().into())
    }

    fn as_namespace(&self) -> Option<Ref<Namespace<'gc>>> {
        Some(Ref::map(self.0.read(), |s| &s.namespace))
    }

    fn as_namespace_object(&self) -> Option<Self> {
        Some(*self)
    }

    fn property_is_enumerable(&self, name: AvmString<'gc>) -> bool {
        &name == b"prefix" || &name == b"uri"
    }

    fn get_next_enumerant(
        self,
        last_index: u32,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<Option<u32>, Error<'gc>> {
        Ok(if last_index < 2 {
            Some(last_index + 1)
        } else {
            Some(0)
        })
    }

    fn get_enumerant_value(
        self,
        index: u32,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        Ok(match index {
            1 => self.namespace().as_uri().into(),
            2 => self.prefix().map(Into::into).unwrap_or(Value::Undefined),
            _ => Value::Undefined,
        })
    }

    fn get_enumerant_name(
        self,
        index: u32,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        Ok(match index {
            1 => "uri".into(),
            2 => "prefix".into(),
            _ => Value::Undefined,
        })
    }
}
