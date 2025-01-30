use crate::avm2::activation::Activation;
use crate::avm2::error::Error;
use crate::avm2::value::Value;
use crate::string::WStr;
use crate::stub::Stub;
use std::borrow::Cow;

pub fn stub_method<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    match args {
        [class, method] => {
            let class = class.coerce_to_string(activation)?;
            let method = method.coerce_to_string(activation)?;
            activation
                .context
                .stub_tracker
                .encounter(&Stub::Avm2Method {
                    class: Cow::Owned(class.to_utf8_lossy().to_string()),
                    method: Cow::Owned(method.to_utf8_lossy().to_string()),
                    specifics: None,
                });
        }
        [class, method, specifics] => {
            let class = class.coerce_to_string(activation)?;
            let method = method.coerce_to_string(activation)?;
            let specifics = specifics.coerce_to_string(activation)?;
            activation
                .context
                .stub_tracker
                .encounter(&Stub::Avm2Method {
                    class: Cow::Owned(class.to_utf8_lossy().to_string()),
                    method: Cow::Owned(method.to_utf8_lossy().to_string()),
                    specifics: Some(Cow::Owned(specifics.to_utf8_lossy().to_string())),
                });
        }
        _ => tracing::warn!("(__ruffle__.stub_method called with wrong args)"),
    }

    Ok(Value::Undefined)
}

pub fn stub_getter<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    match args {
        [class, property] => {
            let class = class.coerce_to_string(activation)?;
            let property = property.coerce_to_string(activation)?;
            activation
                .context
                .stub_tracker
                .encounter(&Stub::Avm2Getter {
                    class: Cow::Owned(class.to_utf8_lossy().to_string()),
                    property: Cow::Owned(property.to_utf8_lossy().to_string()),
                });
        }
        _ => tracing::warn!("(__ruffle__.stub_getter called with wrong args)"),
    }

    Ok(Value::Undefined)
}

pub fn stub_setter<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    match args {
        [class, property] => {
            let class = class.coerce_to_string(activation)?;
            let property = property.coerce_to_string(activation)?;
            activation
                .context
                .stub_tracker
                .encounter(&Stub::Avm2Setter {
                    class: Cow::Owned(class.to_utf8_lossy().to_string()),
                    property: Cow::Owned(property.to_utf8_lossy().to_string()),
                });
        }
        _ => tracing::warn!("(__ruffle__.stub_setter called with wrong args)"),
    }

    Ok(Value::Undefined)
}

pub fn stub_constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    match args {
        [class] => {
            let class = class.coerce_to_string(activation)?;
            activation
                .context
                .stub_tracker
                .encounter(&Stub::Avm2Constructor {
                    class: Cow::Owned(class.to_utf8_lossy().to_string()),
                    specifics: None,
                });
        }
        [class, specifics] => {
            let class = class.coerce_to_string(activation)?;
            let specifics = specifics.coerce_to_string(activation)?;
            activation
                .context
                .stub_tracker
                .encounter(&Stub::Avm2Constructor {
                    class: Cow::Owned(class.to_utf8_lossy().to_string()),
                    specifics: Some(Cow::Owned(specifics.to_utf8_lossy().to_string())),
                });
        }
        _ => tracing::warn!("(__ruffle__.stub_constructor called with wrong args)"),
    }

    Ok(Value::Undefined)
}

pub fn log_warn<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    match args {
        [] => tracing::warn!("(__ruffle__.log_warn called with no arg)"),
        [arg] => {
            let msg = arg.coerce_to_string(activation)?;
            tracing::warn!("{}", &msg.to_utf8_lossy());
        }
        args => {
            let strings = args
                .iter()
                .map(|a| a.coerce_to_string(activation))
                .collect::<Result<Vec<_>, _>>()?;
            let msg = crate::string::join(&strings, &WStr::from_units(b" "));
            tracing::warn!("{}", &msg.to_utf8_lossy());
        }
    }

    Ok(Value::Undefined)
}

pub fn is_dependent<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Value::String(s) = args[0] {
        return Ok(s.is_dependent().into());
    }

    panic!();
}
