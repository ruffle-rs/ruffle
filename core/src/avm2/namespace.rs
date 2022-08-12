use crate::avm2::script::TranslationUnit;
use crate::avm2::Error;
use crate::string::AvmString;
use gc_arena::{Collect, MutationContext};
use std::fmt::Debug;
use swf::avm2::types::{Index, Namespace as AbcNamespace};

/// Represents the name of a namespace.
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Collect, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[collect(no_drop)]
pub enum Namespace<'gc> {
    Namespace(AvmString<'gc>),
    Package(AvmString<'gc>),
    PackageInternal(AvmString<'gc>),
    Protected(AvmString<'gc>),
    Explicit(AvmString<'gc>),
    StaticProtected(AvmString<'gc>),
    Private(AvmString<'gc>),
    Any,
}

impl<'gc> Namespace<'gc> {
    /// Read a namespace declaration from the ABC constant pool and copy it to
    /// a namespace value.
    pub fn from_abc_namespace(
        translation_unit: TranslationUnit<'gc>,
        namespace_index: Index<AbcNamespace>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<Self, Error> {
        if namespace_index.0 == 0 {
            return Ok(Self::Any);
        }

        let actual_index = namespace_index.0 as usize - 1;
        let abc = translation_unit.abc();
        let abc_namespace: Result<_, Error> = abc
            .constant_pool
            .namespaces
            .get(actual_index)
            .ok_or_else(|| format!("Unknown namespace constant {}", namespace_index.0).into());

        Ok(match abc_namespace? {
            AbcNamespace::Namespace(idx) => {
                Self::Namespace(translation_unit.pool_string(idx.0, mc)?)
            }
            AbcNamespace::Package(idx) => Self::Package(translation_unit.pool_string(idx.0, mc)?),
            AbcNamespace::PackageInternal(idx) => {
                Self::PackageInternal(translation_unit.pool_string(idx.0, mc)?)
            }
            AbcNamespace::Protected(idx) => {
                Self::Protected(translation_unit.pool_string(idx.0, mc)?)
            }
            AbcNamespace::Explicit(idx) => Self::Explicit(translation_unit.pool_string(idx.0, mc)?),
            AbcNamespace::StaticProtected(idx) => {
                Self::StaticProtected(translation_unit.pool_string(idx.0, mc)?)
            }
            AbcNamespace::Private(idx) => Self::Private(translation_unit.pool_string(idx.0, mc)?),
        })
    }

    pub fn public() -> Self {
        Self::Package("".into())
    }

    pub fn as3_namespace() -> Self {
        Self::Namespace("http://adobe.com/AS3/2006/builtin".into())
    }

    pub fn package(package_name: impl Into<AvmString<'gc>>) -> Self {
        Self::Package(package_name.into())
    }

    pub fn internal(package_name: impl Into<AvmString<'gc>>) -> Self {
        Self::PackageInternal(package_name.into())
    }

    pub fn private(name: impl Into<AvmString<'gc>>) -> Self {
        Self::Private(name.into())
    }

    pub fn is_public(&self) -> bool {
        matches!(self, Self::Package(name) if name.is_empty())
    }

    pub fn is_any(&self) -> bool {
        matches!(self, Self::Any)
    }

    pub fn is_private(&self) -> bool {
        matches!(self, Self::Private(_))
    }

    pub fn is_package(&self, package_name: impl Into<AvmString<'gc>>) -> bool {
        if let Self::Package(my_name) = self {
            return my_name == &package_name.into();
        }

        false
    }

    pub fn is_namespace(&self) -> bool {
        matches!(self, Self::Namespace(_))
    }

    /// Get the string value of this namespace, ignoring its type.
    ///
    /// TODO: Is this *actually* the namespace URI?
    pub fn as_uri(&self) -> AvmString<'gc> {
        match self {
            Self::Namespace(s) => *s,
            Self::Package(s) => *s,
            Self::PackageInternal(s) => *s,
            Self::Protected(s) => *s,
            Self::Explicit(s) => *s,
            Self::StaticProtected(s) => *s,
            Self::Private(s) => *s,
            Self::Any => "".into(),
        }
    }
}
