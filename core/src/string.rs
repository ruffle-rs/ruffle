pub use ruffle_wstr::*;

use std::ops::Deref;

use gc_arena::{Collect, Gc, MutationContext};
use std::borrow::Cow;

#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
enum Source<'gc> {
    Owned(Gc<'gc, OwnedWStr>),
    Static(&'static WStr),
}

#[derive(Collect)]
#[collect(require_static)]
struct OwnedWStr(WString);

#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct AvmString<'gc> {
    source: Source<'gc>,
}

impl<'gc> AvmString<'gc> {
    pub fn new_utf8<'s, S: Into<Cow<'s, str>>>(
        gc_context: MutationContext<'gc, '_>,
        string: S,
    ) -> Self {
        let buf = match string.into() {
            Cow::Owned(utf8) => WString::from_utf8_owned(utf8),
            Cow::Borrowed(utf8) => WString::from_utf8(utf8),
        };
        Self {
            source: Source::Owned(Gc::allocate(gc_context, OwnedWStr(buf))),
        }
    }

    pub fn new_utf8_bytes(gc_context: MutationContext<'gc, '_>, bytes: &[u8]) -> Self {
        let buf = WString::from_utf8_bytes(bytes.to_vec());
        Self {
            source: Source::Owned(Gc::allocate(gc_context, OwnedWStr(buf))),
        }
    }

    pub fn new<S: Into<WString>>(gc_context: MutationContext<'gc, '_>, string: S) -> Self {
        Self {
            source: Source::Owned(Gc::allocate(gc_context, OwnedWStr(string.into()))),
        }
    }

    pub fn as_wstr(&self) -> &WStr {
        match &self.source {
            Source::Owned(s) => &s.0,
            Source::Static(s) => s,
        }
    }

    pub fn concat(
        gc_context: MutationContext<'gc, '_>,
        left: AvmString<'gc>,
        right: AvmString<'gc>,
    ) -> AvmString<'gc> {
        if left.is_empty() {
            right
        } else if right.is_empty() {
            left
        } else {
            let mut out = WString::from(left.as_wstr());
            out.push_str(&right);
            Self::new(gc_context, out)
        }
    }

    #[inline]
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        match (this.source, other.source) {
            (Source::Owned(this), Source::Owned(other)) => Gc::ptr_eq(this, other),
            (Source::Static(this), Source::Static(other)) => std::ptr::eq(this, other),
            _ => false,
        }
    }
}

impl Default for AvmString<'_> {
    fn default() -> Self {
        Self {
            source: Source::Static(WStr::empty()),
        }
    }
}

impl<'gc> From<&'static str> for AvmString<'gc> {
    #[inline]
    fn from(str: &'static str) -> Self {
        // TODO(moulins): actually check that `str` is valid ASCII.
        Self {
            source: Source::Static(WStr::from_units(str.as_bytes())),
        }
    }
}

impl<'gc> From<&'static WStr> for AvmString<'gc> {
    #[inline]
    fn from(str: &'static WStr) -> Self {
        Self {
            source: Source::Static(str),
        }
    }
}

impl<'gc> Deref for AvmString<'gc> {
    type Target = WStr;
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_wstr()
    }
}

wstr_impl_traits!(impl['gc] for AvmString<'gc>);
