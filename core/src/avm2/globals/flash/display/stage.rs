//! `flash.display.Stage` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, TObject};
use crate::avm2::string::AvmString;
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::display_object::TDisplayObject;
use gc_arena::{GcCell, MutationContext};
use swf::Color;

/// Implements `flash.display.Stage`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;
    }

    Ok(Value::Undefined)
}

/// Implements `flash.display.Stage`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Overrides `accessibilityProperties`'s setter.
pub fn set_accessibility_properties<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot set accessibility properties on the stage.".into())
}

/// Overrides `alpha`'s setter.
pub fn set_alpha<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot set the stage's opacity.".into())
}

/// Overrides `blendMode`'s setter.
pub fn set_blend_mode<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot set the blend mode of the stage.".into())
}

/// Overrides `cacheAsBitmap`'s setter.
pub fn set_cache_as_bitmap<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot set the stage to be cached as a bitmap.".into())
}

/// Overrides `contextMenu`'s setter.
pub fn set_context_menu<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot set the stage's context menu.".into())
}

/// Overrides `filters`'s setter.
pub fn set_filters<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot apply filters to the stage.".into())
}

/// Overrides `focusRect`'s setter.
pub fn set_focus_rect<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot set the stage's focus rect.".into())
}

/// Overrides `loaderInfo`'s setter.
pub fn set_loader_info<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot set the blend mode of the stage.".into())
}

/// Overrides `mask`'s setter.
pub fn set_mask<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot mask the stage.".into())
}

/// Overrides `mouseEnabled`'s setter.
pub fn set_mouse_enabled<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot enable or disable the mouse on the stage.".into())
}

/// Overrides `name`'s getter.
pub fn name<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Null)
}

/// Overrides `name`'s setter.
pub fn set_name<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot set the name of the stage.".into())
}

/// Overrides `opaqueBackground`'s setter.
pub fn set_opaque_background<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot give or take away the stage's opaque background.".into())
}

/// Overrides `rotation`'s setter.
pub fn set_rotation<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot rotate the stage.".into())
}

/// Overrides `scale9Grid`'s setter.
pub fn set_scale_nine_grid<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot set the stage's 9-slice grid.".into())
}

/// Overrides `scaleX`'s setter.
pub fn set_scale_x<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot set the stage's horizontal scale.".into())
}

/// Overrides `scaleY`'s setter.
pub fn set_scale_y<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot set the stage's vertical scale.".into())
}

/// Overrides `scrollRect`'s setter.
pub fn set_scroll_rect<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot set the stage's scroll rectangle.".into())
}

/// Overrides `tabEnabled`'s setter.
pub fn set_tab_enabled<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot enable or disable tabbing the stage.".into())
}

/// Overrides `tabIndex`'s setter.
pub fn set_tab_index<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot set the stage's tab index.".into())
}

/// Overrides `transform`'s setter.
pub fn set_transform<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot transform the stage.".into())
}

/// Overrides `visible`'s setter.
pub fn set_visible<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot hide or unhide the stage.".into())
}

/// Overrides `x`'s setter.
pub fn set_x<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot move the stage horizontally.".into())
}

/// Overrides `y`'s setter.
pub fn set_y<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot move the stage vertically.".into())
}

/// Implement `align`'s getter
pub fn align<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let align = activation.context.stage.align();
    let mut s = String::with_capacity(4);
    // Match string values returned by AS.
    // It's possible to have an oxymoronic "TBLR".
    // This acts the same as "TL" (top-left takes priority).
    // This order is different between AVM1 and AVM2!
    use crate::display_object::StageAlign;
    if align.contains(StageAlign::TOP) {
        s.push('T');
    }
    if align.contains(StageAlign::BOTTOM) {
        s.push('B');
    }
    if align.contains(StageAlign::LEFT) {
        s.push('L');
    }
    if align.contains(StageAlign::RIGHT) {
        s.push('R');
    }
    let align = AvmString::new(activation.context.gc_context, s);
    Ok(align.into())
}

/// Implement `align`'s setter
pub fn set_align<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let align = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?
        .parse()
        .unwrap_or_default();
    activation
        .context
        .stage
        .set_align(&mut activation.context, align);
    Ok(Value::Undefined)
}

/// Implement `browserZoomFactor`'s getter
pub fn browser_zoom_factor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_stage())
    {
        return Ok(dobj.viewport_scale_factor().into());
    }

    Ok(Value::Undefined)
}

/// Implement `color`'s getter
pub fn color<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_stage())
    {
        let color = dobj
            .background_color()
            .unwrap_or_else(|| Color::from_rgb(0xffffff, 255));
        let rgb = color.to_rgb();
        let a = (color.a as u32) << 24;

        return Ok((rgb | a).into());
    }

    Ok(Value::Undefined)
}

/// Implement `color`'s setter
pub fn set_color<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_stage())
    {
        let color = Color::from_rgb(
            args.get(0)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_u32(activation)?,
            255,
        );
        dobj.set_background_color(activation.context.gc_context, Some(color));
    }

    Ok(Value::Undefined)
}

/// Implement `contentsScaleFactor`'s getter
pub fn contents_scale_factor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_stage())
    {
        return Ok(dobj.viewport_scale_factor().into());
    }

    Ok(Value::Undefined)
}

/// Implement `displayState`'s getter
pub fn display_state<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if activation.context.ui.is_fullscreen() {
        Ok("fullScreenInteractive".into())
    } else {
        Ok("normal".into())
    }
}

/// Implement `focus`'s getter
pub fn focus<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(activation
        .context
        .focus_tracker
        .get()
        .and_then(|focus_dobj| focus_dobj.object2().coerce_to_object(activation).ok())
        .map(|o| o.into())
        .unwrap_or(Value::Null))
}

/// Implement `focus`'s setter
pub fn set_focus<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let focus = activation.context.focus_tracker;
    match args.get(0).cloned().unwrap_or(Value::Undefined) {
        Value::Null => focus.set(None, &mut activation.context),
        val => {
            if let Some(dobj) = val.coerce_to_object(activation)?.as_display_object() {
                focus.set(Some(dobj), &mut activation.context);
            } else {
                return Err("Cannot set focus to non-DisplayObject".into());
            }
        }
    };

    Ok(Value::Undefined)
}

/// Implement `frameRate`'s getter
pub fn frame_rate<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok((*activation.context.frame_rate).into())
}

/// Implement `frameRate`'s setter
pub fn set_frame_rate<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let new_frame_rate = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_number(activation)?;
    *activation.context.frame_rate = new_frame_rate;

    Ok(Value::Undefined)
}

/// Implement `scaleMode`'s getter
pub fn scale_mode<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let scale_mode = AvmString::new(
        activation.context.gc_context,
        activation.context.stage.scale_mode().to_string(),
    );
    Ok(scale_mode.into())
}

/// Implement `scaleMode`'s setter
pub fn set_scale_mode<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Ok(scale_mode) = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?
        .parse()
    {
        activation
            .context
            .stage
            .set_scale_mode(&mut activation.context, scale_mode);
    } else {
        return Err(
            "ArgumentError: Error #2008: Parameter scaleMode must be one of the accepted values."
                .into(),
        );
    }
    Ok(Value::Undefined)
}

/// Implement `stageWidth`'s getter
pub fn stage_width<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_stage())
    {
        return Ok(dobj.stage_size().0.into());
    }

    Ok(Value::Undefined)
}

/// Implement `stageWidth`'s setter
pub fn set_stage_width<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    // For some reason this value is settable but it does nothing.
    Ok(Value::Undefined)
}

/// Implement `stageHeight`'s getter
pub fn stage_height<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_stage())
    {
        return Ok(dobj.stage_size().1.into());
    }

    Ok(Value::Undefined)
}

/// Implement `stageHeight`'s setter
pub fn set_stage_height<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    // For some reason this value is settable but it does nothing.
    Ok(Value::Undefined)
}

/// Implement `allowsFullScreen`'s getter
///
/// TODO: This is a stub.
pub fn allows_full_screen<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(true.into())
}

/// Implement `allowsFullScreenInteractive`'s getter
///
/// TODO: This is a stub.
pub fn allows_full_screen_interactive<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(false.into())
}

/// Implement `quality`'s getter
///
/// TODO: This is a stub.
pub fn quality<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok("HIGH".into())
}

/// Construct `Stage`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "Stage"),
        Some(
            QName::new(
                Namespace::package("flash.display"),
                "DisplayObjectContainer",
            )
            .into(),
        ),
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);

    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "accessibilityProperties"),
            Method::from_builtin(set_accessibility_properties),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "alpha"),
            Method::from_builtin(set_alpha),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "blendMode"),
            Method::from_builtin(set_blend_mode),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "cacheAsBitmap"),
            Method::from_builtin(set_cache_as_bitmap),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "contextMenu"),
            Method::from_builtin(set_context_menu),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "filters"),
            Method::from_builtin(set_filters),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "focusRect"),
            Method::from_builtin(set_focus_rect),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "loaderInfo"),
            Method::from_builtin(set_loader_info),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "mask"),
            Method::from_builtin(set_mask),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "mouseEnabled"),
            Method::from_builtin(set_mouse_enabled),
        )
        .with_override(),
    );

    write.define_instance_trait(
        Trait::from_getter(
            QName::new(Namespace::public(), "name"),
            Method::from_builtin(name),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "name"),
            Method::from_builtin(set_name),
        )
        .with_override(),
    );

    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "opaqueBackground"),
            Method::from_builtin(set_opaque_background),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "rotation"),
            Method::from_builtin(set_rotation),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "scale9Grid"),
            Method::from_builtin(set_scale_nine_grid),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "scaleX"),
            Method::from_builtin(set_scale_x),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "scaleY"),
            Method::from_builtin(set_scale_y),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "scrollRect"),
            Method::from_builtin(set_scroll_rect),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "tabEnabled"),
            Method::from_builtin(set_tab_enabled),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "tabIndex"),
            Method::from_builtin(set_tab_index),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "transform"),
            Method::from_builtin(set_transform),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "visible"),
            Method::from_builtin(set_visible),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "x"),
            Method::from_builtin(set_x),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "y"),
            Method::from_builtin(set_y),
        )
        .with_override(),
    );
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "align"),
        Method::from_builtin(align),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::public(), "align"),
        Method::from_builtin(set_align),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "browserZoomFactor"),
        Method::from_builtin(browser_zoom_factor),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "color"),
        Method::from_builtin(color),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::public(), "color"),
        Method::from_builtin(set_color),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "contentsScaleFactor"),
        Method::from_builtin(contents_scale_factor),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "displayState"),
        Method::from_builtin(display_state),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "focus"),
        Method::from_builtin(focus),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::public(), "focus"),
        Method::from_builtin(set_focus),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "frameRate"),
        Method::from_builtin(frame_rate),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::public(), "frameRate"),
        Method::from_builtin(set_frame_rate),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "scaleMode"),
        Method::from_builtin(scale_mode),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::public(), "scaleMode"),
        Method::from_builtin(set_scale_mode),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "stageWidth"),
        Method::from_builtin(stage_width),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::public(), "stageWidth"),
        Method::from_builtin(set_stage_width),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "stageHeight"),
        Method::from_builtin(stage_height),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::public(), "stageHeight"),
        Method::from_builtin(set_stage_height),
    ));

    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "allowsFullScreen"),
        Method::from_builtin(allows_full_screen),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "allowsFullScreenInteractive"),
        Method::from_builtin(allows_full_screen_interactive),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "quality"),
        Method::from_builtin(quality),
    ));

    class
}
