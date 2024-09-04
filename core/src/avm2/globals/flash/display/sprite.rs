//! `flash.display.Sprite` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::globals::flash::display::display_object::initialize_for_allocator;
use crate::avm2::object::{Object, StageObject, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::Multiname;
use crate::avm2::{ClassObject, Error};
use crate::display_object::{MovieClip, SoundTransform, TDisplayObject};
use swf::{Rectangle, Twips};

pub fn sprite_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let sprite_cls = activation.avm2().classes().sprite.inner_class_definition();

    let mut class_def = Some(class.inner_class_definition());
    let orig_class = class;
    while let Some(class) = class_def {
        if class == sprite_cls {
            let movie = activation.caller_movie_or_root();
            let display_object = MovieClip::new(movie, activation.context.gc_context).into();
            return initialize_for_allocator(activation, display_object, orig_class);
        }

        if let Some((movie, symbol)) = activation
            .context
            .library
            .avm2_class_registry()
            .class_symbol(class)
        {
            let child = activation
                .context
                .library
                .library_for_movie_mut(movie)
                .instantiate_by_id(symbol, activation.context.gc_context)?;

            return initialize_for_allocator(activation, child, orig_class);
        }
        class_def = class.super_class();
    }
    unreachable!("A Sprite subclass should have Sprite in superclass chain");
}

/// Implements `dropTarget`'s getter
pub fn get_drop_target<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .as_display_object()
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
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let namespaces = activation.avm2().namespaces;

    if let Some(dobj) = this.as_display_object() {
        // Lazily initialize the `Graphics` object in a hidden property.
        let graphics = match this.get_property(
            &Multiname::new(namespaces.flash_display_internal, "_graphics"),
            activation,
        )? {
            Value::Undefined | Value::Null => {
                let graphics = Value::from(StageObject::graphics(activation, dobj)?);
                this.set_property(
                    &Multiname::new(namespaces.flash_display_internal, "_graphics"),
                    graphics,
                    activation,
                )?;
                graphics
            }
            graphics => graphics,
        };
        return Ok(graphics);
    }

    Ok(Value::Undefined)
}

/// Implements `soundTransform`'s getter
pub fn get_sound_transform<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.as_display_object() {
        let dobj_st = dobj.base().sound_transform().clone();

        return Ok(dobj_st.into_avm2_object(activation)?.into());
    }

    Ok(Value::Undefined)
}

/// Implements `soundTransform`'s setter
pub fn set_sound_transform<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.as_display_object() {
        let as3_st = args.get_object(activation, 0, "value")?;
        let dobj_st = SoundTransform::from_avm2_object(activation, as3_st)?;

        dobj.set_sound_transform(activation.context, dobj_st);
    }

    Ok(Value::Undefined)
}

/// Implements `buttonMode`'s getter
pub fn get_button_mode<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this.as_display_object().and_then(|o| o.as_movie_clip()) {
        return Ok(mc.forced_button_mode().into());
    }

    Ok(Value::Undefined)
}

/// Implements `buttonMode`'s setter
pub fn set_button_mode<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this.as_display_object().and_then(|o| o.as_movie_clip()) {
        let forced_button_mode = args.get_bool(0);

        mc.set_forced_button_mode(activation.context, forced_button_mode);
    }

    Ok(Value::Undefined)
}

/// Starts dragging this display object, making it follow the cursor.
pub fn start_drag<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(display_object) = this.as_display_object() {
        let lock_center = args.get_bool(0);

        let rectangle = args.try_get_object(activation, 1);
        let constraint = if let Some(rectangle) = rectangle {
            let x = rectangle
                .get_public_property("x", activation)?
                .coerce_to_number(activation)?;

            let y = rectangle
                .get_public_property("y", activation)?
                .coerce_to_number(activation)?;

            let width = rectangle
                .get_public_property("width", activation)?
                .coerce_to_number(activation)?;

            let height = rectangle
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
            last_mouse_position: *activation.context.mouse_position,
            lock_center,
            constraint,
        };
        *activation.context.drag_object = Some(drag_object);
    }
    Ok(Value::Undefined)
}

pub fn stop_drag<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // It doesn't matter which clip we call this on; it simply stops any active drag.

    // We might not have had an opportunity to call `update_drag`
    // if AS did `startDrag(mc); stopDrag();` in one go,
    // so let's do it here.
    crate::player::Player::update_drag(activation.context);

    *activation.context.drag_object = None;
    Ok(Value::Undefined)
}

/// Implements `useHandCursor`'s getter
pub fn get_use_hand_cursor<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .as_display_object()
        .and_then(|this| this.as_movie_clip())
    {
        return Ok(mc.avm2_use_hand_cursor().into());
    }

    Ok(Value::Undefined)
}

/// Implements `useHandCursor`'s setter
pub fn set_use_hand_cursor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .as_display_object()
        .and_then(|this| this.as_movie_clip())
    {
        mc.set_avm2_use_hand_cursor(activation.context, args.get_bool(0));
    }

    Ok(Value::Undefined)
}

/// Implements `hitArea`'s getter
pub fn get_hit_area<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .as_display_object()
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
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .as_display_object()
        .and_then(|this| this.as_movie_clip())
    {
        let object = args
            .try_get_object(activation, 0)
            .and_then(|hit_area| hit_area.as_display_object());
        mc.set_hit_area(activation.context, object);
    }

    Ok(Value::Undefined)
}
