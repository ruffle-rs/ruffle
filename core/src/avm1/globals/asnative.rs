use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{FunctionObject, TableNativeFunction};
use crate::avm1::parameters::ParametersExt;
use crate::avm1::{Attribute, Object, Value};
use ruffle_common::avm_string::AvmString;

pub fn asnative<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    use crate::avm1::globals;

    if args.len() != 2 {
        return Ok(Value::Undefined);
    }
    let category = args.get_u32(activation, 0)?;
    let index = args.get_u32(activation, 1)?;

    let category: Option<TableNativeFunction> = match category {
        2 => Some(asnew_method),
        4 => Some(as_set_native_method),
        100 => Some(globals::method),
        101 => Some(globals::object::method),
        103 => Some(globals::date::method),
        200 => Some(globals::math::method),
        1101 => Some(globals::drop_shadow_filter::method),
        1102 => Some(globals::blur_filter::method),
        1103 => Some(globals::glow_filter::method),
        1106 => Some(globals::transform::method),
        1107 => Some(globals::bevel_filter::method),
        1108 => Some(globals::gradient_filter::method),
        1109 => Some(globals::convolution_filter::method),
        1110 => Some(globals::color_matrix_filter::method),
        1111 => Some(globals::displacement_map_filter::method),
        1999 => Some(globals::accessibility::method),
        2102 => Some(globals::camera::method),
        2700 => Some(globals::automation_stage_capture::method),
        2800 => Some(globals::automation_action_generator::method),
        2900 => Some(globals::automation_configuration::method),
        _ => None,
    };

    if let Some(category) = category {
        // No native function accepts u16::MAX as index, so this is OK.
        let index = u16::try_from(index).unwrap_or(u16::MAX);
        let function = FunctionObject::table_native(category, index);
        return Ok(function
            .build(
                &activation.context.strings,
                activation.prototypes().function,
                None,
            )
            .into());
    }

    Ok(Value::Undefined)
}

/// Undocumented internal `ASnew` function: returns a boolean indicating whether or not the
/// enclosing function has been called as a constructor. Note that this 'sees through' any
/// native calls; see the `avm1/asnew` test for examples.
///
/// TODO: It should be exported as `_global.ASnew` when `player_version == 5`, but we don't
/// have a good way of having player-specific definitions so this isn't implemented.
fn asnew_method<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
    id: u16,
) -> Result<Value<'gc>, Error<'gc>> {
    match id {
        0 => Ok(activation.in_bytecode_constructor().into()),
        _ => Ok(Value::Undefined),
    }
}

pub mod as_set_native_method {
    pub const AS_SET_NATIVE: u16 = 0;
    pub const AS_SET_NATIVE_ACCESSOR: u16 = 1;
}

/// Implements `ASSetNative` and `ASSetNativeAccessor`.
pub fn as_set_native_method<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
    id: u16,
) -> Result<Value<'gc>, Error<'gc>> {
    if !(0u16..=1).contains(&id) {
        return Ok(Value::Undefined);
    }

    let [object, major, props, ..] = args else {
        return Ok(Value::Undefined);
    };

    let object = object.coerce_to_object_or_bare(activation)?;
    let major = major.coerce_to_i32(activation)?;
    let props = props.coerce_to_string(activation)?;
    let mut minor = if let Some(minor) = args.get(3) {
        minor.coerce_to_i32(activation)?
    } else {
        0
    };

    for mut prop in props.split(b',') {
        let attributes = match prop.get(0).and_then(|i| u8::try_from(i).ok()) {
            Some(b'6') => {
                prop = &prop[1..];
                Attribute::VERSION_6
            }
            Some(b'7') => {
                prop = &prop[1..];
                Attribute::VERSION_7
            }
            Some(b'8') => {
                prop = &prop[1..];
                Attribute::VERSION_8
            }
            Some(b'9') => {
                prop = &prop[1..];
                Attribute::VERSION_9
            }
            Some(b'1') => {
                prop = &prop[1..];
                if let Some(p) = prop.strip_prefix(b'0') {
                    prop = p;
                    Attribute::VERSION_10
                } else {
                    Attribute::empty()
                }
            }
            _ => Attribute::empty(),
        };

        let prop = AvmString::new(activation.gc(), prop);

        match id {
            as_set_native_method::AS_SET_NATIVE => {
                let f = asnative(activation, this, &[major.into(), minor.into()])?;
                minor += 1;

                if !prop.is_empty() {
                    let had_prop = object.has_own_property(activation, prop);
                    object.set(prop, f, activation)?;
                    if !had_prop {
                        object.set_attributes(
                            activation.gc(),
                            Some(prop),
                            attributes,
                            Attribute::empty(),
                        );
                    }
                }
            }
            as_set_native_method::AS_SET_NATIVE_ACCESSOR => {
                let getter = asnative(activation, this, &[major.into(), minor.into()])?;
                minor += 1;
                let setter = asnative(activation, this, &[major.into(), minor.into()])?;
                minor += 1;

                if !prop.is_empty()
                    && let Some(getter) = getter.as_object(activation)
                {
                    object.add_property(
                        activation.gc(),
                        prop,
                        getter,
                        setter.as_object(activation),
                        Attribute::empty(),
                    );
                    object.set_attributes(
                        activation.gc(),
                        Some(prop),
                        attributes,
                        Attribute::empty(),
                    );
                }
            }
            _ => {}
        }
    }

    Ok(Value::Undefined)
}
