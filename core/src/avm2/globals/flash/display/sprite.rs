//! `flash.display.Sprite` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::object::{Object, StageObject, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::display_object::{MovieClip, SoundTransform, TDisplayObject};
use crate::tag_utils::SwfMovie;
use std::sync::Arc;
use swf::{Rectangle, Twips};

/// Implements `flash.display.Sprite`'s `init` method, which is called from the constructor
pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;

        if this.as_display_object().is_none() {
            init_empty_sprite(activation, this)?;
        }
    }

    Ok(Value::Undefined)
}

pub fn init_empty_sprite<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
) -> Result<(), Error<'gc>> {
    let class_object = this
        .instance_of()
        .ok_or("Attempted to construct Sprite on a bare object")?;
    let movie = Arc::new(SwfMovie::empty(activation.context.swf.version()));
    let new_do = MovieClip::new_with_avm2(movie, this, class_object, activation.context.gc_context);

    this.init_display_object(&mut activation.context, new_do.into());

    Ok(())
}

/// Implements `dropTarget`'s getter
pub fn get_drop_target<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .and_then(|o| o.as_display_object())
        .and_then(|o| o.as_movie_clip())
        .and_then(|o| o.drop_target())
    {
        return Ok(mc.object2());
    }

    Ok(Value::Null)
}

/// Implements `graphics`.
pub fn get_graphics<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut this) = this {
        if let Some(dobj) = this.as_display_object() {
            // Lazily initialize the `Graphics` object in a hidden property.
            let graphics = match this.get_property(
                &Multiname::new(activation.avm2().flash_display_internal, "_graphics"),
                activation,
            )? {
                Value::Undefined | Value::Null => {
                    let graphics = Value::from(StageObject::graphics(activation, dobj)?);
                    this.set_property(
                        &Multiname::new(activation.avm2().flash_display_internal, "_graphics"),
                        graphics,
                        activation,
                    )?;
                    graphics
                }
                graphics => graphics,
            };
            return Ok(graphics);
        }
    }

    Ok(Value::Undefined)
}

/// Implements `soundTransform`'s getter
pub fn get_sound_transform<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|o| o.as_display_object()) {
        let dobj_st = dobj.base().sound_transform().clone();

        return Ok(dobj_st.into_avm2_object(activation)?.into());
    }

    Ok(Value::Undefined)
}

/// Implements `soundTransform`'s setter
pub fn set_sound_transform<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|o| o.as_display_object()) {
        let as3_st = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_object(activation)?;
        let dobj_st = SoundTransform::from_avm2_object(activation, as3_st)?;

        dobj.set_sound_transform(&mut activation.context, dobj_st);
    }

    Ok(Value::Undefined)
}

/// Implements `buttonMode`'s getter
pub fn get_button_mode<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .and_then(|o| o.as_display_object())
        .and_then(|o| o.as_movie_clip())
    {
        return Ok(mc.forced_button_mode().into());
    }

    Ok(Value::Undefined)
}

/// Implements `buttonMode`'s setter
pub fn set_button_mode<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .and_then(|o| o.as_display_object())
        .and_then(|o| o.as_movie_clip())
    {
        let forced_button_mode = args.get_bool(0);

        mc.set_forced_button_mode(&mut activation.context, forced_button_mode);
    }

    Ok(Value::Undefined)
}

/// Starts dragging this display object, making it follow the cursor.
/// Runs via the `startDrag` method or `StartDrag` AVM1 action.
pub fn start_drag<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(display_object) = this.and_then(|this| this.as_display_object()) {
        let lock_center = args.get(0).map(|o| o.coerce_to_boolean()).unwrap_or(false);

        let offset = if lock_center {
            // The object's origin point is locked to the mouse.
            Default::default()
        } else {
            // The object moves relative to current mouse position.
            // Calculate the offset from the mouse to the object in world space.
            let (object_x, object_y) = display_object.local_to_global(Default::default());
            let (mouse_x, mouse_y) = *activation.context.mouse_position;
            (object_x - mouse_x, object_y - mouse_y)
        };

        let constraint = if !matches!(args[1], Value::Null) {
            let rect = args[1].coerce_to_object(activation)?;
            let x = rect
                .get_public_property("x", activation)?
                .coerce_to_number(activation)?;

            let y = rect
                .get_public_property("y", activation)?
                .coerce_to_number(activation)?;

            let width = rect
                .get_public_property("width", activation)?
                .coerce_to_number(activation)?;

            let height = rect
                .get_public_property("height", activation)?
                .coerce_to_number(activation)?;

            // Normalize the bounds.
            let mut x_min = Twips::from_pixels(x);
            let mut y_min = Twips::from_pixels(y);
            let mut x_max = Twips::from_pixels(x + width);
            let mut y_max = Twips::from_pixels(y + height);
            if x_max.get() < x_min.get() {
                std::mem::swap(&mut x_min, &mut x_max);
            }
            if y_max.get() < y_min.get() {
                std::mem::swap(&mut y_min, &mut y_max);
            }

            Rectangle {
                x_min,
                y_min,
                x_max,
                y_max,
            }
        } else {
            // No constraints.
            Default::default()
        };

        let drag_object = crate::player::DragObject {
            display_object,
            offset,
            constraint,
        };
        *activation.context.drag_object = Some(drag_object);
    }
    Ok(Value::Undefined)
}

pub fn stop_drag<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // It doesn't matter which clip we call this on; it simply stops any active drag.

    // we might not have had an opportunity to call `update_drag`
    // if AS did `startDrag(mc);stopDrag();` in one go
    // so let's do it here
    crate::player::Player::update_drag(&mut activation.context);

    *activation.context.drag_object = None;
    Ok(Value::Undefined)
}

/// Implements `useHandCursor`'s getter
pub fn get_use_hand_cursor<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_movie_clip())
    {
        return Ok(mc.use_hand_cursor().into());
    }

    Ok(Value::Undefined)
}

/// Implements `useHandCursor`'s setter
pub fn set_use_hand_cursor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_movie_clip())
    {
        mc.set_use_hand_cursor(
            &mut activation.context,
            args.get(0)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_boolean(),
        );
    }

    Ok(Value::Undefined)
}

/// Implements `hitArea`'s getter
pub fn get_hit_area<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .and_then(|o| o.as_display_object())
        .and_then(|o| o.as_movie_clip())
        .and_then(|o| o.hit_area())
    {
        return Ok(mc.object2());
    }

    Ok(Value::Null)
}

/// Implements `hitArea`'s setter
pub fn set_hit_area<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_movie_clip())
    {
        mc.set_hit_area(
            &mut activation.context,
            args.get(0)
                .and_then(|hit_area| hit_area.as_object())
                .and_then(|hit_area| hit_area.as_display_object()),
        );
    }

    Ok(Value::Undefined)
}
