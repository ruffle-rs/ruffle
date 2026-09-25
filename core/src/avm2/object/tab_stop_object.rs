use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, TObject};
use crate::fte::TabAlignmentValue;
use crate::string::AvmString;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::lock::Lock;
use gc_arena::{Collect, Gc, GcWeak, Mutation};
use ruffle_common::utils::HasPrefixField;
use ruffle_macros::istr;
use std::cell::Cell;

pub fn tab_stop_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    Ok(TabStopObject(Gc::new(
        activation.gc(),
        TabStopObjectData {
            base: ScriptObjectData::new(class),
            alignment: Cell::new(TabAlignmentValue::Start),
            position: Cell::new(0.0),
            decimal_alignment_token: Lock::new(istr!("")),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct TabStopObject<'gc>(pub Gc<'gc, TabStopObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct TabStopObjectWeak<'gc>(pub GcWeak<'gc, TabStopObjectData<'gc>>);

impl fmt::Debug for TabStopObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TabStopObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct TabStopObjectData<'gc> {
    base: ScriptObjectData<'gc>,
    alignment: Cell<TabAlignmentValue>,
    position: Cell<f64>,
    decimal_alignment_token: Lock<AvmString<'gc>>,
}

impl<'gc> TabStopObject<'gc> {
    pub fn alignment(self) -> TabAlignmentValue {
        self.0.alignment.get()
    }

    pub fn set_alignment(self, value: TabAlignmentValue) {
        self.0.alignment.set(value);
    }

    pub fn position(self) -> f64 {
        self.0.position.get()
    }

    pub fn set_position(self, value: f64) {
        self.0.position.set(value);
    }

    pub fn decimal_alignment_token(self) -> AvmString<'gc> {
        self.0.decimal_alignment_token.get()
    }

    pub fn set_decimal_alignment_token(self, value: AvmString<'gc>, mc: &Mutation<'gc>) {
        unlock!(
            Gc::write(mc, self.0),
            TabStopObjectData,
            decimal_alignment_token
        )
        .set(value);
    }
}

impl<'gc> TObject<'gc> for TabStopObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}
