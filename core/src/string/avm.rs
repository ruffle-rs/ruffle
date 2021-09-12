use gc_arena::{Collect, Gc, MutationContext};
use std::cmp::{Eq, Ord, Ordering, PartialOrd};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
enum Source<'gc> {
    Owned(Gc<'gc, String>),
    Static(&'static str),
}

impl fmt::Debug for Source<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Source::Owned(str) => f.debug_tuple("Owned").field(str.deref()).finish(),
            Source::Static(str) => f.debug_tuple("Static").field(str).finish(),
        }
    }
}

#[derive(Debug, Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct AvmString<'gc> {
    source: Source<'gc>,
}

impl<'gc> AvmString<'gc> {
    pub fn new<S: Into<String>>(gc_context: MutationContext<'gc, '_>, string: S) -> Self {
        Self {
            source: Source::Owned(Gc::allocate(gc_context, string.into())),
        }
    }

    pub fn as_str(&self) -> &str {
        self
    }
}

impl Default for AvmString<'_> {
    fn default() -> Self {
        Self {
            source: Source::Static(""),
        }
    }
}

impl<'gc> From<&'static str> for AvmString<'gc> {
    fn from(str: &'static str) -> Self {
        Self {
            source: Source::Static(str),
        }
    }
}

impl fmt::Display for AvmString<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self)
    }
}

impl Deref for AvmString<'_> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        match &self.source {
            Source::Owned(str) => str.deref(),
            Source::Static(str) => str,
        }
    }
}

impl AsRef<str> for AvmString<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        match &self.source {
            Source::Owned(str) => str,
            Source::Static(str) => str,
        }
    }
}

impl<'gc> PartialEq<AvmString<'gc>> for AvmString<'gc> {
    #[inline]
    fn eq(&self, other: &AvmString<'gc>) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl<'gc> Eq for AvmString<'gc> {}

impl<'gc> PartialOrd<AvmString<'gc>> for AvmString<'gc> {
    fn partial_cmp(&self, other: &AvmString<'gc>) -> Option<Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl<'gc> Ord for AvmString<'gc> {
    fn cmp(&self, other: &AvmString<'gc>) -> Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl<'gc> Hash for AvmString<'gc> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.as_ref().hash(state)
    }
}

macro_rules! impl_eq {
    ($lhs:ty, $rhs: ty) => {
        #[allow(unused_lifetimes, clippy::redundant_slicing)]
        impl<'a, 'b> PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                PartialEq::eq(&self[..], &other[..])
            }
        }

        #[allow(unused_lifetimes, clippy::redundant_slicing)]
        impl<'a, 'b> PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                PartialEq::eq(&self[..], &other[..])
            }
        }
    };
}

impl_eq! { AvmString<'_>, str }
impl_eq! { AvmString<'_>, &'a str }
impl_eq! { AvmString<'_>, String }
