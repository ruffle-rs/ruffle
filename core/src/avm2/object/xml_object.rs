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
use super::PrimitiveObject;

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
            .call(Some(self.into()), arguments, activation);
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

    fn set_property_local(
        self,
        name: &Multiname<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        let write = self.0.write(activation.context.gc_context);
        let mut kind = write.node.kind_mut(activation.context.gc_context);
        let E4XNodeKind::Element {
            attributes,
            ..
        } = &mut *kind else {
                return Ok(());
            };

        #[allow(clippy::collapsible_if)]
        if name.contains_public_namespace() && name.is_attribute() {
            if !attributes.iter_mut().any(|attr| attr.matches_name(name)) {
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
                let new_attr = E4XNode::attribute(activation.context.gc_context, local_name, value);
                attributes.push(new_attr);
                return Ok(());
            }
        }

        Err(format!("Modifying an XML object is not yet implemented: {name:?} = {value:?}").into())
    }
}
