use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::object::kind;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, TObject};
use crate::fte::{
    CffHintingValue, FontLookupValue, FontPostureValue, FontWeightValue, RenderingModeValue,
};
use crate::string::AvmString;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::lock::Lock;
use gc_arena::{Collect, Gc, Mutation};
use ruffle_common::utils::HasPrefixField;
use ruffle_macros::istr;
use std::cell::Cell;

pub fn font_description_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    Ok(FontDescriptionObject(Gc::new(
        activation.gc(),
        FontDescriptionObjectData {
            base: ScriptObjectData::new(class),
            font_name: Lock::new(istr!("")),
            font_weight: Cell::new(FontWeightValue::Normal),
            font_posture: Cell::new(FontPostureValue::Normal),
            font_lookup: Cell::new(FontLookupValue::Device),
            rendering_mode: Cell::new(RenderingModeValue::Cff),
            cff_hinting: Cell::new(CffHintingValue::HorizontalStem),
            locked: Cell::new(false),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct FontDescriptionObject<'gc>(pub Gc<'gc, FontDescriptionObjectData<'gc>>);

impl fmt::Debug for FontDescriptionObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FontDescriptionObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct FontDescriptionObjectData<'gc> {
    base: ScriptObjectData<'gc, kind::FontDescriptionObject>,
    font_name: Lock<AvmString<'gc>>,
    font_weight: Cell<FontWeightValue>,
    font_posture: Cell<FontPostureValue>,
    font_lookup: Cell<FontLookupValue>,
    rendering_mode: Cell<RenderingModeValue>,
    cff_hinting: Cell<CffHintingValue>,
    locked: Cell<bool>,
}

impl<'gc> FontDescriptionObject<'gc> {
    pub fn font_name(self) -> AvmString<'gc> {
        self.0.font_name.get()
    }

    pub fn set_font_name(self, value: AvmString<'gc>, mc: &Mutation<'gc>) {
        unlock!(Gc::write(mc, self.0), FontDescriptionObjectData, font_name).set(value);
    }

    pub fn font_weight(self) -> FontWeightValue {
        self.0.font_weight.get()
    }

    pub fn set_font_weight(self, value: FontWeightValue) {
        self.0.font_weight.set(value);
    }

    pub fn font_posture(self) -> FontPostureValue {
        self.0.font_posture.get()
    }

    pub fn set_font_posture(self, value: FontPostureValue) {
        self.0.font_posture.set(value);
    }

    pub fn font_lookup(self) -> FontLookupValue {
        self.0.font_lookup.get()
    }

    pub fn set_font_lookup(self, value: FontLookupValue) {
        self.0.font_lookup.set(value);
    }

    pub fn rendering_mode(self) -> RenderingModeValue {
        self.0.rendering_mode.get()
    }

    pub fn set_rendering_mode(self, value: RenderingModeValue) {
        self.0.rendering_mode.set(value);
    }

    pub fn cff_hinting(self) -> CffHintingValue {
        self.0.cff_hinting.get()
    }

    pub fn set_cff_hinting(self, value: CffHintingValue) {
        self.0.cff_hinting.set(value);
    }

    pub fn locked(self) -> bool {
        self.0.locked.get()
    }

    pub fn set_locked(self, value: bool) {
        self.0.locked.set(value);
    }
}

impl<'gc> TObject<'gc> for FontDescriptionObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        ScriptObjectData::erase_kind(HasPrefixField::as_prefix_gc(self.0))
    }
}
