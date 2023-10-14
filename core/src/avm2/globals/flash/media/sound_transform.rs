//! `flash.media.SoundTransform` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::object::{Object, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::Error;

/// Implements `SoundTransform.pan`'s getter.
pub fn get_pan<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let left_to_right = this
        .get_public_property("leftToRight", activation)?
        .coerce_to_number(activation)?;
    let right_to_left = this
        .get_public_property("rightToLeft", activation)?
        .coerce_to_number(activation)?;

    if left_to_right != 0.0 || right_to_left != 0.0 {
        return Ok(0.0.into());
    }

    let left_to_left = this
        .get_public_property("leftToLeft", activation)?
        .coerce_to_number(activation)?;

    Ok((1.0 - left_to_left.powf(2.0)).into())
}

/// Implements `SoundTransform.pan`'s setter.
pub fn set_pan<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let pan = args.get_f64(activation, 0)?;
    this.set_public_property("leftToLeft", (1.0 - pan).sqrt().into(), activation)?;
    this.set_public_property("rightToRight", (1.0 + pan).sqrt().into(), activation)?;
    this.set_public_property("leftToRight", (0.0).into(), activation)?;
    this.set_public_property("rightToLeft", (0.0).into(), activation)?;

    Ok(Value::Undefined)
}
