//! MovieClip prototype

use crate::avm1::function::Executable;
use crate::avm1::property::Attribute::*;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, ScriptObject, TObject, UpdateContext, Value};
use crate::display_object::{MovieClip, TDisplayObject};
use enumset::EnumSet;
use gc_arena::MutationContext;
use swf::Twips;

/// The depth at which dynamic clips are offset.
const AVM_DEPTH_BIAS: i32 = 16384;

/// Implements `MovieClip`
pub fn constructor<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(Value::Undefined.into())
}

macro_rules! with_movie_clip {
    ( $gc_context: ident, $object:ident, $fn_proto: expr, $($name:expr => $fn:expr),* ) => {{
        $(
            $object.force_set_function(
                $name,
                |avm, context: &mut UpdateContext<'_, 'gc, '_>, this, args| -> Result<ReturnValue<'gc>, Error> {
                    if let Some(display_object) = this.as_display_object() {
                        if let Some(movie_clip) = display_object.as_movie_clip() {
                            return $fn(movie_clip, avm, context, args);
                        }
                    }
                    Ok(Value::Undefined.into())
                } as crate::avm1::function::NativeFunction<'gc>,
                $gc_context,
                DontDelete | ReadOnly | DontEnum,
                $fn_proto
            );
        )*
    }};
}

pub fn overwrite_root<'gc>(
    _avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let new_val = args
        .get(0)
        .map(|v| v.to_owned())
        .unwrap_or(Value::Undefined);
    this.define_value(ac.gc_context, "_root", new_val, EnumSet::new());

    Ok(Value::Undefined.into())
}

pub fn overwrite_global<'gc>(
    _avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let new_val = args
        .get(0)
        .map(|v| v.to_owned())
        .unwrap_or(Value::Undefined);
    this.define_value(ac.gc_context, "_global", new_val, EnumSet::new());

    Ok(Value::Undefined.into())
}

#[allow(clippy::comparison_chain)]
pub fn hit_test<'gc>(
    movie_clip: MovieClip<'gc>,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if args.len() > 1 {
        let x = args.get(0).unwrap().as_number(avm, context)?;
        let y = args.get(1).unwrap().as_number(avm, context)?;
        let shape = args
            .get(2)
            .map(|v| v.as_bool(avm.current_swf_version()))
            .unwrap_or(false);
        if shape {
            log::warn!("Ignoring shape hittest and using bounding box instead. Shape based hit detection is not yet implemented. See https://github.com/ruffle-rs/ruffle/issues/177");
        }
        if x.is_finite() && y.is_finite() {
            // The docs say the point is in "Stage coordinates", but actually they are in root coordinates.
            // root can be moved via _root._x etc., so we actually have to transform from root to world space.
            let point = context
                .root
                .local_to_global((Twips::from_pixels(x), Twips::from_pixels(y)));
            return Ok(movie_clip.hit_test(point).into());
        }
    } else if args.len() == 1 {
        let other = args
            .get(0)
            .unwrap()
            .as_object()
            .ok()
            .and_then(|o| o.as_display_object());
        if let Some(other) = other {
            return Ok(other
                .world_bounds()
                .intersects(&movie_clip.world_bounds())
                .into());
        }
    }

    Ok(false.into())
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let mut object = ScriptObject::object(gc_context, Some(proto));

    with_movie_clip!(
        gc_context,
        object,
        Some(fn_proto),
        "attachMovie" => attach_movie,
        "createEmptyMovieClip" => create_empty_movie_clip,
        "duplicateMovieClip" => |movie_clip: MovieClip<'gc>, avm: &mut Avm1<'gc>, context: &mut UpdateContext<'_, 'gc, '_>, args| {
            // duplicateMovieClip method uses biased depth compared to CloneSprite
            duplicate_movie_clip(movie_clip, avm, context, args, AVM_DEPTH_BIAS)
        },
        "stopDrag" => stop_drag,
        "nextFrame" => |movie_clip: MovieClip<'gc>, _avm: &mut Avm1<'gc>, context: &mut UpdateContext<'_, 'gc, '_>, _args| {
            movie_clip.next_frame(context);
            Ok(Value::Undefined.into())
        },
        "prevFrame" => |movie_clip: MovieClip<'gc>, _avm: &mut Avm1<'gc>, context: &mut UpdateContext<'_, 'gc, '_>, _args| {
            movie_clip.prev_frame(context);
            Ok(Value::Undefined.into())
        },
        "play" => |movie_clip: MovieClip<'gc>, _avm: &mut Avm1<'gc>, context: &mut UpdateContext<'_, 'gc, '_>, _args| {
            movie_clip.play(context);
            Ok(Value::Undefined.into())
        },
        "removeMovieClip" => |movie_clip: MovieClip<'gc>, _avm: &mut Avm1<'gc>, context: &mut UpdateContext<'_, 'gc, '_>, _args| {
            // removeMovieClip method uses biased depth compared to RemoveSprite
            remove_movie_clip(movie_clip, context, AVM_DEPTH_BIAS)
        },
        "stop" => |movie_clip: MovieClip<'gc>, _avm: &mut Avm1<'gc>, context: &mut UpdateContext<'_, 'gc, '_>, _args| {
            movie_clip.stop(context);
            Ok(Value::Undefined.into())
        },
        "getBytesLoaded" => |_movie_clip: MovieClip<'gc>, _avm: &mut Avm1<'gc>, _context: &mut UpdateContext<'_, 'gc, '_>, _args| {
            // TODO find a correct value
            Ok(1.0.into())
        },
        "getBytesTotal" => |_movie_clip: MovieClip<'gc>, _avm: &mut Avm1<'gc>, _context: &mut UpdateContext<'_, 'gc, '_>, _args| {
            // TODO find a correct value
            Ok(1.0.into())
        },
        "hitTest" => |movie_clip: MovieClip<'gc>, avm: &mut Avm1<'gc>, context: &mut UpdateContext<'_, 'gc, '_>, args: &[Value<'gc>]| {
            hit_test(movie_clip, avm, context, args)
        },
        "gotoAndPlay" => goto_and_play,
        "gotoAndStop" => goto_and_stop,
        "startDrag" => start_drag,
        "toString" => |movie_clip: MovieClip<'gc>, _avm: &mut Avm1<'gc>, _context: &mut UpdateContext<'_, 'gc, '_>, _args| {
            Ok(movie_clip.path().into())
        }
    );

    object.add_property(
        gc_context,
        "_global",
        Executable::Native(|avm, context, _this, _args| Ok(avm.global_object(context).into())),
        Some(Executable::Native(overwrite_global)),
        DontDelete | ReadOnly | DontEnum,
    );

    object.add_property(
        gc_context,
        "_root",
        Executable::Native(|avm, context, _this, _args| Ok(avm.root_object(context).into())),
        Some(Executable::Native(overwrite_root)),
        DontDelete | ReadOnly | DontEnum,
    );

    object.add_property(
        gc_context,
        "_parent",
        Executable::Native(|_avm, _context, this, _args| {
            Ok(this
                .as_display_object()
                .and_then(|mc| mc.parent())
                .and_then(|dn| dn.object().as_object().ok())
                .map(Value::Object)
                .unwrap_or(Value::Undefined)
                .into())
        }),
        None,
        DontDelete | ReadOnly | DontEnum,
    );

    object.into()
}

fn attach_movie<'gc>(
    mut movie_clip: MovieClip<'gc>,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let (export_name, new_instance_name, depth) = match &args[0..3] {
        [export_name, new_instance_name, depth] => (
            export_name.clone().coerce_to_string(avm, context)?,
            new_instance_name.clone().coerce_to_string(avm, context)?,
            depth.as_i32().unwrap_or(0).wrapping_add(AVM_DEPTH_BIAS),
        ),
        _ => {
            log::error!("MovieClip.attachMovie: Too few parameters");
            return Ok(Value::Undefined.into());
        }
    };
    let init_object = args.get(3);

    // TODO: What is the derivation of this max value? It shows up a few times in the AVM...
    // 2^31 - 16777220
    if depth < 0 || depth > 2_130_706_428 {
        return Ok(Value::Undefined.into());
    }
    if let Ok(mut new_clip) = context.library.instantiate_by_export_name(
        &export_name,
        context.gc_context,
        &avm.prototypes,
    ) {
        // Set name and attach to parent.
        new_clip.set_name(context.gc_context, &new_instance_name);
        movie_clip.add_child_from_avm(context, new_clip, depth);
        new_clip.run_frame(context);

        // Copy properties from init_object to the movieclip.
        let new_clip = new_clip.object().as_object().unwrap();
        if let Some(Value::Object(o)) = init_object {
            for k in o.get_keys() {
                let value = o.get(&k, avm, context)?.resolve(avm, context)?;
                new_clip.set(&k, value, avm, context)?;
            }
        }
        Ok(new_clip.into())
    } else {
        log::warn!("Unable to attach '{}'", export_name);
        Ok(Value::Undefined.into())
    }
}

fn create_empty_movie_clip<'gc>(
    mut movie_clip: MovieClip<'gc>,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let (new_instance_name, depth) = match &args[0..2] {
        [new_instance_name, depth] => (
            new_instance_name.clone().coerce_to_string(avm, context)?,
            depth.as_i32().unwrap_or(0).wrapping_add(AVM_DEPTH_BIAS),
        ),
        _ => {
            log::error!("MovieClip.attachMovie: Too few parameters");
            return Ok(Value::Undefined.into());
        }
    };

    // Create empty movie clip.
    let mut new_clip = MovieClip::new(avm.current_swf_version(), context.gc_context);
    new_clip.post_instantiation(
        context.gc_context,
        new_clip.into(),
        avm.prototypes.movie_clip,
    );

    // Set name and attach to parent.
    new_clip.set_name(context.gc_context, &new_instance_name);
    movie_clip.add_child_from_avm(context, new_clip.into(), depth);
    new_clip.run_frame(context);

    Ok(new_clip.object().into())
}

pub fn duplicate_movie_clip<'gc>(
    movie_clip: MovieClip<'gc>,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    args: &[Value<'gc>],
    depth_bias: i32,
) -> Result<ReturnValue<'gc>, Error> {
    let (new_instance_name, depth) = match &args[0..2] {
        [new_instance_name, depth] => (
            new_instance_name.clone().coerce_to_string(avm, context)?,
            depth.as_i32().unwrap_or(0).wrapping_add(depth_bias),
        ),
        _ => {
            log::error!("MovieClip.attachMovie: Too few parameters");
            return Ok(Value::Undefined.into());
        }
    };
    let init_object = args.get(2);

    // Can't duplicate the root!
    let mut parent = if let Some(parent) = movie_clip.parent().and_then(|o| o.as_movie_clip()) {
        parent
    } else {
        return Ok(Value::Undefined.into());
    };

    // TODO: What is the derivation of this max value? It shows up a few times in the AVM...
    // 2^31 - 16777220
    if depth < 0 || depth > 2_130_706_428 {
        return Ok(Value::Undefined.into());
    }
    if let Ok(mut new_clip) =
        context
            .library
            .instantiate_by_id(movie_clip.id(), context.gc_context, &avm.prototypes)
    {
        // Set name and attach to parent.
        new_clip.set_name(context.gc_context, &new_instance_name);
        parent.add_child_from_avm(context, new_clip, depth);

        // Copy display properties from previous clip to new clip.
        new_clip.set_matrix(context.gc_context, &*movie_clip.matrix());
        new_clip.set_color_transform(context.gc_context, &*movie_clip.color_transform());
        // TODO: Any other properties we should copy...?
        // Definitely not ScriptObject properties.
        new_clip.run_frame(context);

        // Copy properties from init_object to the movieclip.
        let new_clip = new_clip.object().as_object().unwrap();
        if let Some(Value::Object(o)) = init_object {
            for k in o.get_keys() {
                let value = o.get(&k, avm, context)?.resolve(avm, context)?;
                new_clip.set(&k, value, avm, context)?;
            }
        }
        Ok(new_clip.into())
    } else {
        log::warn!("Unable to duplicate clip '{}'", movie_clip.name());
        Ok(Value::Undefined.into())
    }
}

pub fn goto_and_play<'gc>(
    movie_clip: MovieClip<'gc>,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    goto_frame(movie_clip, avm, context, args, false, 0)
}

pub fn goto_and_stop<'gc>(
    movie_clip: MovieClip<'gc>,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    goto_frame(movie_clip, avm, context, args, true, 0)
}

pub fn goto_frame<'gc>(
    movie_clip: MovieClip<'gc>,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    args: &[Value<'gc>],
    stop: bool,
    scene_offset: u16,
) -> Result<ReturnValue<'gc>, Error> {
    match args.get(0).cloned().unwrap_or(Value::Undefined) {
        Value::Number(n) => {
            // Frame #
            // Gotoing <= 0 has no effect.
            // Gotoing greater than _totalframes jumps to the last frame.
            // Wraps around as an i32.
            // TODO: -1 +1 here to match Flash's behavior.
            // We probably want to change our frame representation to 0-based.
            // Scene offset is only used by GotoFrame2 global opcode.
            let mut frame = crate::avm1::value::f64_to_wrapping_i32(n);
            frame = frame.wrapping_sub(1);
            frame = frame.wrapping_add(i32::from(scene_offset));
            if frame >= 0 {
                let num_frames = movie_clip.total_frames();
                if frame > i32::from(num_frames) {
                    movie_clip.goto_frame(context, num_frames, stop);
                } else {
                    movie_clip.goto_frame(context, frame.saturating_add(1) as u16, stop);
                }
            }
        }
        val => {
            // Coerce to string and search for a frame label.
            let frame_label = val.clone().coerce_to_string(avm, context)?;
            if let Some(mut frame) = movie_clip.frame_label_to_number(&frame_label) {
                frame = frame.wrapping_add(scene_offset);
                movie_clip.goto_frame(context, frame, stop);
            }
        }
    }
    Ok(Value::Undefined.into())
}

pub fn remove_movie_clip<'gc>(
    movie_clip: MovieClip<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    depth_bias: i32,
) -> Result<ReturnValue<'gc>, Error> {
    let depth = movie_clip.depth().wrapping_add(depth_bias);
    // Can only remove positive depths (when offset by the AVM depth bias).
    // Generally this prevents you from removing non-dynamically created clips,
    // although you can get around it with swapDepths.
    // TODO: Figure out the derivation of this range.
    if depth >= AVM_DEPTH_BIAS && depth < 2_130_706_416 {
        // Need a parent to remove from.
        let mut parent = if let Some(parent) = movie_clip.parent().and_then(|o| o.as_movie_clip()) {
            parent
        } else {
            return Ok(Value::Undefined.into());
        };

        parent.remove_child_from_avm(context, movie_clip.into());
    }
    Ok(Value::Undefined.into())
}

pub fn start_drag<'gc>(
    movie_clip: MovieClip<'gc>,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    crate::avm1::start_drag(movie_clip.into(), avm, context, args);
    Ok(Value::Undefined.into())
}

pub fn stop_drag<'gc>(
    _movie_clip: MovieClip<'gc>,
    _avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    // It doesn't matter which clip we call this on; it simply stops any active drag.
    *context.drag_object = None;
    Ok(Value::Undefined.into())
}
