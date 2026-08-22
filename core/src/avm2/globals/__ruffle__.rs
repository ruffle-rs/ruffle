use crate::avm2::activation::Activation;
use crate::avm2::error::Error;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::string::WStr;
use crate::stub::Stub;
use std::borrow::Cow;

pub fn stub_method<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let class = args.get_string(activation, 0);
    let method = args.get_string(activation, 1);
    let specifics = args.try_get_string(2);

    let class = Cow::Owned(class.to_utf8_lossy().to_string());
    let method = Cow::Owned(method.to_utf8_lossy().to_string());
    let specifics = specifics.map(|s| Cow::Owned(s.to_utf8_lossy().to_string()));

    activation
        .context
        .stub_tracker
        .encounter(&Stub::Avm2Method {
            class,
            method,
            specifics,
        });

    Ok(Value::Undefined)
}

pub fn stub_getter<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let class = args.get_string(activation, 0);
    let property = args.get_string(activation, 1);

    let class = Cow::Owned(class.to_utf8_lossy().to_string());
    let property = Cow::Owned(property.to_utf8_lossy().to_string());

    activation
        .context
        .stub_tracker
        .encounter(&Stub::Avm2Getter { class, property });

    Ok(Value::Undefined)
}

pub fn stub_setter<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let class = args.get_string(activation, 0);
    let property = args.get_string(activation, 1);

    let class = Cow::Owned(class.to_utf8_lossy().to_string());
    let property = Cow::Owned(property.to_utf8_lossy().to_string());

    activation
        .context
        .stub_tracker
        .encounter(&Stub::Avm2Setter { class, property });

    Ok(Value::Undefined)
}

pub fn stub_constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let class = args.get_string(activation, 0);
    let specifics = args.try_get_string(1);

    let class = Cow::Owned(class.to_utf8_lossy().to_string());
    let specifics = specifics.map(|s| Cow::Owned(s.to_utf8_lossy().to_string()));

    activation
        .context
        .stub_tracker
        .encounter(&Stub::Avm2Constructor { class, specifics });

    Ok(Value::Undefined)
}

pub fn log_warn<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let strings = args
        .iter()
        .map(|a| a.coerce_to_string(activation))
        .collect::<Result<Vec<_>, _>>()?;
    let msg = crate::string::join(&strings, &WStr::from_units(b" "));
    let msg = msg.to_utf8_lossy();
    tracing::warn!("{}", &msg);

    Ok(Value::Undefined)
}

pub fn is_dependent<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(s) = args.try_get_string(0) {
        return Ok(s.is_dependent().into());
    }

    panic!();
}
