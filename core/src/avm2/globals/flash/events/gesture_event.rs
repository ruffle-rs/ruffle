use crate::avm2::activation::Activation;
use crate::avm2::globals::flash::events::mouse_event;
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;

// Borrow mouse_event's `stageX` getter
pub fn get_stage_x<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    mouse_event::get_stage_x(activation, this, args)
}

// Borrow mouse_event's `stageY` getter
pub fn get_stage_y<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    mouse_event::get_stage_y(activation, this, args)
}
