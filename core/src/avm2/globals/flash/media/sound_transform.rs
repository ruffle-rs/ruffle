//! `flash.media.SoundTransform` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::object::TObject;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::Error;

pub use crate::avm2::object::sound_transform_allocator;

pub fn get_left_to_left<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_sound_transform().unwrap();

    Ok(this.left_to_left().into())
}

pub fn set_left_to_left<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_sound_transform().unwrap();

    let value = args.get_f64(activation, 0)?;
    this.set_left_to_left(value);

    Ok(Value::Undefined)
}

pub fn get_left_to_right<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_sound_transform().unwrap();

    Ok(this.left_to_right().into())
}

pub fn set_left_to_right<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_sound_transform().unwrap();

    let value = args.get_f64(activation, 0)?;
    this.set_left_to_right(value);

    Ok(Value::Undefined)
}

pub fn get_right_to_left<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_sound_transform().unwrap();

    Ok(this.right_to_left().into())
}

pub fn set_right_to_left<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_sound_transform().unwrap();

    let value = args.get_f64(activation, 0)?;
    this.set_right_to_left(value);

    Ok(Value::Undefined)
}

pub fn get_right_to_right<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_sound_transform().unwrap();

    Ok(this.right_to_right().into())
}

pub fn set_right_to_right<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_sound_transform().unwrap();

    let value = args.get_f64(activation, 0)?;
    this.set_right_to_right(value);

    Ok(Value::Undefined)
}

pub fn get_volume<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_sound_transform().unwrap();

    Ok(this.volume().into())
}

pub fn set_volume<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_sound_transform().unwrap();

    let value = args.get_f64(activation, 0)?;
    this.set_volume(value);

    Ok(Value::Undefined)
}

/// Implements `SoundTransform.pan`'s getter.
pub fn get_pan<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_sound_transform().unwrap();

    let left_to_right = this.left_to_right();
    let right_to_left = this.right_to_left();

    if left_to_right != 0.0 || right_to_left != 0.0 {
        return Ok(0.0.into());
    }

    let left_to_left = this.left_to_left();

    Ok((1.0 - left_to_left.powf(2.0)).into())
}

/// Implements `SoundTransform.pan`'s setter.
pub fn set_pan<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_sound_transform().unwrap();

    let pan = args.get_f64(activation, 0)?;

    this.set_left_to_left((1.0 - pan).sqrt());
    this.set_right_to_right((1.0 + pan).sqrt());
    this.set_left_to_right(0.0);
    this.set_right_to_left(0.0);

    Ok(Value::Undefined)
}
