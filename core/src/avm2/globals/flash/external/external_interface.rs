use crate::avm2::{Activation, Error, Object, Value};
use crate::external::{Callback, Value as ExternalValue};

pub fn call<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.is_empty() {
        return Ok(Value::Null);
    }

    let name = args.get(0).unwrap().coerce_to_string(activation)?;
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
        Ok(Value::Null)
    }
}

pub fn get_available<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.external_interface.available().into())
}

pub fn add_callback<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() < 2 {
        return Ok(Value::Undefined);
    }

    let name = args.get(0).unwrap().coerce_to_string(activation)?;
    let method = args.get(1).unwrap();

    if let Value::Object(method) = method {
        activation
            .context
            .external_interface
            .add_callback(name.to_string(), Callback::Avm2 { method: *method });
    }
    Ok(Value::Undefined)
}
