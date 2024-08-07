//! Boxed QNames

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::AvmString;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::{lock::RefLock, Collect, Gc, GcWeak, Mutation};
use std::cell::Ref;

/// A class instance allocator that allocates QName objects.
pub fn q_name_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(QNameObject(Gc::new(
        activation.context.gc_context,
        QNameObjectData {
            base,
            name: RefLock::new(Multiname::any(activation.context.gc_context)),
        },
    ))
    .into())
}

/// An Object which represents a boxed QName.
#[derive(Collect, Clone, Copy)]
#[collect(no_drop)]
pub struct QNameObject<'gc>(pub Gc<'gc, QNameObjectData<'gc>>);

#[derive(Collect, Clone, Copy, Debug)]
#[collect(no_drop)]
pub struct QNameObjectWeak<'gc>(pub GcWeak<'gc, QNameObjectData<'gc>>);

impl fmt::Debug for QNameObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("QNameObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Collect, Clone)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct QNameObjectData<'gc> {
    /// All normal script data.
    base: ScriptObjectData<'gc>,

    /// The Multiname this object is associated with.
    name: RefLock<Multiname<'gc>>,
}

const _: () = assert!(std::mem::offset_of!(QNameObjectData, base) == 0);
const _: () =
    assert!(std::mem::align_of::<QNameObjectData>() == std::mem::align_of::<ScriptObjectData>());

impl<'gc> QNameObject<'gc> {
    /// Box a Multiname into an object.
    pub fn from_name(
        activation: &mut Activation<'_, 'gc>,
        name: Multiname<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let class = activation.avm2().classes().qname;
        let base = ScriptObjectData::new(class);

        let this: Object<'gc> = QNameObject(Gc::new(
            activation.context.gc_context,
            QNameObjectData {
                base,
                name: RefLock::new(name),
            },
        ))
        .into();
        this.install_instance_slots(activation.context.gc_context);

        Ok(this)
    }

    pub fn name(&self) -> Ref<Multiname<'gc>> {
        self.0.name.borrow()
    }

    pub fn set_namespace(&self, mc: &Mutation<'gc>, namespace: Namespace<'gc>) {
        let mut write_name = unlock!(Gc::write(mc, self.0), QNameObjectData, name).borrow_mut();

        write_name.set_single_namespace(namespace);
    }

    pub fn set_local_name(&self, mc: &Mutation<'gc>, local: AvmString<'gc>) {
        let mut write_name = unlock!(Gc::write(mc, self.0), QNameObjectData, name).borrow_mut();

        write_name.set_local_name(local);
    }

    pub fn local_name(&self) -> AvmString<'gc> {
        let name = self.name();

        name.local_name().unwrap_or("*".into())
    }

    pub fn set_is_qname(&self, mc: &Mutation<'gc>, is_qname: bool) {
        let mut write_name = unlock!(Gc::write(mc, self.0), QNameObjectData, name).borrow_mut();

        write_name.set_is_qname(is_qname);
    }

    pub fn uri(&self) -> Option<AvmString<'gc>> {
        let name = self.0.name.borrow();

        if name.is_any_namespace() {
            None
        } else if name.namespace_set().len() > 1 {
            Some("".into())
        } else {
            Some(
                name.namespace_set()
                    .first()
                    .expect("Malformed multiname")
                    .as_uri(),
            )
        }
    }

    pub fn init_name(self, mc: &Mutation<'gc>, name: Multiname<'gc>) {
        let mut write_name = unlock!(Gc::write(mc, self.0), QNameObjectData, name).borrow_mut();

        *write_name = name;
    }
}

impl<'gc> TObject<'gc> for QNameObject<'gc> {
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
        Ok(Value::Object(Object::from(*self)))
    }

    fn as_qname_object(self) -> Option<QNameObject<'gc>> {
        Some(self)
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
        // NOTE: Weird avmplus behavior, get_enumerant_name returns uri first, but get_enumerant_value returns localName first.
        Ok(match index {
            1 => self.local_name().into(),
            2 => self.uri().map(Into::into).unwrap_or("".into()),
            _ => Value::Undefined,
        })
    }

    fn get_enumerant_name(
        self,
        index: u32,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        // NOTE: Weird avmplus behavior, get_enumerant_name returns uri first, but get_enumerant_value returns localName first.
        Ok(match index {
            1 => "uri".into(),
            2 => "localName".into(),
            _ => Value::Undefined,
        })
    }
}
