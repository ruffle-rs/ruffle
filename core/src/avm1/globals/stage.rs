//! Stage object
//!
//! TODO: This is a very rough stub with not much implementation.

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::globals::as_broadcaster::BroadcasterFunctions;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, ScriptObject, Value};
use crate::display_object::StageDisplayState;
use crate::string::{AvmString, StringContext, WStr, WString};

const OBJECT_DECLS: &[Declaration] = declare_properties! {
    "align" => property(align, set_align);
    "height" => property(height);
    "scaleMode" => property(scale_mode, set_scale_mode);
    "displayState" => property(display_state, set_display_state);
    "showMenu" => property(show_menu, set_show_menu);
    "width" => property(width);
};

pub fn create_stage_object<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    array_proto: Object<'gc>,
    fn_proto: Object<'gc>,
    broadcaster_functions: BroadcasterFunctions<'gc>,
) -> Object<'gc> {
    let stage = ScriptObject::new(context.gc_context, Some(proto));
    broadcaster_functions.initialize(context.gc_context, stage.into(), array_proto);
    define_properties_on(OBJECT_DECLS, context, stage, fn_proto);
    stage.into()
}

fn align<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let align = activation.context.stage.align();
    let mut s = WString::with_capacity(4, false);
    // Match string values returned by AS.
    // It's possible to have an oxymoronic "LTRB".
    // This acts the same as "TL" (top-left takes priority).
    // This order is different between AVM1 and AVM2!
    use crate::display_object::StageAlign;
    if align.contains(StageAlign::LEFT) {
        s.push_byte(b'L');
    }
    if align.contains(StageAlign::TOP) {
        s.push_byte(b'T');
    }
    if align.contains(StageAlign::RIGHT) {
        s.push_byte(b'R');
    }
    if align.contains(StageAlign::BOTTOM) {
        s.push_byte(b'B');
    }
    let align = AvmString::new(activation.context.gc_context, s);
    Ok(align.into())
}

fn set_align<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let align = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?
        .parse()
        .unwrap_or_default();
    activation
        .context
        .stage
        .set_align(activation.context, align);
    Ok(Value::Undefined)
}

fn height<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.stage.stage_size().1.into())
}

fn scale_mode<'gc>(
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

fn set_scale_mode<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let scale_mode = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?
        .parse()
        .unwrap_or_default();
    activation
        .context
        .stage
        .set_scale_mode(activation.context, scale_mode, true);
    Ok(Value::Undefined)
}

fn display_state<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if activation.context.stage.is_fullscreen() {
        Ok("fullScreen".into())
    } else {
        Ok("normal".into())
    }
}

fn set_display_state<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let display_state = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;

    if display_state.eq_ignore_case(WStr::from_units(b"fullscreen")) {
        activation
            .context
            .stage
            .set_display_state(activation.context, StageDisplayState::FullScreen);
    } else if display_state.eq_ignore_case(WStr::from_units(b"normal")) {
        activation
            .context
            .stage
            .set_display_state(activation.context, StageDisplayState::Normal);
    }

    Ok(Value::Undefined)
}

fn show_menu<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.stage.show_menu().into())
}

fn set_show_menu<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let show_menu = args
        .get(0)
        .unwrap_or(&true.into())
        .to_owned()
        .as_bool(activation.swf_version());
    activation
        .context
        .stage
        .set_show_menu(activation.context, show_menu);
    Ok(Value::Undefined)
}

fn width<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.stage.stage_size().0.into())
}
