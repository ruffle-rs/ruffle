use std::borrow::Cow;
use std::ops::Deref;

use gc_arena::{Collect, Gc, Mutation};
use ruffle_wstr::{wstr_impl_traits, WStr, WString};

use crate::string::{AvmAtom, AvmStringRepr};

#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
enum Source<'gc> {
    Owned(Gc<'gc, AvmStringRepr>),
    Static(&'static WStr),
}

#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct AvmString<'gc> {
    source: Source<'gc>,
}

impl<'gc> AvmString<'gc> {
    pub(super) fn to_owned(self, gc_context: &Mutation<'gc>) -> Gc<'gc, AvmStringRepr> {
        match self.source {
            Source::Owned(s) => s,
            Source::Static(s) => {
                let repr = AvmStringRepr::from_raw(s.into(), false);
                Gc::new(gc_context, repr)
            }
        }
    }

    pub fn new_utf8<'s, S: Into<Cow<'s, str>>>(gc_context: &Mutation<'gc>, string: S) -> Self {
        let buf = match string.into() {
            Cow::Owned(utf8) => WString::from_utf8_owned(utf8),
            Cow::Borrowed(utf8) => WString::from_utf8(utf8),
        };
        let repr = AvmStringRepr::from_raw(buf, false);
        Self {
            source: Source::Owned(Gc::new(gc_context, repr)),
        }
    }

    pub fn new_utf8_bytes(gc_context: &Mutation<'gc>, bytes: &[u8]) -> Self {
        let buf = WString::from_utf8_bytes(bytes.to_vec());
        Self::new(gc_context, buf)
    }

    pub fn new<S: Into<WString>>(gc_context: &Mutation<'gc>, string: S) -> Self {
        let repr = AvmStringRepr::from_raw(string.into(), false);
        Self {
            source: Source::Owned(Gc::new(gc_context, repr)),
        }
    }

    pub fn as_wstr(&self) -> &WStr {
        match &self.source {
            Source::Owned(s) => s,
            Source::Static(s) => s,
        }
    }

    pub fn as_interned(&self) -> Option<AvmAtom<'gc>> {
        match self.source {
            Source::Owned(s) if s.is_interned() => Some(AvmAtom(s)),
            _ => None,
        }
    }

    pub fn concat(
        gc_context: &Mutation<'gc>,
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
        std::ptr::eq(this.as_wstr(), other.as_wstr())
    }
}

impl<'gc> From<AvmAtom<'gc>> for AvmString<'gc> {
    #[inline]
    fn from(atom: AvmAtom<'gc>) -> Self {
        Self {
            source: Source::Owned(atom.0),
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

// Manual equality implementation with fast paths for owned strings.
impl<'gc> PartialEq for AvmString<'gc> {
    fn eq(&self, other: &Self) -> bool {
        if let (Source::Owned(left), Source::Owned(right)) = (self.source, other.source) {
            // Fast accept for identical strings.
            if Gc::ptr_eq(left, right) {
                return true;
            // Fast reject for distinct interned strings.
            } else if left.is_interned() && right.is_interned() {
                return false;
            }
        }

        // Fallback case.
        self.as_wstr() == other.as_wstr()
    }
}

impl<'gc> PartialEq<AvmString<'gc>> for AvmAtom<'gc> {
    fn eq(&self, other: &AvmString<'gc>) -> bool {
        if let Some(atom) = other.as_interned() {
            *self == atom
        } else {
            self.as_wstr() == other.as_wstr()
        }
    }
}

impl<'gc> PartialEq<AvmAtom<'gc>> for AvmString<'gc> {
    #[inline(always)]
    fn eq(&self, other: &AvmAtom<'gc>) -> bool {
        PartialEq::eq(other, self)
    }
}

impl<'gc> Eq for AvmString<'gc> {}

wstr_impl_traits!(impl['gc] manual_eq for AvmString<'gc>);
