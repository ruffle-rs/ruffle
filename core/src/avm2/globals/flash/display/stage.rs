//! `flash.display.Stage` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::error::make_error_2008;
use crate::avm2::object::{Object, TObject, VectorObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::vector::VectorStorage;
use crate::avm2::Error;
use crate::display_object::{
    StageDisplayState, TDisplayObject, TDisplayObjectContainer, TInteractiveObject,
};
use crate::string::{AvmString, WString};
use crate::{avm2_stub_getter, avm2_stub_setter};
use swf::Color;

/// Implements `flash.display.Stage`'s native instance constructor.
pub fn native_instance_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    activation.super_init(this, args)?;

    Ok(Value::Undefined)
}

/// Implement `align`'s getter
pub fn get_align<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let align = activation.context.stage.align();
    let mut s = WString::with_capacity(4, false);
    // Match string values returned by AS.
    // It's possible to have an oxymoronic "TBLR".
    // This acts the same as "TL" (top-left takes priority).
    // This order is different between AVM1 and AVM2!
    use crate::display_object::StageAlign;
    if align.contains(StageAlign::TOP) {
        s.push_byte(b'T');
    }
    if align.contains(StageAlign::BOTTOM) {
        s.push_byte(b'B');
    }
    if align.contains(StageAlign::LEFT) {
        s.push_byte(b'L');
    }
    if align.contains(StageAlign::RIGHT) {
        s.push_byte(b'R');
    }
    let align = AvmString::new(activation.context.gc_context, s);
    Ok(align.into())
}

/// Implement `align`'s setter
pub fn set_align<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let align = args.get_string(activation, 0)?.parse().unwrap_or_default();
    activation
        .context
        .stage
        .set_align(activation.context, align);
    Ok(Value::Undefined)
}

/// Implement `browserZoomFactor`'s getter
pub fn get_browser_zoom_factor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if this
        .as_display_object()
        .and_then(|this| this.as_stage())
        .is_some()
    {
        return Ok(activation
            .context
            .renderer
            .viewport_dimensions()
            .scale_factor
            .into());
    }

    Ok(Value::Undefined)
}

/// Implement `color`'s getter
pub fn get_color<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.as_display_object().and_then(|this| this.as_stage()) {
        let color = dobj.background_color().unwrap_or(Color::WHITE);
        return Ok(color.to_rgba().into());
    }

    Ok(Value::Undefined)
}

/// Implement `color`'s setter
pub fn set_color<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.as_display_object().and_then(|this| this.as_stage()) {
        let color = Color::from_rgb(args.get_u32(activation, 0)?, 255);
        dobj.set_background_color(activation.context.gc_context, Some(color));
    }

    Ok(Value::Undefined)
}

/// Implement `contentsScaleFactor`'s getter
pub fn get_contents_scale_factor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if this
        .as_display_object()
        .and_then(|this| this.as_stage())
        .is_some()
    {
        return Ok(activation
            .context
            .renderer
            .viewport_dimensions()
            .scale_factor
            .into());
    }

    Ok(Value::Undefined)
}

/// Implement `displayState`'s getter
pub fn get_display_state<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let display_state = AvmString::new_utf8(
        activation.context.gc_context,
        activation.context.stage.display_state().to_string(),
    );
    Ok(display_state.into())
}

/// Implement `displayState`'s setter
pub fn set_display_state<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Ok(mut display_state) = args.get_string(activation, 0)?.parse() {
        // It's not entirely clear why when setting to FullScreen, desktop flash player at least will
        // set its value to FullScreenInteractive. Overriding until flash logic is clearer.
        if display_state == StageDisplayState::FullScreen {
            display_state = StageDisplayState::FullScreenInteractive;
        }
        activation
            .context
            .stage
            .set_display_state(activation.context, display_state);
    } else {
        return Err(make_error_2008(activation, "displayState"));
    }
    Ok(Value::Undefined)
}

/// Implement `focus`'s getter
pub fn get_focus<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation
        .context
        .focus_tracker
        .get()
        .map(|o| o.as_displayobject())
        .and_then(|focus_dobj| focus_dobj.object2().as_object())
        .map(|o| o.into())
        .unwrap_or(Value::Null))
}

/// Implement `focus`'s setter
pub fn set_focus<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let focus = activation.context.focus_tracker;
    match args.try_get_object(activation, 0) {
        None => focus.set(None, activation.context),
        Some(obj) => {
            if let Some(dobj) = obj.as_display_object().and_then(|o| o.as_interactive()) {
                focus.set(Some(dobj), activation.context);
            } else {
                return Err("Cannot set focus to non-DisplayObject".into());
            }
        }
    };

    Ok(Value::Undefined)
}

/// Implement `frameRate`'s getter
pub fn get_frame_rate<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok((*activation.context.frame_rate).into())
}

/// Implement `frameRate`'s setter
pub fn set_frame_rate<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if !activation.context.forced_frame_rate {
        let new_frame_rate = args.get_f64(activation, 0)?;
        *activation.context.frame_rate = new_frame_rate;
    }

    Ok(Value::Undefined)
}

pub fn get_show_default_context_menu<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.stage.show_menu().into())
}

pub fn set_show_default_context_menu<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let show_default_context_menu = args.get_bool(0);
    activation
        .context
        .stage
        .set_show_menu(activation.context, show_default_context_menu);
    Ok(Value::Undefined)
}

/// Implement `scaleMode`'s getter
pub fn get_scale_mode<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let scale_mode = AvmString::new_utf8(
        activation.context.gc_context,
        activation.context.stage.scale_mode().to_string(),
    );
    Ok(scale_mode.into())
}

/// Implement `scaleMode`'s setter
pub fn set_scale_mode<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Ok(scale_mode) = args.get_string(activation, 0)?.parse() {
        activation
            .context
            .stage
            .set_scale_mode(activation.context, scale_mode, true);
    } else {
        return Err(make_error_2008(activation, "scaleMode"));
    }
    Ok(Value::Undefined)
}

/// Implement `stageFocusRect`'s getter
pub fn get_stage_focus_rect<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.as_display_object().and_then(|this| this.as_stage()) {
        return Ok(dobj.stage_focus_rect().into());
    }

    Ok(Value::Undefined)
}

/// Implement `stageFocusRect`'s setter
pub fn set_stage_focus_rect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.as_display_object().and_then(|this| this.as_stage()) {
        let rf = args.get_bool(0);
        dobj.set_stage_focus_rect(activation.context.gc_context, rf);
    }

    Ok(Value::Undefined)
}

/// Implement `stageWidth`'s getter
pub fn get_stage_width<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.as_display_object().and_then(|this| this.as_stage()) {
        return Ok(dobj.stage_size().0.into());
    }

    Ok(Value::Undefined)
}

/// Implement `stageWidth`'s setter
pub fn set_stage_width<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // For some reason this value is settable but it does nothing.
    Ok(Value::Undefined)
}

/// Implement `stageHeight`'s getter
pub fn get_stage_height<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.as_display_object().and_then(|this| this.as_stage()) {
        return Ok(dobj.stage_size().1.into());
    }

    Ok(Value::Undefined)
}

/// Implement `stageHeight`'s setter
pub fn set_stage_height<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // For some reason this value is settable but it does nothing.
    Ok(Value::Undefined)
}

/// Implement `allowsFullScreen`'s getter
pub fn get_allows_full_screen<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.display.Stage", "allowsFullScreen");
    Ok(true.into())
}

/// Implement `allowsFullScreenInteractive`'s getter
pub fn get_allows_full_screen_interactive<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(
        activation,
        "flash.display.Stage",
        "allowsFullScreenInteractive"
    );
    Ok(false.into())
}

/// Implement `quality`'s getter
pub fn get_quality<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let quality = activation.context.stage.quality().into_avm_str();
    Ok(AvmString::from(quality).into())
}

/// Implement `quality`'s setter
pub fn set_quality<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Invalid values result in no change.
    if let Ok(quality) = args.get_string(activation, 0)?.parse() {
        activation
            .context
            .stage
            .set_quality(activation.context, quality);
    }
    Ok(Value::Undefined)
}

/// Implement `stage3Ds`'s getter
pub fn get_stage3ds<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(stage) = this.as_display_object().and_then(|this| this.as_stage()) {
        let storage = VectorStorage::from_values(
            stage
                .stage3ds()
                .iter()
                .map(|obj| Value::Object(*obj))
                .collect(),
            false,
            Some(activation.avm2().classes().stage3d.inner_class_definition()),
        );
        let stage3ds = VectorObject::from_vector(storage, activation)?;
        return Ok(stage3ds.into());
    }
    Ok(Value::Undefined)
}

/// Implement `invalidate`
pub fn invalidate<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(stage) = this.as_display_object().and_then(|this| this.as_stage()) {
        stage.set_invalidated(activation.context.gc_context, true);
    }
    Ok(Value::Undefined)
}

/// Stage.fullScreenSourceRect's getter
pub fn get_full_screen_source_rect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.display.Stage", "fullScreenSourceRect");
    Ok(Value::Undefined)
}

/// Stage.fullScreenSourceRect's setter
pub fn set_full_screen_source_rect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_setter!(activation, "flash.display.Stage", "fullScreenSourceRect");
    Ok(Value::Undefined)
}

/// Stage.fullScreenHeight's getter
pub fn get_full_screen_height<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.display.Stage", "fullScreenHeight");
    Ok(768.into())
}

/// Stage.fullScreenWidth's getter
pub fn get_full_screen_width<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.display.Stage", "fullScreenWidth");
    Ok(1024.into())
}

pub fn set_tab_children<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(stage) = this.as_display_object().and_then(|this| this.as_stage()) {
        // TODO FP actually refers here to the original root,
        //      even if it has been removed.
        //      See the test tab_ordering_stage_tab_children_remove_root.
        if let Some(root) = stage.root_clip() {
            if let Some(root) = root.as_container() {
                // Stage's tabChildren setter just propagates the value to the AVM2 root.
                // It does not affect the value of tabChildren of the stage, which is always true.
                let value = args.get_bool(0);
                root.set_tab_children(activation.context, value);
            }
        }
    }

    Ok(Value::Undefined)
}
