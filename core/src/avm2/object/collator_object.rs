use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, TObject};
use crate::string::AvmString;
use bitflags::bitflags;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::lock::Lock;
use gc_arena::{Collect, Gc, GcWeak, Mutation};
use ruffle_common::utils::HasPrefixField;
use ruffle_macros::{Avm2Enum, istr};
use std::cell::Cell;

/// A class instance allocator that allocates Collator objects.
pub fn collator_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(CollatorObject(Gc::new(
        activation.gc(),
        CollatorObjectData {
            base,
            requested_locale_id_name: Lock::new(istr!("")),
            options: Cell::new(CollatorOptions::default()),
            last_operation_status: Cell::new(LastOperationStatus::NoError),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct CollatorObject<'gc>(pub Gc<'gc, CollatorObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct CollatorObjectWeak<'gc>(pub GcWeak<'gc, CollatorObjectData<'gc>>);

impl fmt::Debug for CollatorObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CollatorObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct CollatorObjectData<'gc> {
    /// Base script object.
    base: ScriptObjectData<'gc>,

    requested_locale_id_name: Lock<AvmString<'gc>>,

    options: Cell<CollatorOptions>,

    last_operation_status: Cell<LastOperationStatus>,
}

bitflags! {
    /// Collation options that can be set on a Collator.
    #[derive(Clone, Copy, Default)]
    pub struct CollatorOptions: u8 {
        const IGNORE_CASE            = 1 << 0;
        const IGNORE_CHARACTER_WIDTH = 1 << 1;
        const IGNORE_DIACRITICS      = 1 << 2;
        const IGNORE_KANA_TYPE       = 1 << 3;
        const IGNORE_SYMBOLS         = 1 << 4;
        const NUMERIC_COMPARISON     = 1 << 5;
    }
}

impl CollatorOptions {
    /// Options used by `CollatorMode.MATCHING`.
    pub fn matching() -> Self {
        Self::IGNORE_CASE
            | Self::IGNORE_CHARACTER_WIDTH
            | Self::IGNORE_DIACRITICS
            | Self::IGNORE_KANA_TYPE
            | Self::IGNORE_SYMBOLS
    }
}

/// Values of `flash.globalization.LastOperationStatus`.
#[derive(Clone, Copy, Avm2Enum)]
pub enum LastOperationStatus {
    #[avm2_variant("noError")]
    NoError,
    #[avm2_variant("unsupportedError")]
    UnsupportedError,
}

impl<'gc> CollatorObject<'gc> {
    pub fn requested_locale_id_name(self) -> AvmString<'gc> {
        self.0.requested_locale_id_name.get()
    }

    pub fn set_requested_locale_id_name(self, value: AvmString<'gc>, mc: &Mutation<'gc>) {
        unlock!(
            Gc::write(mc, self.0),
            CollatorObjectData,
            requested_locale_id_name
        )
        .set(value);
    }

    pub fn options(self) -> CollatorOptions {
        self.0.options.get()
    }

    pub fn set_options(self, options: CollatorOptions) {
        self.0.options.set(options);
    }

    pub fn set_option(self, option: CollatorOptions, value: bool) {
        let mut options = self.0.options.get();
        options.set(option, value);
        self.0.options.set(options);
    }

    pub fn last_operation_status(self) -> LastOperationStatus {
        self.0.last_operation_status.get()
    }

    pub fn set_last_operation_status(self, status: LastOperationStatus) {
        self.0.last_operation_status.set(status);
    }
}

impl<'gc> TObject<'gc> for CollatorObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}
