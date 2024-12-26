use crate::avm2::activation::Activation;
use crate::avm2::error::{make_error_1032, make_error_1080};
use crate::avm2::namespace::{CommonNamespaces, Namespace};
use crate::avm2::script::TranslationUnit;
use crate::avm2::Error;
use crate::avm2::QName;
use crate::avm2::{Object, Value};
use crate::string::{AvmString, StringContext, WStr, WString};
use bitflags::bitflags;
use gc_arena::Gc;
use gc_arena::{Collect, Mutation};
use std::fmt::Debug;
use std::ops::Deref;
use swf::avm2::types::{Index, Multiname as AbcMultiname, NamespaceSet as AbcNamespaceSet};

#[derive(Clone, Copy, Debug, Collect)]
#[collect(no_drop)]
pub enum NamespaceSet<'gc> {
    Multiple(Gc<'gc, Vec<Namespace<'gc>>>),
    Single(Namespace<'gc>),
}

impl<'gc> NamespaceSet<'gc> {
    pub fn new(set: Vec<Namespace<'gc>>, mc: &Mutation<'gc>) -> Self {
        if set.len() == 1 {
            NamespaceSet::single(set[0])
        } else {
            NamespaceSet::multiple(set, mc)
        }
    }

    pub fn multiple(set: Vec<Namespace<'gc>>, mc: &Mutation<'gc>) -> Self {
        Self::Multiple(Gc::new(mc, set))
    }
    pub fn single(ns: Namespace<'gc>) -> Self {
        Self::Single(ns)
    }
    pub fn len(&self) -> usize {
        match self {
            Self::Multiple(ns) => ns.len(),
            Self::Single(_) => 1,
        }
    }
    pub fn get(&self, index: usize) -> Option<Namespace<'gc>> {
        match self {
            Self::Multiple(ns) => ns.get(index).copied(),
            Self::Single(ns) => {
                if index == 0 {
                    Some(*ns)
                } else {
                    None
                }
            }
        }
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Default)]
    pub struct MultinameFlags: u8 {
        /// Whether the namespace needs to be read at runtime before use.
        /// This should only be set when lazy-initialized in Activation.
        const HAS_LAZY_NS = 1 << 0;
        /// Whether the name needs to be read at runtime before use
        /// This should only be set when lazy-initialized in Activation.
        const HAS_LAZY_NAME = 1 << 1;
        /// Whether this was a 'MultinameA' - used for XML attribute lookups
        const ATTRIBUTE = 1 << 2;

        /// Represents the XML concept of "qualified name".
        /// This also distinguishes a QName(x, y) from Multiname(x, [y])
        /// Basically, marks multinames that come from multinames of kind `(RT)QName(L)(A)`
        ///   (and dynamically-generated multinames that are supposed to be equivalent to one).
        /// TODO: There are places (getQName()) where FP sets this where we don't have a direct equivalent,
        /// these should probably be audited eventually
        const IS_QNAME = 1 << 3;
    }
}

/// A `Multiname` consists of a name which could be resolved in one or more
/// potential namespaces.
///
/// All unresolved names are of the form `Multiname`, and the name resolution
/// process consists of searching each name space for a given name.
///
/// The existence of a `name` of `None` indicates the `Any` name.
#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct Multiname<'gc> {
    /// The list of namespaces that satisfy this multiname.
    ns: NamespaceSet<'gc>,

    /// The local name that satisfies this multiname. If `None`, then this
    /// multiname is satisfied by any name in the namespace.
    name: Option<AvmString<'gc>>,

    /// The type parameter required to satisfy this multiname. If None, then
    /// this multiname does not have a type parameter. If Some(None), then
    /// this multiname uses the Any type parameter (`*`).
    param: Option<Option<Gc<'gc, Multiname<'gc>>>>,

    #[collect(require_static)]
    flags: MultinameFlags,
}

impl<'gc> Multiname<'gc> {
    #[inline(always)]
    pub fn has_lazy_ns(&self) -> bool {
        self.flags.contains(MultinameFlags::HAS_LAZY_NS)
    }

    #[inline(always)]
    pub fn has_lazy_name(&self) -> bool {
        self.flags.contains(MultinameFlags::HAS_LAZY_NAME)
    }

    #[inline(always)]
    pub fn has_lazy_component(&self) -> bool {
        self.has_lazy_ns() || self.has_lazy_name()
    }

    #[inline(always)]
    pub fn is_attribute(&self) -> bool {
        self.flags.contains(MultinameFlags::ATTRIBUTE)
    }

    pub fn set_is_attribute(&mut self, is_attribute: bool) {
        self.flags.set(MultinameFlags::ATTRIBUTE, is_attribute);
    }

    #[inline(always)]
    pub fn is_qname(&self) -> bool {
        self.flags.contains(MultinameFlags::IS_QNAME)
    }

    pub fn set_is_qname(&mut self, is_qname: bool) {
        self.flags.set(MultinameFlags::IS_QNAME, is_qname);
    }

    /// Read a namespace set from the ABC constant pool, and return a list of
    /// copied namespaces.
    pub fn abc_namespace_set(
        activation: &mut Activation<'_, 'gc>,
        translation_unit: TranslationUnit<'gc>,
        namespace_set_index: Index<AbcNamespaceSet>,
    ) -> Result<NamespaceSet<'gc>, Error<'gc>> {
        if namespace_set_index.0 == 0 {
            return Err(make_error_1032(activation, 0));
        }

        let actual_index = namespace_set_index.0 as usize - 1;
        let abc = translation_unit.abc();
        let ns_set: Result<_, Error<'gc>> = abc
            .constant_pool
            .namespace_sets
            .get(actual_index)
            .ok_or_else(|| {
                format!("Unknown namespace set constant {}", namespace_set_index.0).into()
            });
        let ns_set = ns_set?;

        if ns_set.len() == 1 {
            let namespace = translation_unit.pool_namespace(activation, ns_set[0])?;

            if namespace.is_any() {
                return Err(make_error_1080(activation));
            }

            Ok(NamespaceSet::single(namespace))
        } else {
            let mut result = Vec::with_capacity(ns_set.len());
            for ns in ns_set {
                let namespace = translation_unit.pool_namespace(activation, *ns)?;

                // Namespace sets must not have Any namespaces in them
                if namespace.is_any() {
                    return Err(make_error_1080(activation));
                }

                result.push(namespace)
            }

            Ok(NamespaceSet::multiple(result, activation.gc()))
        }
    }

    pub fn from_abc_index(
        activation: &mut Activation<'_, 'gc>,
        translation_unit: TranslationUnit<'gc>,
        multiname_index: Index<AbcMultiname>,
    ) -> Result<Self, Error<'gc>> {
        let mc = activation.gc();

        if multiname_index.0 == 0 {
            return Err(make_error_1032(activation, 0));
        }

        let abc = translation_unit.abc();

        let abc_multiname = abc
            .constant_pool
            .multinames
            .get(multiname_index.0 as usize - 1)
            .ok_or_else(|| format!("Unknown multiname constant {}", multiname_index.0))?;

        let mut multiname = match abc_multiname {
            AbcMultiname::QName { namespace, name } | AbcMultiname::QNameA { namespace, name } => {
                Self {
                    ns: NamespaceSet::single(
                        translation_unit.pool_namespace(activation, *namespace)?,
                    ),
                    name: translation_unit
                        .pool_string_option(name.0, activation.strings())?
                        .map(|v| v.into()),
                    param: None,
                    flags: MultinameFlags::IS_QNAME,
                }
            }
            AbcMultiname::RTQName { name } | AbcMultiname::RTQNameA { name } => Self {
                ns: NamespaceSet::multiple(vec![], mc),
                name: translation_unit
                    .pool_string_option(name.0, activation.strings())?
                    .map(|v| v.into()),
                param: None,
                flags: MultinameFlags::HAS_LAZY_NS | MultinameFlags::IS_QNAME,
            },
            AbcMultiname::RTQNameL | AbcMultiname::RTQNameLA => Self {
                ns: NamespaceSet::multiple(vec![], mc),
                name: None,
                param: None,
                flags: MultinameFlags::HAS_LAZY_NS
                    | MultinameFlags::HAS_LAZY_NAME
                    | MultinameFlags::IS_QNAME,
            },
            AbcMultiname::Multiname {
                namespace_set,
                name,
            }
            | AbcMultiname::MultinameA {
                namespace_set,
                name,
            } => Self {
                ns: Self::abc_namespace_set(activation, translation_unit, *namespace_set)?,
                name: translation_unit
                    .pool_string_option(name.0, activation.strings())?
                    .map(|v| v.into()),
                param: None,
                flags: Default::default(),
            },
            AbcMultiname::MultinameL { namespace_set }
            | AbcMultiname::MultinameLA { namespace_set } => Self {
                ns: Self::abc_namespace_set(activation, translation_unit, *namespace_set)?,
                name: None,
                param: None,
                flags: MultinameFlags::HAS_LAZY_NAME,
            },
            AbcMultiname::TypeName {
                base_type,
                parameters,
            } => {
                let mut base = translation_unit
                    .pool_multiname_static(activation, *base_type)?
                    .deref()
                    .clone();

                if parameters.len() > 1 {
                    return Err(format!(
                        "VerifyError: Multiname has {} parameters, no more than 1 is allowed",
                        parameters.len()
                    )
                    .into());
                }

                base.param =
                    Some(translation_unit.pool_multiname_static_any(activation, parameters[0])?);
                base
            }
        };

        if matches!(
            abc_multiname,
            AbcMultiname::QNameA { .. }
                | AbcMultiname::RTQNameA { .. }
                | AbcMultiname::RTQNameLA { .. }
                | AbcMultiname::MultinameA { .. }
                | AbcMultiname::MultinameLA { .. }
        ) {
            multiname.flags |= MultinameFlags::ATTRIBUTE;
        }

        Ok(multiname)
    }

    pub fn fill_with_runtime_params(
        &self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Self, Error<'gc>> {
        let name = if self.has_lazy_name() {
            let name_value = activation.pop_stack();

            if let Value::Object(Object::QNameObject(qname_object)) = name_value {
                if self.has_lazy_ns() {
                    let _ = activation.pop_stack(); // ignore the ns component
                }
                let mut name = qname_object.name().clone();

                if self.is_attribute() {
                    name.set_is_attribute(true);
                }

                return Ok(name);
            }

            Some(name_value.coerce_to_string(activation)?)
        } else {
            self.name
        };

        let ns = if self.has_lazy_ns() {
            let ns_value = activation.pop_stack();
            let ns = ns_value.as_namespace()?;
            NamespaceSet::single(ns)
        } else {
            self.ns
        };

        Ok(Self {
            ns,
            name,
            param: self.param,
            flags: self.flags & (MultinameFlags::ATTRIBUTE | MultinameFlags::IS_QNAME),
        })
    }

    /// Indicates the any type (any name in any namespace).
    pub fn any() -> Self {
        Self {
            ns: NamespaceSet::single(Namespace::any()),
            name: None,
            param: None,
            flags: Default::default(),
        }
    }

    /// Indicates the any attribute type (any attribute in any namespace).
    pub fn any_attribute() -> Self {
        Self {
            ns: NamespaceSet::single(Namespace::any()),
            name: None,
            param: None,
            flags: MultinameFlags::ATTRIBUTE,
        }
    }

    pub fn new(ns: Namespace<'gc>, name: impl Into<AvmString<'gc>>) -> Self {
        Self {
            ns: NamespaceSet::single(ns),
            name: Some(name.into()),
            param: None,
            flags: Default::default(),
        }
    }

    /// Creates a new Multiname with the `MultinameFlags::ATTRIBUTE` flag.
    pub fn attribute(ns: Namespace<'gc>, name: impl Into<AvmString<'gc>>) -> Self {
        Self {
            ns: NamespaceSet::single(ns),
            name: Some(name.into()),
            param: None,
            flags: MultinameFlags::ATTRIBUTE,
        }
    }

    pub fn namespace_set(&self) -> &[Namespace<'gc>] {
        match &self.ns {
            NamespaceSet::Single(ns) => std::slice::from_ref(ns),
            NamespaceSet::Multiple(ns) => ns,
        }
    }

    pub fn local_name(&self) -> Option<AvmString<'gc>> {
        self.name
    }

    pub fn contains_public_namespace(&self) -> bool {
        match self.ns {
            NamespaceSet::Single(ns) => ns.is_public(),
            NamespaceSet::Multiple(ns) => ns.iter().any(|ns| ns.is_public()),
        }
    }

    pub fn has_explicit_namespace(&self) -> bool {
        match self.ns {
            NamespaceSet::Single(ns) => ns.is_namespace() && !ns.is_public(),
            NamespaceSet::Multiple(_) => false,
        }
    }

    pub fn has_nonempty_namespace(&self) -> bool {
        match self.ns {
            NamespaceSet::Single(ns) => !ns.is_public(),
            NamespaceSet::Multiple(_) => true,
        }
    }

    pub fn explicit_namespace(&self) -> Option<AvmString<'gc>> {
        match self.ns {
            NamespaceSet::Single(ns) if ns.is_namespace() && !ns.is_public() => ns.as_uri_opt(),
            _ => None,
        }
    }

    /// Indicates if this multiname matches any type.
    pub fn is_any_name(&self) -> bool {
        self.name.is_none()
    }

    /// Indicates if this multiname matches any namespace.
    pub fn is_any_namespace(&self) -> bool {
        match self.ns {
            NamespaceSet::Single(ns) => ns.is_any(),

            // NamespaceSet::Multiple should not have any Any namespaces in it
            NamespaceSet::Multiple(_) => false,
        }
    }

    /// Determine if this multiname matches a given QName.
    pub fn contains_name(&self, name: &QName<'gc>) -> bool {
        let ns_match = self
            .namespace_set()
            .iter()
            .any(|ns| ns.is_any() || ns.matches_ns(name.namespace()));
        let name_match = self.name.map(|n| n == name.local_name()).unwrap_or(true);

        ns_match && name_match
    }

    /// List the parameters that the selected class must match.
    pub fn param(&self) -> Option<Option<Gc<'gc, Multiname<'gc>>>> {
        self.param
    }

    pub fn to_qualified_name(&self, mc: &Mutation<'gc>) -> AvmString<'gc> {
        let mut uri = WString::new();
        let ns = match self.ns.get(0).filter(|_| self.ns.len() == 1) {
            Some(ns) if ns.is_any() => WStr::from_units(b"*"),
            Some(ns) => ns.as_uri_opt().map(|uri| uri.as_wstr()).unwrap_or_default(),
            None => WStr::empty(),
        };

        uri.push_str(ns);

        if let Some(name) = self.name {
            if !uri.is_empty() {
                uri.push_str(WStr::from_units(b"::"));
            } else if self.param.is_none() {
                // Special-case this to avoid allocating.
                return name;
            }
            uri.push_str(&name);
        } else {
            uri.push_str(WStr::from_units(b"::*"));
        }

        if let Some(param) = self.param {
            uri.push_str(WStr::from_units(b".<"));
            if let Some(param) = param {
                uri.push_str(&param.to_qualified_name(mc));
            } else {
                uri.push_str(WStr::from_units(b"*"));
            }
            uri.push_str(WStr::from_units(b">"));
        }

        AvmString::new(mc, uri)
    }

    /// Like `to_qualified_name`, but returns `*` if `self.is_any()` is true.
    /// This is used by `describeType`
    pub fn to_qualified_name_or_star(&self, context: &mut StringContext<'gc>) -> AvmString<'gc> {
        if self.is_any_name() {
            context.ascii_char(b'*')
        } else {
            self.to_qualified_name(context.gc())
        }
    }

    // note: I didn't look very deeply into how different exactly this should be
    // this is currently generally based on to_qualified_name, without params and leading ::
    pub fn as_uri(&self, context: &mut StringContext<'gc>) -> AvmString<'gc> {
        let ns = match self.ns.get(0).filter(|_| self.ns.len() == 1) {
            Some(ns) if ns.is_any() => WStr::from_units(b"*"),
            Some(ns) => ns.as_uri(context).as_wstr(),
            None => WStr::empty(),
        };

        if ns.is_empty() {
            // Special-case this to avoid allocating.
            self.name.unwrap_or_else(|| context.ascii_char(b'*'))
        } else {
            let mut uri = WString::new();
            uri.push_str(ns);
            uri.push_str(WStr::from_units(b"::"));
            uri.push_str(
                self.name
                    .as_deref()
                    .unwrap_or_else(|| WStr::from_units(b"*")),
            );
            AvmString::new(context.gc(), uri)
        }
    }

    pub fn set_ns(&mut self, ns: NamespaceSet<'gc>) {
        self.ns = ns;
    }

    pub fn set_single_namespace(&mut self, namespace: Namespace<'gc>) {
        self.ns = NamespaceSet::Single(namespace);
    }

    pub fn set_local_name(&mut self, name: AvmString<'gc>) {
        self.name = Some(name);
    }
}

impl<'gc> From<QName<'gc>> for Multiname<'gc> {
    fn from(q: QName<'gc>) -> Self {
        Self {
            ns: NamespaceSet::single(q.namespace()),
            name: Some(q.local_name()),
            param: None,
            flags: Default::default(),
        }
    }
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct CommonMultinames<'gc> {
    pub boolean: Gc<'gc, Multiname<'gc>>,
    pub function: Gc<'gc, Multiname<'gc>>,
    pub int: Gc<'gc, Multiname<'gc>>,
    pub number: Gc<'gc, Multiname<'gc>>,
    pub uint: Gc<'gc, Multiname<'gc>>,
}

impl<'gc> CommonMultinames<'gc> {
    pub fn new(context: &mut StringContext<'gc>, namespaces: &CommonNamespaces<'gc>) -> Self {
        let mut create_pub_multiname = |local_name: &'static [u8]| -> Gc<'gc, Multiname<'gc>> {
            Gc::new(
                context.gc(),
                Multiname::new(
                    namespaces.public_all(),
                    context.intern_static(WStr::from_units(local_name)),
                ),
            )
        };

        Self {
            boolean: create_pub_multiname(b"Boolean"),
            function: create_pub_multiname(b"Function"),
            int: create_pub_multiname(b"int"),
            number: create_pub_multiname(b"Number"),
            uint: create_pub_multiname(b"uint"),
        }
    }
}
