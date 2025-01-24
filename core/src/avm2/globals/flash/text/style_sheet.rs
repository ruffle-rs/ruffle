use crate::avm2::object::{ScriptObject, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::{Activation, Error, Value};
use crate::html::{transform_dashes_to_camel_case, CssStream};
use crate::string::AvmString;
use ruffle_wstr::{WStr, WString};

pub use crate::avm2::object::style_sheet_allocator;

pub fn inner_parse_css<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let document = args.get_string(activation, 0)?;
    let result = ScriptObject::new_object(activation);

    if let Ok(css) = CssStream::new(&document).parse() {
        for (selector, properties) in css.into_iter() {
            let object = ScriptObject::new_object(activation);

            for (key, value) in properties.into_iter() {
                object.set_string_property_local(
                    AvmString::new(activation.gc(), transform_dashes_to_camel_case(key)),
                    Value::String(AvmString::new(activation.gc(), value)),
                    activation,
                )?;
            }

            result.set_string_property_local(
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
    _this: Value<'gc>,
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
    _this: Value<'gc>,
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

pub fn clear_internal<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let Some(this) = this.as_style_sheet() else {
        return Ok(Value::Undefined);
    };

    this.clear();

    Ok(Value::Undefined)
}

pub fn set_style_internal<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let Some(this) = this.as_style_sheet() else {
        return Ok(Value::Undefined);
    };

    let Some(selector) = args.try_get_string(activation, 0)? else {
        return Ok(Value::Undefined);
    };
    let text_format = args
        .try_get_object(activation, 1)
        .and_then(|tf| tf.as_text_format().map(|tf| tf.clone()));

    if let Some(text_format) = text_format {
        this.set_style(selector.as_wstr().to_owned(), text_format);
    } else {
        this.remove_style(selector.as_wstr());
    }

    Ok(Value::Undefined)
}
