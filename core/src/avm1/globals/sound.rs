//! AVM1 Sound object
//! TODO: Sound position, transform, loadSound

use crate::avm1::function::Executable;
use crate::avm1::property::Attribute::*;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, SoundObject, TObject, UpdateContext, Value};
use crate::character::Character;
use crate::display_object::TDisplayObject;
use gc_arena::MutationContext;

/// Implements `Sound`
pub fn constructor<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    // 1st parameter is the movie clip that "owns" all sounds started by this object.
    // `Sound.setTransform`, `Sound.stop`, etc. will affect all sounds owned by this clip.
    let owner = args
        .get(0)
        .map(|o| o.coerce_to_object(avm, context))
        .and_then(|o| o.as_display_object());

    let sound = this.as_sound_object().unwrap();
    sound.set_owner(context.gc_context, owner);

    Ok(this.into())
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = SoundObject::empty_sound(gc_context, Some(proto));

    object.as_script_object().unwrap().force_set_function(
        "attachSound",
        attach_sound,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );

    object.add_property(
        gc_context,
        "duration",
        Executable::Native(duration),
        None,
        DontDelete | ReadOnly | DontEnum,
    );

    object.add_property(
        gc_context,
        "id3",
        Executable::Native(id3),
        None,
        DontDelete | ReadOnly | DontEnum,
    );

    object.as_script_object().unwrap().force_set_function(
        "getBytesLoaded",
        get_bytes_loaded,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );

    object.as_script_object().unwrap().force_set_function(
        "getBytesTotal",
        get_bytes_total,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );

    object.as_script_object().unwrap().force_set_function(
        "getPan",
        get_pan,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );

    object.as_script_object().unwrap().force_set_function(
        "get_transform",
        get_transform,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );

    object.as_script_object().unwrap().force_set_function(
        "get_volume",
        get_volume,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );

    object.as_script_object().unwrap().force_set_function(
        "load_sound",
        load_sound,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );

    object.add_property(
        gc_context,
        "position",
        Executable::Native(position),
        None,
        DontDelete | ReadOnly | DontEnum,
    );

    object.as_script_object().unwrap().force_set_function(
        "set_pan",
        set_pan,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );

    object.as_script_object().unwrap().force_set_function(
        "set_transform",
        set_transform,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );

    object.as_script_object().unwrap().force_set_function(
        "set_volume",
        set_volume,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );

    object.as_script_object().unwrap().force_set_function(
        "start",
        start,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );

    object.as_script_object().unwrap().force_set_function(
        "stop",
        stop,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );

    object.into()
}

fn attach_sound<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let name = args.get(0).unwrap_or(&Value::Undefined);
    if let Some(sound_object) = this.as_sound_object() {
        let name = name.clone().coerce_to_string(avm, context)?;
        let movie = sound_object
            .owner()
            .or_else(|| context.levels.get(&0).copied())
            .and_then(|o| o.movie());
        if let Some(movie) = movie {
            if let Some(Character::Sound(sound)) = context
                .library
                .library_for_movie_mut(movie)
                .get_character_by_export_name(&name)
            {
                sound_object.set_sound(context.gc_context, Some(*sound));
                sound_object.set_duration(
                    context.gc_context,
                    context.audio.get_sound_duration(*sound).unwrap_or(0),
                );
                sound_object.set_position(context.gc_context, 0);
            } else {
                log::warn!("Sound.attachSound: Sound '{}' not found", name);
            }
        } else {
            log::warn!(
                "Sound.attachSound: Cannot attach Sound '{}' without a library to reference",
                name
            );
        }
    } else {
        log::warn!("Sound.attachSound: this is not a Sound");
    }
    Ok(Value::Undefined.into())
}

fn duration<'gc>(
    avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if avm.current_swf_version() >= 6 {
        if let Some(sound_object) = this.as_sound_object() {
            return Ok(sound_object.duration().into());
        } else {
            log::warn!("Sound.duration: this is not a Sound");
        }
    }

    Ok(Value::Undefined.into())
}

fn get_bytes_loaded<'gc>(
    avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if avm.current_swf_version() >= 6 {
        log::warn!("Sound.getBytesLoaded: Unimplemented");
        Ok(1.into())
    } else {
        Ok(Value::Undefined.into())
    }
}

fn get_bytes_total<'gc>(
    avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if avm.current_swf_version() >= 6 {
        log::warn!("Sound.getBytesTotal: Unimplemented");
        Ok(1.into())
    } else {
        Ok(Value::Undefined.into())
    }
}

fn get_pan<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("Sound.getPan: Unimplemented");
    Ok(0.into())
}

fn get_transform<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("Sound.getTransform: Unimplemented");
    Ok(Value::Undefined.into())
}

fn get_volume<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("Sound.getVolume: Unimplemented");
    Ok(100.into())
}

fn id3<'gc>(
    avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if avm.current_swf_version() >= 6 {
        log::warn!("Sound.id3: Unimplemented");
    }
    Ok(Value::Undefined.into())
}

fn load_sound<'gc>(
    avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if avm.current_swf_version() >= 6 {
        log::warn!("Sound.loadSound: Unimplemented");
    }
    Ok(Value::Undefined.into())
}

fn position<'gc>(
    avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if avm.current_swf_version() >= 6 {
        if let Some(sound_object) = this.as_sound_object() {
            // TODO: The position is "sticky"; even if the sound is no longer playing, it should return
            // the previous valid position.
            // Needs some audio backend work for this.
            if sound_object.sound().is_some() {
                if let Some(_sound_instance) = sound_object.sound_instance() {
                    log::warn!("Sound.position: Unimplemented");
                }
                return Ok(sound_object.position().into());
            }
        } else {
            log::warn!("Sound.position: this is not a Sound");
        }
    }
    Ok(Value::Undefined.into())
}

fn set_pan<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("Sound.setPan: Unimplemented");
    Ok(Value::Undefined.into())
}

fn set_transform<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("Sound.setTransform: Unimplemented");
    Ok(Value::Undefined.into())
}

fn set_volume<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("Sound.setVolume: Unimplemented");
    Ok(Value::Undefined.into())
}

fn start<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let start_offset = args
        .get(0)
        .unwrap_or(&Value::Number(0.0))
        .as_number(avm, context)?;
    let loops = args
        .get(1)
        .unwrap_or(&Value::Number(1.0))
        .as_number(avm, context)?;

    let loops = if loops >= 1.0 && loops <= f64::from(std::i16::MAX) {
        loops as u16
    } else {
        1
    };

    use swf::{SoundEvent, SoundInfo};
    if let Some(sound_object) = this.as_sound_object() {
        if let Some(sound) = sound_object.sound() {
            let sound_instance = context.audio.start_sound(
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
            );
            if let Ok(sound_instance) = sound_instance {
                sound_object.set_sound_instance(context.gc_context, Some(sound_instance));
            }
        } else {
            log::warn!("Sound.start: No sound is attached");
        }
    } else {
        log::warn!("Sound.start: Invalid sound");
    }

    Ok(Value::Undefined.into())
}

fn stop<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(sound) = this.as_sound_object() {
        if let Some(name) = args.get(0) {
            // Usage 1: Stop all instances of a particular sound, using the name parameter.
            let name = name.clone().coerce_to_string(avm, context)?;
            let movie = sound
                .owner()
                .or_else(|| context.levels.get(&0).copied())
                .and_then(|o| o.movie());
            if let Some(movie) = movie {
                if let Some(Character::Sound(sound)) = context
                    .library
                    .library_for_movie_mut(movie)
                    .get_character_by_export_name(&name)
                {
                    // Stop all sounds with the given name.
                    context.audio.stop_sounds_with_handle(*sound);
                } else {
                    log::warn!("Sound.stop: Sound '{}' not found", name);
                }
            } else {
                log::warn!(
                    "Sound.stop: Cannot stop Sound '{}' without a library to reference",
                    name
                )
            }
        } else if let Some(_owner) = sound.owner() {
            // Usage 2: Stop all sound running within a given clip.
            // TODO: We just stop the last played sound for now.
            if let Some(sound_instance) = sound.sound_instance() {
                context.audio.stop_sound(sound_instance);
            }
        } else {
            // Usage 3: If there is no owner and no name, this call acts like `stopAllSounds()`.
            context.audio.stop_all_sounds();
        }
    } else {
        log::warn!("Sound.stop: this is not a Sound");
    }

    Ok(Value::Undefined.into())
}
