use crate::avm2::activation::Activation;
use crate::avm2::globals::flash::events::mouse_event;
use crate::avm2::globals::slots::flash_events_press_and_tap_gesture_event as slots;
use crate::avm2::value::Value;
use crate::avm2::Error;

pub fn get_tap_stage_x<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    mouse_event::local_to_stage_x(activation, this, slots::_TAP_LOCAL_X, slots::_TAP_LOCAL_Y)
}

pub fn get_tap_stage_y<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    mouse_event::local_to_stage_y(activation, this, slots::_TAP_LOCAL_X, slots::_TAP_LOCAL_Y)
}
