use crate::avm2::error::{make_error_2060, make_error_2067};
use crate::avm2::function::FunctionArgs;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::{Activation, Error, Value};
use crate::config::NetworkingAccessMode;
use crate::external::{Callback, ExternalInterface, Value as ExternalValue};
use crate::string::AvmString;

pub fn call<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let name = args.get_string(activation, 0);
    check_available(activation)?;

    let external_args = args
        .iter()
        .skip(1)
        .map(|arg| ExternalValue::from_avm2(activation, arg.to_owned()))
        .collect::<Result<Vec<ExternalValue>, Error>>()?;

    let result =
        ExternalInterface::call_method(activation.context, &name.to_utf8_lossy(), &external_args);

    Ok(result.into_avm2(activation.context))
}

fn check_available<'gc>(activation: &mut Activation<'_, 'gc>) -> Result<(), Error<'gc>> {
    Err(match activation.context.allow_networking {
        NetworkingAccessMode::All => {
            if activation.context.external_interface.available() {
                return Ok(());
            }

            make_error_2067(activation)
        }
        NetworkingAccessMode::Internal => make_error_2060(activation),
        NetworkingAccessMode::None => make_error_2067(activation),
    })
}

pub fn get_available<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(match activation.context.allow_networking {
        // For some strange reason internal can still return true.
        NetworkingAccessMode::All | NetworkingAccessMode::Internal => {
            activation.context.external_interface.available()
        }
        NetworkingAccessMode::None => false,
    }
    .into())
}

pub fn add_callback<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let name = args.get_string(activation, 0);
    let callback = args.try_get_function(1);

    check_available(activation)?;

    if let Some(method) = callback {
        activation
            .context
            .external_interface
            .add_callback(name.to_string(), Callback::Avm2 { method });
    } else {
        activation
            .context
            .external_interface
            .remove_callback(name.to_string());
    }

    Ok(Value::Undefined)
}

pub fn get_object_id<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(id) = activation.context.external_interface.get_id() {
        Ok(AvmString::new_utf8(activation.gc(), id).into())
    } else {
        Ok(Value::Null)
    }
}
