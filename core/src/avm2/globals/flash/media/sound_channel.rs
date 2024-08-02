//! `flash.media.SoundChannel` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::display_object::SoundTransform;

pub use crate::avm2::object::sound_channel_allocator;

/// Implements `SoundChannel.leftPeak`
pub fn get_left_peak<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(instance) = this
        .as_sound_channel()
        .and_then(|channel| channel.instance())
    {
        if let Some(peak) = activation.context.audio.get_sound_peak(instance) {
            return Ok(Value::Number(peak[0].into()));
        }
    }

    Ok(Value::Undefined)
}

/// Implements `SoundChannel.rightPeak`
pub fn get_right_peak<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(instance) = this
        .as_sound_channel()
        .and_then(|channel| channel.instance())
    {
        if let Some(peak) = activation.context.audio.get_sound_peak(instance) {
            return Ok(Value::Number(peak[1].into()));
        }
    }

    Ok(Value::Undefined)
}

/// Impl `SoundChannel.position`
pub fn get_position<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(instance) = this.as_sound_channel() {
        return Ok(instance.position(activation.context).into());
    }
    Ok(Value::Undefined)
}

/// Implements `soundTransform`'s getter
pub fn get_sound_transform<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(channel) = this.as_sound_channel() {
        let dobj_st = channel.sound_transform(activation).unwrap_or_default();

        return Ok(dobj_st.into_avm2_object(activation)?.into());
    }

    Ok(Value::Undefined)
}

/// Implements `soundTransform`'s setter
pub fn set_sound_transform<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(sound_channel) = this.as_sound_channel() {
        let as3_st = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_object(activation)?;
        let dobj_st = SoundTransform::from_avm2_object(activation, as3_st)?;

        sound_channel.set_sound_transform(activation, dobj_st);
    }

    Ok(Value::Undefined)
}

/// Impl `SoundChannel.stop`
pub fn stop<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(sound_channel) = this.as_sound_channel() {
        sound_channel.stop(activation);
    }

    Ok(Value::Undefined)
}
