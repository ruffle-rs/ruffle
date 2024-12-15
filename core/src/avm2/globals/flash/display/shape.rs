//! `flash.display.Shape` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::globals::flash::display::display_object::initialize_for_allocator;
use crate::avm2::globals::slots::flash_display_shape as slots;
use crate::avm2::object::{ClassObject, Object, StageObject, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::display_object::Graphic;

pub fn shape_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let display_object = Graphic::empty(activation.context).into();

    initialize_for_allocator(activation, display_object, class)
}

/// Implements `graphics`.
pub fn get_graphics<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(dobj) = this.as_display_object() {
        // Lazily initialize the `Graphics` object in a hidden property.
        let graphics = match this.get_slot(slots::_GRAPHICS) {
            Value::Undefined | Value::Null => {
                let graphics = Value::from(StageObject::graphics(activation, dobj)?);
                this.set_slot(slots::_GRAPHICS, graphics, activation)?;
                graphics
            }
            graphics => graphics,
        };
        return Ok(graphics);
    }

    Ok(Value::Undefined)
}
