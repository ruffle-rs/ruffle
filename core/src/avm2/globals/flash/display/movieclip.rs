//! `flash.display.MovieClip` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, TObject};
use crate::avm2::string::AvmString;
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::display_object::{MovieClip, TDisplayObject};
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.display.MovieClip`'s instance constructor.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `flash.display.MovieClip`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `addFrameScript`, an undocumented method of `MovieClip` used to
/// specify what methods of a clip's class run on which frames.
pub fn add_frame_script<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mc) = this
        .and_then(|o| o.as_display_object())
        .and_then(|dobj| dobj.as_movie_clip())
    {
        for (frame_id, callable) in args.chunks_exact(2).map(|s| (s[0].clone(), s[1].clone())) {
            let frame_id = frame_id.coerce_to_u32(activation)? as u16 + 1;
            let callable = callable.coerce_to_object(activation)?;

            mc.register_frame_script(frame_id, callable, &mut activation.context);
        }
    } else {
        log::error!("Attempted to add frame scripts to non-MovieClip this!");
    }

    Ok(Value::Undefined)
}

/// Implements `currentFrame`.
pub fn current_frame<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mc) = this
        .and_then(|o| o.as_display_object())
        .and_then(|dobj| dobj.as_movie_clip())
    {
        if let Some((_scene, scene_basis)) = mc.current_scene() {
            return Ok(((mc.current_frame() + 1) - scene_basis).into());
        } else {
            return Ok(mc.current_frame().into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `currentFrameLabel`.
pub fn current_frame_label<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mc) = this
        .and_then(|o| o.as_display_object())
        .and_then(|dobj| dobj.as_movie_clip())
    {
        return Ok(mc
            .current_label()
            .and_then(|(label, start_frame)| {
                if start_frame < mc.current_frame() {
                    None
                } else {
                    Some(AvmString::new(activation.context.gc_context, label).into())
                }
            })
            .unwrap_or(Value::Null));
    }

    Ok(Value::Undefined)
}

/// Implements `currentLabel`.
pub fn current_label<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mc) = this
        .and_then(|o| o.as_display_object())
        .and_then(|dobj| dobj.as_movie_clip())
    {
        return Ok(mc
            .current_label()
            .map(|(label, _start_frame)| {
                AvmString::new(activation.context.gc_context, label).into()
            })
            .unwrap_or(Value::Null));
    }

    Ok(Value::Undefined)
}

/// Implements `framesLoaded`.
pub fn frames_loaded<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mc) = this
        .and_then(|o| o.as_display_object())
        .and_then(|dobj| dobj.as_movie_clip())
    {
        return Ok(mc.frames_loaded().into());
    }

    Ok(Value::Undefined)
}

/// Implements `isPlaying`.
pub fn is_playing<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mc) = this
        .and_then(|o| o.as_display_object())
        .and_then(|dobj| dobj.as_movie_clip())
    {
        return Ok(mc.playing().into());
    }

    Ok(Value::Undefined)
}

/// Implements `totalFrames`.
pub fn total_frames<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mc) = this
        .and_then(|o| o.as_display_object())
        .and_then(|dobj| dobj.as_movie_clip())
    {
        return Ok(mc.total_frames().into());
    }

    Ok(Value::Undefined)
}

/// Implements `gotoAndPlay`.
pub fn goto_and_play<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mc) = this
        .and_then(|o| o.as_display_object())
        .and_then(|dobj| dobj.as_movie_clip())
    {
        goto_frame(activation, mc, args, false)?;
    }

    Ok(Value::Undefined)
}

/// Implements `gotoAndStop`.
pub fn goto_and_stop<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mc) = this
        .and_then(|o| o.as_display_object())
        .and_then(|dobj| dobj.as_movie_clip())
    {
        goto_frame(activation, mc, args, true)?;
    }

    Ok(Value::Undefined)
}

pub fn goto_frame<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    mc: MovieClip<'gc>,
    args: &[Value<'gc>],
    stop: bool,
) -> Result<(), Error> {
    let frame_or_label = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Null)
        .coerce_to_object(activation)?;

    if frame_or_label.has_prototype_in_chain(
        activation.avm2().system_prototypes.as_ref().unwrap().string,
        false,
    )? {
        let label = args.get(0).cloned().unwrap().coerce_to_string(activation)?;
        let frame = mc
            .frame_label_to_number(&label)
            .ok_or_else(|| format!("ArgumentError: {} is not a valid frame label.", label))?;

        mc.goto_frame(&mut activation.context, frame - 1, stop);
    } else {
        let frame = args.get(0).cloned().unwrap().coerce_to_u32(activation)? as u16;
        let scene = match args.get(1).cloned().unwrap_or(Value::Null) {
            Value::Null => None,
            v => mc
                .scene_label_to_number(&v.coerce_to_string(activation)?)
                .map(|v| v.saturating_sub(1)),
        }
        .unwrap_or(0);

        mc.goto_frame(&mut activation.context, frame + scene - 1, stop);
    }

    Ok(())
}

/// Implements `stop`.
pub fn stop<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mc) = this
        .and_then(|o| o.as_display_object())
        .and_then(|dobj| dobj.as_movie_clip())
    {
        mc.stop(&mut activation.context);
    }

    Ok(Value::Undefined)
}

/// Implements `play`.
pub fn play<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mc) = this
        .and_then(|o| o.as_display_object())
        .and_then(|dobj| dobj.as_movie_clip())
    {
        mc.play(&mut activation.context);
    }

    Ok(Value::Undefined)
}

/// Implements `prevFrame`.
pub fn prev_frame<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mc) = this
        .and_then(|o| o.as_display_object())
        .and_then(|dobj| dobj.as_movie_clip())
    {
        let target_frame = mc.current_frame().saturating_sub(1);

        if target_frame > 0 {
            mc.goto_frame(&mut activation.context, target_frame, true);
        }
    }

    Ok(Value::Undefined)
}

/// Implements `nextFrame`.
pub fn next_frame<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mc) = this
        .and_then(|o| o.as_display_object())
        .and_then(|dobj| dobj.as_movie_clip())
    {
        let target_frame = mc.current_frame().saturating_add(1);
        let total_frames = mc.total_frames();

        if target_frame <= total_frames {
            mc.goto_frame(&mut activation.context, target_frame, true);
        }
    }

    Ok(Value::Undefined)
}

/// Construct `MovieClip`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "MovieClip"),
        Some(QName::new(Namespace::package("flash.display"), "Sprite").into()),
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    let mut write = class.write(mc);

    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::package(""), "addFrameScript"),
        Method::from_builtin(add_frame_script),
    ));

    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::package(""), "currentFrame"),
        Method::from_builtin(current_frame),
    ));

    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::package(""), "currentFrameLabel"),
        Method::from_builtin(current_frame_label),
    ));

    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::package(""), "currentLabel"),
        Method::from_builtin(current_label),
    ));

    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::package(""), "framesLoaded"),
        Method::from_builtin(frames_loaded),
    ));

    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::package(""), "isPlaying"),
        Method::from_builtin(is_playing),
    ));

    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::package(""), "totalFrames"),
        Method::from_builtin(total_frames),
    ));

    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::package(""), "gotoAndPlay"),
        Method::from_builtin(goto_and_play),
    ));

    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::package(""), "gotoAndStop"),
        Method::from_builtin(goto_and_stop),
    ));

    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::package(""), "stop"),
        Method::from_builtin(stop),
    ));

    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::package(""), "play"),
        Method::from_builtin(play),
    ));

    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::package(""), "prevFrame"),
        Method::from_builtin(prev_frame),
    ));

    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::package(""), "nextFrame"),
        Method::from_builtin(next_frame),
    ));

    class
}
