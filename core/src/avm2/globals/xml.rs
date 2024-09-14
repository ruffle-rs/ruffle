//! XML builtin and prototype

use crate::avm2::e4x::{name_to_multiname, E4XNamespace, E4XNode, E4XNodeKind};
use crate::avm2::error::{make_error_1117, type_error};
pub use crate::avm2::object::xml_allocator;
use crate::avm2::object::{E4XOrXml, QNameObject, TObject, XmlListObject, XmlObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::string::AvmString;
use crate::avm2::{Activation, ArrayObject, ArrayStorage, Error, Multiname, Object, Value};
use crate::avm2_stub_method;

fn ill_formed_markup_err<'gc>(
    activation: &mut Activation<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    type_error(
        activation,
        "Error #1088: The markup in the document following the root element must be well-formed.",
        1088,
    )
}

pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_xml_object().unwrap();
    let value = args[0];
    let ignore_comments = args.get_bool(1);
    let ignore_processing_instructions = args.get_bool(2);
    let ignore_whitespace = args.get_bool(3);

    if let Some(obj) = value.as_object() {
        if let Some(xml_list) = obj.as_xml_list_object() {
            // Note - 'new XML(new XMLList())' throws an error, even though
            // 'new XML("")' does not. We need this special case to ensure that we return
            // an error, since E4XNode::parse would otherwise return an empty array
            // (which would be accepted)
            if xml_list.length() != 1 {
                return Err(Error::AvmError(ill_formed_markup_err(activation)?));
            }
        }
    }

    let nodes = E4XNode::parse(
        value,
        activation,
        ignore_comments,
        ignore_processing_instructions,
        ignore_whitespace,
    )?;

    let node = match nodes.as_slice() {
        // XML defaults to an empty text node when nothing was parsed
        [] => E4XNode::text(activation.context.gc_context, AvmString::default(), None),
        [node] => *node,
        nodes => {
            let mut single_element_node = None;
            for node in nodes {
                match &*node.kind() {
                    E4XNodeKind::CData(_)
                    | E4XNodeKind::Comment(_)
                    | E4XNodeKind::ProcessingInstruction(_) => {}
                    E4XNodeKind::Text(text) => {
                        let mut chars = text.chars();
                        let is_whitespace_text = chars.all(|c| {
                            if let Ok(c) = c {
                                matches!(c, '\t' | '\n' | '\r' | ' ')
                            } else {
                                false
                            }
                        });

                        if !is_whitespace_text {
                            single_element_node = None;
                            break;
                        }
                    }
                    E4XNodeKind::Element { .. } => {
                        if single_element_node.is_none() {
                            single_element_node = Some(node);
                        } else {
                            single_element_node = None;
                            break;
                        }
                    }
                    E4XNodeKind::Attribute(_) => unreachable!(),
                }
            }

            if let Some(element) = single_element_node {
                *element
            } else {
                return Err(Error::AvmError(ill_formed_markup_err(activation)?));
            }
        }
    };
    this.set_node(activation.context.gc_context, node);

    Ok(Value::Undefined)
}

pub fn normalize<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    xml.node().normalize(activation.gc());
    Ok(xml.into())
}

pub fn name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();

    if let Some(local_name) = xml.local_name() {
        let namespace = xml.namespace_object(activation, &[])?.namespace();
        let mut multiname = Multiname::new(namespace, local_name);
        multiname.set_is_attribute(xml.node().is_attribute());
        Ok(QNameObject::from_name(activation, multiname)?.into())
    } else {
        Ok(Value::Null)
    }
}

// ECMA-357 13.4.4.35 XML.prototype.setName (name)
pub fn set_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let node = xml.node();

    // 1. If x.[[Class]] ∈ {"text", "comment"}, return
    if node.is_text() || node.is_comment() {
        return Ok(Value::Undefined);
    }

    let name = match args.get_value(0) {
        // 2. If (Type(name) is Object) and (name.[[Class]] == "QName") and (name.uri == null)
        Value::Object(Object::QNameObject(qname)) if qname.uri().is_none() => {
            // a. Let name = name.localName
            qname.local_name().into()
        }
        value => value,
    };

    // 3. Let n be a new QName created if by calling the constructor new QName(name)
    let new_name = activation
        .avm2()
        .classes()
        .qname
        .construct(activation, &[name])?
        .as_qname_object()
        .unwrap();

    // NOTE: avmplus addition
    if !crate::avm2::e4x::is_xml_name(new_name.local_name()) {
        return Err(make_error_1117(activation, new_name.local_name()));
    }

    // 4. If x.[[Class]] == "processing-instruction", let n.uri be the empty string
    // 6. Let ns be a new Namespace created as if by calling the constructor new Namespace(n.prefix, n.uri)
    // TODO: QName doesn't have a prefix
    let ns = if matches!(&*node.kind(), E4XNodeKind::ProcessingInstruction(_)) {
        None
    } else {
        new_name
            .uri()
            .filter(|uri| !uri.is_empty())
            .map(E4XNamespace::new_uri)
    };

    // 5. Let x.[[Name]] = n
    node.set_namespace(ns, activation.gc());
    node.set_local_name(new_name.local_name(), activation.gc());

    // NOTE: avmplus addition
    if let Some(ns) = ns {
        // 7. If x.[[Class]] == "attribute"
        if node.is_attribute() {
            // 7.a. If x.[[Parent]] == null, return
            // 7.b. Call x.[[Parent]].[[AddInScopeNamespace]](ns)
            if let Some(parent) = node.parent() {
                parent.add_in_scope_namespace(activation.gc(), ns);
            }
        }

        // 7. If x.[[Class]] == "element"
        if node.is_element() {
            // 7.a. Call x.[[AddInScopeNamespace]](ns2)
            node.add_in_scope_namespace(activation.gc(), ns);
        }
    }

    Ok(Value::Undefined)
}

// namespace_internal_impl(hasPrefix:Boolean, prefix:String = null):*
pub fn namespace_internal_impl<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let node = xml.node();

    // 1. Let y = x
    // 2. Let inScopeNS = { }
    // 3. While (y is not null)
    //     a. For each ns in y.[[InScopeNamespaces]]
    //     ....
    let in_scope_ns = node.in_scope_namespaces();

    // 4. If prefix was not specified
    if args[0] == Value::Bool(false) {
        // a. If x.[[Class]] ∈ {"text", "comment", "processing-instruction"}, return null
        if matches!(
            &*node.kind(),
            E4XNodeKind::Text(_)
                | E4XNodeKind::CData(_)
                | E4XNodeKind::Comment(_)
                | E4XNodeKind::ProcessingInstruction(_)
        ) {
            return Ok(Value::Null);
        }

        // b. Return the result of calling the [[GetNamespace]] method of x.[[Name]] with argument inScopeNS
        Ok(xml.namespace_object(activation, &in_scope_ns)?.into())
    } else {
        // a. Let prefix = ToString(prefix)
        let prefix = args.get_string(activation, 1)?;

        // b. Find a Namespace ns ∈ inScopeNS, such that ns.prefix = prefix. If no such ns exists, let ns = undefined.
        // c. Return ns
        Ok(
            if let Some(ns) = in_scope_ns.iter().find(|ns| ns.prefix == Some(prefix)) {
                ns.as_namespace_object(activation)?.into()
            } else {
                Value::Undefined
            },
        )
    }
}

// ECMA-357 13.4.4.2 XML.prototype.addNamespace (namespace)
pub fn add_namespace<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let node = xml.node();

    // 1. Let ns a Namespace constructed as if by calling the function Namespace(namespace)
    let value = args.get_value(0);
    let ns = activation
        .avm2()
        .classes()
        .namespace
        .construct(activation, &[value])?
        .as_namespace_object()
        .unwrap();

    // 2. Call the [[AddInScopeNamespace]] method of x with parameter ns
    node.add_in_scope_namespace(
        activation.gc(),
        E4XNamespace {
            prefix: ns.prefix(),
            uri: ns.namespace().as_uri(),
        },
    );

    // 3. Return x
    Ok(this.into())
}

// ECMA-357 13.4.4.36 XML.prototype.setNamespace (ns)
pub fn set_namespace<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let node = xml.node();

    // 1. If x.[[Class]] ∈ {"text", "comment", "processing-instruction"}, return
    if matches!(
        &*node.kind(),
        E4XNodeKind::Text(_)
            | E4XNodeKind::CData(_)
            | E4XNodeKind::Comment(_)
            | E4XNodeKind::ProcessingInstruction(_)
    ) {
        return Ok(Value::Undefined);
    }

    // 2. Let ns2 be a new Namespace created as if by calling the constructor new Namespace(ns)
    let value = args.get_value(0);
    let ns = activation
        .avm2()
        .classes()
        .namespace
        .construct(activation, &[value])?
        .as_namespace_object()
        .unwrap();
    let ns = E4XNamespace {
        prefix: ns.prefix(),
        uri: ns.namespace().as_uri(),
    };

    // 3. Let x.[[Name]] be a new QName created as if by calling the constructor new QName(ns2, x.[[Name]])
    node.set_namespace(Some(ns), activation.gc());

    // 4. If x.[[Class]] == "attribute"
    if node.is_attribute() {
        // 4.a. If x.[[Parent]] == null, return
        // 4.b. Call x.[[Parent]].[[AddInScopeNamespace]](ns2)
        if let Some(parent) = node.parent() {
            parent.add_in_scope_namespace(activation.gc(), ns);
        }
    }

    // 5. If x.[[Class]] == "element"
    if node.is_element() {
        // 5.a. Call x.[[AddInScopeNamespace]](ns2)
        node.add_in_scope_namespace(activation.gc(), ns);
    }

    Ok(Value::Undefined)
}

// ECMA-357 13.4.4.31 XML.prototype.removeNamespace (namespace)
pub fn remove_namespace<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let node = xml.node();

    // 1. If x.[[Class]] ∈ {"text", "comment", "processing-instruction", "attribute"}, return x
    if !node.is_element() {
        return Ok(this.into());
    }

    // 2. Let ns be a Namespace object created as if by calling the function Namespace( namespace )
    let value = args.get_value(0);
    let ns = activation
        .avm2()
        .classes()
        .namespace
        .construct(activation, &[value])?
        .as_namespace_object()
        .unwrap();
    let ns = E4XNamespace {
        prefix: ns.prefix(),
        uri: ns.namespace().as_uri(),
    };

    // 3. Let thisNS be the result of calling [[GetNamespace]] on x.[[Name]] with argument x.[[InScopeNamespaces]]
    let in_scope_ns = node.in_scope_namespaces();
    let this_ns = node.get_namespace(&in_scope_ns);

    // 4. If (thisNS == ns), return x
    if this_ns == ns {
        return Ok(this.into());
    }

    {
        let E4XNodeKind::Element { attributes, .. } = &*node.kind() else {
            unreachable!()
        };

        // 5. For each a in x.[[Attributes]]
        for attr in attributes {
            // 5.a. Let aNS be the result of calling [[GetNamespace]] on a.[[Name]] with argument x.[[InScopeNamespaces]]
            let attr_ns = attr.get_namespace(&in_scope_ns);
            // 5.b. If (aNS == ns), return x
            if attr_ns == ns {
                return Ok(this.into());
            }
        }
    }

    // 6. If ns.prefix == undefined
    if ns.prefix.is_none() {
        let E4XNodeKind::Element {
            ref mut namespaces, ..
        } = &mut *node.kind_mut(activation.gc())
        else {
            unreachable!()
        };
        // 6.a. If there exists a namespace n ∈ x.[[InScopeNamespaces]],
        // such that n.uri == ns.uri, remove the namespace n from x.[[InScopeNamespaces]]
        namespaces.retain(|namespace| namespace.uri != ns.uri);
    } else {
        // 7. Else
        let E4XNodeKind::Element {
            ref mut namespaces, ..
        } = &mut *node.kind_mut(activation.gc())
        else {
            unreachable!()
        };
        // 7.a. If there exists a namespace n ∈ x.[[InScopeNamespaces]],
        // such that n.uri == ns.uri and n.prefix == ns.prefix, remove the namespace n from x.[[InScopeNamespaces]]
        namespaces.retain(|namespace| *namespace != ns);
    }

    let E4XNodeKind::Element { children, .. } = &*node.kind() else {
        unreachable!()
    };
    // 8. For each property p of x
    for child in children {
        // 8.a. If p.[[Class]] = "element", call the removeNamespace method of p with argument ns
        if child.is_element() {
            let xml = E4XOrXml::E4X(*child).get_or_create_xml(activation);
            remove_namespace(activation, xml.into(), args)?;
        }
    }

    // 9. Return x
    Ok(this.into())
}

// ECMA-357 13.4.4.17 XML.prototype.inScopeNamespaces ()
pub fn in_scope_namespaces<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let node = xml.node();

    // 1. Let y = x
    // 2. Let inScopeNS = { }
    // 3. While (y is not null)
    // ...
    let mut in_scope_ns: Vec<Value<'gc>> = Vec::new();
    for ns in node.in_scope_namespaces() {
        in_scope_ns.push(ns.as_namespace_object(activation)?.into());
    }

    // Note: Non-standard avmplus behavior doesn't allow an empty array.
    if in_scope_ns.is_empty() {
        in_scope_ns.push(
            E4XNamespace::default_namespace()
                .as_namespace_object(activation)?
                .into(),
        );
    }

    // 4. Let a be a new Array created as if by calling the constructor, new Array()
    // ...
    // 7. Return a
    Ok(ArrayObject::from_storage(activation, ArrayStorage::from_iter(in_scope_ns))?.into())
}

pub fn namespace_declarations<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let node = xml.node();

    // 1. Let a be a new Array created as if by calling the constructor, new Array()
    // 2. If x.[[Class]] ∈ {"text", "comment", "processing-instruction", "attribute"}, return a
    if !node.is_element() {
        return Ok(ArrayObject::empty(activation)?.into());
    }

    // 3. Let y = x.[[Parent]]
    // 4. Let ancestorNS = { }
    // 5. While (y is not null)
    // ....
    // Note: in_scope_namespaces implements the whole loop
    let ancestor_namespaces = node
        .parent()
        .map(|parent| parent.in_scope_namespaces())
        .unwrap_or_default();

    // 6. Let declaredNS = { }
    let mut declared_namespaces: Vec<Value<'gc>> = Vec::new();

    // 7. For each ns in x.[[InScopeNamespaces]]
    for ns in node.in_scope_namespaces() {
        // 7.a. If there exists no n ∈ ancestorNS, such that n.prefix == ns.prefix and n.uri == ns.uri
        if !ancestor_namespaces.contains(&ns) {
            // 7.a.i. Let declaredNS = declaredNS ∪ { ns }
            declared_namespaces.push(ns.as_namespace_object(activation)?.into());
        }
    }

    // 8. Let i = 0
    // 9. For each ns in declaredNS
    // 9.a. Call the [[Put]] method of a with arguments ToString(i) and ns
    // 9.b. Let i = i + 1
    // 10. Return a
    Ok(ArrayObject::from_storage(activation, ArrayStorage::from_iter(declared_namespaces))?.into())
}

pub fn local_name<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let node = this.as_xml_object().unwrap();
    Ok(node.local_name().map_or(Value::Null, Value::String))
}

pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let node = xml.node();
    Ok(Value::String(node.xml_to_string(activation)))
}

pub fn to_xml_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    Ok(xml.as_xml_string(activation).into())
}

pub fn child<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let multiname = name_to_multiname(activation, &args[0], false)?;

    let list = xml.child(&multiname, activation);
    Ok(list.into())
}

pub fn child_index<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let node = xml.node();

    Ok(node
        .child_index()
        .map(|x| Value::Number(x as f64))
        .unwrap_or(Value::Number(-1.0)))
}

pub fn children<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let children = if let E4XNodeKind::Element { children, .. } = &*xml.node().kind() {
        children.iter().map(|node| E4XOrXml::E4X(*node)).collect()
    } else {
        Vec::new()
    };

    // FIXME: Spec says to just call [[Get]] with * (any multiname).
    Ok(XmlListObject::new_with_children(
        activation,
        children,
        Some(xml.into()),
        Some(Multiname::any()),
    )
    .into())
}

pub fn contains<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let value = args.get_value(0);

    if let Some(other) = value.as_object().and_then(|obj| obj.as_xml_object()) {
        let result = xml.node().equals(&other.node());
        return Ok(result.into());
    }
    Ok(false.into())
}

pub fn copy<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    Ok(xml.deep_copy(activation).into())
}

pub fn parent<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let node = xml.node();
    Ok(node.parent().map_or(Value::Undefined, |parent| {
        XmlObject::new(parent, activation).into()
    }))
}

pub fn elements<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let multiname = name_to_multiname(activation, &args[0], false)?;

    let list = xml.elements(&multiname, activation);
    Ok(list.into())
}

pub fn attributes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let attributes = if let E4XNodeKind::Element { attributes, .. } = &*xml.node().kind() {
        attributes.iter().map(|node| E4XOrXml::E4X(*node)).collect()
    } else {
        Vec::new()
    };

    // FIXME: Spec/avmplus says to call [[Get]] with * attribute name (any attribute multiname).
    Ok(XmlListObject::new_with_children(
        activation,
        attributes,
        Some(xml.into()),
        Some(Multiname::any_attribute()),
    )
    .into())
}

pub fn attribute<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let multiname = name_to_multiname(activation, &args[0], true)?;
    let attributes = if let E4XNodeKind::Element { attributes, .. } = &*xml.node().kind() {
        attributes
            .iter()
            .filter(|node| node.matches_name(&multiname))
            .map(|node| E4XOrXml::E4X(*node))
            .collect()
    } else {
        Vec::new()
    };

    // FIXME: Spec/avmplus call [[Get]] with attribute name.
    Ok(
        XmlListObject::new_with_children(activation, attributes, Some(xml.into()), Some(multiname))
            .into(),
    )
}

pub fn call_handler<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() == 1 {
        if let Some(obj) = args.try_get_object(activation, 0) {
            // We do *not* create a new object when AS does 'XML(someXML)'
            if let Some(xml) = obj.as_xml_object() {
                return Ok(xml.into());
            }
            // This re-uses the XML object stored in the list
            if let Some(xml_list) = obj.as_xml_list_object() {
                if xml_list.length() == 1 {
                    return Ok(xml_list.children_mut(activation.context.gc_context)[0]
                        .get_or_create_xml(activation)
                        .into());
                }
                return Err(Error::AvmError(ill_formed_markup_err(activation)?));
            }
        }
    }

    Ok(activation
        .avm2()
        .classes()
        .xml
        .construct(activation, args)?
        .into())
}

pub fn node_kind<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let name = match &*xml.node().kind() {
        E4XNodeKind::Text(_) => "text",
        E4XNodeKind::CData(_) => "text", // cdata pretends to be text here
        E4XNodeKind::Comment(_) => "comment",
        E4XNodeKind::ProcessingInstruction(_) => "processing-instruction",
        E4XNodeKind::Attribute(_) => "attribute",
        E4XNodeKind::Element { .. } => "element",
    };
    Ok(name.into())
}

pub fn append_child<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let child = args.get_value(0);
    let child = crate::avm2::e4x::maybe_escape_child(activation, child)?;

    // 1. Let children be the result of calling the [[Get]] method of x with argument "*"
    let name = Multiname::any();
    let children = xml.get_property_local(&name, activation)?;

    // 2. Call the [[Put]] method of children with arguments children.[[Length]] and child
    let xml_list = children
        .as_object()
        .and_then(|o| o.as_xml_list_object())
        .expect("Should have an XMLList");
    let length = xml_list.length();
    let name = Multiname::new(
        activation.avm2().public_namespace_base_version,
        AvmString::new_utf8(activation.context.gc_context, length.to_string()),
    );
    xml_list.set_property_local(&name, child, activation)?;

    // 3. Return x
    Ok(this.into())
}

// ECMA-357 13.4.4.29 XML.prototype.prependChild ( value )
pub fn prepend_child<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let child = args.get_value(0);
    let child = crate::avm2::e4x::maybe_escape_child(activation, child)?;

    // 1. Call the [[Insert]] method of this object with arguments "0" and value
    xml.node().insert(0, child, activation)?;

    // 2. Return x
    Ok(xml.into())
}

pub fn descendants<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let multiname = name_to_multiname(activation, &args[0], false)?;

    // 2. Return the result of calling the [[Descendants]] method of x with argument name
    Ok(xml
        .xml_descendants(activation, &multiname)
        .expect("XmlObject always returns a XmlListObject here")
        .into())
}

// ECMA-357 13.4.4.37 XML.prototype.text ( )
pub fn text<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let nodes = if let E4XNodeKind::Element { children, .. } = &*xml.node().kind() {
        children
            .iter()
            .filter(|node| node.is_text())
            .map(|node| E4XOrXml::E4X(*node))
            .collect()
    } else {
        Vec::new()
    };

    // 1. Let list be a new XMLList with list.[[TargetObject]] = x and list.[[TargetProperty]] = null
    let list = XmlListObject::new_with_children(activation, nodes, Some(xml.into()), None);

    if list.length() > 0 {
        // NOTE: Since avmplus uses appendNode to build the list here, we need to set target dirty flag.
        list.set_dirty_flag();
    }

    // 3. Return list
    Ok(list.into())
}

pub fn length<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Integer(1))
}

pub fn has_complex_content<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml_obj = this.as_xml_object().unwrap();
    let result = xml_obj.node().has_complex_content();
    Ok(result.into())
}

pub fn has_simple_content<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml_obj = this.as_xml_object().unwrap();
    let result = xml_obj.node().has_simple_content();
    Ok(result.into())
}

// ECMA-357 13.4.4.9 XML.prototype.comments ( )
pub fn comments<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let comments = if let E4XNodeKind::Element { children, .. } = &*xml.node().kind() {
        children
            .iter()
            .filter(|node| matches!(&*node.kind(), E4XNodeKind::Comment(_)))
            .map(|node| E4XOrXml::E4X(*node))
            .collect()
    } else {
        Vec::new()
    };

    // 1. Let list be a new XMLList with list.[[TargetObject]] = x and list.[[TargetProperty]] = null
    let list = XmlListObject::new_with_children(activation, comments, Some(xml.into()), None);

    if list.length() > 0 {
        // NOTE: Since avmplus uses appendNode to build the list here, we need to set target dirty flag.
        list.set_dirty_flag();
    }

    // 3. Return list
    Ok(list.into())
}

// ECMA-357 13.4.4.28 XML.prototype.processingInstructions ( [ name ] )
pub fn processing_instructions<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let multiname = name_to_multiname(activation, &args[0], false)?;
    let nodes = if let E4XNodeKind::Element { children, .. } = &*xml.node().kind() {
        children
            .iter()
            .filter(|node| {
                matches!(&*node.kind(), E4XNodeKind::ProcessingInstruction(_))
                    && node.matches_name(&multiname)
            })
            .map(|node| E4XOrXml::E4X(*node))
            .collect()
    } else {
        Vec::new()
    };

    // 3. Let list = a new XMLList with list.[[TargetObject]] = x and list.[[TargetProperty]] = null
    let list = XmlListObject::new_with_children(activation, nodes, Some(xml.into()), None);

    if list.length() > 0 {
        // NOTE: Since avmplus uses appendNode to build the list here, we need to set target dirty flag.
        list.set_dirty_flag();
    }

    // 5. Return list
    Ok(list.into())
}

// ECMA-357 13.4.4.18 XML.prototype.insertChildAfter (child1, child2)
pub fn insert_child_after<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let child1 = args.get_value(0);
    let child2 = args.get_value(1);
    let child2 = crate::avm2::e4x::maybe_escape_child(activation, child2)?;

    // 1. If x.[[Class]] ∈ {"text", "comment", "processing-instruction", "attribute"}, return
    if !xml.node().is_element() {
        return Ok(Value::Undefined);
    }

    // 3. Else if Type(child1) is XML
    if let Some(child1) = child1.as_object().and_then(|x| {
        if let Some(xml) = x.as_xml_object() {
            return Some(xml.node());
        // NOTE: Non-standard avmplus behavior, single element XMLLists are treated as XML objects.
        } else if let Some(list) = x.as_xml_list_object() {
            if list.length() == 1 {
                return Some(list.children()[0].node());
            }
        }

        None
    }) {
        // NOTE: We fetch the index separately to avoid borrowing errors.
        let index = if let E4XNodeKind::Element { children, .. } = &*xml.node().kind() {
            // 3.a. For i = 0 to x.[[Length]]-1
            // 3.a.i. If x[i] is the same object as child1
            children.iter().position(|x| E4XNode::ptr_eq(*x, child1))
        } else {
            None
        };

        if let Some(index) = index {
            // 3.a.i.1. Call the [[Insert]] method of x with arguments ToString(i + 1) and child2
            xml.node().insert(index + 1, child2, activation)?;
            // 3.a.i.2. Return x
            return Ok(xml.into());
        }
    // 2. If (child1 == null)
    } else if matches!(child1, Value::Null) {
        // 2.a. Call the [[Insert]] method of x with arguments "0" and child2
        xml.node().insert(0, child2, activation)?;
        // 2.b. Return x
        return Ok(xml.into());
    }

    // 4. Return
    Ok(Value::Undefined)
}

// ECMA-357 13.4.4.19 XML.prototype.insertChildBefore (child1, child2)
pub fn insert_child_before<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let child1 = args.get_value(0);
    let child2 = args.get_value(1);
    let child2 = crate::avm2::e4x::maybe_escape_child(activation, child2)?;

    // 1. If x.[[Class]] ∈ {"text", "comment", "processing-instruction", "attribute"}, return
    if !xml.node().is_element() {
        return Ok(Value::Undefined);
    }

    // 3. Else if Type(child1) is XML
    if let Some(child1) = child1.as_object().and_then(|x| {
        if let Some(xml) = x.as_xml_object() {
            return Some(xml.node());
        // NOTE: Non-standard avmplus behavior, single element XMLLists are treated as XML objects.
        } else if let Some(list) = x.as_xml_list_object() {
            if list.length() == 1 {
                return Some(list.children()[0].node());
            }
        }

        None
    }) {
        // NOTE: We fetch the index separately to avoid borrowing errors.
        let index = if let E4XNodeKind::Element { children, .. } = &*xml.node().kind() {
            // 3.a. For i = 0 to x.[[Length]]-1
            // 3.a.i. If x[i] is the same object as child1
            children.iter().position(|x| E4XNode::ptr_eq(*x, child1))
        } else {
            None
        };

        if let Some(index) = index {
            // 3.a.i.1. Call the [[Insert]] method of x with arguments ToString(i) and child2
            xml.node().insert(index, child2, activation)?;
            // 3.a.i.2. Return x
            return Ok(xml.into());
        }
    // 2. If (child1 == null)
    } else if matches!(child1, Value::Null) {
        let length = if let E4XNodeKind::Element { children, .. } = &*xml.node().kind() {
            children.len()
        } else {
            0
        };

        // 2.a. Call the [[Insert]] method of x with arguments ToString(x.[[Length]]) and child2
        xml.node().insert(length, child2, activation)?;
        // 2.b. Return x
        return Ok(xml.into());
    }

    // 4. Return
    Ok(Value::Undefined)
}

// ECMA-357 13.4.4.32 XML.prototype.replace (propertyName, value)
pub fn replace<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let self_node = xml.node();
    let multiname = name_to_multiname(activation, &args[0], false)?;
    let value = args.get_value(1);

    // 1. If x.[[Class]] ∈ {"text", "comment", "processing-instruction", "attribute"}, return x
    if !self_node.is_element() {
        return Ok(xml.into());
    }

    // 2. If Type(value) ∉ {XML, XMLList}, let c = ToString(value)
    // 3. Else let c be the result of calling the [[DeepCopy]] method of value
    let value = if let Some(xml) = value.as_object().and_then(|x| x.as_xml_object()) {
        xml.deep_copy(activation).into()
    } else if let Some(list) = value.as_object().and_then(|x| x.as_xml_list_object()) {
        list.deep_copy(activation).into()
    } else {
        // NOTE: Depends on root swf version.
        // See https://github.com/adobe/avmplus/blob/858d034a3bd3a54d9b70909386435cf4aec81d21/core/XMLObject.cpp#L1540
        if activation.context.swf.version() <= 9 {
            // SWF version 9 edge case, call XML constructor.
            // https://github.com/adobe/avmplus/blob/858d034a3bd3a54d9b70909386435cf4aec81d21/core/XMLObject.cpp#L2241-L2242
            activation
                .avm2()
                .classes()
                .xml
                .construct(activation, &[value])?
                .into()
        } else {
            value
        }
    };

    // 4. If ToString(ToUint32(P)) == P
    if let Some(local_name) = multiname.local_name() {
        if let Ok(index) = local_name.parse::<usize>() {
            // 4.a. Call the [[Replace]] method of x with arguments P and c and return x
            self_node.replace(index, value, activation)?;
            return Ok(xml.into());
        }
    }

    // 5. Let n be a QName object created as if by calling the function QName(P)

    // NOTE: Since this part of the E4X spec is annoying to implement in Rust without borrow errors, we do it a bit differently.
    //       1. First we will get the first elements index that matches our multiname.
    //       2. Then we will delete all matches.
    //       2. And then we insert a dummy E4XNode at the previously stored index, and use the replace method to correct it.

    let index =
        if let Some((index, _)) = self_node.remove_matching_children(activation.gc(), &multiname) {
            self_node.insert_at(activation.gc(), index, E4XNode::dummy(activation.gc()));
            index
        // 8. If i == undefined, return x
        } else {
            return Ok(xml.into());
        };

    // 9. Call the [[Replace]] method of x with arguments ToString(i) and c
    self_node.replace(index, value, activation)?;

    // 10. Return x
    Ok(xml.into())
}

// ECMA-357 13.4.4.33 XML.prototype.setChildren (value)
pub fn set_children<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let value = args.get_value(0);

    // 1. Call the [[Put]] method of x with arguments "*" and value
    xml.set_property_local(&Multiname::any(), value, activation)?;

    // 2. Return x
    Ok(xml.into())
}

// ECMA-357 13.4.4.34 XML.prototype.setLocalName ( name )
pub fn set_local_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let node = xml.node();
    let name = args.get_value(0);

    // 1. If x.[[Class]] ∈ {"text", "comment"}, return
    if node.is_text() || node.is_comment() {
        return Ok(Value::Undefined);
    }

    // 2. If (Type(name) is Object) and (name.[[Class]] == "QName")
    let name = if let Some(qname) = name.as_object().and_then(|x| x.as_qname_object()) {
        // 2.a. Let name = name.localName
        qname.local_name()
    // 3. Else
    } else {
        // 3.a. Let name = ToString(name)
        name.coerce_to_string(activation)?
    };

    // NOTE: avmplus check, not in spec.
    if !crate::avm2::e4x::is_xml_name(name) {
        return Err(make_error_1117(activation, name));
    }

    // 4. Let x.[[Name]].localName = name
    node.set_local_name(name, activation.gc());

    Ok(Value::Undefined)
}

pub fn set_notification<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "XML", "setNotification");
    let xml = this.as_xml_object().unwrap();
    let node = xml.node();
    let fun = args.try_get_object(activation, 0);
    node.set_notification(
        fun.and_then(|f| f.as_function_object()),
        activation.context.gc_context,
    );
    Ok(Value::Undefined)
}

pub fn notification<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let node = xml.node();
    Ok(node.notification().map_or(Value::Null, |fun| fun.into()))
}
