//! `flash.media.Sound` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::object::{Object, QueuedPlay, SoundChannelObject, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::Avm2;
use crate::avm2::Error;
use crate::backend::navigator::Request;
use crate::character::Character;
use crate::display_object::SoundTransform;
use crate::string::AvmString;
use crate::{avm2_stub_getter, avm2_stub_method};
use swf::{SoundEvent, SoundInfo};

pub use crate::avm2::object::sound_allocator;

/// Implements `flash.media.Sound`'s 'init' method. which is called from the constructor.
pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(sound_object) = this.as_sound_object() {
        let class_object = this
            .instance_of()
            .ok_or("Attempted to construct Sound on a bare object.")?;

        if let Some((movie, symbol)) = activation
            .context
            .library
            .avm2_class_registry()
            .class_symbol(class_object)
        {
            if let Some(Character::Sound(sound)) = activation
                .context
                .library
                .library_for_movie_mut(movie)
                .character_by_id(symbol)
            {
                let sound = *sound;
                sound_object.set_sound(&mut activation.context, sound)?;
            } else {
                tracing::warn!("Attempted to construct subclass of Sound, {}, which is associated with non-Sound character {}", class_object.inner_class_definition().name().local_name(), symbol);
            }
        }
    }

    if args.try_get_object(activation, 0).is_some() {
        this.call_public_property("load", args, activation)?;
    }

    Ok(Value::Undefined)
}

/// Implements `Sound.bytesTotal`
pub fn get_bytes_total<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(sound) = this.as_sound_object() {
        if let Some(sound_handle) = sound.sound_handle() {
            if let Some(length) = activation.context.audio.get_sound_size(sound_handle) {
                return Ok((length).into());
            }
        }
        return Ok(0.into());
    }

    Ok(Value::Undefined)
}

pub fn get_bytes_loaded<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // This should have a different value from bytesTotal when the sound is loading.
    avm2_stub_getter!(activation, "flash.media.Sound", "bytesLoaded");
    get_bytes_total(activation, this, args)
}

/// Implements `Sound.isBuffering`
pub fn get_is_buffering<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.media.Sound", "isBuffering");
    //STUB: We do not yet support network-loaded sounds.
    Ok(false.into())
}

/// Implements `Sound.isURLInaccessible`
pub fn get_is_url_inaccessible<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.media.Sound", "isURLInaccessible");
    //STUB: We do not yet support network-loaded sounds.
    Ok(false.into())
}

/// Implements `Sound.url`
pub fn get_url<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.media.Sound", "url");
    //STUB: We do not yet support network-loaded sounds.
    Ok(Value::Null)
}

/// Implements `Sound.length`
pub fn get_length<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(sound) = this.as_sound_object() {
        if let Some(sound_handle) = sound.sound_handle() {
            if let Some(duration) = activation.context.audio.get_sound_duration(sound_handle) {
                return Ok((duration).into());
            }
        }
        return Ok(0.into());
    }

    Ok(Value::Undefined)
}

/// Implements `Sound.play`
pub fn play<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(sound_object) = this.as_sound_object() {
        let position = args
            .get(0)
            .cloned()
            .unwrap_or_else(|| 0.0.into())
            .coerce_to_number(activation)?;
        let num_loops = args
            .get(1)
            .cloned()
            .unwrap_or_else(|| 0.into())
            .coerce_to_i32(activation)?;
        let sound_transform = args.get(2).cloned().unwrap_or(Value::Null).as_object();

        let in_sample = if position > 0.0 {
            Some((position / 1000.0 * 44100.0) as u32)
        } else {
            None
        };

        let sound_info = SoundInfo {
            event: SoundEvent::Start,
            in_sample,
            out_sample: None,
            num_loops: num_loops.max(1) as u16,
            envelope: None,
        };

        let sound_transform = if let Some(sound_transform) = sound_transform {
            Some(SoundTransform::from_avm2_object(
                activation,
                sound_transform,
            )?)
        } else {
            None
        };

        let _handle = activation
            .context
            .audio_manager
            .start_generated_sound(activation.context.audio, sound_object)
            .expect("not too many sounds");

        let sound_channel = SoundChannelObject::empty(activation)?;

        let queued_play = QueuedPlay {
            position,
            sound_info,
            sound_transform,
            sound_channel,
        };
        if sound_object.play(queued_play, activation)? {
            return Ok(sound_channel.into());
        }
        // If we start playing a loaded sound with an invalid position,
        // this method returns `null`
        return Ok(Value::Null);
    }

    Ok(Value::Null)
}

/// `Sound.extract`
pub fn extract<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.media.Sound", "extract");

    let bytearray = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation)?;
    let length = args
        .get(1)
        .unwrap_or(&Value::Number(0.0))
        .coerce_to_number(activation)?;

    if let Some(mut bytearray) = bytearray.as_bytearray_mut(activation.context.gc_context) {
        bytearray
            .write_bytes(vec![0u8; length.ceil() as usize].as_slice())
            .map_err(|e| e.to_avm(activation))?;
    }

    Ok(Value::Undefined)
}

/// `Sound.close`
pub fn close<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.media.Sound", "close");
    Ok(Value::Undefined)
}

/// `Sound.load`
pub fn load<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // FIXME - don't allow replacing an existing sound
    let url_request = match args.get(0) {
        Some(Value::Object(request)) => request,
        // This should never actually happen
        _ => return Ok(Value::Undefined),
    };

    let url = url_request
        .get_public_property("url", activation)?
        .coerce_to_string(activation)?;

    // TODO: context parameter currently unused.
    let sound_context = args.try_get_object(activation, 1);
    if sound_context.is_some() {
        avm2_stub_method!(activation, "flash.media.Sound", "load", "with context");
    }

    let s = this.as_sound_object().expect("Not a sound object");
    s.load_called(&mut activation.context);

    let future = activation.context.load_manager.load_sound_avm2(
        activation.context.player.clone(),
        this,
        // FIXME: Set options from the `URLRequest`.
        Request::get(url.to_string()),
    );
    activation.context.navigator.spawn_future(future);

    Ok(Value::Undefined)
}

/// `Sound.loadCompressedDataFromByteArray`
pub fn load_compressed_data_from_byte_array<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bytearray = args.get_object(activation, 0, "bytes")?;
    let bytes_length = args.get_u32(activation, 1)?;
    let bytearray = bytearray.as_bytearray().unwrap();

    // FIXME - determine the actual errors thrown by Flash Player
    let bytes = bytearray.read_bytes(bytes_length as usize).map_err(|e| {
        Error::RustError(format!("Missing bytes from sound bytearray: {e:?}").into())
    })?;

    let handle = activation.context.audio.register_mp3(bytes).map_err(|e| {
        Error::RustError(format!("Failed to register sound from bytearray: {e:?}").into())
    })?;

    let progress_evt = activation
        .avm2()
        .classes()
        .progressevent
        .construct(
            activation,
            &[
                "progress".into(),
                false.into(),
                false.into(),
                bytes.len().into(),
                bytes.len().into(),
            ],
        )
        .map_err(|e| Error::AvmError(AvmString::new_utf8(activation.gc(), e.to_string()).into()))?;

    Avm2::dispatch_event(&mut activation.context, progress_evt, this);

    this.as_sound_object()
        .unwrap()
        .read_and_call_id3_event(activation, bytes);

    this.as_sound_object()
        .unwrap()
        .set_sound(&mut activation.context, handle)?;

    Ok(Value::Undefined)
}

/// `Sound.loadPCMFromByteArray`
pub fn load_pcm_from_byte_array<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.media.Sound", "loadPCMFromByteArray");
    Ok(Value::Undefined)
}

/// Implements `Sound.id3`
pub fn get_id3<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(id3) = this.as_sound_object().unwrap().id3() {
        Ok(id3.into())
    } else {
        Ok(Value::Null)
    }
}
