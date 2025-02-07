//! Boxed namespaces

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Namespace;
use crate::string::AvmString;
use core::fmt;
use gc_arena::{Collect, Gc, GcWeak};
use ruffle_macros::istr;

/// An Object which represents a boxed namespace name.
#[derive(Collect, Clone, Copy)]
#[collect(no_drop)]
pub struct NamespaceObject<'gc>(pub Gc<'gc, NamespaceObjectData<'gc>>);

#[derive(Collect, Clone, Copy, Debug)]
#[collect(no_drop)]
pub struct NamespaceObjectWeak<'gc>(pub GcWeak<'gc, NamespaceObjectData<'gc>>);

impl fmt::Debug for NamespaceObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NamespaceObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Collect, Clone)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct NamespaceObjectData<'gc> {
    /// All normal script data.
    base: ScriptObjectData<'gc>,

    /// The namespace name this object is associated with.
    namespace: Namespace<'gc>,

    /// The prefix that this namespace has been given.
    prefix: Option<AvmString<'gc>>,
}

const _: () = assert!(std::mem::offset_of!(NamespaceObjectData, base) == 0);
const _: () = assert!(
    std::mem::align_of::<NamespaceObjectData>() == std::mem::align_of::<ScriptObjectData>()
);

impl<'gc> NamespaceObject<'gc> {
    pub fn from_ns_and_prefix(
        activation: &mut Activation<'_, 'gc>,
        namespace: Namespace<'gc>,
        prefix: Option<AvmString<'gc>>,
    ) -> Self {
        let class = activation.avm2().classes().namespace;
        let base = ScriptObjectData::new(class);

        NamespaceObject(Gc::new(
            activation.gc(),
            NamespaceObjectData {
                base,
                namespace,
                prefix,
            },
        ))
    }

    /// Box a namespace into an object.
    pub fn from_namespace(activation: &mut Activation<'_, 'gc>, namespace: Namespace<'gc>) -> Self {
        let class = activation.avm2().classes().namespace;
        let base = ScriptObjectData::new(class);

        NamespaceObject(Gc::new(
            activation.gc(),
            NamespaceObjectData {
                base,
                namespace,
                prefix: if namespace.as_uri(activation.strings()).is_empty() {
                    Some(istr!(""))
                } else {
                    None
                },
            },
        ))
    }

    pub fn namespace(self) -> Namespace<'gc> {
        self.0.namespace
    }

    pub fn prefix(&self) -> Option<AvmString<'gc>> {
        self.0.prefix
    }
}

impl<'gc> TObject<'gc> for NamespaceObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        // SAFETY: Object data is repr(C), and a compile-time assert ensures
        // that the ScriptObjectData stays at offset 0 of the struct- so the
        // layouts are compatible

        unsafe { Gc::cast(self.0) }
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn as_namespace(&self) -> Option<Namespace<'gc>> {
        Some(self.0.namespace)
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
    ) -> Result<u32, Error<'gc>> {
        Ok(if last_index < 2 { last_index + 1 } else { 0 })
    }

    fn get_enumerant_value(
        self,
        index: u32,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        Ok(match index {
            1 => self.namespace().as_uri(activation.strings()).into(),
            2 => self.prefix().map(Into::into).unwrap_or(Value::Undefined),
            _ => Value::Undefined,
        })
    }

    fn get_enumerant_name(
        self,
        index: u32,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        Ok(match index {
            1 => istr!("uri").into(),
            2 => istr!("prefix").into(),
            _ => Value::Null,
        })
    }
}
