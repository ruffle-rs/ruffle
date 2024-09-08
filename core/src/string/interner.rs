use core::fmt;
use std::borrow::Cow;
use std::hash::{Hash, Hasher};

use gc_arena::{Collect, Gc, Mutation};

use crate::string::{AvmString, AvmStringRepr, WStr, WString};
use crate::utils::weak_set::WeakSet;

// An interned `AvmString`, with fast by-pointer equality and hashing.
#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct AvmAtom<'gc>(pub(super) Gc<'gc, AvmStringRepr<'gc>>);

impl<'gc> PartialEq for AvmAtom<'gc> {
    fn eq(&self, other: &Self) -> bool {
        Gc::ptr_eq(self.0, other.0)
    }
}

impl<'gc> Eq for AvmAtom<'gc> {}

impl<'gc> Hash for AvmAtom<'gc> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Gc::as_ptr(self.0).hash(state);
    }
}

impl<'gc> fmt::Debug for AvmAtom<'gc> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_wstr(), f)
    }
}

impl<'gc> fmt::Display for AvmAtom<'gc> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_wstr(), f)
    }
}

impl<'gc> AvmAtom<'gc> {
    pub fn as_wstr(&self) -> &WStr {
        &self.0
    }
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct AvmStringInterner<'gc> {
    interned: WeakSet<'gc, AvmStringRepr<'gc>>,

    empty: Gc<'gc, AvmStringRepr<'gc>>,
    chars: [Gc<'gc, AvmStringRepr<'gc>>; INTERNED_CHAR_LEN],
}

const INTERNED_CHAR_LEN: usize = 128;
static INTERNED_CHARS: [u8; INTERNED_CHAR_LEN] = {
    let mut chs = [0; INTERNED_CHAR_LEN];
    let mut i = 0;
    while i < chs.len() {
        chs[i] = i as u8;
        i += 1;
    }
    chs
};

impl<'gc> AvmStringInterner<'gc> {
    pub fn new(mc: &Mutation<'gc>) -> Self {
        let mut interned = WeakSet::new();

        // We can't use `Self::intern_static` because we don't have a Self yet.
        let mut intern_from_static = |s: &'static [u8]| {
            let wstr = WStr::from_units(s);
            let repr = AvmStringRepr::from_raw_static(wstr, true);
            interned.insert_unique_unchecked(mc, Gc::new(mc, repr))
        };

        Self {
            empty: intern_from_static(b""),
            chars: std::array::from_fn(|i| {
                let c = &INTERNED_CHARS[i];
                intern_from_static(std::slice::from_ref(c))
            }),
            interned,
        }
    }

    #[must_use]
    pub fn intern_wstr<'a, S>(&mut self, mc: &Mutation<'gc>, s: S) -> AvmAtom<'gc>
    where
        S: Into<Cow<'a, WStr>>,
    {
        let s = s.into();
        let entry = self.interned.entry(mc, &*s).or_insert(|| {
            let repr = AvmStringRepr::from_raw(s.into_owned(), true);
            Gc::new(mc, repr)
        });
        AvmAtom(entry.get())
    }

    #[must_use]
    pub fn intern_static(&mut self, mc: &Mutation<'gc>, s: &'static WStr) -> AvmAtom<'gc> {
        let entry = self.interned.entry(mc, s).or_insert(|| {
            let repr = AvmStringRepr::from_raw_static(s, true);
            Gc::new(mc, repr)
        });
        AvmAtom(entry.get())
    }

    #[must_use]
    pub fn intern(&mut self, mc: &Mutation<'gc>, s: AvmString<'gc>) -> AvmAtom<'gc> {
        if let Some(atom) = s.as_interned() {
            atom
        } else {
            let entry = self.interned.entry(mc, s.as_wstr()).or_insert(|| {
                let repr = s.to_fully_owned(mc);
                repr.mark_interned();
                repr
            });
            AvmAtom(entry.get())
        }
    }

    #[must_use]
    pub fn empty(&self) -> AvmString<'gc> {
        self.empty.into()
    }

    #[must_use]
    pub fn get(&self, mc: &Mutation<'gc>, s: &WStr) -> Option<AvmAtom<'gc>> {
        self.interned.get(mc, s).map(AvmAtom)
    }

    #[must_use]
    pub fn get_char(&self, mc: &Mutation<'gc>, c: u16) -> AvmString<'gc> {
        if let Some(s) = self.chars.get(c as usize) {
            (*s).into()
        } else {
            AvmString::new(mc, WString::from_unit(c))
        }
    }

    // Like get_char, but panics if the passed char is not ASCII.
    #[must_use]
    pub fn get_ascii_char(&self, c: char) -> AvmString<'gc> {
        self.chars[c as usize].into()
    }

    #[must_use]
    pub fn substring(
        &self,
        mc: &Mutation<'gc>,
        s: AvmString<'gc>,
        start_index: usize,
        end_index: usize,
    ) -> AvmString<'gc> {
        // TODO: return original string if full range

        // It's assumed that start<=end. This is tested later via a range check.
        if start_index == end_index {
            return self.empty.into();
        }
        if end_index == start_index + 1 {
            if let Some(c) = s.get(start_index) {
                if let Some(s) = self.chars.get(c as usize) {
                    return (*s).into();
                }
            }
        }
        AvmString::substring(mc, s, start_index, end_index)
    }
}
