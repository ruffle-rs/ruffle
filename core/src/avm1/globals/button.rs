//! Button/SimpleButton prototype

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::globals::display_object;
use crate::avm1::property::Attribute;
use crate::avm1::{Object, ScriptObject, TObject, Value};
use crate::display_object::{Button, TDisplayObject};
use gc_arena::MutationContext;

macro_rules! with_button_props {
    ($obj:ident, $gc:ident, $fn_proto:ident, $($name:literal => [$get:ident $(, $set:ident)*],)*) => {
        $(
            $obj.add_property(
                $gc,
                $name,
                with_button_props!(getter $gc, $fn_proto, $get),
                with_button_props!(setter $gc, $fn_proto, $($set),*),
                Attribute::DONT_DELETE | Attribute::DONT_ENUM,
            );
        )*
    };

    (getter $gc:ident, $fn_proto:ident, $get:ident) => {
        FunctionObject::function(
            $gc,
            Executable::Native(
                |activation: &mut Activation<'_, 'gc, '_>, this, _args| -> Result<Value<'gc>, Error<'gc>> {
                    if let Some(display_object) = this.as_display_object() {
                        if let Some(button) = display_object.as_button() {
                            return $get(button, activation);
                        }
                    }
                    Ok(Value::Undefined)
                } as crate::avm1::function::NativeFunction<'gc>
            ),
            Some($fn_proto),
            $fn_proto
        )
    };

    (setter $gc:ident, $fn_proto:ident, $set:ident) => {
        Some(FunctionObject::function(
            $gc,
            Executable::Native(
                |activation: &mut Activation<'_, 'gc, '_>, this, args| -> Result<Value<'gc>, Error<'gc>> {
                    if let Some(display_object) = this.as_display_object() {
                        if let Some(button) = display_object.as_button() {
                            let value = args
                                .get(0)
                                .unwrap_or(&Value::Undefined)
                                .clone();
                            $set(button, activation, value)?;
                        }
                    }
                    Ok(Value::Undefined)
                } as crate::avm1::function::NativeFunction<'gc>
            ),
            Some($fn_proto),
            $fn_proto)
        )
    };

    (setter $gc:ident, $fn_proto:ident,) => {
        None
    };
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = ScriptObject::object(gc_context, Some(proto));

    display_object::define_display_object_proto(gc_context, object, fn_proto);

    with_button_props!(
        object, gc_context, fn_proto,
        "enabled" => [enabled, set_enabled],
        "useHandCursor" => [use_hand_cursor, set_use_hand_cursor],
    );

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
    this: Button<'gc>,
    _activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.enabled().into())
}

fn set_enabled<'gc>(
    this: Button<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    let enabled = value.as_bool(activation.swf_version());
    this.set_enabled(&mut activation.context, enabled);
    Ok(())
}

fn use_hand_cursor<'gc>(
    this: Button<'gc>,
    _activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.use_hand_cursor().into())
}

fn set_use_hand_cursor<'gc>(
    this: Button<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    let use_hand_cursor = value.as_bool(activation.swf_version());
    this.set_use_hand_cursor(&mut activation.context, use_hand_cursor);
    Ok(())
}
