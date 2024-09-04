//! XMLList builtin and prototype

pub use crate::avm2::object::xml_list_allocator;
use crate::avm2::{
    e4x::{name_to_multiname, simple_content_to_string, E4XNode, E4XNodeKind},
    error::make_error_1086,
    multiname::Multiname,
    object::{E4XOrXml, XmlListObject, XmlObject},
    parameters::ParametersExt,
    Activation, Error, Object, TObject, Value,
};
use crate::string::AvmString;

fn has_complex_content_inner(children: &[E4XOrXml<'_>]) -> bool {
    match children {
        [] => false,
        [child] => child.node().has_complex_content(),
        _ => children.iter().any(|child| child.node().is_element()),
    }
}

fn has_simple_content_inner(children: &[E4XOrXml<'_>]) -> bool {
    match children {
        [] => true,
        [child] => child.node().has_simple_content(),
        _ => children.iter().all(|child| !child.node().is_element()),
    }
}

pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_xml_list_object().unwrap();
    let value = args[0];
    let ignore_comments = args.get_bool(1);
    let ignore_processing_instructions = args.get_bool(2);
    let ignore_whitespace = args.get_bool(3);

    if let Some(obj) = value.as_object() {
        if let Some(xml) = obj.as_xml_object() {
            // Note - we re-use the XML object that was passed in, which makes
            // `this[0] === xmlObjArg` true.
            // This logic does *not* go in `E4XNode::parse`, as it does not apply
            // to the `XML` constructor: `new XML(xmlObj) === xmlObj` is false.
            this.set_children(activation.context.gc_context, vec![E4XOrXml::Xml(xml)]);
            return Ok(Value::Undefined);
        }
    }

    match E4XNode::parse(
        value,
        activation,
        ignore_comments,
        ignore_processing_instructions,
        ignore_whitespace,
    ) {
        Ok(nodes) => {
            this.set_children(
                activation.context.gc_context,
                nodes.into_iter().map(E4XOrXml::E4X).collect(),
            );
        }
        Err(e) => {
            return Err(Error::RustError(
                format!("Failed to parse XML: {e:?}").into(),
            ))
        }
    }

    Ok(Value::Undefined)
}

pub fn call_handler<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() == 1 {
        // We do *not* create a new object when AS does 'XMLList(someXMLList)'
        if let Some(obj) = args.try_get_object(activation, 0) {
            if let Some(xml_list) = obj.as_xml_list_object() {
                return Ok(xml_list.into());
            }
        }
    }

    Ok(activation
        .avm2()
        .classes()
        .xml_list
        .construct(activation, args)?
        .into())
}

// ECMA-357 13.5.4.11 XMLList.prototype.elements ([name])
pub fn elements<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_xml_list_object().unwrap();
    // 2. Let name = ToXMLName(name)
    let multiname = name_to_multiname(activation, &args[0], false)?;

    // 3. Let m = a new XMLList with m.[[TargetObject]] = list and m.[[TargetProperty]] = name
    let list = XmlListObject::new(activation, Some(this.into()), Some(multiname.clone()));

    // 4. For i = 0 to list.[[Length]]-1
    let mut children = this.children_mut(activation.gc());
    for child in &mut *children {
        // 4.a. If list[i].[[Class]] == "element"
        if child.node().is_element() {
            // 4.a.i. Let r = list[i].elements(name)
            let r = child
                .get_or_create_xml(activation)
                .elements(&multiname, activation);

            // 4.a.ii. If r.[[Length]] > 0, call the [[Append]] method of m with argument r
            if r.length() > 0 {
                list.append(r.into(), activation.gc());
            }
        }
    }

    // 5. Return m
    Ok(list.into())
}

pub fn has_complex_content<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let list = this.as_xml_list_object().unwrap();
    let children = list.children();
    Ok(has_complex_content_inner(&children).into())
}

pub fn has_simple_content<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let list = this.as_xml_list_object().unwrap();
    let children = list.children();
    Ok(has_simple_content_inner(&children).into())
}

pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let list = this.as_xml_list_object().unwrap();
    let children = list.children();
    if has_simple_content_inner(&children) {
        Ok(simple_content_to_string(children.iter().cloned(), activation).into())
    } else {
        to_xml_string(activation, this, args)
    }
}

pub fn to_xml_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let list = this.as_xml_list_object().unwrap();
    Ok(list.as_xml_string(activation).into())
}

pub fn length<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let list = this.as_xml_list_object().unwrap();
    let children = list.children();
    Ok(children.len().into())
}

pub fn child<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_xml_list_object().unwrap();
    let multiname = name_to_multiname(activation, &args[0], false)?;
    let mut children = this.children_mut(activation.gc());

    // 1. Let m be a new XMLList with m.[[TargetObject]] = list
    let list = XmlListObject::new(activation, Some(this.into()), None);

    // 2. For i = 0 to list.[[Length]]-1
    for child in &mut *children {
        // 2.a. Let r = list[i].child(propertyName)
        let child = child.get_or_create_xml(activation);
        let r = child.child(&multiname, activation);
        // 2.b. If r.[[Length]] > 0, call the [[Append]] method of m with argument r
        if r.length() > 0 {
            list.append(r.into(), activation.gc());
        }
    }

    // 3. Return m
    Ok(list.into())
}

pub fn children<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let list = this.as_xml_list_object().unwrap();
    let children = list.children();
    let mut sub_children = Vec::new();
    for child in &*children {
        if let E4XNodeKind::Element { ref children, .. } = &*child.node().kind() {
            sub_children.extend(children.iter().map(|node| E4XOrXml::E4X(*node)));
        }
    }
    // FIXME: This method should just call get_property_local with "*".
    Ok(XmlListObject::new_with_children(
        activation,
        sub_children,
        Some(list.into()),
        Some(Multiname::any()),
    )
    .into())
}

/// 13.5.4.8 XMLList.prototype.contains (value)
pub fn contains<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let list = this.as_xml_list_object().unwrap();
    let value = args.get_value(0);
    let length = list.length();

    // 1. For i = 0 to list.[[Length]]-1
    // NOTE: cannot use children_mut here since the value can be this same list, which causes a panic.
    for index in 0..length {
        let child = list
            .xml_object_child(index, activation)
            .expect("index should be in between 0 and length");

        // 2.a. If the result of the comparison list[i] == value is true, return true
        if child.abstract_eq(&value, activation)? {
            return Ok(true.into());
        }
    }

    // 2. Return false
    Ok(false.into())
}

pub fn copy<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let list = this.as_xml_list_object().unwrap();
    Ok(list.deep_copy(activation).into())
}

pub fn attribute<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let list = this.as_xml_list_object().unwrap();

    let name = args[0];
    let multiname = name_to_multiname(activation, &name, true)?;

    let children = list.children();
    let mut sub_children = Vec::new();
    for child in &*children {
        if let E4XNodeKind::Element { ref attributes, .. } = &*child.node().kind() {
            if let Some(found) = attributes
                .iter()
                .find(|node| node.matches_name(&multiname))
                .copied()
            {
                sub_children.push(E4XOrXml::E4X(found));
            }
        }
    }

    // FIXME: This should just use get_property_local with an attribute Multiname.
    Ok(XmlListObject::new_with_children(
        activation,
        sub_children,
        Some(list.into()),
        Some(multiname),
    )
    .into())
}

pub fn attributes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let list = this.as_xml_list_object().unwrap();

    let mut child_attrs = Vec::new();
    for child in list.children().iter() {
        if let E4XNodeKind::Element { ref attributes, .. } = &*child.node().kind() {
            child_attrs.extend(attributes.iter().map(|node| E4XOrXml::E4X(*node)));
        }
    }

    // FIXME: This should just use get_property_local with an any attribute Multiname.
    Ok(XmlListObject::new_with_children(
        activation,
        child_attrs,
        Some(list.into()),
        Some(Multiname::any_attribute()),
    )
    .into())
}

pub fn descendants<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let multiname = name_to_multiname(activation, &args[0], false)?;
    if let Some(descendants) = this.xml_descendants(activation, &multiname) {
        Ok(descendants.into())
    } else {
        Ok(Value::Undefined)
    }
}

// ECMA-357 13.5.4.20 XMLList.prototype.text ( )
pub fn text<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml_list = this.as_xml_list_object().unwrap();
    let mut nodes = Vec::new();
    for child in xml_list.children().iter() {
        if let E4XNodeKind::Element { ref children, .. } = &*child.node().kind() {
            nodes.extend(
                children
                    .iter()
                    .filter(|node| node.is_text())
                    .map(|node| E4XOrXml::E4X(*node)),
            );
        }
    }
    // FIXME: This should call XmlObject's text() and concat everything together
    //        (Necessary for correct target object/property and dirty flag).
    Ok(XmlListObject::new_with_children(activation, nodes, Some(xml_list.into()), None).into())
}

pub fn comments<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml_list = this.as_xml_list_object().unwrap();
    let mut nodes = Vec::new();
    for child in xml_list.children().iter() {
        if let E4XNodeKind::Element { ref children, .. } = &*child.node().kind() {
            nodes.extend(
                children
                    .iter()
                    .filter(|node| matches!(&*node.kind(), E4XNodeKind::Comment(_)))
                    .map(|node| E4XOrXml::E4X(*node)),
            );
        }
    }

    // FIXME: This should call XmlObject's comments() and concat everything together
    //        (Necessary for correct target object/property and dirty flag).
    Ok(XmlListObject::new_with_children(activation, nodes, Some(xml_list.into()), None).into())
}

// ECMA-357 13.5.4.17 XMLList.prototype.parent ( )
pub fn parent<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let list = this.as_xml_list_object().unwrap();

    // 1. If list.[[Length]] = 0, return undefined
    if list.length() == 0 {
        return Ok(Value::Undefined);
    }

    // 2. Let parent = list[0].[[Parent]]
    let parent = list.children()[0].node().parent();

    // 3. For i = 1 to list.[[Length]]-1, if list[i].[[Parent]] is not equal to parent, return undefined
    for child in list.children().iter().skip(1) {
        let other = child.node().parent();

        match (parent, other) {
            (Some(v1), Some(v2)) if !E4XNode::ptr_eq(v1, v2) => {
                return Ok(Value::Undefined);
            }
            (None, Some(_)) | (Some(_), None) => return Ok(Value::Undefined),
            _ => {}
        }
    }

    // 4. Return parent
    if let Some(parent) = parent {
        Ok(XmlObject::new(parent, activation).into())
    } else {
        Ok(Value::Undefined)
    }
}

pub fn processing_instructions<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml_list = this.as_xml_list_object().unwrap();
    let multiname = name_to_multiname(activation, &args[0], false)?;
    let mut nodes = Vec::new();
    for child in xml_list.children().iter() {
        if let E4XNodeKind::Element { ref children, .. } = &*child.node().kind() {
            nodes.extend(
                children
                    .iter()
                    .filter(|node| {
                        matches!(&*node.kind(), E4XNodeKind::ProcessingInstruction(_))
                            && node.matches_name(&multiname)
                    })
                    .map(|node| E4XOrXml::E4X(*node)),
            );
        }
    }

    // FIXME: This should call XmlObject's processing_instructions() and concat everything together
    //        (Necessary for correct target object/property and dirty flag).
    Ok(XmlListObject::new_with_children(activation, nodes, Some(xml_list.into()), None).into())
}

pub fn normalize<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let namespaces = activation.avm2().namespaces;
    let list = this.as_xml_list_object().unwrap();

    // 1. Let i = 0
    let mut index = 0;

    // 2. While i < list.[[Length]]
    while index < list.length() {
        let child = list
            .node_child(index)
            .expect("index should be between 0 and length");

        // a. If list[i].[[Class]] == "element"
        if child.is_element() {
            // i. Call the normalize method of list[i]
            child.normalize(activation.gc());

            // ii. Let i = i + 1
            index += 1;
        // b. Else if list[i].[[Class]] == "text"
        } else if child.is_text() {
            let should_remove = {
                let (E4XNodeKind::Text(text) | E4XNodeKind::CData(text)) =
                    &mut *child.kind_mut(activation.gc())
                else {
                    unreachable!()
                };

                // i. While ((i+1) < list.[[Length]]) and (list[i + 1].[[Class]] == "text")
                while index + 1 < list.length()
                    && list
                        .node_child(index + 1)
                        .expect("index should be between 0 and length")
                        .is_text()
                {
                    let other = list
                        .node_child(index + 1)
                        .expect("index should be between 0 and length");

                    let (E4XNodeKind::Text(other) | E4XNodeKind::CData(other)) = &*other.kind()
                    else {
                        unreachable!()
                    };

                    // 1. Let list[i].[[Value]] be the result of concatenating list[i].[[Value]] and list[i + 1].[[Value]]
                    *text = AvmString::concat(activation.gc(), *text, *other);

                    // 2. Call the [[Delete]] method of list with argument ToString(i + 1)
                    list.delete_property_local(
                        activation,
                        &Multiname::new(
                            namespaces.public_all(),
                            AvmString::new_utf8(activation.gc(), (index + 1).to_string()),
                        ),
                    )?;
                }

                text.len() == 0
            };

            // ii. If list[i].[[Value]].length == 0
            if should_remove {
                // 1. Call the [[Delete]] method of list with argument ToString(i)
                list.delete_property_local(
                    activation,
                    &Multiname::new(
                        namespaces.public_all(),
                        AvmString::new_utf8(activation.gc(), index.to_string()),
                    ),
                )?;
            // iii. Else
            } else {
                // 1. Let i = i + 1
                index += 1;
            }
        // c. Else
        } else {
            // i. Let i = i + 1
            index += 1;
        }
    }

    // 3. Return list
    Ok(this.into())
}

macro_rules! define_xml_proxy {
    ( $( ($rust_name:ident, $as_name:expr) ),*, ) => {
        $(
            pub fn $rust_name<'gc>(
                activation: &mut Activation<'_, 'gc>,
                this: Object<'gc>,
                args: &[Value<'gc>],
            ) -> Result<Value<'gc>, Error<'gc>> {
                let namespaces = activation.avm2().namespaces;
                let list = this.as_xml_list_object().unwrap();

                let mut children = list.children_mut(activation.context.gc_context);
                match &mut children[..] {
                    [child] => {
                        child
                            .get_or_create_xml(activation)
                            .call_property(&Multiname::new(namespaces.as3, $as_name), args, activation)
                    }
                    _ => Err(make_error_1086(activation, $as_name)),
                }
            }
        )*
    };
}

define_xml_proxy!(
    (add_namespace, "addNamespace"),
    (append_child, "appendChild"),
    (child_index, "childIndex"),
    (in_scope_namespaces, "inScopeNamespaces"),
    (insert_child_after, "insertChildAfter"),
    (insert_child_before, "insertChildBefore"),
    (local_name, "localName"),
    (name, "name"),
    (namespace_declarations, "namespaceDeclarations"),
    (node_kind, "nodeKind"),
    (prepend_child, "prependChild"),
    (remove_namespace, "removeNamespace"),
    (replace, "replace"),
    (set_children, "setChildren"),
    (set_local_name, "setLocalName"),
    (set_name, "setName"),
    (set_namespace, "setNamespace"),
);

// Special case since the XMLObject's method has to know if prefix was passed or not.
// namespace_internal_impl(hasPrefix:Boolean, prefix:String = null):*
pub fn namespace_internal_impl<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let list = this.as_xml_list_object().unwrap();
    let namespaces = activation.avm2().namespaces;
    let mut children = list.children_mut(activation.context.gc_context);

    let args = if args[0] == Value::Bool(true) {
        &args[1..]
    } else {
        &[]
    };

    match &mut children[..] {
        [child] => child.get_or_create_xml(activation).call_property(
            &Multiname::new(namespaces.as3, "namespace"),
            args,
            activation,
        ),
        _ => Err(make_error_1086(activation, "namespace")),
    }
}
