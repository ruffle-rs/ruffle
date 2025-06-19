//! Object representation for XML objects

use crate::avm2::activation::Activation;
use crate::avm2::e4x::{string_to_multiname, E4XNamespace, E4XNode, E4XNodeKind};
use crate::avm2::error::make_error_1087;
use crate::avm2::function::FunctionArgs;
use crate::avm2::multiname::NamespaceSet;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{
    ClassObject, NamespaceObject, Object, ObjectPtr, TObject, XmlListObject,
};
use crate::avm2::string::AvmString;
use crate::avm2::value::Value;
use crate::avm2::{Error, Multiname};
use crate::utils::HasPrefixField;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::{lock::Lock, Collect, Gc, GcWeak, Mutation};
use ruffle_wstr::WString;

use super::xml_list_object::{E4XOrXml, XmlOrXmlListObject};

/// A class instance allocator that allocates XML objects.
pub fn xml_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(XmlObject(Gc::new(
        activation.gc(),
        XmlObjectData {
            base,
            node: Lock::new(E4XNode::dummy(activation.gc())),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct XmlObject<'gc>(pub Gc<'gc, XmlObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct XmlObjectWeak<'gc>(pub GcWeak<'gc, XmlObjectData<'gc>>);

impl fmt::Debug for XmlObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("XmlObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Clone, Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct XmlObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    node: Lock<E4XNode<'gc>>,
}

impl<'gc> XmlObject<'gc> {
    pub fn new(node: E4XNode<'gc>, activation: &mut Activation<'_, 'gc>) -> Self {
        XmlObject(Gc::new(
            activation.gc(),
            XmlObjectData {
                base: ScriptObjectData::new(activation.context.avm2.classes().xml),
                node: Lock::new(node),
            },
        ))
    }

    fn get_child_list(
        self,
        activation: &mut Activation<'_, 'gc>,
        name: &Multiname<'gc>,
    ) -> XmlListObject<'gc> {
        let matched_children = if let E4XNodeKind::Element {
            children,
            attributes,
            ..
        } = &*self.0.node.get().kind()
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

        // NOTE: avmplus does set the target_dirty flag on the list object if there was at least one child
        //       due to the way avmplus implemented this.
        let list = XmlListObject::new_with_children(
            activation,
            matched_children,
            Some(self.into()),
            Some(name.clone()),
        );

        if list.length() > 0 {
            list.set_dirty_flag();
        }

        list
    }

    // 13.4.4.6 XML.prototype.child ( propertyName )
    pub fn child(
        &self,
        name: &Multiname<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> XmlListObject<'gc> {
        // 1. If ToString(ToUint32(propertyName)) == propertyName
        if let Some(local_name) = name.local_name() {
            if let Ok(index) = local_name.parse::<usize>() {
                let result = if let E4XNodeKind::Element { children, .. } = &*self.node().kind() {
                    if let Some(node) = children.get(index) {
                        vec![E4XOrXml::E4X(*node)]
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                };

                let list = XmlListObject::new_with_children(activation, result, None, None);

                if list.length() > 0 {
                    // NOTE: Since avmplus uses appendNode here, when the node exists, that implicitly sets the target_dirty flag.
                    list.set_dirty_flag();
                }

                return list;
            }
        }

        // 2. Let temporary be the result of calling the [[Get]] method of x with argument propertyName
        // 3. Return ToXMLList(temporary)
        self.get_child_list(activation, name)
    }

    pub fn elements(
        &self,
        name: &Multiname<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> XmlListObject<'gc> {
        let children = if let E4XNodeKind::Element { children, .. } = &*self.node().kind() {
            children
                .iter()
                .filter(|node| node.is_element() && node.matches_name(name))
                .map(|node| E4XOrXml::E4X(*node))
                .collect()
        } else {
            Vec::new()
        };

        let list = XmlListObject::new_with_children(
            activation,
            children,
            Some(XmlOrXmlListObject::Xml(*self)),
            // NOTE: Spec says to set target property here, but avmplus doesn't, so we do the same.
            None,
        );

        if list.length() > 0 {
            // NOTE: Since avmplus uses appendNode to build the list here, we need to set target dirty flag.
            list.set_dirty_flag();
        }

        list
    }

    pub fn length(&self) -> Option<usize> {
        self.node().length()
    }

    pub fn set_node(&self, mc: &Mutation<'gc>, node: E4XNode<'gc>) {
        unlock!(Gc::write(mc, self.0), XmlObjectData, node).set(node);
    }

    pub fn local_name(&self) -> Option<AvmString<'gc>> {
        self.0.node.get().local_name()
    }

    pub fn namespace_object(
        &self,
        activation: &mut Activation<'_, 'gc>,
        in_scope_ns: &[E4XNamespace<'gc>],
    ) -> Result<NamespaceObject<'gc>, Error<'gc>> {
        self.node()
            .get_namespace(activation.strings(), in_scope_ns)
            .as_namespace_object(activation)
    }

    pub fn matches_name(&self, multiname: &Multiname<'gc>) -> bool {
        self.0.node.get().matches_name(multiname)
    }

    pub fn node(&self) -> E4XNode<'gc> {
        self.0.node.get()
    }

    pub fn deep_copy(&self, activation: &mut Activation<'_, 'gc>) -> XmlObject<'gc> {
        let node = self.node();
        XmlObject::new(node.deep_copy(activation.gc()), activation)
    }

    pub fn as_xml_string(&self, activation: &mut Activation<'_, 'gc>) -> AvmString<'gc> {
        let node = self.node();
        node.xml_to_xml_string(activation)
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
                // 3.a.i. If ((x.[[Class]] ∈ {"text", "attribute"}) and (y.hasSimpleContent())
                // or ((y.[[Class]] ∈ {"text", "attribute"}) and (x.hasSimpleContent())
                if ((self.node().is_text() || self.node().is_attribute())
                    && xml_obj.node().has_simple_content())
                    || ((xml_obj.node().is_text() || xml_obj.node().is_attribute())
                        && self.node().has_simple_content())
                {
                    // 3.a.i.1. Return the result of the comparison ToString(x) == ToString(y)
                    return Ok(self.node().xml_to_string(activation)
                        == xml_obj.node().xml_to_string(activation));
                }

                // 3.a.i. Else return the result of calling the [[Equals]] method of x with argument y
                return self.equals(other, activation);
            }
        }

        // 4. If (Type(x) is XML and x.hasSimpleContent() == true)
        if self.node().has_simple_content() {
            // 4.a. Return the result of the comparison ToString(x) == ToString(y)
            return Ok(self.node().xml_to_string(activation) == other.coerce_to_string(activation)?);
        }

        // It seems like everything else will just ultimately fall-through to the last step.
        Ok(false)
    }
}

impl<'gc> TObject<'gc> for XmlObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn as_xml_object(&self) -> Option<Self> {
        Some(*self)
    }

    fn xml_descendants(
        &self,
        activation: &mut Activation<'_, 'gc>,
        multiname: &Multiname<'gc>,
    ) -> Option<XmlListObject<'gc>> {
        let mut descendants = Vec::new();
        self.0.node.get().descendants(multiname, &mut descendants);

        let list = XmlListObject::new_with_children(activation, descendants, None, None);
        // NOTE: avmplus does not set a target property/object here, but if there was at least one child
        //       then the target_dirty flag would be set, since avmplus used appendNode which always sets it.
        if list.length() > 0 {
            list.set_dirty_flag();
        }

        Some(list)
    }

    fn get_property_local(
        self,
        name: &Multiname<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        // FIXME - implement everything from E4X spec (XMLObject::getMultinameProperty in avmplus)

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
        }

        let name = handle_input_multiname(name.clone(), activation);
        let list = self.get_child_list(activation, &name);
        Ok(list.into())
    }

    fn call_property_local(
        self,
        multiname: &Multiname<'gc>,
        arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let this = self.as_xml_object().unwrap();

        let method = Value::from(self.proto().expect("XMLList missing prototype"))
            .get_property(multiname, activation)?;

        // If the method doesn't exist on the prototype, and we have simple content,
        // then coerce this XML to a string and call the method on that.
        // This lets things like `new XML("<p>Hello world</p>").split(" ")` work.
        if matches!(method, Value::Undefined) {
            // Checking if we have a child with the same name as the method is probably
            // unnecessary - if we had such a child, then we wouldn't have simple content,
            // so we already would bail out before calling the method. Nevertheless,
            // avmplus has this check, so we do it out of an abundance of caution.
            // Compare to the very similar case in XMLListObject::call_property_local
            let prop = self.get_property_local(multiname, activation)?;
            if let Some(list) = prop.as_object().and_then(|obj| obj.as_xml_list_object()) {
                if list.length() == 0 && this.node().has_simple_content() {
                    let receiver = Value::String(this.node().xml_to_string(activation));

                    return receiver.call_property(
                        multiname,
                        FunctionArgs::AsArgSlice { arguments },
                        activation,
                    );
                }
            }
        }

        method.call(activation, self.into(), arguments)
    }

    fn has_own_property(self, name: &Multiname<'gc>) -> bool {
        if self.node().has_property(name) {
            return true;
        }

        self.base().has_own_dynamic_property(name)
    }

    fn has_property_via_in(
        self,
        activation: &mut Activation<'_, 'gc>,
        name: &Multiname<'gc>,
    ) -> Result<bool, Error<'gc>> {
        let multiname = handle_input_multiname(name.clone(), activation);
        Ok(self.has_property(&multiname))
    }

    fn has_own_property_string(
        self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<bool, Error<'gc>> {
        let multiname = string_to_multiname(activation, name);
        Ok(self.has_own_property(&multiname))
    }

    // ECMA-357 9.1.1.2 [[Put]] (P, V)
    fn set_property_local(
        self,
        name: &Multiname<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        let name = handle_input_multiname(name.clone(), activation);

        // 1. If ToString(ToUint32(P)) == P, throw a TypeError exception
        if let Some(local_name) = name.local_name() {
            if local_name.parse::<usize>().is_ok() {
                return Err(make_error_1087(activation));
            }
        }

        // 2. If x.[[Class]] ∈ {"text", "comment", "processing-instruction", "attribute"}, return
        if !self.node().is_element() {
            return Ok(());
        }

        // 3. If (Type(V) ∉ {XML, XMLList}) or (V.[[Class]] ∈ {"text", "attribute"})
        // 3.a. Let c = ToString(V)
        // 4. Else
        // 4.a. Let c be the result of calling the [[DeepCopy]] method of V
        let value = if let Some(xml) = value.as_object().and_then(|x| x.as_xml_object()) {
            // NOTE: avmplus contrary to specification doesn't consider CData here.
            if matches!(
                *xml.node().kind(),
                E4XNodeKind::Attribute(_) | E4XNodeKind::Text(_)
            ) {
                Value::String(value.coerce_to_string(activation)?)
            } else {
                xml.deep_copy(activation).into()
            }
        } else if let Some(list) = value.as_object().and_then(|x| x.as_xml_list_object()) {
            if list.length() == 1 {
                let xml = list
                    .xml_object_child(0, activation)
                    .expect("List length was just verified");

                if matches!(
                    *xml.node().kind(),
                    E4XNodeKind::Attribute(_) | E4XNodeKind::Text(_)
                ) {
                    value.coerce_to_string(activation)?.into()
                } else {
                    list.deep_copy(activation).into()
                }
            } else {
                list.deep_copy(activation).into()
            }
        } else {
            value.coerce_to_string(activation)?.into()
        };

        // 5. Let n = ToXMLName(P)
        // 6. If Type(n) is AttributeName
        if name.is_attribute() {
            // 6.b. If Type(c) is XMLList
            let value = if let Some(list) = value.as_object().and_then(|x| x.as_xml_list_object()) {
                let mut out = WString::new();

                // 6.b.i. If c.[[Length]] == 0, let c be the empty string, NOTE: String is already empty, no case needed.
                // 6.b.ii. Else
                if list.length() != 0 {
                    // 6.b.ii.1. Let s = ToString(c[0])
                    out.push_str(
                        list.children()[0]
                            .node()
                            .xml_to_string(activation)
                            .as_wstr(),
                    );

                    // 6.b.ii.2. For i = 1 to c.[[Length]]-1
                    for child in list.children().iter().skip(1) {
                        // 6.b.ii.2.a. Let s be the result of concatenating s, the string " " (space) and ToString(c[i])
                        out.push_char(' ');
                        out.push_str(child.node().xml_to_string(activation).as_wstr())
                    }
                }

                AvmString::new(activation.gc(), out)
            // 6.c. Else
            } else {
                value.coerce_to_string(activation)?
            };

            let mc = activation.gc();
            self.delete_property_local(activation, &name)?;
            let Some(local_name) = name.local_name() else {
                return Err(format!("Cannot set attribute {name:?} without a local name").into());
            };
            let ns = name.explicit_namespace().map(E4XNamespace::new_uri);
            let new_attr = E4XNode::attribute(mc, ns, local_name, value, Some(self.node()));

            let node = self.0.node.get();
            let mut kind = node.kind_mut(mc);
            let E4XNodeKind::Element { attributes, .. } = &mut *kind else {
                return Ok(());
            };

            attributes.push(new_attr);
            return Ok(());
        }

        // 7. Let isValidName be the result of calling the function isXMLName (section 13.1.2.1) with argument n
        let is_valid_name = name
            .local_name()
            .map(crate::avm2::e4x::is_xml_name)
            .unwrap_or(false);
        // 8. If isValidName is false and n.localName is not equal to the string "*", return
        if !is_valid_name && !name.is_any_name() {
            return Ok(());
        }

        // 10. Let primitiveAssign = (Type(c) ∉ {XML, XMLList}) and (n.localName is not equal to the string "*")
        let primitive_assign = !value
            .as_object()
            .is_some_and(|x| x.as_xml_list_object().is_some() || x.as_xml_object().is_some())
            && !name.is_any_name();

        let self_node = self.node();

        // 9. Let i = undefined
        // 11.
        let index = self_node.remove_matching_children(activation.gc(), &name);

        let index = if let Some((index, node)) = index {
            self_node.insert_at(activation.gc(), index, node);
            index
        // 12. If i == undefined
        } else {
            // 12.a. Let i = x.[[Length]]
            let index = self_node.length().expect("Node should be of element kind");
            self_node.insert_at(activation.gc(), index, E4XNode::dummy(activation.gc()));

            // 12.b. If (primitiveAssign == true)
            if primitive_assign {
                // 12.b.i. If (n.uri == null)
                // 12.b.i.1. Let name be a new QName created as if by calling the constructor new
                //           QName(GetDefaultNamespace(), n)
                // 12.b.ii. Else
                // 12.b.ii.1. Let name be a new QName created as if by calling the constructor new QName(n)

                // 12.b.iii. Create a new XML object y with y.[[Name]] = name, y.[[Class]] = "element" and y.[[Parent]] = x
                let node = E4XNode::element(
                    activation.gc(),
                    name.explicit_namespace().map(E4XNamespace::new_uri),
                    name.local_name().unwrap(),
                    Some(self_node),
                );
                // 12.b.v. Call the [[Replace]] method of x with arguments ToString(i) and y
                self_node.replace(index, XmlObject::new(node, activation).into(), activation)?;
                // FIXME: 12.b.iv. Let ns be the result of calling [[GetNamespace]] on name with no arguments
                // 12.b.vi. Call [[AddInScopeNamespace]] on y with argument ns
            }

            index
        };

        // 13. If (primitiveAssign == true)
        if primitive_assign {
            let E4XNodeKind::Element { children, .. } = &mut *self_node.kind_mut(activation.gc())
            else {
                unreachable!("Node should be of Element kind");
            };

            // 13.a. Delete all the properties of the XML object x[i]
            children[index].remove_all_children(activation.gc());

            // 13.b. Let s = ToString(c)
            let val = value.coerce_to_string(activation)?;

            // 13.c. If s is not the empty string, call the [[Replace]] method of x[i] with arguments "0" and s
            if !val.is_empty() {
                children[index].replace(0, value, activation)?;
            }
        // 14. Else
        } else {
            // 14.a. Call the [[Replace]] method of x with arguments ToString(i) and c
            self_node.replace(index, value, activation)?;
        }

        // 15. Return
        Ok(())
    }

    fn get_next_enumerant(
        self,
        last_index: u32,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<u32, Error<'gc>> {
        Ok(if last_index == 0 { 1 } else { 0 })
    }

    fn get_enumerant_value(
        self,
        index: u32,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        if index == 1 {
            Ok(self.into())
        } else {
            Ok(Value::Undefined)
        }
    }

    fn get_enumerant_name(
        self,
        index: u32,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        if index == 1 {
            Ok(0.into())
        } else {
            Ok(Value::Null)
        }
    }

    // ECMA-357 9.1.1.3 [[Delete]] (P)
    fn delete_property_local(
        self,
        activation: &mut Activation<'_, 'gc>,
        name: &Multiname<'gc>,
    ) -> Result<bool, Error<'gc>> {
        let name = handle_input_multiname(name.clone(), activation);

        // 1. If ToString(ToUint32(P)) == P, throw a TypeError exception
        // NOTE: This doesn't actually throw in Flash.
        if let Some(local_name) = name.local_name() {
            if local_name.parse::<usize>().is_ok() {
                return Ok(true);
            }
        }

        let node = self.0.node.get();

        // 2. Let n = ToXMLName(P)

        // 3. If Type(n) is AttributeName
        if name.is_attribute() {
            let E4XNodeKind::Element { attributes, .. } = &mut *node.kind_mut(activation.gc())
            else {
                return Ok(true);
            };

            // 3.a. For each a in x.[[Attributes]]
            attributes.retain(|attr| {
                // 3.a.i. If ((n.[[Name]].localName == "*") or
                //            (n.[[Name]].localName == a.[[Name]].localName))
                // and ((n.[[Name]].uri == null) or (n.[[Name]].uri == a.[[Name]].uri))
                if attr.matches_name(&name) {
                    // 3.a.i.1. Let a.[[Parent]] = null
                    attr.set_parent(None, activation.gc());
                    // 3.a.i.2. Remove the attribute a from x.[[Attributes]]
                    false
                } else {
                    true
                }
            });

            // 3.b. Return true
            return Ok(true);
        }

        let E4XNodeKind::Element { children, .. } = &mut *node.kind_mut(activation.gc()) else {
            return Ok(true);
        };

        // 4. Let dp = 0
        // 5. For q = 0 to x.[[Length]]-1
        children.retain(|child| {
            // 5.a. If ((n.localName == "*")
            //   or (x[q].[[Class]] == "element" and x[q].[[Name]].localName == n.localName))
            //   and ((n.uri == null) or (x[q].[[Class]] == “element” and n.uri == x[q].[[Name]].uri ))
            let should_retain = if name.is_any_name() {
                false
            } else if child.is_element() {
                !child.matches_name(&name)
            } else {
                true
            };

            if !should_retain {
                child.set_parent(None, activation.gc());
            }

            should_retain
        });

        // 6. Let x.[[Length]] = x.[[Length]] - dp
        // 7. Return true.
        Ok(true)
    }
}

fn handle_input_multiname<'gc>(
    name: Multiname<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Multiname<'gc> {
    // Special case to handle code like: xml["@attr"]
    // FIXME: Figure out the exact semantics.
    // NOTE: It is very important the code within the if-statement is not run
    // when the passed name has the Any namespace. Otherwise, we run the risk of
    // creating a NamespaceSet::Multiple with an Any namespace in it.
    if !name.has_explicit_namespace()
        && !name.is_attribute()
        && !name.is_any_name()
        && !name.is_any_namespace()
    {
        if let Some(mut new_name) = name
            .local_name()
            .map(|name| string_to_multiname(activation, name))
        {
            // Copy the namespaces from the previous name,
            // but make sure to definitely include the public namespace.
            if !new_name.is_any_namespace() {
                let mut ns = Vec::new();
                ns.extend(name.namespace_set());
                if !name.contains_public_namespace() {
                    ns.push(activation.avm2().namespaces.public_all());
                }
                new_name.set_ns(NamespaceSet::new(ns, activation.gc()));
            }

            return new_name;
        }
    }

    name
}
