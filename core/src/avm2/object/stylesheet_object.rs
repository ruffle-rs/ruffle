use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::Error;
use crate::html::{StyleSheet, TextFormat};
use core::fmt;
use gc_arena::{Collect, Gc, GcWeak};
use ruffle_wstr::{WStr, WString};

/// A class instance allocator that allocates StyleSheet objects.
pub fn style_sheet_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(StyleSheetObject(Gc::new(
        activation.gc(),
        StyleSheetObjectData {
            base,
            style_sheet: StyleSheet::new(activation.gc()),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct StyleSheetObject<'gc>(pub Gc<'gc, StyleSheetObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct StyleSheetObjectWeak<'gc>(pub GcWeak<'gc, StyleSheetObjectData<'gc>>);

impl fmt::Debug for StyleSheetObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StyleSheetObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .field("style_sheet", &self.0.style_sheet)
            .finish()
    }
}

#[derive(Collect)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct StyleSheetObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    style_sheet: StyleSheet<'gc>,
}

const _: () = assert!(std::mem::offset_of!(StyleSheetObjectData, base) == 0);
const _: () = assert!(
    std::mem::align_of::<StyleSheetObjectData>() == std::mem::align_of::<ScriptObjectData>()
);

impl<'gc> TObject<'gc> for StyleSheetObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        // SAFETY: Object data is repr(C), and a compile-time assert ensures
        // that the ScriptObjectData stays at offset 0 of the struct- so the
        // layouts are compatible

        unsafe { Gc::cast(self.0) }
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn as_style_sheet(&self) -> Option<StyleSheetObject<'gc>> {
        Some(*self)
    }
}

impl<'gc> StyleSheetObject<'gc> {
    pub fn set_style(self, selector: WString, format: TextFormat) {
        self.0.style_sheet.set_style(selector, format);
    }

    pub fn remove_style(self, selector: &WStr) {
        self.0.style_sheet.remove_style(selector);
    }

    pub fn clear(self) {
        self.0.style_sheet.clear();
    }

    pub fn style_sheet(self) -> StyleSheet<'gc> {
        self.0.style_sheet
    }
}
