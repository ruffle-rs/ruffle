//! Boxed QNames

use crate::avm2::AvmString;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::activation::Activation;
use crate::avm2::object::TObject;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::value::Value;
use crate::string::StringContext;
use core::fmt;
use gc_arena::{Collect, Gc, GcWeak};
use ruffle_common::utils::HasPrefixField;
use ruffle_macros::istr;

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

#[derive(Collect, Clone, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct QNameObjectData<'gc> {
    /// All normal script data.
    base: ScriptObjectData<'gc>,

    /// The Multiname this object is associated with.
    name: Multiname<'gc>,
}

impl<'gc> QNameObject<'gc> {
    /// Box a Multiname into an object.
    pub fn from_name(activation: &mut Activation<'_, 'gc>, name: Multiname<'gc>) -> Self {
        let class = activation.avm2().classes().qname;
        let base = ScriptObjectData::new(class);

        QNameObject(Gc::new(activation.gc(), QNameObjectData { base, name }))
    }

    pub fn name(&self) -> &Multiname<'gc> {
        &self.0.name
    }

    pub fn local_name(self, context: &mut StringContext<'gc>) -> AvmString<'gc> {
        let name = self.name();

        name.local_name().unwrap_or_else(|| istr!(context, "*"))
    }

    pub fn uri(self, context: &mut StringContext<'gc>) -> Option<AvmString<'gc>> {
        let name = &self.0.name;

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

    pub fn is_any_namespace(self) -> bool {
        self.0.name.is_any_namespace()
    }
}

impl<'gc> TObject<'gc> for QNameObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
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
