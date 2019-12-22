//! AVM1 Sound object

use crate::avm1::function::Executable;
use crate::avm1::property::Attribute::*;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, SoundObject, TObject, UpdateContext, Value};
use crate::character::Character;
use gc_arena::MutationContext;

/// Implements `Sound`
pub fn constructor<'gc>(
    _avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    // 1st parameter is the movie clip that "owns" all sounds started by this object.
    // `Sound.setTransform`, `Sound.stop`, etc. will affect all sounds owned by this clip.
    let owner = args
        .get(0)
        .and_then(|o| o.as_object().ok())
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
        if let Some(Character::Sound(sound)) = context.library.get_character_by_export_name(&name) {
            sound_object.set_sound(context.gc_context, Some(*sound));
        } else {
            log::warn!("Sound.attachSound: Sound '{}' not found", name);
        }
    } else {
        log::warn!("Sound.attachSound: this is not a Sound");
    }
    Ok(Value::Undefined.into())
}

fn duration<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("Sound.duration: Unimplemented");
    Ok(1.into())
}

fn get_bytes_loaded<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("Sound.getBytesLoaded: Unimplemented");
    Ok(1.into())
}

fn get_bytes_total<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("Sound.getBytesTotal: Unimplemented");
    Ok(1.into())
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
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("Sound.id3: Unimplemented");
    Ok(Value::Undefined.into())
}

fn load_sound<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("Sound.loadSound: Unimplemented");
    Ok(Value::Undefined.into())
}

fn position<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("Sound.position: Unimplemented");
    Ok(0.into())
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
    if let Some(sound) = this.as_sound_object().and_then(|o| o.sound()) {
        context.audio.start_sound(
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
    } else {
        log::error!("Sound.start: Invalid sound");
    }

    Ok(Value::Undefined.into())
}

fn stop<'gc>(
    _avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(sound) = this.as_sound_object() {
        if let Some(_owner) = sound.owner() {
            // TODO
        } else {
            // If there is no owner, this call acts like `stopAllSounds()`.
            context.audio.stop_all_sounds();
        }
    } else {
        log::warn!("Sound.stop: this is not a Sound");
    }

    Ok(Value::Undefined.into())
}
