use crate::avm2::activation::Activation;
use crate::avm2::script::TranslationUnit;
use crate::avm2::Error;
use crate::avm2::Namespace;
use crate::avm2::QName;
use crate::avm2::{Object, Value};
use crate::context::UpdateContext;
use crate::string::{AvmString, WStr, WString};
use bitflags::bitflags;
use gc_arena::Gc;
use gc_arena::{Collect, Mutation};
use std::fmt::Debug;
use std::ops::Deref;
use swf::avm2::types::{
    AbcFile, Index, Multiname as AbcMultiname, NamespaceSet as AbcNamespaceSet,
};

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
    /// this multiname is satisfied by any type parameter or no type parameter
    param: Option<Gc<'gc, Multiname<'gc>>>,

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
        translation_unit: TranslationUnit<'gc>,
        namespace_set_index: Index<AbcNamespaceSet>,
        context: &mut UpdateContext<'gc>,
    ) -> Result<NamespaceSet<'gc>, Error<'gc>> {
        if namespace_set_index.0 == 0 {
            return Err(Error::RustError(
                "Multiname namespace set must not be null".into(),
            ));
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
            Ok(NamespaceSet::single(
                translation_unit.pool_namespace(ns_set[0], context)?,
            ))
        } else {
            let mut result = Vec::with_capacity(ns_set.len());
            for ns in ns_set {
                result.push(translation_unit.pool_namespace(*ns, context)?)
            }
            Ok(NamespaceSet::multiple(result, context.gc_context))
        }
    }

    pub fn from_abc_index(
        translation_unit: TranslationUnit<'gc>,
        multiname_index: Index<AbcMultiname>,
        context: &mut UpdateContext<'gc>,
    ) -> Result<Self, Error<'gc>> {
        let mc = context.gc_context;
        let abc = translation_unit.abc();
        let abc_multiname = Self::resolve_multiname_index(&abc, multiname_index)?;

        let mut multiname = match abc_multiname {
            AbcMultiname::QName { namespace, name } | AbcMultiname::QNameA { namespace, name } => {
                Self {
                    ns: NamespaceSet::single(translation_unit.pool_namespace(*namespace, context)?),
                    name: translation_unit
                        .pool_string_option(name.0, &mut context.borrow_gc())?
                        .map(|v| v.into()),
                    param: None,
                    flags: MultinameFlags::IS_QNAME,
                }
            }
            AbcMultiname::RTQName { name } | AbcMultiname::RTQNameA { name } => Self {
                ns: NamespaceSet::multiple(vec![], mc),
                name: translation_unit
                    .pool_string_option(name.0, &mut context.borrow_gc())?
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
                ns: Self::abc_namespace_set(translation_unit, *namespace_set, context)?,
                name: translation_unit
                    .pool_string_option(name.0, &mut context.borrow_gc())?
                    .map(|v| v.into()),
                param: None,
                flags: Default::default(),
            },
            AbcMultiname::MultinameL { namespace_set }
            | AbcMultiname::MultinameLA { namespace_set } => Self {
                ns: Self::abc_namespace_set(translation_unit, *namespace_set, context)?,
                name: None,
                param: None,
                flags: MultinameFlags::HAS_LAZY_NAME,
            },
            AbcMultiname::TypeName {
                base_type,
                parameters,
            } => {
                let mut base = translation_unit
                    .pool_multiname_static(*base_type, context)?
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
                    Some(translation_unit.pool_multiname_static_any(parameters[0], context)?);
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

    /// Retrieve a given multiname index from the ABC file, yielding an error
    /// if the multiname index is zero.
    pub fn resolve_multiname_index(
        abc: &AbcFile,
        multiname_index: Index<AbcMultiname>,
    ) -> Result<&AbcMultiname, Error<'gc>> {
        let actual_index: Result<usize, Error<'gc>> = (multiname_index.0 as usize)
            .checked_sub(1)
            .ok_or_else(|| "Attempted to resolve a multiname at index zero. This is a bug.".into());

        let actual_index = actual_index?;
        abc.constant_pool
            .multinames
            .get(actual_index)
            .ok_or_else(|| format!("Unknown multiname constant {}", multiname_index.0).into())
    }

    /// Indicates the any type (any name in any namespace).
    pub fn any(mc: &Mutation<'gc>) -> Self {
        Self {
            ns: NamespaceSet::single(Namespace::any(mc)),
            name: None,
            param: None,
            flags: Default::default(),
        }
    }

    /// Indicates the any attribute type (any attribute in any namespace).
    pub fn any_attribute(mc: &Mutation<'gc>) -> Self {
        Self {
            ns: NamespaceSet::single(Namespace::any(mc)),
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
            NamespaceSet::Single(ns) if ns.is_namespace() && !ns.is_public() => Some(ns.as_uri()),
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
            NamespaceSet::Multiple(ns) => ns.iter().any(|ns| ns.is_any()),
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
    pub fn param(&self) -> Option<Gc<'gc, Multiname<'gc>>> {
        self.param
    }

    pub fn to_qualified_name(&self, mc: &Mutation<'gc>) -> AvmString<'gc> {
        let mut uri = WString::new();
        let ns = match self.ns.get(0).filter(|_| self.ns.len() == 1) {
            Some(ns) if ns.is_any() => "*".into(),
            Some(ns) => ns.as_uri(),
            None => "".into(),
        };

        uri.push_str(&ns);

        if let Some(name) = self.name {
            if !uri.is_empty() {
                uri.push_str(WStr::from_units(b"::"));
            }
            uri.push_str(&name);
        } else {
            uri.push_str(WStr::from_units(b"::*"));
        }

        if let Some(param) = self.param {
            uri.push_str(WStr::from_units(b".<"));
            uri.push_str(&param.to_qualified_name(mc));
            uri.push_str(WStr::from_units(b">"));
        }

        AvmString::new(mc, uri)
    }

    /// Like `to_qualified_name`, but returns `*` if `self.is_any()` is true.
    /// This is used by `describeType`
    pub fn to_qualified_name_or_star(&self, mc: &Mutation<'gc>) -> AvmString<'gc> {
        if self.is_any_name() {
            AvmString::new_utf8(mc, "*")
        } else {
            self.to_qualified_name(mc)
        }
    }

    // note: I didn't look very deeply into how different exactly this should be
    // this is currently generally based on to_qualified_name, without params and leading ::
    pub fn as_uri(&self, mc: &Mutation<'gc>) -> AvmString<'gc> {
        let mut uri = WString::new();
        let ns = match self.ns.get(0).filter(|_| self.ns.len() == 1) {
            Some(ns) if ns.is_any() => "*".into(),
            Some(ns) => ns.as_uri(),
            None => "".into(),
        };

        if !ns.is_empty() {
            uri.push_str(&ns);
            uri.push_str(WStr::from_units(b"::"));
        }

        if let Some(name) = self.name {
            uri.push_str(&name);
        } else {
            uri.push_str(WStr::from_units(b"*"));
        }

        AvmString::new(mc, uri)
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
