use crate::avm2::activation::Activation;
use crate::avm2::globals::slots::flash_events_mouse_event as slots;
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::display_object::TDisplayObject;
use swf::Point;

/// Implements `stageX`'s getter.
pub fn get_stage_x<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    local_to_stage_x(activation, this, slots::LOCAL_X, slots::LOCAL_Y)
}

/// Implements `stageY`'s getter.
pub fn get_stage_y<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    local_to_stage_y(activation, this, slots::LOCAL_X, slots::LOCAL_Y)
}

pub fn update_after_event<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    *activation.context.needs_render = true;
    Ok(Value::Undefined)
}

pub(super) fn local_to_stage_x<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    slot_x: u32,
    slot_y: u32,
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(evt) = this.as_event() {
        let local_x = this.get_slot(slot_x).coerce_to_number(activation)?;
        let local_y = this.get_slot(slot_y).coerce_to_number(activation)?;

        if local_x.is_nan() || local_y.is_nan() {
            return Ok(Value::Number(local_x));
        } else if let Some(target) = evt.target().and_then(|t| t.as_display_object()) {
            let local = Point::from_pixels(local_x, local_y);
            // `local_to_global` does a matrix multiplication, which in general
            // depends on both the x and y coordinates.
            let global = target.local_to_global(local);
            return Ok(Value::Number(global.x.to_pixels()));
        } else {
            return Ok(Value::Number(local_x * 0.0));
        }
    }

    Ok(Value::Undefined)
}

pub(super) fn local_to_stage_y<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    slot_x: u32,
    slot_y: u32,
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(evt) = this.as_event() {
        let local_x = this.get_slot(slot_x).coerce_to_number(activation)?;
        let local_y = this.get_slot(slot_y).coerce_to_number(activation)?;

        if local_x.is_nan() || local_y.is_nan() {
            return Ok(Value::Number(local_y));
        } else if let Some(target) = evt.target().and_then(|t| t.as_display_object()) {
            let local = Point::from_pixels(local_x, local_y);
            // `local_to_global` does a matrix multiplication, which in general
            // depends on both the x and y coordinates.
            let global = target.local_to_global(local);
            return Ok(Value::Number(global.y.to_pixels()));
        } else {
            return Ok(Value::Number(local_y * 0.0));
        }
    }

    Ok(Value::Undefined)
}
