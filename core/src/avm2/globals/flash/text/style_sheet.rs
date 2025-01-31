use crate::avm2::object::{ScriptObject, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::{Activation, Error, Value};
use crate::html::{transform_dashes_to_camel_case, CssStream};
use crate::string::AvmString;
use ruffle_wstr::WStr;

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
    let parsed_font_list = crate::html::parse_font_list(input.as_wstr());
    Ok(Value::String(AvmString::new(
        activation.gc(),
        parsed_font_list,
    )))
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
