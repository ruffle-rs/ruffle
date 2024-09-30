use std::{borrow::Cow, ops::Range};

use gc_arena::{Gc, Mutation};

use super::{AvmAtom, AvmString, AvmStringInterner, AvmStringRepr, WStr, WString};

/// Context for managing `AvmString`s: allocating them, interning them, etc...
pub struct StringContext<'gc> {
    /// The mutation context to allocate and mutate `Gc` pointers.
    pub gc_context: &'gc Mutation<'gc>,

    /// The global string interner.
    interner: &'gc mut AvmStringInterner<'gc>,
}

impl<'gc> StringContext<'gc> {
    pub fn from_parts(
        gc_context: &'gc Mutation<'gc>,
        interner: &'gc mut AvmStringInterner<'gc>,
    ) -> Self {
        Self {
            gc_context,
            interner,
        }
    }

    #[inline(always)]
    pub fn gc(&self) -> &'gc Mutation<'gc> {
        self.gc_context
    }

    #[must_use]
    pub fn intern_wstr<'a, S>(&mut self, s: S) -> AvmAtom<'gc>
    where
        S: Into<Cow<'a, WStr>>,
    {
        let s = s.into();
        let mc = self.gc();
        self.interner.intern_inner(mc, s, |s| {
            let repr = AvmStringRepr::from_raw(s.into_owned(), true);
            Gc::new(mc, repr)
        })
    }

    #[must_use]
    pub fn intern_static(&mut self, s: &'static WStr) -> AvmAtom<'gc> {
        let mc = self.gc();
        self.interner.intern_inner(mc, s, |s| {
            let repr = AvmStringRepr::from_raw_static(s, true);
            Gc::new(mc, repr)
        })
    }

    #[must_use]
    pub fn intern(&mut self, s: AvmString<'gc>) -> AvmAtom<'gc> {
        if let Some(atom) = s.as_interned() {
            atom
        } else {
            let mc = self.gc();
            self.interner.intern_inner(mc, s, |s| {
                let repr = s.to_fully_owned(mc);
                repr.mark_interned();
                repr
            })
        }
    }

    #[must_use]
    pub fn get_interned(&self, s: &WStr) -> Option<AvmAtom<'gc>> {
        self.interner.get(self.gc(), s)
    }

    #[must_use]
    pub fn empty(&self) -> AvmString<'gc> {
        self.interner.empty.into()
    }

    #[must_use]
    pub fn make_char(&self, c: u16) -> AvmString<'gc> {
        if let Some(s) = self.interner.chars.get(c as usize) {
            (*s).into()
        } else {
            AvmString::new(self.gc(), WString::from_unit(c))
        }
    }

    /// Like `make_char`, but panics if the passed char is not ASCII.
    #[must_use]
    pub fn ascii_char(&self, c: u8) -> AvmString<'gc> {
        self.interner.chars[c as usize].into()
    }

    #[must_use]
    pub fn substring(&self, s: AvmString<'gc>, range: Range<usize>) -> AvmString<'gc> {
        self.interner
            .substring(self.gc(), s, range.start, range.end)
    }
}
