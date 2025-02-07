//! Boxed QNames

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::AvmString;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::string::StringContext;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::{lock::RefLock, Collect, Gc, GcWeak, Mutation};
use ruffle_macros::istr;
use std::cell::Ref;

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
    pub fn new_empty(activation: &mut Activation<'_, 'gc>) -> Self {
        let base = ScriptObjectData::new(activation.avm2().classes().qname);

        QNameObject(Gc::new(
            activation.gc(),
            QNameObjectData {
                base,
                name: RefLock::new(Multiname::any()),
            },
        ))
    }

    /// Box a Multiname into an object.
    pub fn from_name(activation: &mut Activation<'_, 'gc>, name: Multiname<'gc>) -> Self {
        let class = activation.avm2().classes().qname;
        let base = ScriptObjectData::new(class);

        QNameObject(Gc::new(
            activation.gc(),
            QNameObjectData {
                base,
                name: RefLock::new(name),
            },
        ))
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

    pub fn local_name(&self, context: &mut StringContext<'gc>) -> AvmString<'gc> {
        let name = self.name();

        name.local_name().unwrap_or_else(|| istr!(context, "*"))
    }

    pub fn set_is_qname(&self, mc: &Mutation<'gc>, is_qname: bool) {
        let mut write_name = unlock!(Gc::write(mc, self.0), QNameObjectData, name).borrow_mut();

        write_name.set_is_qname(is_qname);
    }

    pub fn uri(&self, context: &mut StringContext<'gc>) -> Option<AvmString<'gc>> {
        let name = self.0.name.borrow();

        if name.is_any_namespace() {
            None
        } else if name.namespace_set().len() > 1 {
            Some(context.empty())
        } else {
            name.namespace_set()
                .first()
                .expect("Malformed multiname")
                .as_uri_opt()
        }
    }

    pub fn is_any_namespace(&self) -> bool {
        self.0.name.borrow().is_any_namespace()
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

    fn as_qname_object(self) -> Option<QNameObject<'gc>> {
        Some(self)
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
        // NOTE: Weird avmplus behavior, get_enumerant_name returns uri first, but get_enumerant_value returns localName first.
        Ok(match index {
            1 => self.local_name(activation.strings()).into(),
            2 => self
                .uri(activation.strings())
                .unwrap_or_else(|| istr!(""))
                .into(),
            _ => Value::Undefined,
        })
    }

    fn get_enumerant_name(
        self,
        index: u32,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        // NOTE: Weird avmplus behavior, get_enumerant_name returns uri first, but get_enumerant_value returns localName first.
        Ok(match index {
            1 => istr!("uri").into(),
            2 => istr!("localName").into(),
            _ => Value::Null,
        })
    }
}
