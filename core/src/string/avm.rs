use gc_arena::{Collect, Gc, MutationContext};
use std::ops::Deref;

use super::{BorrowWStr, WStr, WString};

#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
enum Source<'gc> {
    // Store the string both in UTF8 and UCS2, to be able to have
    // both `impl Deref<&str>` and O(1) UCS2 char access.
    // TODO(moulins): remove the extra `String`
    Owned(Gc<'gc, (String, WString)>),
    // Should be an ASCII string, for zero-copy conversion into `Str<'_>`.
    Static(&'static str),
}

#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct AvmString<'gc> {
    source: Source<'gc>,
}

impl<'gc> AvmString<'gc> {
    pub fn new<S: Into<String>>(gc_context: MutationContext<'gc, '_>, string: S) -> Self {
        let utf8 = string.into();
        let buf = WString::from_utf8(&utf8);
        Self {
            source: Source::Owned(Gc::allocate(gc_context, (utf8, buf))),
        }
    }

    pub fn new_ucs2(gc_context: MutationContext<'gc, '_>, string: WString) -> Self {
        // TODO(moulins): this loose unpaired surrogates
        let utf8 = string.to_string();
        Self {
            source: Source::Owned(Gc::allocate(gc_context, (utf8, string))),
        }
    }

    pub fn as_str(&self) -> &str {
        self
    }

    pub fn as_ucs2(&self) -> WStr<'_> {
        match &self.source {
            Source::Owned(str) => str.1.borrow(),
            // `str` is valid ASCII, per invariant.
            Source::Static(str) => WStr::from_units(str.as_bytes()),
        }
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
        // TODO(moulins): actually check that `str` is valid ASCII.
        Self {
            source: Source::Static(str),
        }
    }
}

impl Deref for AvmString<'_> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        match &self.source {
            Source::Owned(str) => str.0.deref(),
            Source::Static(str) => str,
        }
    }
}

impl AsRef<str> for AvmString<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        match &self.source {
            Source::Owned(str) => &str.0,
            Source::Static(str) => str,
        }
    }
}

impl<'gc> BorrowWStr for AvmString<'gc> {
    #[inline]
    fn borrow(&self) -> WStr<'_> {
        self.as_ucs2()
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
