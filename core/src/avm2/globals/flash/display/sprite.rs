//! `flash.display.Sprite` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::globals::NS_RUFFLE_INTERNAL;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::{Object, StageObject, TObject};
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::avm2::QName;
use crate::display_object::{MovieClip, SoundTransform, TDisplayObject};
use crate::tag_utils::SwfMovie;
use gc_arena::{GcCell, MutationContext};
use ruffle_render::bounding_box::BoundingBox;
use std::sync::Arc;
use swf::Twips;

/// Implements `flash.display.Sprite`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;

        if this.as_display_object().is_none() {
            let class_object = this
                .instance_of()
                .ok_or("Attempted to construct Sprite on a bare object")?;
            let movie = Arc::new(SwfMovie::empty(activation.context.swf.version()));
            let new_do =
                MovieClip::new_with_avm2(movie, this, class_object, activation.context.gc_context);

            this.init_display_object(activation.context.gc_context, new_do.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `flash.display.Sprite`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

/// Implements `graphics`.
pub fn graphics<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut this) = this {
        if let Some(dobj) = this.as_display_object() {
            // Lazily initialize the `Graphics` object in a hidden property.
            let graphics = match this.get_property(
                &Multiname::new(Namespace::private(NS_RUFFLE_INTERNAL), "graphics"),
                activation,
            )? {
                Value::Undefined | Value::Null => {
                    let graphics = Value::from(StageObject::graphics(activation, dobj)?);
                    this.set_property(
                        &Multiname::new(Namespace::private(NS_RUFFLE_INTERNAL), "graphics"),
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
pub fn sound_transform<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
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
    activation: &mut Activation<'_, 'gc, '_>,
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
pub fn button_mode<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
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
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mc) = this
        .and_then(|o| o.as_display_object())
        .and_then(|o| o.as_movie_clip())
    {
        let forced_button_mode = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_boolean();

        mc.set_forced_button_mode(&mut activation.context, forced_button_mode);
    }

    Ok(Value::Undefined)
}

/// Starts dragging this display object, making it follow the cursor.
/// Runs via the `startDrag` method or `StartDrag` AVM1 action.
pub fn start_drag<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
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

        let constraint = if let Some(rect) = args.get(1) {
            let rect = rect.coerce_to_object(activation)?;
            let x = rect
                .get_property(&Multiname::public("x"), activation)?
                .coerce_to_number(activation)?;

            let y = rect
                .get_property(&Multiname::public("y"), activation)?
                .coerce_to_number(activation)?;

            let width = rect
                .get_property(&Multiname::public("width"), activation)?
                .coerce_to_number(activation)?;

            let height = rect
                .get_property(&Multiname::public("height"), activation)?
                .coerce_to_number(activation)?;

            BoundingBox {
                valid: true,
                x_min: Twips::from_pixels(x),
                y_min: Twips::from_pixels(y),
                x_max: Twips::from_pixels(x + width),
                y_max: Twips::from_pixels(y + height),
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

fn stop_drag<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
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
pub fn use_hand_cursor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
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
    activation: &mut Activation<'_, 'gc, '_>,
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

/// Construct `Sprite`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "Sprite"),
        Some(Multiname::new(
            Namespace::package("flash.display"),
            "DisplayObjectContainer",
        )),
        Method::from_builtin(instance_init, "<Sprite instance initializer>", mc),
        Method::from_builtin(class_init, "<Sprite class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("graphics", Some(graphics), None),
        (
            "soundTransform",
            Some(sound_transform),
            Some(set_sound_transform),
        ),
        ("buttonMode", Some(button_mode), Some(set_button_mode)),
        (
            "useHandCursor",
            Some(use_hand_cursor),
            Some(set_use_hand_cursor),
        ),
    ];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] =
        &[("startDrag", start_drag), ("stopDrag", stop_drag)];
    write.define_public_builtin_instance_methods(mc, PUBLIC_INSTANCE_METHODS);

    // Slot for lazy-initialized Graphics object.
    write.define_instance_trait(Trait::from_slot(
        QName::new(Namespace::private(NS_RUFFLE_INTERNAL), "graphics"),
        Multiname::new(Namespace::package("flash.display"), "Graphics"),
        None,
    ));

    class
}
