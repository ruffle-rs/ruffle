//! `flash.text.TextField` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::{Object, TObject, TextFormatObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::avm2::QName;
use crate::display_object::{AutoSizeMode, EditText, TDisplayObject, TextSelection};
use crate::html::TextFormat;
use crate::string::AvmString;
use crate::tag_utils::SwfMovie;
use gc_arena::{GcCell, MutationContext};
use std::sync::Arc;
use swf::Color;

/// Implements `flash.text.TextField`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;

        if this.as_display_object().is_none() {
            let movie = Arc::new(SwfMovie::empty(activation.context.swf.version()));
            let new_do = EditText::new(&mut activation.context, movie, 0.0, 0.0, 100.0, 100.0);

            this.init_display_object(activation.context.gc_context, new_do.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `flash.text.TextField`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

pub fn autosize<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(match this.autosize() {
            AutoSizeMode::None => "none".into(),
            AutoSizeMode::Left => "left".into(),
            AutoSizeMode::Center => "center".into(),
            AutoSizeMode::Right => "right".into(),
        });
    }

    Ok(Value::Undefined)
}

pub fn set_autosize<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let value = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_string(activation)?;
        this.set_autosize(
            if &value == b"left" {
                AutoSizeMode::Left
            } else if &value == b"center" {
                AutoSizeMode::Center
            } else if &value == b"right" {
                AutoSizeMode::Right
            } else {
                AutoSizeMode::None
            },
            &mut activation.context,
        );
    }

    Ok(Value::Undefined)
}

pub fn background<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok((this.has_background()).into());
    }

    Ok(Value::Undefined)
}

pub fn set_background<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let has_background = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_boolean();
        this.set_has_background(activation.context.gc_context, has_background);
    }

    Ok(Value::Undefined)
}

pub fn background_color<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.background_color().to_rgb().into());
    }

    Ok(Value::Undefined)
}

pub fn set_background_color<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let rgb = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_u32(activation)?;
        let color = Color::from_rgb(rgb, 255);
        this.set_background_color(activation.context.gc_context, color);
    }

    Ok(Value::Undefined)
}

pub fn border<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.has_border().into());
    }

    Ok(Value::Undefined)
}

pub fn set_border<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let border = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_boolean();
        this.set_has_border(activation.context.gc_context, border);
    }

    Ok(Value::Undefined)
}

pub fn border_color<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.border_color().to_rgb().into());
    }

    Ok(Value::Undefined)
}

pub fn set_border_color<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let rgb = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_u32(activation)?;
        let color = Color::from_rgb(rgb, 255);
        this.set_border_color(activation.context.gc_context, color);
    }

    Ok(Value::Undefined)
}

pub fn default_text_format<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(TextFormatObject::from_text_format(activation, this.new_text_format())?.into());
    }

    Ok(Value::Undefined)
}

pub fn set_default_text_format<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let new_text_format = args.get(0).unwrap_or(&Value::Undefined).as_object();

        if let Some(new_text_format) = new_text_format {
            if let Some(new_text_format) = new_text_format.as_text_format() {
                this.set_new_text_format(new_text_format.clone(), &mut activation.context);
            }
        }
    }

    Ok(Value::Undefined)
}

pub fn display_as_password<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.is_password().into());
    }

    Ok(Value::Undefined)
}

pub fn set_display_as_password<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let is_password = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_boolean();

        this.set_password(is_password, &mut activation.context);
    }

    Ok(Value::Undefined)
}

pub fn embed_fonts<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok((!this.is_device_font()).into());
    }

    Ok(Value::Undefined)
}

pub fn set_embed_fonts<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let is_embed_fonts = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_boolean();

        this.set_is_device_font(&mut activation.context, !is_embed_fonts);
    }

    Ok(Value::Undefined)
}

pub fn html_text<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(AvmString::new(activation.context.gc_context, this.html_text()).into());
    }

    Ok(Value::Undefined)
}

pub fn set_html_text<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let html_text = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_string(activation)?;

        this.set_is_html(&mut activation.context, true);
        this.set_html_text(&html_text, &mut activation.context);
    }

    Ok(Value::Undefined)
}

pub fn length<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.text_length().into());
    }

    Ok(Value::Undefined)
}

pub fn multiline<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.is_multiline().into());
    }

    Ok(Value::Undefined)
}

pub fn set_multiline<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let is_multiline = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_boolean();

        this.set_multiline(is_multiline, &mut activation.context);
    }

    Ok(Value::Undefined)
}

pub fn selectable<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.is_selectable().into());
    }

    Ok(Value::Undefined)
}

pub fn set_selectable<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let is_selectable = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_boolean();

        this.set_selectable(is_selectable, &mut activation.context);
    }

    Ok(Value::Undefined)
}

pub fn text<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(AvmString::new(activation.context.gc_context, this.text()).into());
    }

    Ok(Value::Undefined)
}

pub fn set_text<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let text = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_string(activation)?;

        this.set_is_html(&mut activation.context, false);
        this.set_text(&text, &mut activation.context);
    }

    Ok(Value::Undefined)
}

pub fn text_color<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        if let Some(color) = this.new_text_format().color {
            return Ok(color.to_rgb().into());
        } else {
            return Ok(0u32.into());
        }
    }

    Ok(Value::Undefined)
}

pub fn set_text_color<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let text_color = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_u32(activation)?;
        let desired_format = TextFormat {
            color: Some(swf::Color::from_rgb(text_color, 0xFF)),
            ..TextFormat::default()
        };

        this.set_text_format(
            0,
            this.text_length(),
            desired_format.clone(),
            &mut activation.context,
        );
        this.set_new_text_format(desired_format, &mut activation.context);
    }

    Ok(Value::Undefined)
}

pub fn text_height<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let metrics = this.measure_text(&mut activation.context);
        return Ok(metrics.1.to_pixels().into());
    }

    Ok(Value::Undefined)
}

pub fn text_width<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let metrics = this.measure_text(&mut activation.context);
        return Ok(metrics.0.to_pixels().into());
    }

    Ok(Value::Undefined)
}

pub fn get_type<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        match this.is_editable() {
            true => return Ok("input".into()),
            false => return Ok("dynamic".into()),
        }
    }

    Ok(Value::Undefined)
}

pub fn set_type<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let is_editable = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_string(activation)?;

        if &is_editable == b"input" {
            this.set_editable(true, &mut activation.context);
        } else if &is_editable == b"dynamic" {
            this.set_editable(false, &mut activation.context);
        } else {
            return Err(format!("Invalid TextField.type: {is_editable}").into());
        }
    }

    Ok(Value::Undefined)
}

pub fn word_wrap<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.is_word_wrap().into());
    }

    Ok(Value::Undefined)
}

pub fn set_word_wrap<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let is_word_wrap = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_boolean();

        this.set_word_wrap(is_word_wrap, &mut activation.context);
    }

    Ok(Value::Undefined)
}

pub fn append_text<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let new_text = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_string(activation)?;
        let existing_length = this.text_length();

        this.replace_text(
            existing_length,
            existing_length,
            &new_text,
            &mut activation.context,
        );
    }

    Ok(Value::Undefined)
}

pub fn get_text_format<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let mut begin_index = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Integer(-1))
            .coerce_to_i32(activation)?;
        let mut end_index = args
            .get(1)
            .cloned()
            .unwrap_or(Value::Integer(-1))
            .coerce_to_i32(activation)?;

        if begin_index < 0 {
            begin_index = 0;
        }

        if end_index < 0 {
            end_index = this.text_length() as i32;
        }

        let tf = this.text_format(begin_index as usize, end_index as usize);
        return Ok(TextFormatObject::from_text_format(activation, tf)?.into());
    }

    Ok(Value::Undefined)
}

pub fn replace_selected_text<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let value = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_string(activation)?;
        let selection = this
            .selection()
            .unwrap_or_else(|| TextSelection::for_position(0));

        this.replace_text(
            selection.start(),
            selection.end(),
            &value,
            &mut activation.context,
        );
    }

    Ok(Value::Undefined)
}

pub fn replace_text<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let begin_index = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_u32(activation)?;
        let end_index = args
            .get(1)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_u32(activation)?;
        let value = args
            .get(2)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_string(activation)?;

        this.replace_text(
            begin_index as usize,
            end_index as usize,
            &value,
            &mut activation.context,
        );
    }

    Ok(Value::Undefined)
}

pub fn set_selection<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let begin_index = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_u32(activation)?;
        let end_index = args
            .get(1)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_u32(activation)?;

        this.set_selection(
            Some(TextSelection::for_range(
                begin_index as usize,
                end_index as usize,
            )),
            activation.context.gc_context,
        );
    }

    Ok(Value::Undefined)
}

pub fn set_text_format<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let tf = args.get(0).unwrap_or(&Value::Undefined).as_object();
        if let Some(tf) = tf {
            if let Some(tf) = tf.as_text_format() {
                let mut begin_index = args
                    .get(1)
                    .unwrap_or(&Value::Undefined)
                    .coerce_to_i32(activation)?;
                let mut end_index = args
                    .get(2)
                    .unwrap_or(&Value::Undefined)
                    .coerce_to_i32(activation)?;

                if begin_index < 0 {
                    begin_index = 0;
                }

                if begin_index as usize > this.text_length() {
                    return Err("RangeError: The supplied index is out of bounds.".into());
                }

                if end_index < 0 {
                    end_index = this.text_length() as i32;
                }

                if end_index as usize > this.text_length() {
                    return Err("RangeError: The supplied index is out of bounds.".into());
                }

                this.set_text_format(
                    begin_index as usize,
                    end_index as usize,
                    tf.clone(),
                    &mut activation.context,
                );
            }
        }
    }

    Ok(Value::Undefined)
}

pub fn anti_alias_type<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return if this.render_settings().is_advanced() {
            Ok("advanced".into())
        } else {
            Ok("normal".into())
        };
    }

    Ok(Value::Undefined)
}

pub fn set_anti_alias_type<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let old_settings = this.render_settings();
        let new_type = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_string(activation)?;

        if &new_type == b"advanced" {
            this.set_render_settings(
                activation.context.gc_context,
                old_settings.with_advanced_rendering(),
            );
        } else if &new_type == b"normal" {
            this.set_render_settings(
                activation.context.gc_context,
                old_settings.with_normal_rendering(),
            );
        }
    }
    Ok(Value::Undefined)
}

pub fn grid_fit_type<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return match this.render_settings().grid_fit() {
            swf::TextGridFit::None => Ok("none".into()),
            swf::TextGridFit::Pixel => Ok("pixel".into()),
            swf::TextGridFit::SubPixel => Ok("subpixel".into()),
        };
    }

    Ok(Value::Undefined)
}

pub fn set_grid_fit_type<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let old_settings = this.render_settings();
        let new_type = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_string(activation)?;

        if &new_type == b"pixel" {
            this.set_render_settings(
                activation.context.gc_context,
                old_settings.with_grid_fit(swf::TextGridFit::Pixel),
            );
        } else if &new_type == b"subpixel" {
            this.set_render_settings(
                activation.context.gc_context,
                old_settings.with_grid_fit(swf::TextGridFit::SubPixel),
            );
        } else {
            //NOTE: In AS3 invalid values are treated as None.
            this.set_render_settings(
                activation.context.gc_context,
                old_settings.with_grid_fit(swf::TextGridFit::None),
            );
        }
    }
    Ok(Value::Undefined)
}

pub fn thickness<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.render_settings().thickness().into());
    }

    Ok(0.into())
}

pub fn set_thickness<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let old_settings = this.render_settings();
        let mut new_thickness = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;

        // NOTE: The thickness clamp is ONLY enforced on AS3.
        new_thickness = new_thickness.clamp(-200.0, 200.0);

        this.set_render_settings(
            activation.context.gc_context,
            old_settings.with_thickness(new_thickness as f32),
        );
    }

    Ok(Value::Undefined)
}

pub fn sharpness<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.render_settings().sharpness().into());
    }

    Ok(0.into())
}

pub fn set_sharpness<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let old_settings = this.render_settings();
        let mut new_sharpness = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;

        // NOTE: The sharpness clamp is only enforced on AS3.
        new_sharpness = new_sharpness.clamp(-400.0, 400.0);

        this.set_render_settings(
            activation.context.gc_context,
            old_settings.with_sharpness(new_sharpness as f32),
        );
    }

    Ok(Value::Undefined)
}

pub fn num_lines<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.layout_lines().into());
    }

    Ok(Value::Undefined)
}

pub fn get_line_metrics<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let line_num = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_i32(activation)?;
        let metrics = this.layout_metrics(Some(line_num as usize));

        if let Some(metrics) = metrics {
            let metrics_class = activation.avm2().classes().textlinemetrics;
            return Ok(metrics_class
                .construct(
                    activation,
                    &[
                        metrics.x.to_pixels().into(),
                        metrics.width.to_pixels().into(),
                        metrics.height.to_pixels().into(),
                        metrics.ascent.to_pixels().into(),
                        metrics.descent.to_pixels().into(),
                        metrics.leading.to_pixels().into(),
                    ],
                )?
                .into());
        } else {
            return Err("RangeError".into());
        }
    }

    Ok(Value::Undefined)
}

pub fn bottom_scroll_v<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.bottom_scroll().into());
    }

    Ok(Value::Undefined)
}

pub fn max_scroll_v<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.maxscroll().into());
    }

    Ok(Value::Undefined)
}

pub fn max_scroll_h<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.maxhscroll().into());
    }

    Ok(Value::Undefined)
}

pub fn scroll_v<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.scroll().into());
    }

    Ok(Value::Undefined)
}

pub fn set_scroll_v<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let input = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_i32(activation)?;
        this.set_scroll(input as f64, &mut activation.context);
    }

    Ok(Value::Undefined)
}

pub fn scroll_h<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.hscroll().into());
    }

    Ok(Value::Undefined)
}

pub fn set_scroll_h<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        // NOTE: The clamping behavior here is identical to AVM1.
        // This is incorrect, SWFv9 uses more complex behavior and AS3 can only
        // be present in v9 SWFs.
        let input = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_i32(activation)?;
        let clamped = input.clamp(0, this.maxhscroll() as i32);
        this.set_hscroll(clamped as f64, &mut activation.context);
    }

    Ok(Value::Undefined)
}

/// Construct `TextField`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.text"), "TextField"),
        Some(Multiname::new(
            Namespace::package("flash.display"),
            "InteractiveObject",
        )),
        Method::from_builtin(instance_init, "<TextField instance initializer>", mc),
        Method::from_builtin(class_init, "<TextField class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("autoSize", Some(autosize), Some(set_autosize)),
        ("background", Some(background), Some(set_background)),
        (
            "backgroundColor",
            Some(background_color),
            Some(set_background_color),
        ),
        ("border", Some(border), Some(set_border)),
        ("borderColor", Some(border_color), Some(set_border_color)),
        ("bottomScrollV", Some(bottom_scroll_v), None),
        (
            "defaultTextFormat",
            Some(default_text_format),
            Some(set_default_text_format),
        ),
        (
            "displayAsPassword",
            Some(display_as_password),
            Some(set_display_as_password),
        ),
        ("embedFonts", Some(embed_fonts), Some(set_embed_fonts)),
        ("htmlText", Some(html_text), Some(set_html_text)),
        ("length", Some(length), None),
        ("maxScrollH", Some(max_scroll_h), None),
        ("maxScrollV", Some(max_scroll_v), None),
        ("multiline", Some(multiline), Some(set_multiline)),
        ("scrollH", Some(scroll_h), Some(set_scroll_h)),
        ("scrollV", Some(scroll_v), Some(set_scroll_v)),
        ("selectable", Some(selectable), Some(set_selectable)),
        ("text", Some(text), Some(set_text)),
        ("textColor", Some(text_color), Some(set_text_color)),
        ("textHeight", Some(text_height), None),
        ("textWidth", Some(text_width), None),
        ("type", Some(get_type), Some(set_type)),
        ("wordWrap", Some(word_wrap), Some(set_word_wrap)),
        (
            "antiAliasType",
            Some(anti_alias_type),
            Some(set_anti_alias_type),
        ),
        ("gridFitType", Some(grid_fit_type), Some(set_grid_fit_type)),
        ("thickness", Some(thickness), Some(set_thickness)),
        ("sharpness", Some(sharpness), Some(set_sharpness)),
        ("numLines", Some(num_lines), None),
    ];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("appendText", append_text),
        ("getTextFormat", get_text_format),
        ("replaceSelectedText", replace_selected_text),
        ("replaceText", replace_text),
        ("setSelection", set_selection),
        ("setTextFormat", set_text_format),
        ("getLineMetrics", get_line_metrics),
    ];
    write.define_public_builtin_instance_methods(mc, PUBLIC_INSTANCE_METHODS);

    class
}
