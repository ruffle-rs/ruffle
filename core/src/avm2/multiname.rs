use crate::avm2::activation::Activation;
use crate::avm2::script::TranslationUnit;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Namespace;
use crate::avm2::QName;
use crate::string::{AvmString, WStr, WString};
use gc_arena::{Collect, MutationContext};
use std::fmt::Debug;
use swf::avm2::types::{
    AbcFile, Index, Multiname as AbcMultiname, NamespaceSet as AbcNamespaceSet,
};

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
    ns: Vec<Namespace<'gc>>,

    /// The local name that satisfies this multiname. If `None`, then this
    /// multiname is satisfied by any name in the namespace.
    name: Option<AvmString<'gc>>,

    /// The type parameters required to satisfy this multiname. If empty, then
    /// this multiname is satisfied by any type parameters in any amount.
    params: Vec<Multiname<'gc>>,
}

impl<'gc> Multiname<'gc> {
    /// Read a namespace set from the ABC constant pool, and return a list of
    /// copied namespaces.
    fn abc_namespace_set(
        translation_unit: TranslationUnit<'gc>,
        namespace_set_index: Index<AbcNamespaceSet>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<Vec<Namespace<'gc>>, Error> {
        if namespace_set_index.0 == 0 {
            //TODO: What is namespace set zero?
            return Ok(vec![]);
        }

        let actual_index = namespace_set_index.0 as usize - 1;
        let abc = translation_unit.abc();
        let ns_set: Result<_, Error> = abc
            .constant_pool
            .namespace_sets
            .get(actual_index)
            .ok_or_else(|| {
                format!("Unknown namespace set constant {}", namespace_set_index.0).into()
            });
        let mut result = vec![];

        for ns in ns_set? {
            result.push(Namespace::from_abc_namespace(translation_unit, *ns, mc)?)
        }

        Ok(result)
    }

    /// Assemble a multiname from an ABC `MultinameL` and the late-bound name.
    ///
    /// Intended for use by code that wants to inspect the late-bound name's
    /// value first before using standard namespace lookup.
    pub fn from_multiname_late(
        translation_unit: TranslationUnit<'gc>,
        abc_multiname: &AbcMultiname,
        name: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Self, Error> {
        match abc_multiname {
            AbcMultiname::MultinameL { namespace_set }
            | AbcMultiname::MultinameLA { namespace_set } => Ok(Self {
                ns: Self::abc_namespace_set(
                    translation_unit,
                    *namespace_set,
                    activation.context.gc_context,
                )?,
                name: Some(name.coerce_to_string(activation)?),
                params: Vec::new(),
            }),
            _ => Err("Cannot assemble early-bound multinames using from_multiname_late".into()),
        }
    }

    /// Resolve an ABC multiname's parameters and yields an AVM multiname with
    /// those parameters filled in.
    ///
    /// This function deliberately errors out if handed a `TypeName`, as it
    /// assumes that this is an attempt to construct a recursive generic type.
    /// Type parameters may themselves be typenames, but not the base type.
    /// This is valid: `Vector.<Vector.<int>>`, but this is not:
    /// `Vector.<int>.<int>`
    fn resolve_multiname_params(
        translation_unit: TranslationUnit<'gc>,
        abc_multiname: &AbcMultiname,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Self, Error> {
        Ok(match abc_multiname {
            AbcMultiname::QName { namespace, name } | AbcMultiname::QNameA { namespace, name } => {
                Self {
                    ns: vec![Namespace::from_abc_namespace(
                        translation_unit,
                        *namespace,
                        activation.context.gc_context,
                    )?],
                    name: translation_unit
                        .pool_string_option(name.0, activation.context.gc_context)?,
                    params: Vec::new(),
                }
            }
            AbcMultiname::RTQName { name } | AbcMultiname::RTQNameA { name } => {
                let ns_value = activation.avm2().pop();
                let ns = ns_value.as_namespace()?;
                Self {
                    ns: vec![*ns],
                    name: translation_unit
                        .pool_string_option(name.0, activation.context.gc_context)?,
                    params: Vec::new(),
                }
            }
            AbcMultiname::RTQNameL | AbcMultiname::RTQNameLA => {
                let name = activation.avm2().pop().coerce_to_string(activation)?;
                let ns_value = activation.avm2().pop();
                let ns = ns_value.as_namespace()?;
                Self {
                    ns: vec![*ns],
                    name: Some(name),
                    params: Vec::new(),
                }
            }
            AbcMultiname::Multiname {
                namespace_set,
                name,
            }
            | AbcMultiname::MultinameA {
                namespace_set,
                name,
            } => Self {
                ns: Self::abc_namespace_set(
                    translation_unit,
                    *namespace_set,
                    activation.context.gc_context,
                )?,
                name: translation_unit.pool_string_option(name.0, activation.context.gc_context)?,
                params: Vec::new(),
            },
            AbcMultiname::MultinameL { .. } | AbcMultiname::MultinameLA { .. } => {
                let name = activation.avm2().pop();
                Self::from_multiname_late(translation_unit, abc_multiname, name, activation)?
            }
            AbcMultiname::TypeName { .. } => {
                return Err("Recursive TypeNames are not supported!".into())
            }
        })
    }

    /// Retrieve a given multiname index from the ABC file, yielding an error
    /// if the multiname index is zero.
    pub fn resolve_multiname_index(
        abc: &AbcFile,
        multiname_index: Index<AbcMultiname>,
    ) -> Result<&AbcMultiname, Error> {
        let actual_index: Result<usize, Error> = (multiname_index.0 as usize)
            .checked_sub(1)
            .ok_or_else(|| "Attempted to resolve a multiname at index zero. This is a bug.".into());
        let actual_index = actual_index?;
        let abc_multiname: Result<_, Error> = abc
            .constant_pool
            .multinames
            .get(actual_index)
            .ok_or_else(|| format!("Unknown multiname constant {}", multiname_index.0).into());

        abc_multiname
    }

    /// Read a multiname from the ABC constant pool, copying it into the most
    /// general form of multiname.
    pub fn from_abc_multiname(
        translation_unit: TranslationUnit<'gc>,
        multiname_index: Index<AbcMultiname>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Self, Error> {
        let abc = translation_unit.abc();
        let abc_multiname = Self::resolve_multiname_index(&abc, multiname_index)?;

        match abc_multiname {
            AbcMultiname::TypeName {
                base_type,
                parameters,
            } => {
                let base_multiname = Self::resolve_multiname_index(&abc, *base_type)?;
                let mut base =
                    Self::resolve_multiname_params(translation_unit, base_multiname, activation)?;

                if parameters.len() > 1 {
                    return Err(format!(
                        "VerifyError: Multiname has {} parameters, no more than 1 is allowed",
                        parameters.len()
                    )
                    .into());
                }

                for param_type in parameters {
                    let param_multiname =
                        Self::from_abc_multiname(translation_unit, *param_type, activation)?;

                    base.params.push(param_multiname);
                }

                Ok(base)
            }
            abc_multiname => {
                Self::resolve_multiname_params(translation_unit, abc_multiname, activation)
            }
        }
    }

    /// Read a static multiname from the ABC constant pool
    ///
    /// This function prohibits the use of runtime-qualified and late-bound
    /// names. Runtime multinames will instead result in an error.
    ///
    /// Multiname index zero is also treated as an error, you must check for it
    /// and substitute it with whatever default is called for by AVM2.
    pub fn from_abc_multiname_static(
        translation_unit: TranslationUnit<'gc>,
        multiname_index: Index<AbcMultiname>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<Self, Error> {
        let actual_index: Result<usize, Error> =
            (multiname_index.0 as usize).checked_sub(1).ok_or_else(|| {
                "Attempted to resolve a (static) multiname at index zero. This is a bug.".into()
            });
        let actual_index = actual_index?;
        let abc = translation_unit.abc();
        let abc_multiname: Result<_, Error> = abc
            .constant_pool
            .multinames
            .get(actual_index)
            .ok_or_else(|| format!("Unknown multiname constant {}", multiname_index.0).into());

        Ok(match abc_multiname? {
            AbcMultiname::QName { namespace, name } | AbcMultiname::QNameA { namespace, name } => {
                Self {
                    ns: vec![Namespace::from_abc_namespace(
                        translation_unit,
                        *namespace,
                        mc,
                    )?],
                    name: translation_unit.pool_string_option(name.0, mc)?,
                    params: Vec::new(),
                }
            }
            AbcMultiname::Multiname {
                namespace_set,
                name,
            }
            | AbcMultiname::MultinameA {
                namespace_set,
                name,
            } => Self {
                ns: Self::abc_namespace_set(translation_unit, *namespace_set, mc)?,
                name: translation_unit.pool_string_option(name.0, mc)?,
                params: Vec::new(),
            },
            AbcMultiname::TypeName {
                base_type,
                parameters,
            } => {
                let mut base = Self::from_abc_multiname_static(translation_unit, *base_type, mc)?;

                if parameters.len() > 1 {
                    return Err(format!(
                        "VerifyError: Multiname has {} parameters, no more than 1 is allowed",
                        parameters.len()
                    )
                    .into());
                }

                for param_type in parameters {
                    let param_multiname = if param_type.0 == 0 {
                        Self::any()
                    } else {
                        Self::from_abc_multiname_static(translation_unit, *param_type, mc)?
                    };

                    base.params.push(param_multiname);
                }

                base
            }
            _ => return Err(format!("Multiname {} is not static", multiname_index.0).into()),
        })
    }

    /// Indicates the any type (any name in any namespace).
    pub fn any() -> Self {
        Self {
            ns: vec![Namespace::Any],
            name: None,
            params: Vec::new(),
        }
    }

    pub fn public(name: impl Into<AvmString<'gc>>) -> Self {
        Self {
            ns: vec![Namespace::public()],
            name: Some(name.into()),
            params: Vec::new(),
        }
    }

    pub fn namespace_set(&self) -> impl Iterator<Item = &Namespace<'gc>> {
        self.ns.iter()
    }

    pub fn local_name(&self) -> Option<AvmString<'gc>> {
        self.name
    }

    pub fn contains_public_namespace(&self) -> bool {
        self.ns.iter().any(|ns| ns.is_public())
    }

    /// Indicates if this multiname matches any type in any namespace.
    pub fn is_any(&self) -> bool {
        self.ns.contains(&Namespace::Any) && self.name.is_none()
    }

    /// Determine if this multiname matches a given QName.
    pub fn contains_name(&self, name: &QName<'gc>) -> bool {
        let ns_match = self
            .ns
            .iter()
            .any(|ns| *ns == Namespace::Any || *ns == name.namespace());
        let name_match = self.name.map(|n| n == name.local_name()).unwrap_or(true);

        ns_match && name_match
    }

    /// List the parameters that the selected class must match.
    pub fn params(&self) -> &[Multiname<'gc>] {
        &self.params[..]
    }

    pub fn to_qualified_name(&self, mc: MutationContext<'gc, '_>) -> AvmString<'gc> {
        let mut uri = WString::new();
        let ns = match self.ns.get(0).filter(|_| self.ns.len() == 1) {
            Some(Namespace::Any) => "*".into(),
            Some(ns) => ns.as_uri(),
            None => "".into(),
        };

        uri.push_str(&ns);

        if let Some(name) = self.name {
            uri.push_str(WStr::from_units(b"::"));
            uri.push_str(&name);
        } else {
            uri.push_str(WStr::from_units(b"::*"));
        }

        if !self.params.is_empty() {
            uri.push_str(WStr::from_units(b"<"));

            for (i, param) in self.params.iter().enumerate() {
                uri.push_str(&param.to_qualified_name(mc));
                if i < self.params.len() - 1 {
                    uri.push_str(WStr::from_units(b","));
                }
            }

            uri.push_str(WStr::from_units(b">"));
        }

        AvmString::new(mc, uri)
    }
}

impl<'gc> From<QName<'gc>> for Multiname<'gc> {
    fn from(q: QName<'gc>) -> Self {
        Self {
            ns: vec![q.namespace()],
            name: Some(q.local_name()),
            params: Vec::new(),
        }
    }
}
