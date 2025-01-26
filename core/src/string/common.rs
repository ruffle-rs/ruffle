use super::{AvmAtom, AvmStringInterner, AvmStringRepr, WStr};
use gc_arena::{Gc, Mutation};

pub struct CommonStrings<'gc> {
    pub str_x: AvmAtom<'gc>,
    pub str_y: AvmAtom<'gc>,
    pub str_width: AvmAtom<'gc>,
    pub str_height: AvmAtom<'gc>,
}

impl<'gc> CommonStrings<'gc> {
    pub fn new(mc: &Mutation<'gc>, interner: &mut AvmStringInterner<'gc>) -> Self {
        let mut intern_from_static = |s: &'static [u8]| {
            let wstr = WStr::from_units(s);
            interner.intern_inner(mc, wstr, |wstr| {
                let repr = AvmStringRepr::from_raw_static(wstr, true);
                Gc::new(mc, repr)
            })
        };

        Self {
            str_x: intern_from_static(b"x"),
            str_y: intern_from_static(b"y"),
            str_width: intern_from_static(b"width"),
            str_height: intern_from_static(b"height"),
        }
    }
}
