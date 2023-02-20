//! `flash.media.Sound` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::{sound_allocator, Object, QueuedPlay, SoundChannelObject, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::avm2::QName;
use crate::backend::navigator::Request;
use crate::character::Character;
use crate::display_object::SoundTransform;
use crate::{avm2_stub_constructor, avm2_stub_getter, avm2_stub_method};
use gc_arena::GcCell;
use swf::{SoundEvent, SoundInfo};

/// Implements `flash.media.Sound`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;

        if !args.is_empty() {
            avm2_stub_constructor!(
                activation.context.stub_tracker,
                "flash.media.Sound",
                "with arguments"
            );
        }

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
                    tracing::warn!("Attempted to construct subclass of Sound, {}, which is associated with non-Sound character {}", class_object.inner_class_definition().read().name().local_name(), symbol);
                }
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `flash.media.Sound`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

/// Implements `Sound.bytesTotal`
pub fn bytes_total<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(sound) = this.and_then(|this| this.as_sound_object()) {
        if let Some(sound_handle) = sound.sound_handle() {
            if let Some(length) = activation.context.audio.get_sound_size(sound_handle) {
                return Ok((length).into());
            }
        }
        return Ok(0.into());
    }

    Ok(Value::Undefined)
}

/// Implements `Sound.isBuffering`
pub fn is_buffering<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(
        activation.context.stub_tracker,
        "flash.media.Sound",
        "isBuffering"
    );
    //STUB: We do not yet support network-loaded sounds.
    Ok(false.into())
}

/// Implements `Sound.url`
pub fn url<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation.context.stub_tracker, "flash.media.Sound", "url");
    //STUB: We do not yet support network-loaded sounds.
    Ok(Value::Null)
}

/// Implements `Sound.length`
pub fn length<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(sound) = this.and_then(|this| this.as_sound_object()) {
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
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(sound_object) = this.and_then(|this| this.as_sound_object()) {
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
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(
        activation.context.stub_tracker,
        "flash.media.Sound",
        "extract"
    );
    Ok(Value::Undefined)
}

/// `Sound.close`
pub fn close<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(
        activation.context.stub_tracker,
        "flash.media.Sound",
        "close"
    );
    Ok(Value::Undefined)
}

/// `Sound.load`
pub fn load<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
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
        let _sound_context = args.get(1);
        if _sound_context.is_some() {
            avm2_stub_method!(
                activation.context.stub_tracker,
                "flash.media.Sound",
                "load",
                "with context"
            );
        }

        let future = activation.context.load_manager.load_sound_avm2(
            activation.context.player.clone(),
            this,
            // FIXME: Set options from the `URLRequest`.
            Request::get(url.to_string()),
        );
        activation.context.navigator.spawn_future(future);
    }
    Ok(Value::Undefined)
}

/// `Sound.loadCompressedDataFromByteArray`
pub fn load_compressed_data_from_byte_array<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(
        activation.context.stub_tracker,
        "flash.media.Sound",
        "loadCompressedDataFromByteArray"
    );
    Ok(Value::Undefined)
}

/// `Sound.loadPCMFromByteArray`
pub fn load_pcm_from_byte_array<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(
        activation.context.stub_tracker,
        "flash.media.Sound",
        "loadPCMFromByteArray"
    );
    Ok(Value::Undefined)
}

/// Construct `Sound`'s class.
pub fn create_class<'gc>(activation: &mut Activation<'_, 'gc>) -> GcCell<'gc, Class<'gc>> {
    let mc = activation.context.gc_context;
    let class = Class::new(
        QName::new(Namespace::package("flash.media", mc), "Sound"),
        Some(Multiname::new(
            Namespace::package("flash.events", mc),
            "EventDispatcher",
        )),
        Method::from_builtin(instance_init, "<Sound instance initializer>", mc),
        Method::from_builtin(class_init, "<Sound class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);
    write.set_instance_allocator(sound_allocator);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("bytesLoaded", Some(bytes_total), None),
        ("bytesTotal", Some(bytes_total), None),
        ("isBuffering", Some(is_buffering), None),
        ("isURLInaccessible", Some(is_buffering), None),
        ("url", Some(url), None),
        ("length", Some(length), None),
    ];
    write.define_builtin_instance_properties(
        mc,
        activation.avm2().public_namespace,
        PUBLIC_INSTANCE_PROPERTIES,
    );

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("play", play),
        ("extract", extract),
        ("load", load),
        ("close", close),
        (
            "loadCompressedDataFromByteArray",
            load_compressed_data_from_byte_array,
        ),
        ("loadPCMFromByteArray", load_pcm_from_byte_array),
    ];
    write.define_builtin_instance_methods(
        mc,
        activation.avm2().public_namespace,
        PUBLIC_INSTANCE_METHODS,
    );

    class
}
