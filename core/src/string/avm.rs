use gc_arena::{Collect, Gc, MutationContext};

use super::{BorrowWStr, WStr, WString};

#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
enum Source<'gc> {
    Owned(Gc<'gc, WString>),
    Static(WStr<'static>),
}

#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct AvmString<'gc> {
    source: Source<'gc>,
}

impl<'gc> AvmString<'gc> {
    pub fn new_utf8<S: Into<String>>(gc_context: MutationContext<'gc, '_>, string: S) -> Self {
        let utf8 = string.into();
        let buf = WString::from_utf8(&utf8);
        Self {
            source: Source::Owned(Gc::allocate(gc_context, buf)),
        }
    }

    pub fn new<S: Into<WString>>(gc_context: MutationContext<'gc, '_>, string: S) -> Self {
        Self {
            source: Source::Owned(Gc::allocate(gc_context, string.into())),
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
            let mut out = WString::from(left.borrow());
            out.push_str(right.borrow());
            Self::new(gc_context, out)
        }
    }

    #[inline]
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        match (this.source, other.source) {
            (Source::Owned(this), Source::Owned(other)) => Gc::ptr_eq(this, other),
            (Source::Static(this), Source::Static(other)) => this.to_ptr() == other.to_ptr(),
            _ => false,
        }
    }

    impl_str_methods! {
        lifetime: '_;
        self: &Self;
        deref: self.borrow();
        pattern['a,]: 'a, &'a Self;
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

impl<'gc> From<WStr<'static>> for AvmString<'gc> {
    #[inline]
    fn from(str: WStr<'static>) -> Self {
        Self {
            source: Source::Static(str),
        }
    }
}

impl<'gc> BorrowWStr for AvmString<'gc> {
    #[inline]
    fn borrow(&self) -> WStr<'_> {
        match &self.source {
            Source::Owned(s) => s.borrow(),
            Source::Static(s) => *s,
        }
    }
}
