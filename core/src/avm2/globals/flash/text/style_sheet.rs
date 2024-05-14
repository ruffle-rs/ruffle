use crate::avm2::parameters::ParametersExt;
use crate::avm2::{Activation, Error, Object, TObject, Value};
use crate::html::{transform_dashes_to_camel_case, CssStream};
use crate::string::AvmString;
use ruffle_wstr::{WStr, WString};

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
                    AvmString::new(activation.gc(), transform_dashes_to_camel_case(key)),
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

pub fn inner_parse_color<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let input = args.get_string(activation, 0)?;

    if let Some(stripped) = input.strip_prefix(WStr::from_units(b"#")) {
        if stripped.len() <= 6 {
            if let Ok(number) = u32::from_str_radix(&stripped.to_string(), 16) {
                return Ok(number.into());
            }
        }
    }

    Ok(0.into())
}

pub fn inner_parse_font_family<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let input = args.get_string(activation, 0)?;
    let mut result = WString::new();

    let mut pos = 0;
    while pos < input.len() {
        // Skip whitespace
        while input.get(pos) == Some(' ' as u16) {
            pos += 1;
        }

        // Find the whole value
        let start = pos;
        while input.get(pos) != Some(',' as u16) && pos < input.len() {
            pos += 1;
        }

        let mut value = &input[start..pos];

        if pos < input.len() {
            pos += 1; // move past the comma
        }

        // Transform some names
        if value == b"mono" {
            value = WStr::from_units(b"_typewriter");
        } else if value == b"sans-serif" {
            value = WStr::from_units(b"_sans");
        } else if value == b"serif" {
            value = WStr::from_units(b"_serif");
        }

        // Add it to the result (without any extra space)
        if !value.is_empty() {
            if !result.is_empty() {
                result.push_char(',');
            }
            result.push_str(value);
        }
    }

    Ok(Value::String(AvmString::new(activation.gc(), result)))
}
