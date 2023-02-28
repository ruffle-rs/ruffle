//! Object representation for XML objects

use crate::avm2::activation::Activation;
use crate::avm2::e4x::{E4XNode, E4XNodeKind};
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject, XmlListObject};
use crate::avm2::string::AvmString;
use crate::avm2::value::Value;
use crate::avm2::{Error, Multiname};
use core::fmt;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};

use super::xml_list_object::E4XOrXml;

/// A class instance allocator that allocates XML objects.
pub fn xml_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(XmlObject(GcCell::allocate(
        activation.context.gc_context,
        XmlObjectData {
            base,
            node: E4XNode::dummy(activation.context.gc_context),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct XmlObject<'gc>(GcCell<'gc, XmlObjectData<'gc>>);

impl fmt::Debug for XmlObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("XmlObject")
            .field("ptr", &self.0.as_ptr())
            .finish()
    }
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct XmlObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    node: E4XNode<'gc>,
}

impl<'gc> XmlObject<'gc> {
    pub fn new(node: E4XNode<'gc>, activation: &mut Activation<'_, 'gc>) -> Self {
        XmlObject(GcCell::allocate(
            activation.context.gc_context,
            XmlObjectData {
                base: ScriptObjectData::new(activation.context.avm2.classes().xml),
                node,
            },
        ))
    }
    pub fn set_node(&self, mc: MutationContext<'gc, '_>, node: E4XNode<'gc>) {
        self.0.write(mc).node = node;
    }

    pub fn local_name(&self) -> Option<AvmString<'gc>> {
        self.0.read().node.local_name()
    }

    pub fn matches_name(&self, multiname: &Multiname<'gc>) -> bool {
        self.0.read().node.matches_name(multiname)
    }

    pub fn node(&self) -> Ref<'_, E4XNode<'gc>> {
        Ref::map(self.0.read(), |data| &data.node)
    }
}

impl<'gc> TObject<'gc> for XmlObject<'gc> {
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

    fn as_xml_object(&self) -> Option<Self> {
        Some(*self)
    }

    fn get_property_local(
        self,
        name: &Multiname<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        // FIXME - implement everything from E4X spec (XMLObject::getMultinameProperty in avmplus)
        let read = self.0.read();

        if name.contains_public_namespace() {
            if let Some(local_name) = name.local_name() {
                // The only supported numerical index is 0
                if let Ok(index) = local_name.parse::<usize>() {
                    if index == 0 {
                        return Ok(self.into());
                    } else {
                        return Ok(Value::Undefined);
                    }
                }

                let matched_children = if let E4XNodeKind::Element {
                    children,
                    attributes,
                } = &*read.node.kind()
                {
                    let search_children = if name.is_attribute() {
                        attributes
                    } else {
                        children
                    };

                    search_children
                        .iter()
                        .filter_map(|child| {
                            if child.matches_name(name) {
                                Some(E4XOrXml::E4X(*child))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                } else {
                    Vec::new()
                };
                return Ok(XmlListObject::new(activation, matched_children).into());
            }
        }

        read.base.get_property_local(name, activation)
    }

    fn has_own_property(self, name: &Multiname<'gc>) -> bool {
        let read = self.0.read();

        // FIXME - see if we can deduplicate this with get_property_local in
        // an efficient way
        if name.contains_public_namespace() {
            if let Some(local_name) = name.local_name() {
                // The only supported numerical index is 0
                if let Ok(index) = local_name.parse::<usize>() {
                    return index == 0;
                }

                if let E4XNodeKind::Element {
                    children,
                    attributes,
                } = &*read.node.kind()
                {
                    let search_children = if name.is_attribute() {
                        attributes
                    } else {
                        children
                    };

                    return search_children.iter().any(|child| child.matches_name(name));
                }
            }
        }
        read.base.has_own_dynamic_property(name)
    }
}
