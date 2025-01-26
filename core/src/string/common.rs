use super::{AvmString, AvmStringInterner, AvmStringRepr, WStr};
use gc_arena::{Gc, Mutation};

#[allow(non_snake_case)]
pub struct CommonStrings<'gc> {
    pub str_boolean: AvmString<'gc>,
    pub str_callee: AvmString<'gc>,
    pub str_false: AvmString<'gc>,
    pub str_function: AvmString<'gc>,
    pub str_height: AvmString<'gc>,
    pub str_Infinity: AvmString<'gc>,
    pub str_NaN: AvmString<'gc>,
    pub str_null: AvmString<'gc>,
    pub str_number: AvmString<'gc>,
    pub str_object: AvmString<'gc>,
    pub str_string: AvmString<'gc>,
    pub str_toJSON: AvmString<'gc>,
    pub str_toString: AvmString<'gc>,
    pub str_true: AvmString<'gc>,
    pub str_undefined: AvmString<'gc>,
    pub str_valueOf: AvmString<'gc>,
    pub str_width: AvmString<'gc>,
    pub str_x: AvmString<'gc>,
    pub str_xml: AvmString<'gc>,
    pub str_y: AvmString<'gc>,
}

impl<'gc> CommonStrings<'gc> {
    pub fn new(mc: &Mutation<'gc>, interner: &mut AvmStringInterner<'gc>) -> Self {
        let mut intern_from_static = |s: &'static [u8]| {
            let wstr = WStr::from_units(s);

            let atom = interner.intern_inner(mc, wstr, |wstr| {
                let repr = AvmStringRepr::from_raw_static(wstr, true);
                Gc::new(mc, repr)
            });

            atom.into()
        };

        Self {
            str_boolean: intern_from_static(b"boolean"),
            str_callee: intern_from_static(b"callee"),
            str_false: intern_from_static(b"false"),
            str_function: intern_from_static(b"function"),
            str_height: intern_from_static(b"height"),
            str_Infinity: intern_from_static(b"Infinity"),
            str_NaN: intern_from_static(b"NaN"),
            str_null: intern_from_static(b"null"),
            str_number: intern_from_static(b"number"),
            str_object: intern_from_static(b"object"),
            str_string: intern_from_static(b"string"),
            str_toJSON: intern_from_static(b"toJSON"),
            str_toString: intern_from_static(b"toString"),
            str_true: intern_from_static(b"true"),
            str_undefined: intern_from_static(b"undefined"),
            str_valueOf: intern_from_static(b"valueOf"),
            str_width: intern_from_static(b"width"),
            str_x: intern_from_static(b"x"),
            str_xml: intern_from_static(b"xml"),
            str_y: intern_from_static(b"y"),
        }
    }
}
