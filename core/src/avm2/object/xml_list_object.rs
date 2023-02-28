use crate::avm2::activation::Activation;
use crate::avm2::e4x::E4XNode;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::{Error, Multiname};
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};
use std::fmt::{self, Debug};
use std::ops::Deref;

use super::{ClassObject, XmlObject};

/// A class instance allocator that allocates XMLList objects.
pub fn xml_list_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(XmlListObject(GcCell::allocate(
        activation.context.gc_context,
        XmlListObjectData {
            base,
            children: Vec::new(),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct XmlListObject<'gc>(GcCell<'gc, XmlListObjectData<'gc>>);

impl<'gc> Debug for XmlListObject<'gc> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("XmlListObject")
            .field("ptr", &self.0.as_ptr())
            .finish()
    }
}

impl<'gc> XmlListObject<'gc> {
    pub fn new(activation: &mut Activation<'_, 'gc>, children: Vec<E4XOrXml<'gc>>) -> Self {
        let base = ScriptObjectData::new(activation.context.avm2.classes().xml_list);
        XmlListObject(GcCell::allocate(
            activation.context.gc_context,
            XmlListObjectData { base, children },
        ))
    }

    pub fn children(&self) -> Ref<'_, Vec<E4XOrXml<'gc>>> {
        Ref::map(self.0.read(), |d| &d.children)
    }

    pub fn set_children(&self, mc: MutationContext<'gc, '_>, children: Vec<E4XOrXml<'gc>>) {
        self.0.write(mc).children = children;
    }
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct XmlListObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    children: Vec<E4XOrXml<'gc>>,
}

/// Holds either an `E4XNode` or an `XmlObject`. This can be converted
/// in-palce to an `XmlObject` via `get_or_create_xml`.
/// This deliberately does not implement `Copy`, since `get_or_create_xml`
/// takes `&mut self`
#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub enum E4XOrXml<'gc> {
    E4X(E4XNode<'gc>),
    Xml(XmlObject<'gc>),
}

impl<'gc> E4XOrXml<'gc> {
    pub fn get_or_create_xml(&mut self, activation: &mut Activation<'_, 'gc>) -> XmlObject<'gc> {
        match self {
            E4XOrXml::E4X(node) => {
                let xml = XmlObject::new(*node, activation);
                *self = E4XOrXml::Xml(xml);
                xml
            }
            E4XOrXml::Xml(xml) => *xml,
        }
    }

    pub fn node(&self) -> E4XWrapper<'_, 'gc> {
        match self {
            E4XOrXml::E4X(node) => E4XWrapper::E4X(*node),
            E4XOrXml::Xml(xml) => E4XWrapper::XmlRef(xml.node()),
        }
    }
}

// Allows using `E4XOrXml` as an `E4XNode` via deref coercions, while
// storing the needed `Ref` wrappers
#[derive(Debug)]
pub enum E4XWrapper<'a, 'gc> {
    E4X(E4XNode<'gc>),
    XmlRef(Ref<'a, E4XNode<'gc>>),
}

impl<'a, 'gc> Deref for E4XWrapper<'a, 'gc> {
    type Target = E4XNode<'gc>;

    fn deref(&self) -> &Self::Target {
        match self {
            E4XWrapper::E4X(node) => node,
            E4XWrapper::XmlRef(node) => node,
        }
    }
}

impl<'gc> TObject<'gc> for XmlListObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn as_xml_list_object(&self) -> Option<Self> {
        Some(*self)
    }

    fn get_property_local(
        self,
        name: &Multiname<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        // FIXME - implement everything from E4X spec (XMLListObject::getMultinameProperty in avmplus)
        let mut write = self.0.write(activation.context.gc_context);

        if name.contains_public_namespace() {
            if let Some(local_name) = name.local_name() {
                if let Ok(index) = local_name.parse::<usize>() {
                    if let Some(child) = write.children.get_mut(index) {
                        return Ok(Value::Object(child.get_or_create_xml(activation).into()));
                    } else {
                        return Ok(Value::Undefined);
                    }
                }

                let matched_children = write
                    .children
                    .iter_mut()
                    .flat_map(|child| {
                        let child_prop = child
                            .get_or_create_xml(activation)
                            .get_property_local(name, activation)
                            .unwrap();
                        if let Some(prop_xml) =
                            child_prop.as_object().and_then(|obj| obj.as_xml_object())
                        {
                            vec![E4XOrXml::Xml(prop_xml)]
                        } else if let Some(prop_xml_list) = child_prop
                            .as_object()
                            .and_then(|obj| obj.as_xml_list_object())
                        {
                            // Flatten children
                            prop_xml_list.children().clone()
                        } else {
                            vec![]
                        }
                    })
                    .collect();

                return Ok(XmlListObject::new(activation, matched_children).into());
            }
        }

        write.base.get_property_local(name, activation)
    }

    fn set_property_local(
        self,
        _name: &Multiname<'gc>,
        _value: Value<'gc>,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        Err("Modifying an XMLList object is not yet implemented".into())
    }

    fn get_next_enumerant(
        self,
        last_index: u32,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<Option<u32>, Error<'gc>> {
        let read = self.0.read();
        if (last_index as usize) < read.children.len() {
            return Ok(Some(last_index + 1));
        }
        Ok(None)
    }

    fn get_enumerant_name(
        self,
        index: u32,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let children_len = self.0.read().children.len() as u32;
        if children_len >= index {
            Ok(index
                .checked_sub(1)
                .map(|index| index.into())
                .unwrap_or(Value::Undefined))
        } else {
            Ok(self
                .base()
                .get_enumerant_name(index - children_len)
                .unwrap_or(Value::Undefined))
        }
    }
}
