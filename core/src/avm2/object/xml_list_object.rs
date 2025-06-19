use crate::avm2::activation::Activation;
use crate::avm2::api_version::ApiVersion;
use crate::avm2::e4x::{string_to_multiname, E4XNamespace, E4XNode, E4XNodeKind};
use crate::avm2::error::make_error_1089;
use crate::avm2::function::FunctionArgs;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::{Error, Multiname, Namespace};
use crate::string::AvmString;
use crate::utils::HasPrefixField;
use gc_arena::barrier::unlock;
use gc_arena::{
    lock::{Lock, RefLock},
    Collect, Gc, GcWeak, Mutation,
};
use ruffle_macros::istr;
use ruffle_wstr::WString;
use std::cell::{Cell, Ref, RefMut};
use std::fmt::{self, Debug};

use super::{ClassObject, XmlObject};

/// A class instance allocator that allocates XMLList objects.
pub fn xml_list_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(XmlListObject(Gc::new(
        activation.gc(),
        XmlListObjectData {
            base,
            children: RefLock::new(Vec::new()),
            // An XMLList created by 'new XMLList()' is not linked
            // to any object
            target_object: Lock::new(None),
            target_property: RefLock::new(None),
            target_dirty: Cell::new(false),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct XmlListObject<'gc>(pub Gc<'gc, XmlListObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct XmlListObjectWeak<'gc>(pub GcWeak<'gc, XmlListObjectData<'gc>>);

impl Debug for XmlListObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("XmlListObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

impl<'gc> XmlListObject<'gc> {
    pub fn new(
        activation: &mut Activation<'_, 'gc>,
        target_object: Option<XmlOrXmlListObject<'gc>>,
        target_property: Option<Multiname<'gc>>,
    ) -> Self {
        Self::new_with_children(activation, Vec::new(), target_object, target_property)
    }

    pub fn new_with_children(
        activation: &mut Activation<'_, 'gc>,
        children: Vec<E4XOrXml<'gc>>,
        target_object: Option<XmlOrXmlListObject<'gc>>,
        target_property: Option<Multiname<'gc>>,
    ) -> XmlListObject<'gc> {
        let base = ScriptObjectData::new(activation.context.avm2.classes().xml_list);
        XmlListObject(Gc::new(
            activation.gc(),
            XmlListObjectData {
                base,
                children: RefLock::new(children),
                target_object: Lock::new(target_object),
                target_property: RefLock::new(target_property),
                target_dirty: Cell::new(false),
            },
        ))
    }

    pub fn set_dirty_flag(&self) {
        self.0.target_dirty.set(true);
    }

    pub fn length(&self) -> usize {
        self.0.children.borrow().len()
    }

    pub fn xml_object_child(
        &self,
        index: usize,
        activation: &mut Activation<'_, 'gc>,
    ) -> Option<XmlObject<'gc>> {
        let mut children = self.children_mut(activation.gc());
        if let Some(child) = children.get_mut(index) {
            Some(child.get_or_create_xml(activation))
        } else {
            None
        }
    }

    pub fn node_child(&self, index: usize) -> Option<E4XNode<'gc>> {
        self.0.children.borrow().get(index).map(|x| x.node())
    }

    pub fn children(&self) -> Ref<'_, Vec<E4XOrXml<'gc>>> {
        self.0.children.borrow()
    }

    pub fn children_mut(&self, mc: &Mutation<'gc>) -> RefMut<'_, Vec<E4XOrXml<'gc>>> {
        unlock!(Gc::write(mc, self.0), XmlListObjectData, children).borrow_mut()
    }

    pub fn set_children(&self, mc: &Mutation<'gc>, children: Vec<E4XOrXml<'gc>>) {
        *unlock!(Gc::write(mc, self.0), XmlListObjectData, children).borrow_mut() = children;
    }

    fn target_object(&self) -> Option<XmlOrXmlListObject<'gc>> {
        self.0.target_object.get()
    }

    fn target_property(&self) -> Option<Multiname<'gc>> {
        self.0.target_property.borrow().clone()
    }

    pub fn deep_copy(&self, activation: &mut Activation<'_, 'gc>) -> XmlListObject<'gc> {
        self.reevaluate_target_object(activation);

        let children = self
            .children()
            .iter()
            .map(|child| E4XOrXml::E4X(child.node().deep_copy(activation.gc())))
            .collect();
        XmlListObject::new_with_children(
            activation,
            children,
            self.target_object(),
            self.target_property(),
        )
    }

    pub fn as_xml_string(&self, activation: &mut Activation<'_, 'gc>) -> AvmString<'gc> {
        let children = self.children();
        let mut out = WString::new();
        for (i, child) in children.iter().enumerate() {
            if i != 0 {
                out.push_char('\n');
            }
            out.push_str(child.node().xml_to_xml_string(activation).as_wstr())
        }
        AvmString::new(activation.gc(), out)
    }

    // Based on https://github.com/adobe/avmplus/blob/858d034a3bd3a54d9b70909386435cf4aec81d21/core/XMLListObject.cpp#L621
    pub fn reevaluate_target_object(&self, activation: &mut Activation<'_, 'gc>) {
        if self.0.target_dirty.get() && !self.0.children.borrow().is_empty() {
            let last_node = self
                .0
                .children
                .borrow()
                .last()
                .expect("At least one child exists")
                .node();

            if let Some(parent) = last_node.parent() {
                if let Some(XmlOrXmlListObject::Xml(target_obj)) = self.0.target_object.get() {
                    if !E4XNode::ptr_eq(target_obj.node(), parent) {
                        unlock!(
                            Gc::write(activation.gc(), self.0),
                            XmlListObjectData,
                            target_object
                        )
                        .set(Some(XmlObject::new(parent, activation).into()));
                    }
                }
            } else {
                unlock!(
                    Gc::write(activation.gc(), self.0),
                    XmlListObjectData,
                    target_object
                )
                .set(None);
            }

            if !matches!(*last_node.kind(), E4XNodeKind::ProcessingInstruction(_)) {
                if let Some(name) = last_node.local_name() {
                    let ns = match last_node.namespace() {
                        Some(ns) => Namespace::package(
                            ns.uri,
                            ApiVersion::AllVersions,
                            activation.strings(),
                        ),
                        None => activation.avm2().namespaces.public_all(),
                    };

                    *unlock!(
                        Gc::write(activation.gc(), self.0),
                        XmlListObjectData,
                        target_property
                    )
                    .borrow_mut() = Some(Multiname::new(ns, name));
                }
            }

            self.0.target_dirty.set(false);
        }
    }

    // ECMA-357 9.2.1.6 [[Append]] (V)
    pub fn append(&self, value: Value<'gc>, mc: &Mutation<'gc>) {
        let mut children = self.children_mut(mc);

        // 3. If Type(V) is XMLList,
        if let Some(list) = value.as_object().and_then(|x| x.as_xml_list_object()) {
            self.0.target_dirty.set(false);
            // 3.a. Let x.[[TargetObject]] = V.[[TargetObject]]
            unlock!(Gc::write(mc, self.0), XmlListObjectData, target_object)
                .set(list.target_object());
            // 3.b. Let x.[[TargetProperty]] = V.[[TargetProperty]]
            *unlock!(Gc::write(mc, self.0), XmlListObjectData, target_property).borrow_mut() =
                list.target_property();

            for el in &*list.children() {
                children.push(el.clone());
            }
        }

        if let Some(xml) = value.as_object().and_then(|x| x.as_xml_object()) {
            self.0.target_dirty.set(true);
            children.push(E4XOrXml::Xml(xml));
        }
    }

    // ECMA-357 9.2.1.10 [[ResolveValue]] ( )
    pub fn resolve_value(
        &self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Option<XmlOrXmlListObject<'gc>>, Error<'gc>> {
        // 1. If x.[[Length]] > 0, return x
        if self.length() > 0 {
            Ok(Some(XmlOrXmlListObject::XmlList(*self)))
        // 2. Else
        } else {
            self.reevaluate_target_object(activation);

            // 2.a. If (x.[[TargetObject]] == null)
            let Some(target_object) = self.target_object() else {
                // 2.a.i. Return null
                return Ok(None);
            };
            // or (x.[[TargetProperty]] == null)
            let Some(target_property) = self.target_property() else {
                // 2.a.i. Return null
                return Ok(None);
            };

            // or (type(x.[[TargetProperty]]) is AttributeName) or (x.[[TargetProperty]].localName == "*")
            if target_property.is_attribute() || target_property.is_any_name() {
                // 2.a.i. Return null
                return Ok(None);
            }

            // 2.b. Let base be the result of calling the [[ResolveValue]] method of x.[[TargetObject]] recursively
            let Some(base) = target_object.resolve_value(activation)? else {
                // 2.c. If base == null, return null
                return Ok(None);
            };

            // 2.d. Let target be the result of calling [[Get]] on base with argument x.[[TargetProperty]]
            let Some(target) = base.get_property_local(&target_property, activation)? else {
                // NOTE: Not specified in spec, but avmplus checks if null/undefined was returned, so we do the same, since there is
                //       an invariant in get_property_local of XmlListObject/XmlObject.
                return Ok(None);
            };

            // 2.e. If (target.[[Length]] == 0)
            if target.length().unwrap_or(0) == 0 {
                // 2.e.i. If (Type(base) is XMLList) and (base.[[Length]] > 1), return null
                if let XmlOrXmlListObject::XmlList(x) = &base {
                    if x.length() > 1 {
                        // NOTE: Not mentioned in the spec, but avmplus throws an Error 1089 here.
                        return Err(make_error_1089(activation));
                    }
                }

                // 2.e.ii. Call [[Put]] on base with arguments x.[[TargetProperty]] and the empty string
                base.as_object().set_property_local(
                    &target_property,
                    istr!("").into(),
                    activation,
                )?;

                // 2.e.iii. Let target be the result of calling [[Get]] on base with argument x.[[TargetProperty]]
                return base.get_property_local(&target_property, activation);
            }

            // 2.f. Return target
            Ok(Some(target))
        }
    }

    pub fn equals(
        &self,
        other: &Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<bool, Error<'gc>> {
        if *other == Value::Undefined && self.length() == 0 {
            return Ok(true);
        }

        if let Value::Object(obj) = other {
            if let Some(xml_list_obj) = obj.as_xml_list_object() {
                if self.length() != xml_list_obj.length() {
                    return Ok(false);
                }

                for n in 0..self.length() {
                    let value = xml_list_obj.xml_object_child(n, activation).unwrap().into();
                    if !self
                        .xml_object_child(n, activation)
                        .unwrap()
                        .abstract_eq(&value, activation)?
                    {
                        return Ok(false);
                    }
                }

                return Ok(true);
            }
        }

        if self.length() == 1 {
            return self
                .xml_object_child(0, activation)
                .unwrap()
                .abstract_eq(other, activation);
        }

        Ok(false)
    }
}

#[derive(Clone, Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct XmlListObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// The children stored by this list.
    children: RefLock<Vec<E4XOrXml<'gc>>>,

    /// The XML or XMLList object that this list was created from.
    /// If `Some`, then modifications to this list are reflected
    /// in the original object.
    target_object: Lock<Option<XmlOrXmlListObject<'gc>>>,

    target_property: RefLock<Option<Multiname<'gc>>>,

    target_dirty: Cell<bool>,
}

/// Holds either an `E4XNode` or an `XmlObject`. This can be converted
/// in-place to an `XmlObject` via `get_or_create_xml`.
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

    pub fn node(&self) -> E4XNode<'gc> {
        match self {
            E4XOrXml::E4X(node) => *node,
            E4XOrXml::Xml(xml) => xml.node(),
        }
    }
}

/// Represents either a XmlObject or a XmlListObject. Used
/// for resolving the value of empty XMLLists.
#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub enum XmlOrXmlListObject<'gc> {
    XmlList(XmlListObject<'gc>),
    Xml(XmlObject<'gc>),
}

impl<'gc> XmlOrXmlListObject<'gc> {
    pub fn length(&self) -> Option<usize> {
        match self {
            XmlOrXmlListObject::Xml(x) => x.length(),
            XmlOrXmlListObject::XmlList(x) => Some(x.length()),
        }
    }

    pub fn resolve_value(
        &self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Option<XmlOrXmlListObject<'gc>>, Error<'gc>> {
        match self {
            // NOTE: XmlObjects just resolve to themselves.
            XmlOrXmlListObject::Xml(x) => Ok(Some(XmlOrXmlListObject::Xml(*x))),
            XmlOrXmlListObject::XmlList(x) => x.resolve_value(activation),
        }
    }

    pub fn as_object(&self) -> Object<'gc> {
        match self {
            XmlOrXmlListObject::Xml(x) => Object::XmlObject(*x),
            XmlOrXmlListObject::XmlList(x) => Object::XmlListObject(*x),
        }
    }

    pub fn get_property_local(
        &self,
        name: &Multiname<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Option<XmlOrXmlListObject<'gc>>, Error<'gc>> {
        let value = self.as_object().get_property_local(name, activation)?;

        if let Some(xml) = value.as_object().and_then(|x| x.as_xml_object()) {
            return Ok(Some(XmlOrXmlListObject::Xml(xml)));
        }

        if let Some(list) = value.as_object().and_then(|x| x.as_xml_list_object()) {
            return Ok(Some(XmlOrXmlListObject::XmlList(list)));
        }

        if matches!(value, Value::Null | Value::Undefined) {
            return Ok(None);
        }

        unreachable!(
            "Invalid value {:?}, expected XmlListObject/XmlObject or a null value",
            value
        );
    }
}

impl<'gc> From<XmlListObject<'gc>> for XmlOrXmlListObject<'gc> {
    fn from(value: XmlListObject<'gc>) -> XmlOrXmlListObject<'gc> {
        XmlOrXmlListObject::XmlList(value)
    }
}

impl<'gc> From<XmlObject<'gc>> for XmlOrXmlListObject<'gc> {
    fn from(value: XmlObject<'gc>) -> XmlOrXmlListObject<'gc> {
        XmlOrXmlListObject::Xml(value)
    }
}

impl<'gc> TObject<'gc> for XmlListObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn as_xml_list_object(&self) -> Option<Self> {
        Some(*self)
    }

    fn xml_descendants(
        &self,
        activation: &mut Activation<'_, 'gc>,
        multiname: &Multiname<'gc>,
    ) -> Option<XmlListObject<'gc>> {
        let mut descendants = Vec::new();
        for child in self.0.children.borrow().iter() {
            child.node().descendants(multiname, &mut descendants);
        }

        // NOTE: The way avmplus implemented this means we do not need to set target_dirty flag.
        //       avmplus used the _append method which explicitly unsets the target_dirty flag when appending an XMLListObject.
        //       and XMLObject's getDescendants method never returns a XMLList with target object/property set,
        //       so we do not need to do anything special here.
        Some(XmlListObject::new_with_children(
            activation,
            descendants,
            None,
            None,
        ))
    }

    fn get_property_local(
        self,
        name: &Multiname<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let mut children = self.children_mut(activation.gc());

        // 1. If ToString(ToUint32(P)) == P
        if !name.has_explicit_namespace() {
            if let Some(local_name) = name.local_name() {
                if let Ok(index) = local_name.parse::<usize>() {
                    if let Some(child) = children.get_mut(index) {
                        return Ok(Value::Object(child.get_or_create_xml(activation).into()));
                    } else {
                        return Ok(Value::Undefined);
                    }
                }
            }
        }

        // 2. Let list be a new XMLList with list.[[TargetObject]] = x and list.[[TargetProperty]] = P
        let out = XmlListObject::new(activation, Some(self.into()), Some(name.clone()));

        // 3. For i = 0 to x.[[Length]]-1,
        for child in children.iter_mut() {
            let child = child.get_or_create_xml(activation);

            // 3.a. If x[i].[[Class]] == "element",
            if child.node().is_element() {
                // 3.a.i. Let gq be the result of calling the [[Get]] method of x[i] with argument P
                let gq = child.get_property_local(name, activation)?;

                // 3.a.ii. If gq.[[Length]] > 0, call the [[Append]] method of list with argument gq
                if let Some(obj) = gq.as_object() {
                    if let Some(xml) = obj.as_xml_object() {
                        let length = xml.length().unwrap_or(0);
                        if length > 0 {
                            out.append(gq, activation.gc());
                        }
                    } else if let Some(list) = obj.as_xml_list_object() {
                        let length = list.length();
                        if length > 0 {
                            out.append(gq, activation.gc());
                        }
                    }
                }
            }
        }

        // 4. Return list
        Ok(out.into())
    }

    fn call_property_local(
        self,
        multiname: &Multiname<'gc>,
        arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let method = Value::from(self.proto().expect("XMLList missing prototype"))
            .get_property(multiname, activation)?;

        // See https://github.com/adobe/avmplus/blob/858d034a3bd3a54d9b70909386435cf4aec81d21/core/XMLListObject.cpp#L50
        // in avmplus.
        // If we have exactly one child, then we forward the method to the child,
        // so long as none of our *children* have a property matching the method name
        // (it doesn't matter if a child's *name* matches, because XMLList methods work
        //  by running an operation on each child. For example,
        // 'new XMLList('<child attr="Outer"><name attr="Inner"></name</child>').name'
        // gives us back an XMLList with '<name attr=Inner></name>'
        //
        // It seems like it may be unnecessary to check if any of our children contain
        // a property matching the method name:
        // * XMLList defines all of the same methods as XML on its prototype (e.g. 'name', 'nodeType', etc.)
        //   If we're attempting to call one of these XML-related methods, then we'll find it on the prototype
        //   in the above check.
        // * If we're calling a method that *doesn't* exist on the prototype, it must not be an XML-related
        //   method. In that case, the method will only be callable on our XML child if the child has simple
        //   content (as we'll automatically convert it to a String, and call the method on that String).
        // * However, in order for a child to have a property matching the method name, it must be
        //   a non-simple XML object (simple XML objects have no properties to match).
        //
        // Nevertheless, there may be some weird edge case where this actually matters.
        // To be safe, we'll just perform exactly the same check that avmplus does.
        if matches!(method, Value::Undefined) {
            let prop = self.get_property_local(multiname, activation)?;
            if let Some(list) = prop.as_object().and_then(|obj| obj.as_xml_list_object()) {
                if list.length() == 0 && self.length() == 1 {
                    let mut children = self.children_mut(activation.gc());

                    let child = children.first_mut().unwrap().get_or_create_xml(activation);

                    return Value::from(child).call_property(
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
        // 1. If ToString(ToUint32(P)) == P
        if let Some(name) = name.local_name() {
            if let Ok(index) = name.parse::<usize>() {
                // 1.a. Return (ToUint32(P) < x.[[Length]])
                return index < self.length();
            }
        }

        // 2. For i = 0 to x.[[Length]]-1
        // 2.a. If x[i].[[Class]] == "element" and the result of calling the [[HasProperty]] method of x[i] with argument P == true, return true
        // 3. Return false
        self.children().iter().any(|x| {
            let node = x.node();
            node.is_element() && node.has_property(name)
        })
    }

    fn has_own_property_string(
        self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<bool, Error<'gc>> {
        let multiname = string_to_multiname(activation, name);
        Ok(self.has_own_property(&multiname))
    }

    // ECMA-357 9.2.1.2 [[Put]] (P, V)
    fn set_property_local(
        self,
        name: &Multiname<'gc>,
        mut value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        // 1. Let i = ToUint32(P)
        // 2. If ToString(i) == P
        if !name.is_any_name() && !name.is_attribute() {
            if let Some(local_name) = name.local_name() {
                if let Ok(mut index) = local_name.parse::<usize>() {
                    self.reevaluate_target_object(activation);

                    // 2.a. If x.[[TargetObject]] is not null
                    let r = if let Some(target) = self.target_object() {
                        // 2.a.i. Let r be the result of calling the [[ResolveValue]] method of x.[[TargetObject]]
                        let r = target.resolve_value(activation)?;

                        // 2.a.ii. If r == null, return
                        let Some(r) = r else {
                            return Ok(());
                        };

                        Some(r)
                    // 2.b. Else let r = null
                    } else {
                        None
                    };

                    // 2.c. If i is greater than or equal to x.[[Length]]
                    if index >= self.length() {
                        let r = match r {
                            Some(XmlOrXmlListObject::Xml(x)) => Some(x.node()),
                            // 2.c.i. If Type(r) is XMLList
                            Some(XmlOrXmlListObject::XmlList(x)) => {
                                // 2.c.i.1. If r.[[Length]] is not equal to 1, return
                                if x.length() != 1 {
                                    return Ok(());
                                }

                                // 2.c.i.2. Else let r = r[0]
                                Some(x.children()[0].node())
                            }
                            None => None,
                        };

                        // 2.c.ii. If r.[[Class]] is not equal to "element", return
                        if let Some(r) = r {
                            if !r.is_element() {
                                return Ok(());
                            }
                        }

                        // 2.c.iii Create a new XML object y with y.[[Parent]] = r, y.[[Name]] = x.[[TargetProperty]],
                        //         y.[[Attributes]] = {}, y.[[Length]] = 0
                        let y = match self.target_property() {
                            // 2.c.iv. If Type(x.[[TargetProperty]]) is AttributeName
                            Some(x) if x.is_attribute() => {
                                // 2.c.iv.1. Let attributeExists be the result of calling the [[Get]] method of r with argument y.[[Name]]
                                let attribute_exists = XmlObject::new(r.unwrap(), activation)
                                    .get_property_local(&x, activation)?;

                                // 2.c.iv.2. If (attributeExists.[[Length]] > 0), return
                                if let Some(list) = attribute_exists
                                    .as_object()
                                    .and_then(|x| x.as_xml_list_object())
                                {
                                    if list.length() > 0 {
                                        return Ok(());
                                    }
                                }

                                // 2.c.iv.3. Let y.[[Class]] = "attribute"
                                E4XNode::attribute(
                                    activation.gc(),
                                    x.explicit_namespace().map(E4XNamespace::new_uri),
                                    x.local_name().unwrap(),
                                    istr!(""),
                                    r,
                                )
                            }
                            // 2.c.v. Else if x.[[TargetProperty]] == null or x.[[TargetProperty]].localName == "*"
                            // 2.c.v.1. Let y.[[Name]] = null
                            // 2.c.v.2. Let y.[[Class]] = "text"
                            Some(x) if x.is_any_name() => {
                                E4XNode::text(activation.gc(), istr!(""), r)
                            }
                            None => E4XNode::text(activation.gc(), istr!(""), r),
                            // NOTE: avmplus edge case.
                            //       See https://github.com/adobe/avmplus/blob/858d034a3bd3a54d9b70909386435cf4aec81d21/core/XMLListObject.cpp#L297-L300
                            _ if value
                                .as_object()
                                .and_then(|x| x.as_xml_object())
                                .is_some_and(|x| x.node().is_text() || x.node().is_attribute()) =>
                            {
                                E4XNode::text(activation.gc(), istr!(""), r)
                            }

                            // 2.c.vi. Else let y.[[Class]] = "element"
                            Some(property) => E4XNode::element(
                                activation.gc(),
                                property.explicit_namespace().map(E4XNamespace::new_uri),
                                property.local_name().expect("Local name should exist"),
                                r,
                            ),
                        };

                        // 2.c.vii. Let i = x.[[Length]]
                        index = self.length();

                        // 2.c.viii. If (y.[[Class]] is not equal to "attribute")
                        if !y.is_attribute() {
                            // 2.c.viii.1. If r is not null
                            if let Some(r) = r {
                                let j = if let E4XNodeKind::Element { children, .. } = &*r.kind() {
                                    // 2.c.viii.1.a. If (i > 0)
                                    let j = if index > 0 {
                                        // 2.c.viii.1.a.i. Let j = 0
                                        let mut j = 0;

                                        // 2.c.viii.1.a.ii. While (j < r.[[Length]]-1) and (r[j] is not the same object as x[i-1])
                                        while j < children.len() - 1
                                            && !E4XNode::ptr_eq(
                                                children[j],
                                                self.children()[index - 1].node(),
                                            )
                                        {
                                            // 2.c.viii.1.a.ii.1. Let j = j + 1
                                            j += 1;
                                        }

                                        // NOTE: Not listed in spec, but avmplus does this, so we do the same.
                                        j + 1
                                    // 2.c.viii.1.b. Else
                                    } else {
                                        // 2.c.viii.1.b.i. Let j = r.[[Length]]-1
                                        children.len()
                                    };

                                    Some(j)
                                } else {
                                    None
                                };

                                // NOTE: This is to bypass borrow errors.
                                if let Some(j) = j {
                                    // 2.c.viii.1.c. Call the [[Insert]] method of r with arguments ToString(j+1) and y
                                    r.insert(j, XmlObject::new(y, activation).into(), activation)?;
                                }
                            }

                            // 2.c.viii.2. If Type(V) is XML, let y.[[Name]] = V.[[Name]]
                            if let Some(xml) = value.as_object().and_then(|x| x.as_xml_object()) {
                                if let Some(name) = xml.node().local_name() {
                                    y.set_local_name(name, activation.gc());
                                }
                                y.set_namespace(xml.node().namespace(), activation.gc());
                            }

                            // 2.c.viii.3. Else if Type(V) is XMLList, let y.[[Name]] = V.[[TargetProperty]]
                            if let Some(list) =
                                value.as_object().and_then(|x| x.as_xml_list_object())
                            {
                                // Note: Don't set anything when there is no [[TargetProperty]].
                                if let Some(target_property) = list.target_property() {
                                    if let Some(name) = target_property.local_name() {
                                        y.set_local_name(name, activation.gc());
                                    }
                                    if let Some(namespace) = target_property.explicit_namespace() {
                                        y.set_namespace(
                                            Some(E4XNamespace::new_uri(namespace)),
                                            activation.gc(),
                                        );
                                    }
                                }
                            }
                        }

                        // 2.c.ix. Call the [[Append]] method of x with argument y
                        self.append(XmlObject::new(y, activation).into(), activation.gc());
                    }

                    // 2.d. If (Type(V) ∉ {XML, XMLList}) or (V.[[Class]] ∈ {"text", "attribute"}), let V = ToString(V)
                    if let Some(list) = value.as_object().and_then(|x| x.as_xml_list_object()) {
                        if list.length() == 1 {
                            let xml = list
                                .xml_object_child(0, activation)
                                .expect("List length was just verified");

                            // NOTE: avmplus contrary to specification doesn't consider CData here.
                            if matches!(
                                *xml.node().kind(),
                                E4XNodeKind::Attribute(_) | E4XNodeKind::Text(_)
                            ) {
                                value = Value::Object(xml.into())
                                    .coerce_to_string(activation)?
                                    .into();
                            }
                        }
                    } else if let Some(xml) = value.as_object().and_then(|x| x.as_xml_object()) {
                        // NOTE: This also doesn't consider CData.
                        if matches!(
                            *xml.node().kind(),
                            E4XNodeKind::Attribute(_) | E4XNodeKind::Text(_)
                        ) {
                            value = value.coerce_to_string(activation)?.into();
                        }
                    } else {
                        value = value.coerce_to_string(activation)?.into();
                    }

                    // NOTE: Get x[i] for future operations. Also we need to drop ref to the children as we need to borrow as mutable later.
                    let children = self.children();
                    let child = children[index].node();
                    drop(children);

                    // 2.e. If x[i].[[Class]] == "attribute"
                    if child.is_attribute() {
                        // FIXME: We probably need to take the namespace too.
                        // 2.e.i. Let z = ToAttributeName(x[i].[[Name]])
                        let z = Multiname::attribute(
                            activation.avm2().namespaces.public_all(),
                            child.local_name().expect("Attribute should have a name"),
                        );
                        // 2.e.ii. Call the [[Put]] method of x[i].[[Parent]] with arguments z and V
                        if let Some(parent) = child.parent() {
                            let parent = XmlObject::new(parent, activation);
                            parent.set_property_local(&z, value, activation)?;

                            // 2.e.iii. Let attr be the result of calling [[Get]] on x[i].[[Parent]] with argument z
                            let attr = parent
                                .get_property_local(&z, activation)?
                                .as_object()
                                .and_then(|x| x.as_xml_list_object())
                                .expect("XmlObject get_property_local should return XmlListObject");
                            // 2.e.iv. Let x[i] = attr[0]
                            self.children_mut(activation.gc())[index] = attr.children()[0].clone();
                        }
                    // 2.f. Else if Type(V) is XMLList
                    } else if let Some(list) =
                        value.as_object().and_then(|x| x.as_xml_list_object())
                    {
                        // 2.f.i. Create a shallow copy c of V
                        let c = XmlListObject::new_with_children(
                            activation,
                            list.children().clone(),
                            None,
                            None,
                        );
                        // 2.f.ii. Let parent = x[i].[[Parent]]
                        let parent = child.parent();

                        // 2.f.iii. If parent is not null
                        if let Some(parent) = parent {
                            // 2.f.iii.1. Let q be the property of parent, such that parent[q] is the same object as x[i]
                            let q = if let E4XNodeKind::Element { children, .. } = &*parent.kind() {
                                children.iter().position(|x| E4XNode::ptr_eq(*x, child))
                            } else {
                                None
                            };

                            if let Some(q) = q {
                                // 2.f.iii.2. Call the [[Replace]] method of parent with arguments q and c
                                parent.replace(q, c.into(), activation)?;

                                let E4XNodeKind::Element { children, .. } = &*parent.kind() else {
                                    unreachable!()
                                };

                                // 2.f.iii.3. For j = 0 to c.[[Length]]-1
                                for (index, child) in
                                    c.children_mut(activation.gc()).iter_mut().enumerate()
                                {
                                    // 2.f.iii.3.a. Let c[j] = parent[ToUint32(q)+j]
                                    *child = E4XOrXml::E4X(children[q + index]);
                                }
                            }
                        }

                        // 2.f.iv - 2.f.viii.
                        let mut children = self.children_mut(activation.gc());
                        children.remove(index);
                        for (index2, child) in c.children().iter().enumerate() {
                            children.insert(index + index2, child.clone());
                        }
                    // 2.g. Else if (Type(V) is XML) or (x[i].[[Class]] ∈ {"text", "comment", "processing-instruction"})
                    } else if value
                        .as_object()
                        .is_some_and(|x| x.as_xml_object().is_some())
                        || matches!(
                            *child.kind(),
                            E4XNodeKind::Text(_)
                                | E4XNodeKind::Comment(_)
                                | E4XNodeKind::ProcessingInstruction(_)
                                | E4XNodeKind::CData(_)
                        )
                    {
                        // 2.g.i. Let parent = x[i].[[Parent]]
                        let parent = child.parent();

                        // 2.g.ii. If parent is not null
                        if let Some(parent) = parent {
                            // 2.g.ii.1. Let q be the property of parent, such that parent[q] is the same object as x[i]
                            let q = if let E4XNodeKind::Element { children, .. } = &*parent.kind() {
                                children.iter().position(|x| E4XNode::ptr_eq(*x, child))
                            } else {
                                None
                            };

                            if let Some(q) = q {
                                // 2.g.ii.2. Call the [[Replace]] method of parent with arguments q and V
                                parent.replace(q, value, activation)?;

                                let E4XNodeKind::Element { children, .. } = &*parent.kind() else {
                                    unreachable!()
                                };

                                // 2.g.ii.3. Let V = parent[q]
                                value = XmlObject::new(children[q], activation).into();
                            }
                        }

                        let mut children = self.children_mut(activation.gc());
                        // NOTE: Avmplus does not follow the spec here, it instead checks if value is XML
                        //       and sets it, otherwise uses ToXML (our closest equivalent is the XML constructor).
                        if let Some(xml) = value.as_object().and_then(|x| x.as_xml_object()) {
                            children[index] = E4XOrXml::Xml(xml);
                        } else {
                            let xml = activation
                                .avm2()
                                .classes()
                                .xml
                                .construct(activation, &[value])?
                                .as_object()
                                .unwrap()
                                .as_xml_object()
                                .expect("Should be XML Object");
                            children[index] = E4XOrXml::Xml(xml);
                        }
                    // 2.h. Else
                    } else {
                        // 2.h.i. Call the [[Put]] method of x[i] with arguments "*" and V
                        self.xml_object_child(index, activation)
                            .unwrap()
                            .set_property_local(&Multiname::any(), value, activation)?;
                    }

                    // NOTE: Not specified in the spec, but avmplus returns here, so we do the same.
                    return Ok(());
                }
            }
        }

        // 3. Else if x.[[Length]] is less than or equal to 1
        if self.length() <= 1 {
            // 3.a. If x.[[Length]] == 0
            if self.length() == 0 {
                // 3.a.i. Let r be the result of calling the [[ResolveValue]] method of x
                let r = self.resolve_value(activation)?;

                // 3.a.ii. If (r == null)
                let Some(r) = r else {
                    return Ok(());
                };

                // or (r.[[Length]] is not equal to 1), return
                if r.length().unwrap_or(0) != 1 {
                    return Ok(());
                }

                // 3.a.iii. Call the [[Append]] method of x with argument r
                self.append(r.as_object().into(), activation.gc());
            }

            let mut children = self.children_mut(activation.gc());

            // 3.b. Call the [[Put]] method of x[0] with arguments P and V
            let xml = children.first_mut().unwrap().get_or_create_xml(activation);
            return xml.set_property_local(name, value, activation);
        }

        // 4. Return
        Err(make_error_1089(activation))
    }

    fn get_next_enumerant(
        self,
        last_index: u32,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<u32, Error<'gc>> {
        if (last_index as usize) < self.0.children.borrow().len() {
            return Ok(last_index + 1);
        }

        Ok(0)
    }

    fn get_enumerant_value(
        self,
        index: u32,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let mut children = self.children_mut(activation.gc());
        let children_len = children.len() as u32;

        if children_len >= index {
            Ok(index
                .checked_sub(1)
                .map(|index| {
                    children
                        .get_mut(index as usize)
                        .unwrap()
                        .get_or_create_xml(activation)
                        .into()
                })
                .unwrap_or(Value::Undefined))
        } else {
            Ok(Value::Undefined)
        }
    }

    fn get_enumerant_name(
        self,
        index: u32,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let children_len = self.0.children.borrow().len() as u32;
        if children_len >= index {
            Ok(index
                .checked_sub(1)
                .map(|index| index.into())
                .unwrap_or(Value::Null))
        } else {
            Ok(self
                .base()
                .get_enumerant_name(index - children_len)
                .unwrap_or(Value::Null))
        }
    }

    fn delete_property_local(
        self,
        activation: &mut Activation<'_, 'gc>,
        name: &Multiname<'gc>,
    ) -> Result<bool, Error<'gc>> {
        let mut children = self.children_mut(activation.gc());

        if !name.is_any_name() && !name.is_attribute() {
            if let Some(local_name) = name.local_name() {
                if let Ok(index) = local_name.parse::<usize>() {
                    if index < children.len() {
                        let removed = children.remove(index);
                        let removed_node = removed.node();
                        if let Some(parent) = removed_node.parent() {
                            if removed_node.is_attribute() {
                                parent.remove_attribute(activation.gc(), &removed_node);
                            } else {
                                parent.remove_child(activation.gc(), &removed_node);
                            }
                        }
                    }
                    return Ok(true);
                }
            }
        }

        for child in children.iter_mut() {
            if child.node().is_element() {
                child
                    .get_or_create_xml(activation)
                    .delete_property_local(activation, name)?;
            }
        }

        Ok(true)
    }
}
