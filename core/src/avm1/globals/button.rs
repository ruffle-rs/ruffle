//! Button prototype

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{globals, Object, ScriptObject, TObject, Value};
use crate::display_object::{Avm1Button, TDisplayObject};
use crate::string::AvmString;
use gc_arena::MutationContext;
use std::str::FromStr;
use swf::BlendMode;

macro_rules! button_getter {
    ($name:ident) => {
        |activation, this, _args| {
            if let Some(display_object) = this.as_display_object() {
                if let Some(button) = display_object.as_avm1_button() {
                    return $name(button, activation);
                }
            }
            Ok(Value::Undefined)
        }
    };
}

macro_rules! button_setter {
    ($name:ident) => {
        |activation, this, args| {
            if let Some(display_object) = this.as_display_object() {
                if let Some(button) = display_object.as_avm1_button() {
                    let value = args.get(0).unwrap_or(&Value::Undefined).clone();
                    $name(button, activation, value)?;
                }
            }
            Ok(Value::Undefined)
        }
    };
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "enabled" => property(button_getter!(enabled), button_setter!(set_enabled));
    "getDepth" => method(globals::get_depth; DONT_ENUM | DONT_DELETE | READ_ONLY; version(6));
    "useHandCursor" => property(button_getter!(use_hand_cursor), button_setter!(set_use_hand_cursor));
    "blendMode" => property(button_getter!(blend_mode), button_setter!(set_blend_mode); DONT_DELETE | DONT_ENUM);
};

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = ScriptObject::object(gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, gc_context, object, fn_proto);
    object.into()
}

/// Implements `Button` constructor.
pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.into())
}

fn enabled<'gc>(
    this: Avm1Button<'gc>,
    _activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.enabled().into())
}

fn set_enabled<'gc>(
    this: Avm1Button<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    let enabled = value.as_bool(activation.swf_version());
    this.set_enabled(&mut activation.context, enabled);
    Ok(())
}

fn use_hand_cursor<'gc>(
    this: Avm1Button<'gc>,
    _activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.use_hand_cursor().into())
}

fn set_use_hand_cursor<'gc>(
    this: Avm1Button<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    let use_hand_cursor = value.as_bool(activation.swf_version());
    this.set_use_hand_cursor(&mut activation.context, use_hand_cursor);
    Ok(())
}

fn blend_mode<'gc>(
    this: Avm1Button<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Value<'gc>, Error<'gc>> {
    let mode = AvmString::new_utf8(activation.context.gc_context, this.blend_mode().to_string());
    Ok(mode.into())
}

fn set_blend_mode<'gc>(
    this: Avm1Button<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    // No-op if value is not a string.
    if let Value::String(mode) = value {
        if let Ok(mode) = BlendMode::from_str(&mode.to_string()) {
            this.set_blend_mode(activation.context.gc_context, mode);
        } else {
            log::error!("Unknown blend mode {}", mode);
        };
    }
    Ok(())
}
