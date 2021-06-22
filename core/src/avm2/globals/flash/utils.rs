//! `flash.utils` namespace

use crate::avm2::{Activation, Error, Object, Value};

pub mod bytearray;
pub mod compression_algorithm;
pub mod endian;

/// Implements `flash.utils.getTimer`
pub fn get_timer<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok((activation.context.navigator.time_since_launch().as_millis() as u32).into())
}
