//! `flash.media.Sound` builtin/prototype

use crate::avm2::Avm2;
use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::bytearray::ByteArrayStorage;
use crate::avm2::error::{make_error_2037, make_error_2084};
use crate::avm2::function::FunctionArgs;
use crate::avm2::globals::slots::flash_net_url_request as url_request_slots;
use crate::avm2::object::{
    ByteArrayObject, EventObject, QueuedPlay, SoundChannelObject, SoundLoadingState, TObject as _,
};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::backend::audio::{DYNAMIC_SOUND_MIN_SAMPLES, dynamic_sound_samples_from_bytearray};
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

        let sound_transform = if let Some(sound_transform) = sound_transform {
            Some(SoundTransform::from_avm2_object(sound_transform))
        } else {
            None
        };

        let sound_channel = SoundChannelObject::empty(activation);

        // A new Sound without loaded audio uses SampleDataEvent
        // to provide dynamically generated audio.
        if sound_object.sound_handle().is_none()
            && sound_object.loading_state() == SoundLoadingState::New
        {
            let storage = ByteArrayStorage::new(activation.context);
            let data = ByteArrayObject::from_storage(activation.context, storage);

            let event_type = AvmString::new_utf8(activation.gc(), "sampleData");
            let sample_data_event_class = activation.avm2().classes().sampledataevent;

            let event = EventObject::from_class_and_args(
                activation,
                sample_data_event_class,
                &[
                    event_type.into(),
                    false.into(),
                    false.into(),
                    0.0.into(),
                    data.into(),
                ],
            );

            Avm2::dispatch_event(activation.context, event, this);

            let written = data.storage().position();
            let samples = dynamic_sound_samples_from_bytearray(data, written);

            let initial_position = samples.len() as u32;

            let dynamic_start_offset = if position > 0.0 {
                (position / 1000.0 * 44100.0) as u32
            } else {
                0
            };

            if let Some(instance) = activation.context.start_dynamic_sound(
                sound_channel,
                sound_object,
                data,
                initial_position,
                dynamic_start_offset,
            ) {
                activation.context.append_dynamic_sound(instance, &samples);

                if initial_position < DYNAMIC_SOUND_MIN_SAMPLES {
                    activation.context.finish_dynamic_sound(instance);

                    activation
                        .context
                        .audio_manager
                        .mark_dynamic_sound_finished(instance);
                }

                sound_channel.set_sound_instance(activation.context, instance);

                if let Some(sound_transform) = sound_transform {
                    activation
                        .context
                        .set_local_sound_transform(instance, sound_transform);
                }

                activation
                    .context
                    .audio_manager
                    .mark_dynamic_sound_play_started(instance);

                sound_object.set_loading_state(SoundLoadingState::Generated);
                return Ok(sound_channel.into());
            }

            return Ok(Value::Null);
        }

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
        // this method returns `null`.
        return Ok(Value::Null);
    }

    Ok(Value::Null)
}

/// `Sound.extract`
pub fn extract<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.media.Sound", "extract");

    let bytearray = args.try_get_object(0);
    let length = args.get_f64(1);

    if let Some(bytearray) = bytearray
        && let Some(mut bytearray) = bytearray.as_bytearray_mut()
    {
        bytearray
            .write_bytes(vec![0u8; length.ceil() as usize].as_slice())
            .map_err(|e| e.to_avm(activation))?;
    }

    Ok(Value::Undefined)
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
