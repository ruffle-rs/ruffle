use crate::avm2::parameters::ParametersExt;
use crate::avm2::{Activation, Error, Object, TObject, Value};
use crate::html::CssStream;
use crate::string::AvmString;

pub fn inner_parse_css<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let document = args.get_string(activation, 0)?;
    let result = activation
        .avm2()
        .classes()
        .object
        .construct(activation, &[])?;

    if let Ok(css) = CssStream::new(&document).parse() {
        for (selector, properties) in css.into_iter() {
            let object = activation
                .avm2()
                .classes()
                .object
                .construct(activation, &[])?;
            for (key, value) in properties.into_iter() {
                object.set_public_property(
                    AvmString::new(activation.gc(), key),
                    Value::String(AvmString::new(activation.gc(), value)),
                    activation,
                )?;
            }
            result.set_public_property(
                AvmString::new(activation.gc(), selector),
                Value::Object(object),
                activation,
            )?;
        }
    }

    Ok(Value::Object(result))
}
