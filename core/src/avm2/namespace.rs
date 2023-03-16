use crate::avm2::Error;
use crate::string::AvmString;
use crate::{avm2::script::TranslationUnit, context::GcContext};
use gc_arena::{Collect, Gc, MutationContext};
use std::fmt::Debug;
use swf::avm2::types::{Index, Namespace as AbcNamespace};

#[derive(Clone, Copy, Collect, Debug)]
#[collect(no_drop)]
pub struct Namespace<'gc>(pub Gc<'gc, NamespaceData<'gc>>);

impl<'gc> PartialEq for Namespace<'gc> {
    fn eq(&self, other: &Self) -> bool {
        if Gc::as_ptr(self.0) == Gc::as_ptr(other.0) {
            true
        } else if self.is_private() || other.is_private() {
            false
        } else {
            *self.0 == *other.0
        }
    }
}
impl<'gc> Eq for Namespace<'gc> {}

/// Represents the name of a namespace.
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Collect, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[collect(no_drop)]
pub enum NamespaceData<'gc> {
    // note: this is the default "public namespace", corresponding to both
    // ABC Namespace and PackageNamespace
    Namespace(AvmString<'gc>),
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
    /// NOTE: you should use the TranslationUnit.pool_namespace instead of calling this.
    /// otherwise you run a risk of creating a duplicate of private ns singleton.
    pub fn from_abc_namespace(
        translation_unit: TranslationUnit<'gc>,
        namespace_index: Index<AbcNamespace>,
        context: &mut GcContext<'_, 'gc>,
    ) -> Result<Self, Error<'gc>> {
        if namespace_index.0 == 0 {
            return Ok(Self::any(context.gc_context));
        }

        let actual_index = namespace_index.0 as usize - 1;
        let abc = translation_unit.abc();
        let abc_namespace: Result<_, Error<'gc>> = abc
            .constant_pool
            .namespaces
            .get(actual_index)
            .ok_or_else(|| format!("Unknown namespace constant {}", namespace_index.0).into());

        let ns = match abc_namespace? {
            AbcNamespace::Namespace(idx) => {
                NamespaceData::Namespace(translation_unit.pool_string(idx.0, context)?)
            }
            AbcNamespace::Package(idx) => {
                NamespaceData::Namespace(translation_unit.pool_string(idx.0, context)?)
            }
            AbcNamespace::PackageInternal(idx) => {
                NamespaceData::PackageInternal(translation_unit.pool_string(idx.0, context)?)
            }
            AbcNamespace::Protected(idx) => {
                NamespaceData::Protected(translation_unit.pool_string(idx.0, context)?)
            }
            AbcNamespace::Explicit(idx) => {
                NamespaceData::Explicit(translation_unit.pool_string(idx.0, context)?)
            }
            AbcNamespace::StaticProtected(idx) => {
                NamespaceData::StaticProtected(translation_unit.pool_string(idx.0, context)?)
            }
            AbcNamespace::Private(idx) => {
                NamespaceData::Private(translation_unit.pool_string(idx.0, context)?)
            }
        };
        Ok(Self(Gc::allocate(context.gc_context, ns)))
    }

    pub fn any(mc: MutationContext<'gc, '_>) -> Self {
        Self(Gc::allocate(mc, NamespaceData::Any))
    }

    pub fn package(package_name: impl Into<AvmString<'gc>>, mc: MutationContext<'gc, '_>) -> Self {
        Self(Gc::allocate(
            mc,
            NamespaceData::Namespace(package_name.into()),
        ))
    }

    pub fn internal(package_name: impl Into<AvmString<'gc>>, mc: MutationContext<'gc, '_>) -> Self {
        Self(Gc::allocate(
            mc,
            NamespaceData::PackageInternal(package_name.into()),
        ))
    }

    // note: since private namespaces are compared by identity,
    // if you try using it to create temporary namespaces it will likely not work.
    pub fn private(name: impl Into<AvmString<'gc>>, mc: MutationContext<'gc, '_>) -> Self {
        Self(Gc::allocate(mc, NamespaceData::Private(name.into())))
    }

    pub fn is_public(&self) -> bool {
        matches!(*self.0, NamespaceData::Namespace(name) if name.is_empty())
    }

    pub fn is_any(&self) -> bool {
        matches!(*self.0, NamespaceData::Any)
    }

    pub fn is_private(&self) -> bool {
        matches!(*self.0, NamespaceData::Private(_))
    }

    pub fn is_namespace(&self) -> bool {
        matches!(*self.0, NamespaceData::Namespace(_))
    }

    /// Get the string value of this namespace, ignoring its type.
    ///
    /// TODO: Is this *actually* the namespace URI?
    pub fn as_uri(&self) -> AvmString<'gc> {
        match &*self.0 {
            NamespaceData::Namespace(s) => *s,
            NamespaceData::PackageInternal(s) => *s,
            NamespaceData::Protected(s) => *s,
            NamespaceData::Explicit(s) => *s,
            NamespaceData::StaticProtected(s) => *s,
            NamespaceData::Private(s) => *s,
            NamespaceData::Any => "".into(),
        }
    }
}
