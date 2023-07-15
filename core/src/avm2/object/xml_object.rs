//! Object representation for XML objects

use crate::avm2::activation::Activation;
use crate::avm2::e4x::{name_to_multiname, E4XNode, E4XNodeKind};
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject, XmlListObject};
use crate::avm2::string::AvmString;
use crate::avm2::value::Value;
use crate::avm2::{Error, Multiname};
use core::fmt;
use gc_arena::{Collect, GcCell, GcWeakCell, MutationContext};
use std::cell::{Ref, RefMut};

use super::xml_list_object::E4XOrXml;
use super::PrimitiveObject;

/// A class instance allocator that allocates XML objects.
pub fn xml_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(XmlObject(GcCell::new(
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
pub struct XmlObject<'gc>(pub GcCell<'gc, XmlObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct XmlObjectWeak<'gc>(pub GcWeakCell<'gc, XmlObjectData<'gc>>);

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
        XmlObject(GcCell::new(
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

    pub fn equals(
        &self,
        other: &Value<'gc>,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<bool, Error<'gc>> {
        // 1. If Type(V) is not XML, return false.
        let other = if let Some(xml_obj) = other.as_object().and_then(|obj| obj.as_xml_object()) {
            xml_obj
        } else {
            return Ok(false);
        };

        // It seems like an XML object should always be equal to itself
        if Object::ptr_eq(*self, other) {
            return Ok(true);
        }

        let node = other.node();
        Ok(self.node().equals(&node))
    }

    // Implements "The Abstract Equality Comparison Algorithm" as defined
    // in ECMA-357 when one side is an XML type (object).
    pub fn abstract_eq(
        &self,
        other: &Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<bool, Error<'gc>> {
        // 3.a. If both x and y are the same type (XML)
        if let Value::Object(obj) = other {
            if let Some(xml_obj) = obj.as_xml_object() {
                if (matches!(
                    &*self.node().kind(),
                    E4XNodeKind::Text(_) | E4XNodeKind::CData(_) | E4XNodeKind::Attribute(_)
                ) && xml_obj.node().has_simple_content())
                    || (matches!(
                        &*xml_obj.node().kind(),
                        E4XNodeKind::Text(_) | E4XNodeKind::CData(_) | E4XNodeKind::Attribute(_)
                    ) && self.node().has_simple_content())
                {
                    return Ok(self.node().xml_to_string(activation)?
                        == xml_obj.node().xml_to_string(activation)?);
                }

                return self.equals(other, activation);
            }
        }

        // 4. If (Type(x) is XML) and x.hasSimpleContent() == true)
        if self.node().has_simple_content() {
            return Ok(
                self.node().xml_to_string(activation)? == other.coerce_to_string(activation)?
            );
        }

        // It seems like everything else will just ultimately fall-through to the last step.
        Ok(false)
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

        if !name.has_explicit_namespace() {
            if let Some(local_name) = name.local_name() {
                // The only supported numerical index is 0
                if let Ok(index) = local_name.parse::<usize>() {
                    if index == 0 {
                        return Ok(self.into());
                    } else {
                        return Ok(Value::Undefined);
                    }
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

            return Ok(XmlListObject::new(activation, matched_children, Some(self.into())).into());
        }

        read.base.get_property_local(name, activation)
    }

    fn call_property_local(
        self,
        multiname: &Multiname<'gc>,
        arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let this = self.as_xml_object().unwrap();

        let method = self
            .proto()
            .expect("XMLList misisng prototype")
            .get_property(multiname, activation)?;

        // If the method doesn't exist on the prototype, and we have simple content,
        // then coerce this XML to a string and call the method on that.
        // This lets things like `new XML("<p>Hello world</p>").split(" ")` work.
        if matches!(method, Value::Undefined) {
            // Checking if we have a child with the same name as the method is probably
            // unecessary - if we had such a child, then we wouldn't have simple content,
            // so we already would bail out before calling the method. Nevertheless,
            // avmplus has this check, so we do it out of an abundance of caution.
            // Compare to the very similar case in XMLListObject::call_property_local
            let prop = self.get_property_local(multiname, activation)?;
            if let Some(list) = prop.as_object().and_then(|obj| obj.as_xml_list_object()) {
                if list.length() == 0 && this.node().has_simple_content() {
                    let receiver = PrimitiveObject::from_primitive(
                        this.node().xml_to_string(activation)?.into(),
                        activation,
                    )?;
                    return receiver.call_property(multiname, arguments, activation);
                }
            }
        }

        return method
            .as_callable(activation, Some(multiname), Some(self.into()))?
            .call(self.into(), arguments, activation);
    }

    fn has_own_property(self, name: &Multiname<'gc>) -> bool {
        let read = self.0.read();

        // FIXME - see if we can deduplicate this with get_property_local in
        // an efficient way
        if !name.has_explicit_namespace() {
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

    fn has_own_property_string(
        self,
        name: impl Into<AvmString<'gc>>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<bool, Error<'gc>> {
        let name = name_to_multiname(activation, &Value::String(name.into()), false)?;
        Ok(self.has_own_property(&name))
    }

    fn set_property_local(
        self,
        name: &Multiname<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        if name.has_explicit_namespace() {
            return Err(format!(
                "Can not set property {:?} with an explicit namespace yet",
                name
            )
            .into());
        }

        let mc = activation.context.gc_context;

        if name.is_attribute() {
            self.delete_property_local(activation, name)?;
            if let Some(obj) = value.as_object() {
                if obj.as_xml_object().is_some() || obj.as_xml_list_object().is_some() {
                    return Err(format!(
                        "Cannot set an XML/XMLList object {:?} as an attribute",
                        obj
                    )
                    .into());
                }
            }
            let Some(local_name) = name.local_name() else {
                return Err(format!("Cannot set attribute {:?} without a local name", name).into());
            };
            let value = value.coerce_to_string(activation)?;
            let new_attr = E4XNode::attribute(mc, local_name, value, *self.node());

            let write = self.0.write(mc);
            let mut kind = write.node.kind_mut(mc);
            let E4XNodeKind::Element { attributes, .. } = &mut *kind else {
                return Ok(());
            };

            attributes.push(new_attr);
            return Ok(());
        }

        let self_node = *self.node();
        let write = self.0.write(mc);
        let mut kind = write.node.kind_mut(mc);
        let E4XNodeKind::Element { children, .. } = &mut *kind else {
            return Ok(());
        };

        if value.as_object().map_or(false, |obj| {
            obj.as_xml_object().is_some() || obj.as_xml_list_object().is_some()
        }) {
            return Err(
                format!("Cannot set an XML/XMLList object {value:?} as an element yet").into(),
            );
        }

        if name.is_any_name() {
            return Err("Any name (*) not yet implemented for set".into());
        }

        let text = value.coerce_to_string(activation)?;

        let matches: Vec<_> = children
            .iter()
            .filter(|child| child.matches_name(name))
            .collect();
        match matches.as_slice() {
            [] => {
                let element_with_text =
                    E4XNode::element(mc, name.local_name().unwrap(), write.node);
                element_with_text.append_child(mc, E4XNode::text(mc, text, Some(self_node)))?;
                children.push(element_with_text);
                Ok(())
            }
            [child] => {
                child.remove_all_children(mc);
                child.append_child(mc, E4XNode::text(mc, text, Some(self_node)))?;
                Ok(())
            }
            _ => Err(format!("Can not replace multiple elements yet: {name:?} = {value:?}").into()),
        }
    }

    fn delete_property_local(
        self,
        activation: &mut Activation<'_, 'gc>,
        name: &Multiname<'gc>,
    ) -> Result<bool, Error<'gc>> {
        if name.has_explicit_namespace() {
            return Err(format!(
                "Can not set property {:?} with an explicit namespace yet",
                name
            )
            .into());
        }

        let mc = activation.context.gc_context;
        let write = self.0.write(mc);
        let mut kind = write.node.kind_mut(mc);
        let E4XNodeKind::Element {
            children,
            attributes,
            ..
        } = &mut *kind
        else {
            return Ok(false);
        };

        let retain_non_matching = |node: &E4XNode<'gc>| {
            if node.matches_name(name) {
                node.set_parent(None, mc);
                false
            } else {
                true
            }
        };

        if name.is_attribute() {
            attributes.retain(retain_non_matching);
        } else {
            children.retain(retain_non_matching);
        }
        Ok(true)
    }
}
