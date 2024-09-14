use std::borrow::Cow;
use std::ops::Deref;

use gc_arena::{Collect, Gc, Mutation};
use ruffle_wstr::{wstr_impl_traits, WStr, WString};

use crate::string::{AvmAtom, AvmStringRepr};

#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
enum Source<'gc> {
    Managed(Gc<'gc, AvmStringRepr<'gc>>),
    Static(&'static WStr),
}

#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct AvmString<'gc> {
    source: Source<'gc>,
}

impl<'gc> AvmString<'gc> {
    /// Turns a string to a fully owned (non-dependent) managed string.
    pub(super) fn to_fully_owned(self, mc: &Mutation<'gc>) -> Gc<'gc, AvmStringRepr<'gc>> {
        match self.source {
            Source::Managed(s) => {
                if s.is_dependent() {
                    let repr = AvmStringRepr::from_raw(WString::from(self.as_wstr()), false);
                    Gc::new(mc, repr)
                } else {
                    s
                }
            }
            Source::Static(s) => {
                let repr = AvmStringRepr::from_raw_static(s, false);
                Gc::new(mc, repr)
            }
        }
    }

    pub fn as_managed(self) -> Option<Gc<'gc, AvmStringRepr<'gc>>> {
        match self.source {
            Source::Managed(s) => Some(s),
            Source::Static(_) => None,
        }
    }

    pub fn new_utf8<'s, S: Into<Cow<'s, str>>>(gc_context: &Mutation<'gc>, string: S) -> Self {
        let buf = match string.into() {
            Cow::Owned(utf8) => WString::from_utf8_owned(utf8),
            Cow::Borrowed(utf8) => WString::from_utf8(utf8),
        };
        let repr = AvmStringRepr::from_raw(buf, false);
        Self {
            source: Source::Managed(Gc::new(gc_context, repr)),
        }
    }

    pub fn new_utf8_bytes(gc_context: &Mutation<'gc>, bytes: &[u8]) -> Self {
        let buf = WString::from_utf8_bytes(bytes.to_vec());
        Self::new(gc_context, buf)
    }

    pub fn new<S: Into<WString>>(gc_context: &Mutation<'gc>, string: S) -> Self {
        let repr = AvmStringRepr::from_raw(string.into(), false);
        Self {
            source: Source::Managed(Gc::new(gc_context, repr)),
        }
    }

    pub fn substring(mc: &Mutation<'gc>, string: AvmString<'gc>, start: usize, end: usize) -> Self {
        match string.source {
            Source::Managed(repr) => {
                let repr = AvmStringRepr::new_dependent(repr, start, end);
                Self {
                    source: Source::Managed(Gc::new(mc, repr)),
                }
            }
            Source::Static(s) => Self {
                source: Source::Static(&s[start..end]),
            },
        }
    }

    pub fn is_dependent(&self) -> bool {
        match &self.source {
            Source::Managed(s) => s.is_dependent(),
            Source::Static(_) => false,
        }
    }

    pub fn as_wstr(&self) -> &WStr {
        match &self.source {
            Source::Managed(s) => s,
            Source::Static(s) => s,
        }
    }

    pub fn as_interned(&self) -> Option<AvmAtom<'gc>> {
        match self.source {
            Source::Managed(s) if s.is_interned() => Some(AvmAtom(s)),
            _ => None,
        }
    }

    pub fn concat(
        mc: &Mutation<'gc>,
        left: AvmString<'gc>,
        right: AvmString<'gc>,
    ) -> AvmString<'gc> {
        if left.is_empty() {
            right
        } else if right.is_empty() {
            left
        } else if let Some(repr) = left
            .as_managed()
            .and_then(|l| AvmStringRepr::try_append_inline(l, &right))
        {
            Self {
                source: Source::Managed(Gc::new(mc, repr)),
            }
        } else {
            // When doing a non-in-place append,
            // Overallocate a bit so that further appends can be in-place.
            // (Note that this means that all first-time appends will happen here and
            // overallocate, even if done only once)
            // This growth logic should be equivalent to AVM's, except I capped the growth at 1MB instead of 4MB.
            let new_size = left.len() + right.len();
            let new_capacity = if new_size < 32 {
                32
            } else if new_size > 1024 * 1024 {
                new_size + 1024 * 1024
            } else {
                new_size * 2
            };

            let mut out = WString::with_capacity(new_capacity, left.is_wide() || right.is_wide());
            out.push_str(&left);
            out.push_str(&right);
            Self::new(mc, out)
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
            source: Source::Managed(atom.0),
        }
    }
}

impl<'gc> From<Gc<'gc, AvmStringRepr<'gc>>> for AvmString<'gc> {
    #[inline]
    fn from(repr: Gc<'gc, AvmStringRepr<'gc>>) -> Self {
        Self {
            source: Source::Managed(repr),
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
        if let (Source::Managed(left), Source::Managed(right)) = (self.source, other.source) {
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
