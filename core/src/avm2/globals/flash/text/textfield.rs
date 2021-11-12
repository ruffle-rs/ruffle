//! `flash.text.TextField` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, TObject, TextFormatObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::display_object::{AutoSizeMode, EditText, TDisplayObject, TextSelection};
use crate::html::TextFormat;
use crate::string::AvmString;
use crate::tag_utils::SwfMovie;
use crate::vminterface::AvmType;
use gc_arena::{GcCell, MutationContext};
use std::sync::Arc;

/// Implements `flash.text.TextField`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;

        if this.as_display_object().is_none() {
            let movie = Arc::new(SwfMovie::empty(activation.context.swf.version()));
            let movie_library = activation
                .context
                .library
                .library_for_movie_mut(movie.clone());
            movie_library.force_avm_type(AvmType::Avm2);

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
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

pub fn autosize<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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

pub fn background_color<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok((this.background_color()).into());
    }

    Ok(Value::Undefined)
}

pub fn set_background_color<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let new_color = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_u32(activation)?;
        this.set_background_color(activation.context.gc_context, new_color);
    }

    Ok(Value::Undefined)
}

pub fn border<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.border_color().into());
    }

    Ok(Value::Undefined)
}

pub fn set_border_color<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let border_color = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_u32(activation)?;
        this.set_border_color(activation.context.gc_context, border_color);
    }

    Ok(Value::Undefined)
}

pub fn default_text_format<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let new_text_format = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?;
        if let Some(new_text_format) = new_text_format.as_text_format() {
            this.set_new_text_format(new_text_format.clone(), &mut activation.context);
        };
    }

    Ok(Value::Undefined)
}

pub fn display_as_password<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let html_text = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_string(activation)?;

        this.set_is_html(&mut activation.context, true);
        this.set_html_text(&html_text, &mut activation.context)?;
    }

    Ok(Value::Undefined)
}

pub fn length<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let text = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_string(activation)?;

        this.set_is_html(&mut activation.context, false);
        this.set_text(&text, &mut activation.context)?;
    }

    Ok(Value::Undefined)
}

pub fn text_color<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
            return Err(format!("Invalid TextField.type: {}", is_editable).into());
        }
    }

    Ok(Value::Undefined)
}

pub fn word_wrap<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let tf = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?;
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
        };
    }

    Ok(Value::Undefined)
}

/// Construct `TextField`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.text"), "TextField"),
        Some(QName::new(Namespace::package("flash.display"), "InteractiveObject").into()),
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
        (
            "backgroundColor",
            Some(background_color),
            Some(set_background_color),
        ),
        ("border", Some(border), Some(set_border)),
        ("borderColor", Some(border_color), Some(set_border_color)),
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
        ("multiline", Some(multiline), Some(set_multiline)),
        ("selectable", Some(selectable), Some(set_selectable)),
        ("text", Some(text), Some(set_text)),
        ("textColor", Some(text_color), Some(set_text_color)),
        ("textHeight", Some(text_height), None),
        ("textWidth", Some(text_width), None),
        ("type", Some(get_type), Some(set_type)),
        ("wordWrap", Some(word_wrap), Some(set_word_wrap)),
    ];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("appendText", append_text),
        ("getTextFormat", get_text_format),
        ("replaceSelectedText", replace_selected_text),
        ("replaceText", replace_text),
        ("setSelection", set_selection),
        ("setTextFormat", set_text_format),
    ];
    write.define_public_builtin_instance_methods(mc, PUBLIC_INSTANCE_METHODS);

    class
}
