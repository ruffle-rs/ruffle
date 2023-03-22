use crate::avm2::parameters::ParametersExt;
use crate::avm2::{Activation, Error, Object, Value};
use crate::external::{Callback, Value as ExternalValue};

pub fn call<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let name = args.get_string(activation, 0)?;
    if let Some(method) = activation
        .context
        .external_interface
        .get_method_for(&name.to_utf8_lossy())
    {
        let mut external_args = Vec::with_capacity(args.len() - 1);
        for arg in &args[1..] {
            external_args.push(ExternalValue::from_avm2(arg.to_owned()));
        }
        Ok(method
            .call(&mut activation.context, &external_args)
            .into_avm2(activation))
    } else {
        Ok(ExternalValue::check_avm2_value(Value::Undefined))
    }
}

pub fn get_available<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.external_interface.available().into())
}

pub fn add_callback<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let name = args.get_string(activation, 0)?;
    let method = args.get_object(activation, 1, "closure")?;

    activation
        .context
        .external_interface
        .add_callback(name.to_string(), Callback::Avm2 { method });
    Ok(Value::Undefined)
}
