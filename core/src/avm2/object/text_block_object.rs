use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, TObject, VectorObject};
use crate::fte::{TextBaselineValue, TextRotationValue};
use crate::string::AvmString;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::lock::Lock;
use gc_arena::{Collect, Gc, GcWeak, Mutation};
use ruffle_common::utils::HasPrefixField;
use std::cell::Cell;

pub fn text_block_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    Ok(TextBlockObject(Gc::new(
        activation.gc(),
        TextBlockObjectData {
            base: ScriptObjectData::new(class),
            apply_non_linear_font_scaling: Cell::new(false),
            baseline_font_description: Lock::new(None),
            baseline_font_size: Cell::new(12.0),
            baseline_zero: Cell::new(TextBaselineValue::Roman),
            bidi_level: Cell::new(0),
            line_rotation: Cell::new(TextRotationValue::Rotate0),
            tab_stops: Lock::new(None),
            text_justifier: Lock::new(None),
            content: Lock::new(None),
            text_line_creation_result: Lock::new(None),
            first_line: Lock::new(None),
            last_line: Lock::new(None),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct TextBlockObject<'gc>(pub Gc<'gc, TextBlockObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct TextBlockObjectWeak<'gc>(pub GcWeak<'gc, TextBlockObjectData<'gc>>);

impl fmt::Debug for TextBlockObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextBlockObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct TextBlockObjectData<'gc> {
    base: ScriptObjectData<'gc>,
    apply_non_linear_font_scaling: Cell<bool>,
    baseline_font_description: Lock<Option<Object<'gc>>>,
    baseline_font_size: Cell<f64>,
    baseline_zero: Cell<TextBaselineValue>,
    bidi_level: Cell<i32>,
    line_rotation: Cell<TextRotationValue>,
    tab_stops: Lock<Option<VectorObject<'gc>>>,
    text_justifier: Lock<Option<Object<'gc>>>,
    content: Lock<Option<Object<'gc>>>,
    text_line_creation_result: Lock<Option<AvmString<'gc>>>,
    first_line: Lock<Option<Object<'gc>>>,
    /// Last line broken out of this block. Set by the native line breaker;
    /// added on top of upstream's TextBlockObject for the FTE implementation.
    last_line: Lock<Option<Object<'gc>>>,
}

impl<'gc> TextBlockObject<'gc> {
    pub fn apply_non_linear_font_scaling(self) -> bool {
        self.0.apply_non_linear_font_scaling.get()
    }

    pub fn set_apply_non_linear_font_scaling(self, value: bool) {
        self.0.apply_non_linear_font_scaling.set(value);
    }

    pub fn baseline_font_description(self) -> Option<Object<'gc>> {
        self.0.baseline_font_description.get()
    }

    pub fn set_baseline_font_description(self, value: Option<Object<'gc>>, mc: &Mutation<'gc>) {
        unlock!(
            Gc::write(mc, self.0),
            TextBlockObjectData,
            baseline_font_description
        )
        .set(value);
    }

    pub fn baseline_font_size(self) -> f64 {
        self.0.baseline_font_size.get()
    }

    pub fn set_baseline_font_size(self, value: f64) {
        self.0.baseline_font_size.set(value);
    }

    pub fn baseline_zero(self) -> TextBaselineValue {
        self.0.baseline_zero.get()
    }

    pub fn set_baseline_zero(self, value: TextBaselineValue) {
        self.0.baseline_zero.set(value);
    }

    pub fn bidi_level(self) -> i32 {
        self.0.bidi_level.get()
    }

    pub fn set_bidi_level(self, value: i32) {
        self.0.bidi_level.set(value);
    }

    pub fn line_rotation(self) -> TextRotationValue {
        self.0.line_rotation.get()
    }

    pub fn set_line_rotation(self, value: TextRotationValue) {
        self.0.line_rotation.set(value);
    }

    pub fn tab_stops(self) -> Option<VectorObject<'gc>> {
        self.0.tab_stops.get()
    }

    pub fn set_tab_stops(self, value: Option<VectorObject<'gc>>, mc: &Mutation<'gc>) {
        unlock!(Gc::write(mc, self.0), TextBlockObjectData, tab_stops).set(value);
    }

    pub fn text_justifier(self) -> Option<Object<'gc>> {
        self.0.text_justifier.get()
    }

    pub fn set_text_justifier(self, value: Object<'gc>, mc: &Mutation<'gc>) {
        unlock!(Gc::write(mc, self.0), TextBlockObjectData, text_justifier).set(Some(value));
    }

    pub fn content(self) -> Option<Object<'gc>> {
        self.0.content.get()
    }

    pub fn set_content(self, value: Option<Object<'gc>>, mc: &Mutation<'gc>) {
        unlock!(Gc::write(mc, self.0), TextBlockObjectData, content).set(value);
    }

    pub fn text_line_creation_result(self) -> Option<AvmString<'gc>> {
        self.0.text_line_creation_result.get()
    }

    pub fn set_text_line_creation_result(self, value: Option<AvmString<'gc>>, mc: &Mutation<'gc>) {
        unlock!(
            Gc::write(mc, self.0),
            TextBlockObjectData,
            text_line_creation_result
        )
        .set(value);
    }

    pub fn first_line(self) -> Option<Object<'gc>> {
        self.0.first_line.get()
    }

    pub fn set_first_line(self, value: Option<Object<'gc>>, mc: &Mutation<'gc>) {
        unlock!(Gc::write(mc, self.0), TextBlockObjectData, first_line).set(value);
    }

    pub fn last_line(self) -> Option<Object<'gc>> {
        self.0.last_line.get()
    }

    pub fn set_last_line(self, value: Option<Object<'gc>>, mc: &Mutation<'gc>) {
        unlock!(Gc::write(mc, self.0), TextBlockObjectData, last_line).set(value);
    }
}

impl<'gc> TObject<'gc> for TextBlockObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}
