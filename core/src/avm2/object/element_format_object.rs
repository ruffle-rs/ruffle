use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::object::font_description_object::FontDescriptionObject;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, TObject};
use crate::fte::{
    BreakOpportunityValue, DigitCaseValue, DigitWidthValue, KerningValue, LigatureLevelValue,
    TextBaselineValue, TextRotationValue, TypographicCaseValue,
};
use crate::string::AvmString;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::lock::Lock;
use gc_arena::{Collect, Gc, GcWeak, Mutation};
use ruffle_common::utils::HasPrefixField;
use ruffle_macros::istr;
use std::cell::Cell;

pub fn element_format_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    Ok(ElementFormatObject(Gc::new(
        activation.gc(),
        ElementFormatObjectData {
            base: ScriptObjectData::new(class),
            alignment_baseline: Cell::new(TextBaselineValue::UseDominantBaseline),
            alpha: Cell::new(1.0),
            baseline_shift: Cell::new(0.0),
            break_opportunity: Cell::new(BreakOpportunityValue::Auto),
            color: Cell::new(swf::Color::BLACK),
            digit_case: Cell::new(DigitCaseValue::Default),
            digit_width: Cell::new(DigitWidthValue::Default),
            dominant_baseline: Cell::new(TextBaselineValue::Roman),
            font_description: Lock::new(None),
            font_size: Cell::new(12.0),
            kerning: Cell::new(KerningValue::On),
            ligature_level: Cell::new(LigatureLevelValue::Common),
            locale: Lock::new(istr!("en")),
            text_rotation: Cell::new(TextRotationValue::Auto),
            tracking_left: Cell::new(0.0),
            tracking_right: Cell::new(0.0),
            typographic_case: Cell::new(TypographicCaseValue::Default),
            locked: Cell::new(false),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct ElementFormatObject<'gc>(pub Gc<'gc, ElementFormatObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct ElementFormatObjectWeak<'gc>(pub GcWeak<'gc, ElementFormatObjectData<'gc>>);

impl fmt::Debug for ElementFormatObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ElementFormatObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct ElementFormatObjectData<'gc> {
    base: ScriptObjectData<'gc>,
    alignment_baseline: Cell<TextBaselineValue>,
    alpha: Cell<f64>,
    baseline_shift: Cell<f64>,
    break_opportunity: Cell<BreakOpportunityValue>,
    color: Cell<swf::Color>,
    digit_case: Cell<DigitCaseValue>,
    digit_width: Cell<DigitWidthValue>,
    dominant_baseline: Cell<TextBaselineValue>,
    font_description: Lock<Option<FontDescriptionObject<'gc>>>,
    font_size: Cell<f64>,
    kerning: Cell<KerningValue>,
    ligature_level: Cell<LigatureLevelValue>,
    locale: Lock<AvmString<'gc>>,
    text_rotation: Cell<TextRotationValue>,
    tracking_left: Cell<f64>,
    tracking_right: Cell<f64>,
    typographic_case: Cell<TypographicCaseValue>,
    locked: Cell<bool>,
}

impl<'gc> ElementFormatObject<'gc> {
    pub fn alignment_baseline(self) -> TextBaselineValue {
        self.0.alignment_baseline.get()
    }

    pub fn set_alignment_baseline(self, value: TextBaselineValue) {
        self.0.alignment_baseline.set(value);
    }

    pub fn alpha(self) -> f64 {
        self.0.alpha.get()
    }

    pub fn set_alpha(self, value: f64) {
        let value = if value.is_nan() { 0.0 } else { value };
        self.0.alpha.set(value.clamp(0.0, 1.0));
    }

    pub fn baseline_shift(self) -> f64 {
        self.0.baseline_shift.get()
    }

    pub fn set_baseline_shift(self, value: f64) {
        self.0.baseline_shift.set(value);
    }

    pub fn break_opportunity(self) -> BreakOpportunityValue {
        self.0.break_opportunity.get()
    }

    pub fn set_break_opportunity(self, value: BreakOpportunityValue) {
        self.0.break_opportunity.set(value);
    }

    pub fn color(self) -> swf::Color {
        self.0.color.get()
    }

    pub fn set_color(self, value: swf::Color) {
        self.0.color.set(value);
    }

    pub fn digit_case(self) -> DigitCaseValue {
        self.0.digit_case.get()
    }

    pub fn set_digit_case(self, value: DigitCaseValue) {
        self.0.digit_case.set(value);
    }

    pub fn digit_width(self) -> DigitWidthValue {
        self.0.digit_width.get()
    }

    pub fn set_digit_width(self, value: DigitWidthValue) {
        self.0.digit_width.set(value);
    }

    pub fn dominant_baseline(self) -> TextBaselineValue {
        self.0.dominant_baseline.get()
    }

    pub fn set_dominant_baseline(self, value: TextBaselineValue) {
        self.0.dominant_baseline.set(value);
    }

    pub fn font_description(self) -> Option<FontDescriptionObject<'gc>> {
        self.0.font_description.get()
    }

    pub fn set_font_description(self, value: FontDescriptionObject<'gc>, mc: &Mutation<'gc>) {
        unlock!(
            Gc::write(mc, self.0),
            ElementFormatObjectData,
            font_description
        )
        .set(Some(value));
    }

    pub fn font_size(self) -> f64 {
        self.0.font_size.get()
    }

    pub fn set_font_size(self, value: f64) {
        self.0
            .font_size
            .set(if value.is_nan() { 0.0 } else { value });
    }

    pub fn kerning(self) -> KerningValue {
        self.0.kerning.get()
    }

    pub fn set_kerning(self, value: KerningValue) {
        self.0.kerning.set(value);
    }

    pub fn ligature_level(self) -> LigatureLevelValue {
        self.0.ligature_level.get()
    }

    pub fn set_ligature_level(self, value: LigatureLevelValue) {
        self.0.ligature_level.set(value);
    }

    pub fn locale(self) -> AvmString<'gc> {
        self.0.locale.get()
    }

    pub fn set_locale(self, value: AvmString<'gc>, mc: &Mutation<'gc>) {
        unlock!(Gc::write(mc, self.0), ElementFormatObjectData, locale).set(value);
    }

    pub fn text_rotation(self) -> TextRotationValue {
        self.0.text_rotation.get()
    }

    pub fn set_text_rotation(self, value: TextRotationValue) {
        self.0.text_rotation.set(value);
    }

    pub fn tracking_left(self) -> f64 {
        self.0.tracking_left.get()
    }

    pub fn set_tracking_left(self, value: f64) {
        self.0
            .tracking_left
            .set(if value.is_nan() { 0.0 } else { value });
    }

    pub fn tracking_right(self) -> f64 {
        self.0.tracking_right.get()
    }

    pub fn set_tracking_right(self, value: f64) {
        self.0
            .tracking_right
            .set(if value.is_nan() { 0.0 } else { value });
    }

    pub fn typographic_case(self) -> TypographicCaseValue {
        self.0.typographic_case.get()
    }

    pub fn set_typographic_case(self, value: TypographicCaseValue) {
        self.0.typographic_case.set(value);
    }

    pub fn locked(self) -> bool {
        self.0.locked.get()
    }

    pub fn set_locked(self, value: bool) {
        self.0.locked.set(value);
    }
}

impl<'gc> TObject<'gc> for ElementFormatObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}
