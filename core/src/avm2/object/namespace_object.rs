//! Boxed namespaces

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Namespace;
use crate::string::AvmString;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::{lock::Lock, Collect, Gc, GcWeak, Mutation};

/// A class instance allocator that allocates namespace objects.
pub fn namespace_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    let namespace = activation.context.avm2.public_namespace_base_version;
    Ok(NamespaceObject(Gc::new(
        activation.context.gc_context,
        NamespaceObjectData {
            base,
            namespace: Lock::new(namespace),
            prefix: Lock::new(if namespace.as_uri().is_empty() {
                Some("".into())
            } else {
                None
            }),
        },
    ))
    .into())
}

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
    namespace: Lock<Namespace<'gc>>,

    /// The prefix that this namespace has been given.
    prefix: Lock<Option<AvmString<'gc>>>,
}

const _: () = assert!(std::mem::offset_of!(NamespaceObjectData, base) == 0);
const _: () = assert!(
    std::mem::align_of::<NamespaceObjectData>() == std::mem::align_of::<ScriptObjectData>()
);

impl<'gc> NamespaceObject<'gc> {
    /// Box a namespace into an object.
    pub fn from_namespace(
        activation: &mut Activation<'_, 'gc>,
        namespace: Namespace<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let class = activation.avm2().classes().namespace;
        let base = ScriptObjectData::new(class);

        let this: Object<'gc> = NamespaceObject(Gc::new(
            activation.context.gc_context,
            NamespaceObjectData {
                base,
                namespace: Lock::new(namespace),
                prefix: Lock::new(if namespace.as_uri().is_empty() {
                    Some("".into())
                } else {
                    None
                }),
            },
        ))
        .into();
        this.install_instance_slots(activation.context.gc_context);

        class.call_native_init(this.into(), &[], activation)?;

        Ok(this)
    }

    pub fn namespace(self) -> Namespace<'gc> {
        self.0.namespace.get()
    }

    pub fn init_namespace(&self, mc: &Mutation<'gc>, namespace: Namespace<'gc>) {
        unlock!(Gc::write(mc, self.0), NamespaceObjectData, namespace).set(namespace);
    }

    pub fn prefix(&self) -> Option<AvmString<'gc>> {
        self.0.prefix.get()
    }

    pub fn set_prefix(&self, mc: &Mutation<'gc>, prefix: Option<AvmString<'gc>>) {
        unlock!(Gc::write(mc, self.0), NamespaceObjectData, prefix).set(prefix);
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

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(self.0.namespace.get().as_uri().into())
    }

    fn as_namespace(&self) -> Option<Namespace<'gc>> {
        Some(self.0.namespace.get())
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
