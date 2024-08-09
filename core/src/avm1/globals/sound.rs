//! AVM1 Sound object
//! TODO: Sound position, transform, loadSound

use crate::avm1::activation::Activation;
use crate::avm1::clamp::Clamp;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, ScriptObject, SoundObject, TObject, Value};
use crate::backend::navigator::Request;
use crate::character::Character;
use crate::context::GcContext;
use crate::display_object::{SoundTransform, TDisplayObject};
use crate::{avm1_stub, avm_warn};

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "attachSound" => method(attach_sound; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "duration" => property(duration; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getDuration" => method(duration; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "setDuration" => method(set_duration; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "id3" => method(id3; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getBytesLoaded" => method(get_bytes_loaded; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getBytesTotal" => method(get_bytes_total; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getPan" => method(get_pan; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getTransform" => method(get_transform; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getVolume" => method(get_volume; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "loadSound" => method(load_sound; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "position" => property(position; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "setPan" => method(set_pan; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "setTransform" => method(set_transform; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "setVolume" => method(set_volume; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "start" => method(start; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "stop" => method(stop; DONT_ENUM | DONT_DELETE | READ_ONLY);
};

/// Implements `Sound`
pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // 1st parameter is the movie clip that "owns" all sounds started by this object.
    // `Sound.setTransform`, `Sound.stop`, etc. will affect all sounds owned by this clip.
    let owner = if let Some(owner) = args.get(0) {
        let start_clip = activation.target_clip_or_root();
        activation.resolve_target_display_object(start_clip, *owner, false)?
    } else {
        None
    };

    if let Some(sound) = this.as_sound_object() {
        sound.set_owner(activation.context.gc_context, owner);
    } else {
        tracing::error!("Tried to construct a Sound on a non-SoundObject");
    }

    Ok(this.into())
}

pub fn create_proto<'gc>(
    context: &mut GcContext<'_, 'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let sound = SoundObject::empty_sound(context.gc_context, proto);
    let object = sound.raw_script_object();
    define_properties_on(PROTO_DECLS, context, object, fn_proto);
    sound.into()
}

fn attach_sound<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let name = args.get(0).unwrap_or(&Value::Undefined);
    if let Some(sound_object) = this.as_sound_object() {
        let name = name.coerce_to_string(activation)?;
        let movie = sound_object
            .owner()
            .unwrap_or_else(|| activation.base_clip().avm1_root())
            .movie();
        if let Some((_, Character::Sound(sound))) = activation
            .context
            .library
            .library_for_movie_mut(movie)
            .character_by_export_name(name)
        {
            sound_object.set_sound(activation.context.gc_context, Some(*sound));
            sound_object.set_is_streaming(activation.context.gc_context, false);
            sound_object.set_duration(
                activation.context.gc_context,
                activation
                    .context
                    .audio
                    .get_sound_duration(*sound)
                    .map(|d| d.round() as u32),
            );
            sound_object.set_position(activation.context.gc_context, 0);
        } else {
            avm_warn!(activation, "Sound.attachSound: Sound '{}' not found", name);
        }
    } else {
        avm_warn!(activation, "Sound.attachSound: this is not a Sound");
    }
    Ok(Value::Undefined)
}

fn duration<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // TODO: Sound.duration was only added in SWFv6, but it is not version gated.
    // Return undefined for player <6 if we ever add player version emulation.
    if let Some(sound_object) = this.as_sound_object() {
        return Ok(sound_object
            .duration()
            .map_or(Value::Undefined, |d| d.into()));
    } else {
        avm_warn!(activation, "Sound.duration: this is not a Sound");
    }
    Ok(Value::Undefined)
}

fn set_duration<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

fn get_bytes_loaded<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if activation.swf_version() >= 6 {
        avm1_stub!(activation, "Sound", "getBytesLoaded");
        Ok(1.into())
    } else {
        Ok(Value::Undefined)
    }
}

fn get_bytes_total<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if activation.swf_version() >= 6 {
        avm1_stub!(activation, "Sound", "getBytesTotal");
        Ok(1.into())
    } else {
        Ok(Value::Undefined)
    }
}

fn get_pan<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let transform = this.as_sound_object().map(|sound| {
        sound
            .owner()
            .map(|owner| owner.base().sound_transform().clone())
            .unwrap_or_else(|| activation.context.global_sound_transform().clone())
    });

    if let Some(transform) = transform {
        Ok(transform.pan().into())
    } else {
        Ok(Value::Undefined)
    }
}

fn get_transform<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let transform = this.as_sound_object().map(|sound| {
        sound
            .owner()
            .map(|owner| owner.base().sound_transform().clone())
            .unwrap_or_else(|| activation.context.global_sound_transform().clone())
    });

    if let Some(transform) = transform {
        let obj = ScriptObject::new(
            activation.context.gc_context,
            Some(activation.context.avm1.prototypes().object),
        );
        // Surprisingly `lr` means "right-to-left" and `rl` means "left-to-right".
        obj.set("ll", transform.left_to_left.into(), activation)?;
        obj.set("lr", transform.right_to_left.into(), activation)?;
        obj.set("rl", transform.left_to_right.into(), activation)?;
        obj.set("rr", transform.right_to_right.into(), activation)?;
        Ok(obj.into())
    } else {
        Ok(Value::Undefined)
    }
}

fn get_volume<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let transform = this.as_sound_object().map(|sound| {
        sound
            .owner()
            .map(|owner| owner.base().sound_transform().clone())
            .unwrap_or_else(|| activation.context.global_sound_transform().clone())
    });

    if let Some(transform) = transform {
        Ok(transform.volume.into())
    } else {
        Ok(Value::Undefined)
    }
}

fn id3<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if activation.swf_version() >= 6 {
        avm1_stub!(activation, "Sound", "id3");
    }
    Ok(Value::Undefined)
}

fn load_sound<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(sound) = this.as_sound_object() {
        if let Some(url) = args.get(0) {
            let url = url.coerce_to_string(activation)?;
            let is_streaming = args
                .get(1)
                .unwrap_or(&Value::Undefined)
                .as_bool(activation.swf_version());
            if is_streaming {
                // Streaming MP3s can only have a single active instance.
                // (Previous `attachSound` instances will continue to play.)
                if let Some(sound_instance) = sound.sound_instance() {
                    activation.context.stop_sound(sound_instance);
                }
            }
            sound.set_is_streaming(activation.context.gc_context, is_streaming);
            let future = activation.context.load_manager.load_sound_avm1(
                activation.context.player.clone(),
                sound,
                Request::get(url.to_utf8_lossy().into_owned()),
                is_streaming,
            );
            activation.context.navigator.spawn_future(future);
        }
    }
    Ok(Value::Undefined)
}

fn position<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // TODO: Sound.position was only added in SWFv6, but it is not version gated.
    // Return undefined for player <6 if we ever add player version emulation.
    if let Some(sound_object) = this.as_sound_object() {
        if sound_object.sound().is_some() {
            return Ok(sound_object.position().into());
        }
    } else {
        avm_warn!(activation, "Sound.position: this is not a Sound");
    }
    Ok(Value::Undefined)
}

fn set_pan<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let pan = args
        .get(0)
        .unwrap_or(&0.into())
        .coerce_to_f64(activation)?
        .clamp_to_i32();
    if let Some(sound) = this.as_sound_object() {
        if let Some(owner) = sound.owner() {
            let mut transform = owner.base().sound_transform().clone();
            transform.set_pan(pan);
            owner.set_sound_transform(activation.context, transform);
        } else {
            let mut transform = activation.context.global_sound_transform().clone();
            transform.set_pan(pan);
            activation.context.set_global_sound_transform(transform);
        }
    }

    Ok(Value::Undefined)
}

fn set_transform<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let obj = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation);

    if let Some(sound) = this.as_sound_object() {
        let mut transform = if let Some(owner) = sound.owner() {
            owner.base().sound_transform().clone()
        } else {
            activation.context.global_sound_transform().clone()
        };

        if obj.has_own_property(activation, "ll".into()) {
            transform.left_to_left = obj.get("ll", activation)?.coerce_to_i32(activation)?;
        }
        // Surprisingly `lr` means "right-to-left" and `rl` means "left-to-right".
        if obj.has_own_property(activation, "rl".into()) {
            transform.left_to_right = obj.get("rl", activation)?.coerce_to_i32(activation)?;
        }
        if obj.has_own_property(activation, "lr".into()) {
            transform.right_to_left = obj.get("lr", activation)?.coerce_to_i32(activation)?;
        }
        if obj.has_own_property(activation, "rr".into()) {
            transform.right_to_right = obj.get("rr", activation)?.coerce_to_i32(activation)?;
        }

        if let Some(owner) = sound.owner() {
            owner.set_sound_transform(activation.context, transform);
        } else {
            activation.context.set_global_sound_transform(transform);
        };
    }
    Ok(Value::Undefined)
}

fn set_volume<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let volume = args
        .get(0)
        .unwrap_or(&0.into())
        .coerce_to_f64(activation)?
        .clamp_to_i32();
    if let Some(sound) = this.as_sound_object() {
        if let Some(owner) = sound.owner() {
            let transform = SoundTransform {
                volume,
                ..*owner.base().sound_transform()
            };
            owner.set_sound_transform(activation.context, transform);
        } else {
            let transform = SoundTransform {
                volume,
                ..*activation.context.global_sound_transform()
            };
            activation.context.set_global_sound_transform(transform);
        }
    }

    Ok(Value::Undefined)
}

pub fn start<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let start_offset = args.get(0).unwrap_or(&0.into()).coerce_to_f64(activation)?;
    let loops = args.get(1).unwrap_or(&1.into()).coerce_to_f64(activation)?;

    // TODO: Handle loops > u16::MAX.
    let loops = (loops as u16).max(1);

    use swf::{SoundEvent, SoundInfo};
    if let Some(sound_object) = this.as_sound_object() {
        if let Some(sound) = sound_object.sound() {
            if sound_object.is_streaming() {
                // Streaming MP3s can only have a single active instance.
                if let Some(sound_instance) = sound_object.sound_instance() {
                    activation.context.stop_sound(sound_instance);
                }
            }
            let sound_instance = activation.context.start_sound(
                sound,
                &SoundInfo {
                    event: SoundEvent::Start,
                    in_sample: if start_offset > 0.0 {
                        Some((start_offset * 44100.0) as u32)
                    } else {
                        None
                    },
                    out_sample: None,
                    num_loops: loops,
                    envelope: None,
                },
                sound_object.owner(),
                Some(sound_object),
            );
            if let Some(sound_instance) = sound_instance {
                sound_object
                    .set_sound_instance(activation.context.gc_context, Some(sound_instance));
            }
        } else {
            avm_warn!(activation, "Sound.start: No sound is attached");
        }
    } else {
        avm_warn!(activation, "Sound.start: Invalid sound");
    }

    Ok(Value::Undefined)
}

fn stop<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(sound) = this.as_sound_object() {
        if let Some(name) = args.get(0) {
            // Usage 1: Stop all instances of a particular sound, using the name parameter.
            let name = name.coerce_to_string(activation)?;
            let movie = sound
                .owner()
                .unwrap_or_else(|| activation.base_clip().avm1_root())
                .movie();
            if let Some((_, Character::Sound(sound))) = activation
                .context
                .library
                .library_for_movie_mut(movie)
                .character_by_export_name(name)
            {
                // Stop all sounds with the given name.
                let sound = *sound;
                activation.context.stop_sounds_with_handle(sound);
            } else {
                avm_warn!(activation, "Sound.stop: Sound '{}' not found", name);
            }
        } else if let Some(owner) = sound.owner() {
            // Usage 2: Stop all sound running within a given clip.
            activation.context.stop_sounds_with_display_object(owner);
            sound.set_sound_instance(activation.context.gc_context, None);
        } else {
            // Usage 3: If there is no owner and no name, this call acts like `stopAllSounds()`.
            activation.context.stop_all_sounds();
        }
    } else {
        avm_warn!(activation, "Sound.stop: this is not a Sound");
    }

    Ok(Value::Undefined)
}
