use gc_arena::{Collect, Gc, MutationContext};
use std::fmt;
use std::ops::Deref;

#[derive(Debug, Clone, Collect)]
#[collect(no_drop)]
enum Source<'gc> {
    Owned(Gc<'gc, String>),
    Static(&'static str),
}

#[derive(Debug, Clone, Collect)]
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

macro_rules! impl_eq {
    ($lhs:ty, $rhs: ty) => {
        #[allow(unused_lifetimes)]
        impl<'a, 'b> PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                PartialEq::eq(&self[..], &other[..])
            }
        }

        #[allow(unused_lifetimes)]
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
