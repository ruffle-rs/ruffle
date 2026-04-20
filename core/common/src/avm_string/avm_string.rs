use super::interner::AvmAtom;
use super::repr::AvmStringRepr;

use gc_arena::{Collect, Gc, Mutation};
use ruffle_wstr::{WStr, WString, wstr_impl_traits};
use std::borrow::Cow;
use std::ops::Deref;

#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct AvmString<'gc>(Gc<'gc, AvmStringRepr<'gc>>);

impl<'gc> AvmString<'gc> {
    /// Turns a string to a fully owned (non-dependent) managed string.
    pub(super) fn to_fully_owned(self, mc: &Mutation<'gc>) -> Gc<'gc, AvmStringRepr<'gc>> {
        if self.0.is_dependent() {
            let repr = AvmStringRepr::from_raw(WString::from(self.as_wstr()), false);
            Gc::new(mc, repr)
        } else {
            self.0
        }
    }

    pub fn new_ascii_static(gc_context: &Mutation<'gc>, bytes: &'static [u8]) -> Self {
        let repr = AvmStringRepr::from_raw_static(WStr::from_units(bytes), false);
        Self(Gc::new(gc_context, repr))
    }

    pub fn new_utf8<'s, S: Into<Cow<'s, str>>>(gc_context: &Mutation<'gc>, string: S) -> Self {
        let buf = match string.into() {
            Cow::Owned(utf8) => WString::from_utf8_owned(utf8),
            Cow::Borrowed(utf8) => WString::from_utf8(utf8),
        };
        let repr = AvmStringRepr::from_raw(buf, false);
        Self(Gc::new(gc_context, repr))
    }

    pub fn new_utf8_bytes(gc_context: &Mutation<'gc>, bytes: &[u8]) -> Self {
        let buf = WString::from_utf8_bytes(bytes.to_vec());
        Self::new(gc_context, buf)
    }

    pub fn new<S: Into<WString>>(gc_context: &Mutation<'gc>, string: S) -> Self {
        let repr = AvmStringRepr::from_raw(string.into(), false);
        Self(Gc::new(gc_context, repr))
    }

    pub fn substring(mc: &Mutation<'gc>, string: AvmString<'gc>, start: usize, end: usize) -> Self {
        let repr = AvmStringRepr::new_dependent(string.0, start, end);
        Self(Gc::new(mc, repr))
    }

    pub fn is_dependent(&self) -> bool {
        self.0.is_dependent()
    }

    pub fn as_wstr(&self) -> &'gc WStr {
        Gc::as_ref(self.0).as_wstr()
    }

    pub fn as_interned(&self) -> Option<AvmAtom<'gc>> {
        if self.0.is_interned() {
            Some(AvmAtom(self.0))
        } else {
            None
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
        } else if let Some(repr) = AvmStringRepr::try_append_inline(left.0, &right) {
            Self(Gc::new(mc, repr))
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
        Self(atom.0)
    }
}

impl<'gc> From<Gc<'gc, AvmStringRepr<'gc>>> for AvmString<'gc> {
    #[inline]
    fn from(repr: Gc<'gc, AvmStringRepr<'gc>>) -> Self {
        Self(repr)
    }
}

impl Deref for AvmString<'_> {
    type Target = WStr;
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_wstr()
    }
}

// Manual equality implementation with fast paths for owned strings.
impl PartialEq for AvmString<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        if Gc::ptr_eq(self.0, other.0) {
            // Fast accept for identical strings.
            true
        } else if self.0.is_interned() && other.0.is_interned() {
            // Fast reject for distinct interned strings.
            false
        } else {
            // Fallback case.
            self.as_wstr() == other.as_wstr()
        }
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

impl Eq for AvmString<'_> {}

wstr_impl_traits!(impl['gc] manual_eq for AvmString<'gc>);
