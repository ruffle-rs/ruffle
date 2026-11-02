//! `flash.media.Sound` builtin/prototype

use crate::avm2::Avm2;
use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::bytearray::Endian;
use crate::avm2::error::{make_error_2037, make_error_2068, make_error_2084};
use crate::avm2::function::FunctionArgs;
use crate::avm2::globals::slots::flash_events_event_dispatcher as event_dispatcher_slots;
use crate::avm2::globals::slots::flash_net_url_request as url_request_slots;
use crate::avm2::object::{
    EventObject, QueuedPlay, SoundChannelObject, SoundLoadingState, TObject as _,
};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
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
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(sound_object) = this.as_object().and_then(|o| o.as_sound_object()) {
        let class_def = this.instance_class(activation);

        if let Some((movie, symbol)) = activation
            .context
            .library
            .avm2_class_registry()
            .class_symbol(class_def)
        {
            if let Some(Character::Sound(sound)) = activation
                .context
                .library
                .library_for_movie_mut(movie)
                .character_by_id(symbol)
            {
                sound_object.set_sound(activation.context, sound);
            } else {
                tracing::warn!(
                    "Attempted to construct subclass of Sound, {}, which is associated with non-Sound character {}",
                    class_def.name().local_name(),
                    symbol
                );
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Sound.bytesTotal`
pub fn get_bytes_total<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(sound) = this.as_sound_object() {
        if let Some(sound_handle) = sound.sound_handle()
            && let Some(length) = activation.context.audio.get_sound_size(sound_handle)
        {
            return Ok((length).into());
        }
        return Ok(0.into());
    }

    Ok(Value::Undefined)
}

pub fn get_bytes_loaded<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    // This should have a different value from bytesTotal when the sound is loading.
    avm2_stub_getter!(activation, "flash.media.Sound", "bytesLoaded");
    get_bytes_total(activation, this, args)
}

/// Implements `Sound.isBuffering`
pub fn get_is_buffering<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.media.Sound", "isBuffering");
    //STUB: We do not yet support network-loaded sounds.
    Ok(false.into())
}

/// Implements `Sound.isURLInaccessible`
pub fn get_is_url_inaccessible<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.media.Sound", "isURLInaccessible");
    //STUB: We do not yet support network-loaded sounds.
    Ok(false.into())
}

/// Implements `Sound.url`
pub fn get_url<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.media.Sound", "url");
    //STUB: We do not yet support network-loaded sounds.
    Ok(Value::Null)
}

/// Implements `Sound.length`
pub fn get_length<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(sound) = this.as_sound_object() {
        if let Some(sound_handle) = sound.sound_handle()
            && let Some(duration) = activation.context.audio.get_sound_duration(sound_handle)
        {
            return Ok((duration.as_millis()).into());
        }
        return Ok(0.into());
    }

    Ok(Value::Undefined)
}

/// Implements `Sound.play`
pub fn play<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(sound_object) = this.as_sound_object() {
        let position = args.get_f64(0);
        let num_loops = args.get_i32(1);
        let sound_transform = args.try_get_object(2);

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
            Some(SoundTransform::from_avm2_object(sound_transform))
        } else {
            None
        };

        if sound_object.sound_handle().is_none()
            && sound_object.loading_state() == SoundLoadingState::New
        {
            // A sound with no data plays what its `sampleData` listeners
            // generate, and is invalid without any.
            let sample_data = AvmString::new_ascii_static(activation.gc(), b"sampleData");
            let has_listener = this
                .get_slot(event_dispatcher_slots::DISPATCH_LIST)
                .as_object()
                .and_then(|list| {
                    list.as_dispatch_mut(activation.gc())
                        .map(|list| list.has_event_listener(sample_data))
                })
                .unwrap_or(false);
            if !has_listener {
                return Err(make_error_2068(activation));
            }

            let sound_channel = SoundChannelObject::generated(activation, sound_object);
            if sound_channel.start_generator(activation.context, sound_transform) {
                return Ok(sound_channel.into());
            }
            return Ok(Value::Null);
        }

        let sound_channel = SoundChannelObject::empty(activation);

        let queued_play = QueuedPlay {
            position,
            sound_info,
            sound_transform,
            sound_channel,
        };
        if sound_object.play(queued_play, activation) {
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
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_sound_object().unwrap();
    let target = args.get_object(activation, 0, "target")?;
    let length = args.get_f64(1);
    let start_position = args.get_f64(2);

    let Some(sound) = this.sound_handle() else {
        // Flash extracts nothing from a sound without data.
        return Ok(0.into());
    };

    // Lengths and positions are rounded down, as in Flash Player, where an
    // infinite length extracts nothing. A negative position, the default,
    // goes on where the last call ended; NaN, and positions from 2^31 on,
    // start at 0.
    let length = if length.is_finite() && length > 0.0 {
        length as usize
    } else {
        0
    };
    let start = if start_position < 0.0 {
        this.extract_position()
    } else if start_position < 2147483648.0 {
        start_position as usize
    } else {
        0
    };

    // In pieces, written as they are decoded: a long sound takes 8 bytes per
    // sample frame.
    const CHUNK_LENGTH: usize = 8192;
    let mut target = target.as_bytearray_mut().unwrap();
    let little_endian = target.endian() == Endian::Little;
    let mut extracted = 0;
    this.set_extract_position(start);
    while extracted < length {
        let chunk_length = (length - extracted).min(CHUNK_LENGTH);
        let Some(samples) =
            activation
                .context
                .audio
                .extract_sound(sound, start + extracted, chunk_length)
        else {
            // The backend can't decode it: extract nothing, but return
            // `undefined` like a stub, so that loops such as
            // `while (len > 0) len -= sound.extract(bytes, len)` end.
            return Ok(Value::Undefined);
        };
        let mut bytes = Vec::with_capacity(samples.len() * 8);
        for sample in samples.iter().flatten() {
            if little_endian {
                bytes.extend_from_slice(&sample.to_le_bytes());
            } else {
                bytes.extend_from_slice(&sample.to_be_bytes());
            }
        }
        target
            .write_bytes(&bytes)
            .map_err(|e| e.to_avm(activation))?;
        extracted += samples.len();
        this.set_extract_position(start + extracted);
        if samples.len() < chunk_length {
            break;
        }
    }

    Ok((extracted as f64).into())
}

/// `Sound.close`
pub fn close<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.media.Sound", "close");
    Ok(Value::Undefined)
}

/// `Sound.load`
pub fn load<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this_object = this.as_object().unwrap();

    let this = this_object.as_sound_object().unwrap();
    if this.loading_state() != SoundLoadingState::New {
        return Err(make_error_2037(activation));
    }

    let url_request = match args.try_get_object(0) {
        Some(request) => request,
        // FP ignores calls of `load(null)`
        None => return Ok(Value::Undefined),
    };

    let url = url_request
        .get_slot(url_request_slots::_URL)
        .coerce_to_string(activation)?;

    // TODO: context parameter currently unused.
    let sound_context = args.try_get_object(1);
    if sound_context.is_some() {
        avm2_stub_method!(activation, "flash.media.Sound", "load", "with context");
    }

    let future = crate::loader::load_sound_avm2(
        activation.context,
        this,
        // FIXME: Set options from the `URLRequest`.
        Request::get(url.to_string()),
    );
    activation.context.navigator.spawn_future(future);
    this.set_loading_state(SoundLoadingState::Loading);

    Ok(Value::Undefined)
}

/// `Sound.loadCompressedDataFromByteArray`
pub fn load_compressed_data_from_byte_array<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this_object = this.as_object().unwrap();

    let this = this_object.as_sound_object().unwrap();
    if this.loading_state() == SoundLoadingState::Loaded {
        return Ok(Value::Undefined);
    }

    let bytearray = args.get_object(activation, 0, "bytes")?;
    let bytes_length = args.get_u32(1);
    let bytearray = bytearray.as_bytearray().unwrap();

    let bytes = if let Ok(bytes) = bytearray.read_bytes(bytes_length as usize) {
        bytes
    } else {
        // This is the error Flash throws
        return Err(make_error_2084(activation));
    };

    // FIXME - determine the actual error thrown by Flash Player
    let handle = activation.context.audio.register_mp3(bytes).map_err(|e| {
        Error::rust_error(format!("Failed to register sound from bytearray: {e:?}").into())
    })?;

    let progress_evt =
        EventObject::progress_event(activation, "progress", bytes.len(), bytes.len());

    Avm2::dispatch_event(activation.context, progress_evt, this_object);

    this.read_and_call_id3_event(activation, bytes);
    this.set_sound(activation.context, handle);

    Ok(Value::Undefined)
}

/// `Sound.loadPCMFromByteArray`
pub fn load_pcm_from_byte_array<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this_object = this.as_object().unwrap();

    let this = this_object.as_sound_object().unwrap();
    if this.loading_state() == SoundLoadingState::Loaded {
        return Ok(Value::Undefined);
    }

    // TODO Add proper implementation.
    //   The following line ensures proper behavior
    //   when calling load multiple times.
    this.set_loading_state(SoundLoadingState::Loaded);
    avm2_stub_method!(activation, "flash.media.Sound", "loadPCMFromByteArray");

    Ok(Value::Undefined)
}

/// Implements `Sound.id3`
pub fn get_id3<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(id3) = this.as_sound_object().unwrap().id3() {
        Ok(id3.into())
    } else {
        Ok(Value::Null)
    }
}
