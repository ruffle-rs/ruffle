//! `flash.display.MovieClip` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::array::ArrayStorage;
use crate::avm2::error::argument_error;
use crate::avm2::object::{ArrayObject, Object, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::display_object::{MovieClip, Scene, TDisplayObject};
use crate::string::{AvmString, WString};

/// Implements `addFrameScript`, an undocumented method of `MovieClip` used to
/// specify what methods of a clip's class run on which frames.
pub fn add_frame_script<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_movie_clip())
    {
        for (frame_id, callable) in args.chunks_exact(2).map(|s| (s[0], s[1])) {
            let frame_id = frame_id.coerce_to_u32(activation)? as u16 + 1;
            let callable = callable.as_callable(activation, None, None, false).ok();

            mc.register_frame_script(frame_id, callable, activation.context);
        }
    } else {
        tracing::error!("Attempted to add frame scripts to non-MovieClip this!");
    }

    Ok(Value::Undefined)
}

/// Implements `currentFrame`.
pub fn get_current_frame<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_movie_clip())
    {
        if let Some(Scene {
            name: _,
            start,
            length: _,
        }) = mc.current_scene()
        {
            return Ok(((mc.current_frame() + 1) - start).into());
        } else {
            return Ok(mc.current_frame().into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `currentFrameLabel`.
pub fn get_current_frame_label<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .as_display_object()
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
pub fn get_current_label<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .as_display_object()
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

/// Given a scene, produce its name, length, and a list of frame labels.
///
/// The intended purpose of this output is to be sent directly into the
/// constructor of `flash.display.Scene`.
fn labels_for_scene<'gc>(
    activation: &mut Activation<'_, 'gc>,
    mc: MovieClip<'gc>,
    scene: &Scene,
) -> Result<(String, u16, Object<'gc>), Error<'gc>> {
    let Scene {
        name: scene_name,
        start: scene_start,
        length: scene_length,
    } = scene;
    let frame_label_class = activation.context.avm2.classes().framelabel;
    let labels = mc.labels_in_range(*scene_start, scene_start + scene_length);
    let mut frame_labels = Vec::with_capacity(labels.len());

    for (name, frame) in labels {
        let name: Value<'gc> = AvmString::new(activation.context.gc_context, name).into();
        let local_frame = frame - scene_start + 1;
        let args = [name, local_frame.into()];
        let frame_label = frame_label_class.construct(activation, &args)?;

        frame_labels.push(Some(frame_label.into()));
    }

    Ok((
        scene_name.to_string(),
        *scene_length,
        ArrayObject::from_storage(activation, ArrayStorage::from_storage(frame_labels))?,
    ))
}

/// Implements `currentLabels`.
pub fn get_current_labels<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_movie_clip())
    {
        let scene = mc.current_scene().unwrap_or_else(|| Scene {
            name: WString::default(),
            start: 1,
            length: mc.total_frames(),
        });
        return Ok(labels_for_scene(activation, mc, &scene)?.2.into());
    }

    Ok(Value::Undefined)
}

/// Implements `currentScene`.
pub fn get_current_scene<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_movie_clip())
    {
        let scene = mc.current_scene().unwrap_or_else(|| Scene {
            name: WString::default(),
            start: 1,
            length: mc.total_frames(),
        });
        let (scene_name, scene_length, scene_labels) = labels_for_scene(activation, mc, &scene)?;
        let scene_class = activation.context.avm2.classes().scene;
        let args = [
            AvmString::new_utf8(activation.context.gc_context, scene_name).into(),
            scene_labels.into(),
            scene_length.into(),
        ];

        let scene = scene_class.construct(activation, &args)?;

        return Ok(scene.into());
    }

    Ok(Value::Undefined)
}

pub fn get_enabled<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_movie_clip())
    {
        return Ok(mc.avm2_enabled().into());
    }

    Ok(Value::Undefined)
}

pub fn set_enabled<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_movie_clip())
    {
        let enabled = args.get_bool(0);

        mc.set_avm2_enabled(activation.context, enabled);
    }

    Ok(Value::Undefined)
}

/// Implements `scenes`.
pub fn get_scenes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_movie_clip())
    {
        let mut mc_scenes = mc.scenes();
        if mc.scenes().is_empty() {
            mc_scenes.push(Scene {
                name: WString::default(),
                start: 1,
                length: mc.total_frames(),
            });
        }

        let mut scene_objects = Vec::with_capacity(mc_scenes.len());
        for scene in mc_scenes {
            let (scene_name, scene_length, scene_labels) =
                labels_for_scene(activation, mc, &scene)?;
            let scene_class = activation.context.avm2.classes().scene;
            let args = [
                AvmString::new_utf8(activation.context.gc_context, scene_name).into(),
                scene_labels.into(),
                scene_length.into(),
            ];

            let scene = scene_class.construct(activation, &args)?;

            scene_objects.push(Some(scene.into()));
        }

        return Ok(ArrayObject::from_storage(
            activation,
            ArrayStorage::from_storage(scene_objects),
        )?
        .into());
    }

    Ok(Value::Undefined)
}

/// Implements `framesLoaded`.
pub fn get_frames_loaded<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_movie_clip())
    {
        return Ok(mc.frames_loaded().into());
    }

    Ok(Value::Undefined)
}

/// Implements `isPlaying`.
pub fn get_is_playing<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_movie_clip())
    {
        return Ok((mc.programmatically_played() && mc.playing()).into());
    }

    Ok(Value::Undefined)
}

/// Implements `totalFrames`.
pub fn get_total_frames<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_movie_clip())
    {
        return Ok(mc.total_frames().into());
    }

    Ok(Value::Undefined)
}

/// Implements `gotoAndPlay`.
pub fn goto_and_play<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_movie_clip())
    {
        mc.set_programmatically_played(activation.context.gc_context);
        goto_frame(activation, mc, args, false)?;
    }

    Ok(Value::Undefined)
}

/// Implements `gotoAndStop`.
pub fn goto_and_stop<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_movie_clip())
    {
        goto_frame(activation, mc, args, true)?;
    }

    Ok(Value::Undefined)
}

pub fn goto_frame<'gc>(
    activation: &mut Activation<'_, 'gc>,
    mc: MovieClip<'gc>,
    args: &[Value<'gc>],
    stop: bool,
) -> Result<(), Error<'gc>> {
    let frame_or_label = args.get(0).cloned().unwrap_or(Value::Null);

    let scene = match args.try_get_string(activation, 1)? {
        None => mc
            .current_scene()
            .and_then(|scene| mc.scene_label_to_number(&scene.name))
            .map(|v| v.saturating_sub(1)),
        Some(label) => mc
            .scene_label_to_number(&label)
            .map(|v| v.saturating_sub(1)),
    }
    .unwrap_or(0) as i32;
    let frame = match frame_or_label {
        Value::Integer(i) => i + scene,
        frame_or_label => {
            let frame_or_label = frame_or_label.coerce_to_string(activation)?;
            let frame = crate::avm2::value::string_to_int(&frame_or_label, 10, true);
            if !frame.is_nan() {
                (frame as i32)
                    .wrapping_sub(1)
                    .wrapping_add(scene)
                    .saturating_add(1)
            } else {
                if !matches!(args[1], Value::Null) {
                    //If the user specified a scene, we need to validate that
                    //the requested frame exists within that scene.
                    let scene = args[1].coerce_to_string(activation)?;
                    if !mc.frame_exists_within_scene(&frame_or_label, &scene, activation.context) {
                        return Err(Error::AvmError(argument_error(
                            activation,
                            &format!("Error #2109: Frame label {frame_or_label} not found in scene {scene}."),
                            2109,
                        )?));
                    }
                }

                let frame = mc.frame_label_to_number(&frame_or_label, activation.context);

                if activation.caller_movie_or_root().version() >= 11 {
                    frame.ok_or(
                        // TODO: Also include the scene in the error message, as done above
                        Error::AvmError(argument_error(
                            activation,
                            &format!("Error #2109: {frame_or_label} is not a valid frame label."),
                            2109,
                        )?),
                    )? as i32
                } else {
                    frame.unwrap_or(0) as i32 // Old swf versions silently jump to frame 1 for invalid labels.
                }
            }
        }
    };

    mc.goto_frame(activation.context, frame.max(1) as u16, stop);

    Ok(())
}

/// Implements `stop`.
pub fn stop<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_movie_clip())
    {
        mc.stop(activation.context);
    }

    Ok(Value::Undefined)
}

/// Implements `play`.
pub fn play<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_movie_clip())
    {
        mc.set_programmatically_played(activation.context.gc_context);
        mc.play(activation.context);
    }

    Ok(Value::Undefined)
}

/// Implements `prevFrame`.
pub fn prev_frame<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_movie_clip())
    {
        mc.prev_frame(activation.context);
    }

    Ok(Value::Undefined)
}

/// Implements `nextFrame`.
pub fn next_frame<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_movie_clip())
    {
        mc.next_frame(activation.context);
    }

    Ok(Value::Undefined)
}

/// Implements `prevScene`.
pub fn prev_scene<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_movie_clip())
    {
        if let Some(Scene {
            name: _,
            start,
            length: _,
        }) = mc.previous_scene()
        {
            mc.goto_frame(activation.context, start, false);
        }
    }

    Ok(Value::Undefined)
}

/// Implements `nextScene`.
pub fn next_scene<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_movie_clip())
    {
        if let Some(Scene {
            name: _,
            start,
            length: _,
        }) = mc.next_scene()
        {
            mc.goto_frame(activation.context, start, false);
        }
    }

    Ok(Value::Undefined)
}
