//! Button prototype

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{globals, Object, ScriptObject, TObject, Value};
use crate::context::GcContext;
use crate::display_object::{Avm1Button, TDisplayObject};
use crate::string::AvmString;

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
    "enabled" => bool(true);
    "useHandCursor" => bool(true);
    "getDepth" => method(globals::get_depth; DONT_DELETE | READ_ONLY | VERSION_6);
    "blendMode" => property(button_getter!(blend_mode), button_setter!(set_blend_mode); DONT_DELETE | VERSION_8);
};

pub fn create_proto<'gc>(
    context: &mut GcContext<'_, 'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, context, object, fn_proto);
    object.into()
}

/// Implements `Button` constructor.
pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.into())
}

fn blend_mode<'gc>(
    this: Avm1Button<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let mode = AvmString::new_utf8(activation.context.gc_context, this.blend_mode().to_string());
    Ok(mode.into())
}

fn set_blend_mode<'gc>(
    this: Avm1Button<'gc>,
    activation: &mut Activation<'_, 'gc>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    // No-op if value is not a valid blend mode.
    if let Some(mode) = value.as_blend_mode() {
        this.set_blend_mode(activation.context.gc_context, mode);
    } else {
        tracing::error!("Unknown blend mode {value:?}");
    }
    Ok(())
}
