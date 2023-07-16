use crate::avm2::Error;
use crate::string::{AvmAtom, AvmString};
use crate::{avm2::script::TranslationUnit, context::GcContext};
use gc_arena::{Collect, Gc, MutationContext};
use std::fmt::Debug;
use swf::avm2::types::{Index, Namespace as AbcNamespace};

#[derive(Clone, Copy, Collect, Debug)]
#[collect(no_drop)]
pub struct Namespace<'gc>(Gc<'gc, NamespaceData<'gc>>);

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
#[derive(Copy, Clone, Collect, Debug, PartialEq, Eq)]
#[collect(no_drop)]
enum NamespaceData<'gc> {
    // note: this is the default "public namespace", corresponding to both
    // ABC Namespace and PackageNamespace
    Namespace(AvmAtom<'gc>),
    PackageInternal(AvmAtom<'gc>),
    Protected(AvmAtom<'gc>),
    Explicit(AvmAtom<'gc>),
    StaticProtected(AvmAtom<'gc>),
    // note: private namespaces are always compared by pointer identity
    // of the enclosing `Gc`.
    Private(AvmAtom<'gc>),
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
        Ok(Self(Gc::new(context.gc_context, ns)))
    }

    pub fn any(mc: MutationContext<'gc, '_>) -> Self {
        Self(Gc::new(mc, NamespaceData::Any))
    }

    // TODO(moulins): allow passing an AvmAtom or a non-static `&WStr` directly
    pub fn package(
        package_name: impl Into<AvmString<'gc>>,
        context: &mut GcContext<'_, 'gc>,
    ) -> Self {
        let atom = context
            .interner
            .intern(context.gc_context, package_name.into());
        Self(Gc::new(context.gc_context, NamespaceData::Namespace(atom)))
    }

    // TODO(moulins): allow passing an AvmAtom or a non-static `&WStr` directly
    pub fn internal(
        package_name: impl Into<AvmString<'gc>>,
        context: &mut GcContext<'_, 'gc>,
    ) -> Self {
        let atom = context
            .interner
            .intern(context.gc_context, package_name.into());
        Self(Gc::new(
            context.gc_context,
            NamespaceData::PackageInternal(atom),
        ))
    }

    pub fn is_public(&self) -> bool {
        matches!(*self.0, NamespaceData::Namespace(name) if name.as_wstr().is_empty())
    }

    pub fn is_public_ignoring_ns(&self) -> bool {
        matches!(*self.0, NamespaceData::Namespace(_))
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

    pub fn as_uri_opt(&self) -> Option<AvmString<'gc>> {
        match *self.0 {
            NamespaceData::Namespace(a) => Some(a.into()),
            NamespaceData::PackageInternal(a) => Some(a.into()),
            NamespaceData::Protected(a) => Some(a.into()),
            NamespaceData::Explicit(a) => Some(a.into()),
            NamespaceData::StaticProtected(a) => Some(a.into()),
            NamespaceData::Private(a) => Some(a.into()),
            NamespaceData::Any => None,
        }
    }

    /// Get the string value of this namespace, ignoring its type.
    ///
    /// TODO: Is this *actually* the namespace URI?
    pub fn as_uri(&self) -> AvmString<'gc> {
        self.as_uri_opt().unwrap_or_else(|| "".into())
    }
}
