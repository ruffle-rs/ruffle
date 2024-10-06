use crate::avm2::activation::Activation;
use crate::avm2::error::make_error_2008;
use crate::avm2::object::{ArrayObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::ecma_conversions::round_to_even;
use crate::html::TextDisplay;
use crate::string::{AvmString, WStr};

pub use crate::avm2::object::textformat_allocator as text_format_allocator;

pub fn get_align<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(text_format) = this.as_text_format() {
        return Ok(text_format
            .align
            .as_ref()
            .map_or(Value::Null, |align| match align {
                swf::TextAlign::Left => "left".into(),
                swf::TextAlign::Center => "center".into(),
                swf::TextAlign::Right => "right".into(),
                swf::TextAlign::Justify => "justify".into(),
            }));
    }

    Ok(Value::Undefined)
}

pub fn set_align<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut text_format) = this.as_text_format_mut() {
        let value = args.get(0).unwrap_or(&Value::Undefined);
        let value = match value {
            Value::Undefined | Value::Null => {
                text_format.align = None;
                return Ok(Value::Undefined);
            }
            value => value.coerce_to_string(activation)?,
        };

        text_format.align = if value == WStr::from_units(b"left") {
            Some(swf::TextAlign::Left)
        } else if value == WStr::from_units(b"center") {
            Some(swf::TextAlign::Center)
        } else if value == WStr::from_units(b"right") {
            Some(swf::TextAlign::Right)
        } else if value == WStr::from_units(b"justify") {
            Some(swf::TextAlign::Justify)
        } else {
            return Err(make_error_2008(activation, "align"));
        };
    }

    Ok(Value::Undefined)
}

pub fn get_block_indent<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(text_format) = this.as_text_format() {
        return Ok(text_format
            .block_indent
            .as_ref()
            .map_or(Value::Null, |&block_indent| block_indent.into()));
    }

    Ok(Value::Undefined)
}

pub fn set_block_indent<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut text_format) = this.as_text_format_mut() {
        let value = args.get(0).unwrap_or(&Value::Undefined);
        text_format.block_indent = match value {
            Value::Undefined | Value::Null => None,
            value => Some(round_to_even(value.coerce_to_number(activation)?).into()),
        };
    }

    Ok(Value::Undefined)
}

pub fn get_bold<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(text_format) = this.as_text_format() {
        return Ok(text_format
            .bold
            .as_ref()
            .map_or(Value::Null, |&bold| bold.into()));
    }

    Ok(Value::Undefined)
}

pub fn set_bold<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut text_format) = this.as_text_format_mut() {
        let value = args.get(0).unwrap_or(&Value::Undefined);
        text_format.bold = match value {
            Value::Undefined | Value::Null => None,
            value => Some(value.coerce_to_boolean()),
        };
    }

    Ok(Value::Undefined)
}

pub fn get_bullet<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(text_format) = this.as_text_format() {
        return Ok(text_format
            .bullet
            .as_ref()
            .map_or(Value::Null, |&bullet| bullet.into()));
    }

    Ok(Value::Undefined)
}

pub fn set_bullet<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut text_format) = this.as_text_format_mut() {
        let value = args.get(0).unwrap_or(&Value::Undefined);
        text_format.bullet = match value {
            Value::Undefined | Value::Null => None,
            value => Some(value.coerce_to_boolean()),
        };
    }

    Ok(Value::Undefined)
}

pub fn get_color<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(text_format) = this.as_text_format() {
        return Ok(text_format
            .color
            .as_ref()
            .map_or(Value::Null, |color| (color.to_rgba() as i32).into()));
    }

    Ok(Value::Undefined)
}

pub fn set_color<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut text_format) = this.as_text_format_mut() {
        let value = args.get(0).unwrap_or(&Value::Undefined);
        text_format.color = match value {
            Value::Undefined | Value::Null => None,
            value => Some(swf::Color::from_rgba(value.coerce_to_u32(activation)?)),
        };
    }

    Ok(Value::Undefined)
}

pub fn get_display<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(text_format) = this.as_text_format() {
        return Ok(text_format
            .display
            .as_ref()
            .map_or(Value::Null, |display| match display {
                TextDisplay::Block => "block".into(),
                TextDisplay::Inline => "inline".into(),
                TextDisplay::None => "none".into(),
            }));
    }

    Ok(Value::Undefined)
}

pub fn set_display<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut text_format) = this.as_text_format_mut() {
        let value = args.get(0).unwrap_or(&Value::Undefined);
        let value = match value {
            Value::Undefined | Value::Null => {
                text_format.display = None;
                return Ok(Value::Undefined);
            }
            value => value.coerce_to_string(activation)?,
        };

        text_format.display = if &value == b"block" {
            Some(TextDisplay::Block)
        } else if &value == b"inline" {
            Some(TextDisplay::Inline)
        } else if &value == b"none" {
            Some(TextDisplay::None)
        } else {
            // No error message for this, silently set it to None/null
            None
        };
    }

    Ok(Value::Undefined)
}

pub fn get_font<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(text_format) = this.as_text_format() {
        return Ok(text_format.font.as_ref().map_or(Value::Null, |font| {
            AvmString::new(activation.context.gc_context, font.as_wstr()).into()
        }));
    }

    Ok(Value::Undefined)
}

pub fn set_font<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut text_format) = this.as_text_format_mut() {
        let value = args.get(0).unwrap_or(&Value::Undefined);
        text_format.font = match value {
            Value::Undefined | Value::Null => None,
            value => Some(value.coerce_to_string(activation)?.as_wstr().into()),
        };
    }

    Ok(Value::Undefined)
}

pub fn get_indent<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(text_format) = this.as_text_format() {
        return Ok(text_format
            .indent
            .as_ref()
            .map_or(Value::Null, |&indent| indent.into()));
    }

    Ok(Value::Undefined)
}

pub fn set_indent<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut text_format) = this.as_text_format_mut() {
        let value = args.get(0).unwrap_or(&Value::Undefined);
        text_format.indent = match value {
            Value::Undefined | Value::Null => None,
            value => Some(round_to_even(value.coerce_to_number(activation)?).into()),
        };
    }

    Ok(Value::Undefined)
}

pub fn get_italic<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(text_format) = this.as_text_format() {
        return Ok(text_format
            .italic
            .as_ref()
            .map_or(Value::Null, |&italic| italic.into()));
    }

    Ok(Value::Undefined)
}

pub fn set_italic<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut text_format) = this.as_text_format_mut() {
        let value = args.get(0).unwrap_or(&Value::Undefined);
        text_format.italic = match value {
            Value::Undefined | Value::Null => None,
            value => Some(value.coerce_to_boolean()),
        };
    }

    Ok(Value::Undefined)
}

pub fn get_kerning<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(text_format) = this.as_text_format() {
        return Ok(text_format
            .kerning
            .as_ref()
            .map_or(Value::Null, |&kerning| kerning.into()));
    }

    Ok(Value::Undefined)
}

pub fn set_kerning<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut text_format) = this.as_text_format_mut() {
        let value = args.get(0).unwrap_or(&Value::Undefined);
        text_format.kerning = match value {
            Value::Undefined | Value::Null => None,
            value => Some(value.coerce_to_boolean()),
        };
    }

    Ok(Value::Undefined)
}

pub fn get_leading<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(text_format) = this.as_text_format() {
        return Ok(text_format
            .leading
            .as_ref()
            .map_or(Value::Null, |&leading| leading.into()));
    }

    Ok(Value::Undefined)
}

pub fn set_leading<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut text_format) = this.as_text_format_mut() {
        let value = args.get(0).unwrap_or(&Value::Undefined);
        text_format.leading = match value {
            Value::Undefined | Value::Null => None,
            value => Some(round_to_even(value.coerce_to_number(activation)?).into()),
        };
    }

    Ok(Value::Undefined)
}

pub fn get_left_margin<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(text_format) = this.as_text_format() {
        return Ok(text_format
            .left_margin
            .as_ref()
            .map_or(Value::Null, |&left_margin| left_margin.into()));
    }

    Ok(Value::Undefined)
}

pub fn set_left_margin<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut text_format) = this.as_text_format_mut() {
        let value = args.get(0).unwrap_or(&Value::Undefined);
        text_format.left_margin = match value {
            Value::Undefined | Value::Null => None,
            value => Some(round_to_even(value.coerce_to_number(activation)?).into()),
        };
    }

    Ok(Value::Undefined)
}

pub fn get_letter_spacing<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(text_format) = this.as_text_format() {
        return Ok(text_format
            .letter_spacing
            .as_ref()
            .map_or(Value::Null, |&letter_spacing| letter_spacing.into()));
    }

    Ok(Value::Undefined)
}

pub fn set_letter_spacing<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut text_format) = this.as_text_format_mut() {
        let value = args.get(0).unwrap_or(&Value::Undefined);
        text_format.letter_spacing = match value {
            Value::Undefined | Value::Null => None,
            value => Some(value.coerce_to_number(activation)?),
        };
    }

    Ok(Value::Undefined)
}

pub fn get_right_margin<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(text_format) = this.as_text_format() {
        return Ok(text_format
            .right_margin
            .as_ref()
            .map_or(Value::Null, |&right_margin| right_margin.into()));
    }

    Ok(Value::Undefined)
}

pub fn set_right_margin<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut text_format) = this.as_text_format_mut() {
        let value = args.get(0).unwrap_or(&Value::Undefined);
        text_format.right_margin = match value {
            Value::Undefined | Value::Null => None,
            value => Some(round_to_even(value.coerce_to_number(activation)?).into()),
        };
    }

    Ok(Value::Undefined)
}

pub fn get_size<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(text_format) = this.as_text_format() {
        return Ok(text_format
            .size
            .as_ref()
            .map_or(Value::Null, |&size| size.into()));
    }

    Ok(Value::Undefined)
}

pub fn set_size<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut text_format) = this.as_text_format_mut() {
        let value = args.get(0).unwrap_or(&Value::Undefined);
        text_format.size = match value {
            Value::Undefined | Value::Null => None,
            value => Some(round_to_even(value.coerce_to_number(activation)?).into()),
        };
    }

    Ok(Value::Undefined)
}

pub fn get_tab_stops<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(text_format) = this.as_text_format() {
        return text_format
            .tab_stops
            .as_ref()
            .map_or(Ok(Value::Null), |tab_stops| {
                let tab_stop_storage = tab_stops.iter().copied().collect();
                Ok(ArrayObject::from_storage(activation, tab_stop_storage)?.into())
            });
    }

    Ok(Value::Undefined)
}

pub fn set_tab_stops<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut text_format) = this.as_text_format_mut() {
        let value = args.get(0).unwrap_or(&Value::Undefined);
        text_format.tab_stops = match value {
            Value::Null => None,
            Value::Object(obj) => {
                let length = obj.as_array_storage().map_or(0, |v| v.length());

                let tab_stops: Result<Vec<_>, Error<'gc>> = (0..length)
                    .map(|i| {
                        let element = obj.get_public_property(
                            AvmString::new_utf8(activation.context.gc_context, i.to_string()),
                            activation,
                        )?;
                        Ok(round_to_even(element.coerce_to_number(activation)?).into())
                    })
                    .collect();
                Some(tab_stops?)
            }
            _ => unreachable!("Array-typed argument can only be Object or Null"),
        };
    }

    Ok(Value::Undefined)
}

pub fn get_target<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(text_format) = this.as_text_format() {
        return Ok(text_format.target.as_ref().map_or(Value::Null, |target| {
            AvmString::new(activation.context.gc_context, target.as_wstr()).into()
        }));
    }

    Ok(Value::Undefined)
}

pub fn set_target<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut text_format) = this.as_text_format_mut() {
        let value = args.get(0).unwrap_or(&Value::Undefined);
        text_format.target = match value {
            Value::Undefined | Value::Null => None,
            value => Some(value.coerce_to_string(activation)?.as_wstr().into()),
        };
    }

    Ok(Value::Undefined)
}

pub fn get_underline<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(text_format) = this.as_text_format() {
        return Ok(text_format
            .underline
            .as_ref()
            .map_or(Value::Null, |&underline| underline.into()));
    }

    Ok(Value::Undefined)
}

pub fn set_underline<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut text_format) = this.as_text_format_mut() {
        let value = args.get(0).unwrap_or(&Value::Undefined);
        text_format.underline = match value {
            Value::Undefined | Value::Null => None,
            value => Some(value.coerce_to_boolean()),
        };
    }

    Ok(Value::Undefined)
}

pub fn get_url<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(text_format) = this.as_text_format() {
        return Ok(text_format.url.as_ref().map_or(Value::Null, |url| {
            AvmString::new(activation.context.gc_context, url.as_wstr()).into()
        }));
    }

    Ok(Value::Undefined)
}

pub fn set_url<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut text_format) = this.as_text_format_mut() {
        let value = args.get(0).unwrap_or(&Value::Undefined);
        text_format.url = match value {
            Value::Undefined | Value::Null => None,
            value => Some(value.coerce_to_string(activation)?.as_wstr().into()),
        };
    }

    Ok(Value::Undefined)
}
