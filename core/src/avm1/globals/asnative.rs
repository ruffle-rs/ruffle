use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{FunctionObject, TableNativeFunction};
use crate::avm1::parameters::ParametersExt;
use crate::avm1::{Object, Value};

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
        100 => Some(globals::method),
        101 => Some(globals::object::method),
        103 => Some(globals::date::method),
        1101 => Some(globals::drop_shadow_filter::method),
        1102 => Some(globals::blur_filter::method),
        1103 => Some(globals::glow_filter::method),
        1106 => Some(globals::transform::method),
        1107 => Some(globals::bevel_filter::method),
        1108 => Some(globals::gradient_filter::method),
        1109 => Some(globals::convolution_filter::method),
        1110 => Some(globals::color_matrix_filter::method),
        1111 => Some(globals::displacement_map_filter::method),
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
