//! `flash.text.TextField` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::error::{make_error_2006, make_error_2008};
use crate::avm2::globals::flash::display::display_object::initialize_for_allocator;
use crate::avm2::object::{ClassObject, Object, TextFormatObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::{ArrayObject, ArrayStorage, Error};
use crate::display_object::{AutoSizeMode, EditText, TextSelection};
use crate::html::TextFormat;
use crate::string::AvmString;
use crate::{avm2_stub_getter, avm2_stub_setter};
use ruffle_macros::istr;
use swf::{Color, Point};

pub fn text_field_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    // Creating a TextField from AS ignores SymbolClass linkage.
    let movie = activation.caller_movie_or_root();
    let display_object = EditText::new(activation.context, movie, 0.0, 0.0, 100.0, 100.0).into();

    Ok(initialize_for_allocator(
        activation.context,
        display_object,
        class,
    ))
}

pub fn get_always_show_selection<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    else {
        return Ok(Value::Undefined);
    };

    Ok(this.always_show_selection().into())
}

pub fn set_always_show_selection<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    else {
        return Ok(Value::Undefined);
    };

    let value = args.get_bool(0);
    this.set_always_show_selection(value);

    Ok(Value::Undefined)
}

pub fn get_auto_size<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let autosize = match this.autosize() {
            AutoSizeMode::None => istr!("none"),
            AutoSizeMode::Left => istr!("left"),
            AutoSizeMode::Center => istr!("center"),
            AutoSizeMode::Right => istr!("right"),
        };

        return Ok(autosize.into());
    }

    Ok(Value::Undefined)
}

pub fn set_auto_size<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let value = args.get_string_non_null(activation, 0, "autoSize")?;
        this.set_autosize(
            if &value == b"left" {
                AutoSizeMode::Left
            } else if &value == b"center" {
                AutoSizeMode::Center
            } else if &value == b"right" {
                AutoSizeMode::Right
            } else if &value == b"none" {
                AutoSizeMode::None
            } else {
                return Err(make_error_2008(activation, "autoSize"));
            },
            activation.context,
        );
    }

    Ok(Value::Undefined)
}

pub fn get_background<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        return Ok((this.has_background()).into());
    }

    Ok(Value::Undefined)
}

pub fn set_background<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let has_background = args.get_bool(0);
        this.set_has_background(has_background);
    }

    Ok(Value::Undefined)
}

pub fn get_background_color<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.background_color().to_rgb().into());
    }

    Ok(Value::Undefined)
}

pub fn set_background_color<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let rgb = args.get_u32(0);
        let color = Color::from_rgb(rgb, 255);
        this.set_background_color(color);
    }

    Ok(Value::Undefined)
}

pub fn get_border<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.has_border().into());
    }

    Ok(Value::Undefined)
}

pub fn set_border<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let border = args.get_bool(0);
        this.set_has_border(border);
    }

    Ok(Value::Undefined)
}

pub fn get_border_color<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.border_color().to_rgb().into());
    }

    Ok(Value::Undefined)
}

pub fn set_border_color<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let rgb = args.get_u32(0);
        let color = Color::from_rgb(rgb, 255);
        this.set_border_color(color);
    }

    Ok(Value::Undefined)
}

pub fn get_condense_white<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.condense_white().into());
    }

    Ok(Value::Undefined)
}

pub fn set_condense_white<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let value = args.get_bool(0);
        this.set_condense_white(value);
    }

    Ok(Value::Undefined)
}

pub fn get_default_text_format<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        return Ok(TextFormatObject::from_text_format(activation, this.new_text_format())?.into());
    }

    Ok(Value::Undefined)
}

pub fn set_default_text_format<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let new_text_format = args.try_get_object(0);

        if let Some(new_text_format) = new_text_format {
            if let Some(new_text_format) = new_text_format.as_text_format() {
                this.set_new_text_format(new_text_format.clone());
            }
        }
    }

    Ok(Value::Undefined)
}

pub fn get_display_as_password<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.is_password().into());
    }

    Ok(Value::Undefined)
}

pub fn set_display_as_password<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let is_password = args.get_bool(0);

        this.set_password(is_password, activation.context);
    }

    Ok(Value::Undefined)
}

pub fn get_embed_fonts<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        return Ok((!this.is_device_font()).into());
    }

    Ok(Value::Undefined)
}

pub fn set_embed_fonts<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let is_embed_fonts = args.get_bool(0);

        this.set_is_device_font(activation.context, !is_embed_fonts);
    }

    Ok(Value::Undefined)
}

pub fn get_html_text<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        return Ok(AvmString::new(activation.gc(), this.html_text()).into());
    }

    Ok(Value::Undefined)
}

pub fn set_html_text<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let html_text = args.get_string(activation, 0);

        this.set_is_html(true);
        this.set_html_text(&html_text, activation.context);
    }

    Ok(Value::Undefined)
}

pub fn get_length<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.text_length().into());
    }

    Ok(Value::Undefined)
}

pub fn get_multiline<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.is_multiline().into());
    }

    Ok(Value::Undefined)
}

pub fn set_multiline<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let is_multiline = args.get_bool(0);

        this.set_multiline(is_multiline, activation.context);
    }

    Ok(Value::Undefined)
}

pub fn get_selectable<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.is_selectable().into());
    }

    Ok(Value::Undefined)
}

pub fn set_selectable<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let is_selectable = args.get_bool(0);

        this.set_selectable(is_selectable);
    }

    Ok(Value::Undefined)
}

pub fn get_text<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        return Ok(AvmString::new(activation.gc(), this.text()).into());
    }

    Ok(Value::Undefined)
}

pub fn set_text<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let text = args.get_string_non_null(activation, 0, "text")?;

        this.set_text(&text, activation.context);
    }

    Ok(Value::Undefined)
}

pub fn get_text_color<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
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
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let text_color = args.get_u32(0);
        let desired_format = TextFormat {
            color: Some(swf::Color::from_rgb(text_color, 0xFF)),
            ..TextFormat::default()
        };

        this.set_text_format(
            0,
            this.text_length(),
            desired_format.clone(),
            activation.context,
        );
        this.set_new_text_format(desired_format);
    }

    Ok(Value::Undefined)
}

pub fn get_text_height<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let metrics = this.measure_text(activation.context);
        return Ok(metrics.1.to_pixels().into());
    }

    Ok(Value::Undefined)
}

pub fn get_text_width<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let metrics = this.measure_text(activation.context);
        return Ok(metrics.0.to_pixels().into());
    }

    Ok(Value::Undefined)
}

pub fn get_type<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let type_ = match this.is_editable() {
            true => istr!("input"),
            false => istr!("dynamic"),
        };

        return Ok(type_.into());
    }

    Ok(Value::Undefined)
}

pub fn set_type<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let is_editable = args.get_string_non_null(activation, 0, "type")?;

        if &is_editable == b"input" {
            this.set_editable(true);
        } else if &is_editable == b"dynamic" {
            this.set_editable(false);
        } else {
            return Err(make_error_2008(activation, "type"));
        }
    }

    Ok(Value::Undefined)
}

pub fn get_word_wrap<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.is_word_wrap().into());
    }

    Ok(Value::Undefined)
}

pub fn set_word_wrap<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let is_word_wrap = args.get_bool(0);

        this.set_word_wrap(is_word_wrap, activation.context);
    }

    Ok(Value::Undefined)
}

pub fn append_text<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let new_text = args.get_string_non_null(activation, 0, "text")?;
        let existing_length = this.text_length();

        this.replace_text(
            existing_length,
            existing_length,
            &new_text,
            activation.context,
        );
    }

    Ok(Value::Undefined)
}

pub fn get_text_format<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let mut begin_index = args.get_i32(0);
        let mut end_index = args.get_i32(1);

        if end_index >= 0 && (begin_index >= end_index || begin_index < 0) {
            return Err(make_error_2006(activation));
        }

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
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let value = args.get_string_non_null(activation, 0, "text")?;
        let selection = this
            .selection()
            .unwrap_or_else(|| TextSelection::for_position(0));

        this.replace_text(
            selection.start(),
            selection.end(),
            &value,
            activation.context,
        );
    }

    Ok(Value::Undefined)
}

pub fn replace_text<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        // FIXME what is the behavior for negative beginIndex and endIndex?
        let begin_index = args.get_i32(0);
        let end_index = args.get_i32(1);
        let value = args.get_string_non_null(activation, 2, "text")?;

        this.replace_text(
            begin_index as usize,
            end_index as usize,
            &value,
            activation.context,
        );
    }

    Ok(Value::Undefined)
}

pub fn get_caret_index<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        return if let Some(selection) = this.selection() {
            Ok(selection.to().into())
        } else {
            Ok(0.into())
        };
    }

    Ok(Value::Undefined)
}

pub fn get_selection_begin_index<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        return if let Some(selection) = this.selection() {
            Ok(selection.start().into())
        } else {
            Ok(0.into())
        };
    }

    Ok(Value::Undefined)
}

pub fn get_selection_end_index<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        return if let Some(selection) = this.selection() {
            Ok(selection.end().into())
        } else {
            Ok(0.into())
        };
    }

    Ok(Value::Undefined)
}

pub fn set_selection<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        // FIXME what is the behavior for negative beginIndex and endIndex?
        let begin_index = args.get_i32(0);
        let end_index = args.get_i32(1);

        this.set_selection(Some(TextSelection::for_range(
            begin_index as usize,
            end_index as usize,
        )));
    }

    Ok(Value::Undefined)
}

pub fn set_text_format<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let tf = args.try_get_object(0);
        if let Some(tf) = tf {
            if let Some(tf) = tf.as_text_format() {
                let mut begin_index = args.get_i32(1);
                let mut end_index = args.get_i32(2);

                if begin_index < 0 {
                    begin_index = 0;
                }

                if begin_index as usize > this.text_length() {
                    return Err(make_error_2006(activation));
                }

                if end_index < 0 {
                    end_index = this.text_length() as i32;
                }

                if end_index as usize > this.text_length() {
                    return Err(make_error_2006(activation));
                }

                this.set_text_format(
                    begin_index as usize,
                    end_index as usize,
                    tf.clone(),
                    activation.context,
                );
            }
        }
    }

    Ok(Value::Undefined)
}

pub fn get_anti_alias_type<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let anti_alias_type = if this.render_settings().is_advanced() {
            istr!("advanced")
        } else {
            istr!("normal")
        };

        return Ok(anti_alias_type.into());
    }

    Ok(Value::Undefined)
}

pub fn set_anti_alias_type<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let old_settings = this.render_settings();
        let new_type = args.get_string_non_null(activation, 0, "antiAliasType")?;

        if &new_type == b"advanced" {
            this.set_render_settings(old_settings.with_advanced_rendering());
        } else if &new_type == b"normal" {
            this.set_render_settings(old_settings.with_normal_rendering());
        }
    }
    Ok(Value::Undefined)
}

pub fn get_grid_fit_type<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let grid_fit_type = match this.render_settings().grid_fit() {
            swf::TextGridFit::None => istr!("none"),
            swf::TextGridFit::Pixel => istr!("pixel"),
            swf::TextGridFit::SubPixel => istr!("subpixel"),
        };

        return Ok(grid_fit_type.into());
    }

    Ok(Value::Undefined)
}

pub fn set_grid_fit_type<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let old_settings = this.render_settings();
        let new_type = args.get_string_non_null(activation, 0, "gridFitType")?;

        if &new_type == b"pixel" {
            this.set_render_settings(old_settings.with_grid_fit(swf::TextGridFit::Pixel));
        } else if &new_type == b"subpixel" {
            this.set_render_settings(old_settings.with_grid_fit(swf::TextGridFit::SubPixel));
        } else {
            //NOTE: In AS3 invalid values are treated as None.
            this.set_render_settings(old_settings.with_grid_fit(swf::TextGridFit::None));
        }
    }
    Ok(Value::Undefined)
}

pub fn get_thickness<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.render_settings().thickness().into());
    }

    Ok(0.into())
}

pub fn set_thickness<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let old_settings = this.render_settings();
        let mut new_thickness = args.get_f64(0);

        // NOTE: The thickness clamp is ONLY enforced on AS3.
        new_thickness = new_thickness.clamp(-200.0, 200.0);

        this.set_render_settings(old_settings.with_thickness(new_thickness as f32));
    }

    Ok(Value::Undefined)
}

pub fn get_sharpness<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.render_settings().sharpness().into());
    }

    Ok(0.into())
}

pub fn set_sharpness<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let old_settings = this.render_settings();
        let mut new_sharpness = args.get_f64(0);

        // NOTE: The sharpness clamp is only enforced on AS3.
        new_sharpness = new_sharpness.clamp(-400.0, 400.0);

        this.set_render_settings(old_settings.with_sharpness(new_sharpness as f32));
    }

    Ok(Value::Undefined)
}

pub fn get_num_lines<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.layout_lines().into());
    }

    Ok(Value::Undefined)
}

pub fn get_line_metrics<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    else {
        return Ok(Value::Undefined);
    };

    let line_num = args.get_i32(0);
    if line_num < 0 {
        return Err(make_error_2006(activation));
    }

    let metrics = this.line_metrics(line_num as usize);

    let Some(metrics) = metrics else {
        return Err(make_error_2006(activation));
    };

    let metrics_class = activation.avm2().classes().textlinemetrics;
    metrics_class.construct(
        activation,
        &[
            metrics.x.to_pixels().into(),
            metrics.width.to_pixels().into(),
            metrics.height.to_pixels().into(),
            metrics.ascent.to_pixels().into(),
            metrics.descent.to_pixels().into(),
            metrics.leading.to_pixels().into(),
        ],
    )
}

pub fn get_line_length<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    else {
        return Ok(Value::Undefined);
    };

    let line_num = args.get_i32(0);
    if line_num < 0 {
        return Err(make_error_2006(activation));
    }

    if let Some(length) = this.line_length(line_num as usize) {
        Ok(length.into())
    } else {
        Err(make_error_2006(activation))
    }
}

pub fn get_line_text<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let line_num = args.get_i32(0);
        return if let Some(text) = this.line_text(line_num as usize) {
            Ok(AvmString::new(activation.gc(), text).into())
        } else {
            Err(make_error_2006(activation))
        };
    }

    Ok(Value::Undefined)
}

pub fn get_line_offset<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    else {
        return Ok(Value::Undefined);
    };

    let line_num = args.get_i32(0);
    if line_num < 0 {
        return Err(make_error_2006(activation));
    }

    if let Some(offset) = this.line_offset(line_num as usize) {
        Ok(offset.into())
    } else {
        Err(make_error_2006(activation))
    }
}

pub fn get_bottom_scroll_v<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.bottom_scroll().into());
    }

    Ok(Value::Undefined)
}

pub fn get_max_scroll_v<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.maxscroll().into());
    }

    Ok(Value::Undefined)
}

pub fn get_max_scroll_h<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.maxhscroll().into());
    }

    Ok(Value::Undefined)
}

pub fn get_scroll_v<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.scroll().into());
    }

    Ok(Value::Undefined)
}

pub fn set_scroll_v<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let input = args.get_i32(0);
        this.set_scroll(input as f64);
    }

    Ok(Value::Undefined)
}

pub fn get_scroll_h<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.hscroll().into());
    }

    Ok(Value::Undefined)
}

pub fn set_scroll_h<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        // NOTE: The clamping behavior here is identical to AVM1.
        // This is incorrect, SWFv9 uses more complex behavior and AS3 can only
        // be present in v9 SWFs.
        let input = args.get_i32(0);
        let clamped = input.abs().min(this.maxhscroll() as i32);
        this.set_hscroll(clamped as f64);
    }

    Ok(Value::Undefined)
}

pub fn get_max_chars<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.max_chars().into());
    }

    Ok(Value::Undefined)
}

pub fn set_max_chars<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let input = args.get_i32(0);
        this.set_max_chars(input);
    }

    Ok(Value::Undefined)
}

pub fn get_mouse_wheel_enabled<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.text.TextField", "mouseWheelEnabled");
    Ok(true.into())
}

pub fn set_mouse_wheel_enabled<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_setter!(activation, "flash.text.TextField", "mouseWheelEnabled");
    Ok(Value::Undefined)
}

pub fn get_restrict<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        return match this.restrict() {
            Some(value) => Ok(AvmString::new(activation.gc(), value).into()),
            None => Ok(Value::Null),
        };
    }

    Ok(Value::Undefined)
}

pub fn set_restrict<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        this.set_restrict(args.try_get_string(0).as_deref());
    }
    Ok(Value::Undefined)
}

pub fn get_selected_text<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    {
        let text = this.text();
        let mut selection = this
            .selection()
            .unwrap_or_else(|| TextSelection::for_position(0));
        selection.clamp(text.len());

        let start_index = selection.start();
        let end_index = selection.end();

        return Ok(AvmString::new(activation.gc(), &text[start_index..end_index]).into());
    }
    Ok(istr!("").into())
}

pub fn get_text_runs<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    else {
        return Ok(Value::Undefined);
    };

    let textrun_class = activation.avm2().classes().textrun;

    let array = this
        .spans()
        .iter_spans()
        .filter(|(start, end, _, _)| {
            // Flash never returns empty spans here, but we currently require
            // that at least one span is present albeit an empty one.
            start != end
        })
        .map(|(start, end, _, format)| {
            let tf = TextFormatObject::from_text_format(activation, format.get_text_format())?;
            textrun_class.construct(activation, &[start.into(), end.into(), tf.into()])
        })
        .collect::<Result<ArrayStorage<'gc>, Error<'gc>>>()?;
    Ok(ArrayObject::from_storage(activation.context, array).into())
}

pub fn get_line_index_of_char<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    else {
        return Ok(Value::Undefined);
    };

    let index = args.get_i32(0);
    if index < 0 {
        // Docs say "throw RangeError", reality says "return -1".
        return Ok(Value::Number(-1f64));
    }

    if let Some(line) = this.line_index_of_char(index as usize) {
        Ok(line.into())
    } else {
        Ok(Value::Number(-1f64))
    }
}

pub fn get_char_index_at_point<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    else {
        return Ok(Value::Undefined);
    };

    // No idea why FP does this weird 1px translation...
    let x = args.get_f64(0) + 1.0;
    let y = args.get_f64(1);

    if let Some(index) = this.char_index_at_point(Point::from_pixels(x, y)) {
        Ok(index.into())
    } else {
        Ok(Value::Number(-1f64))
    }
}

pub fn get_line_index_at_point<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    else {
        return Ok(Value::Undefined);
    };

    // No idea why FP does this weird 1px translation...
    let x = args.get_f64(0) + 1.0;
    let y = args.get_f64(1);

    if let Some(index) = this.line_index_at_point(Point::from_pixels(x, y)) {
        Ok(index.into())
    } else {
        Ok(Value::Number(-1f64))
    }
}

pub fn get_first_char_in_paragraph<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    else {
        return Ok(Value::Undefined);
    };

    let char_index = args.get_i32(0);
    if char_index < 0 {
        return Ok((-1).into());
    }

    Ok(this
        .paragraph_start_index_at(char_index as usize)
        .map(|i| i as i32)
        .unwrap_or(-1)
        .into())
}

pub fn get_paragraph_length<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    else {
        return Ok(Value::Undefined);
    };

    let char_index = args.get_i32(0);
    if char_index < 0 {
        return Ok((-1).into());
    }

    Ok(this
        .paragraph_length_at(char_index as usize)
        .map(|i| i as i32)
        .unwrap_or(-1)
        .into())
}

pub fn get_char_boundaries<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    else {
        return Ok(Value::Undefined);
    };

    let char_index = args.get_i32(0);
    if char_index < 0 {
        return Ok(Value::Null);
    }

    let Some(bounds) = this.char_bounds(char_index as usize) else {
        return Ok(Value::Null);
    };

    if bounds.width() == swf::Twips::ZERO {
        return Ok(Value::Null);
    }

    let rect = activation.avm2().classes().rectangle.construct(
        activation,
        &[
            bounds.x_min.to_pixels().into(),
            bounds.y_min.to_pixels().into(),
            bounds.width().to_pixels().into(),
            bounds.height().to_pixels().into(),
        ],
    )?;

    Ok(rect)
}

pub fn get_style_sheet<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    else {
        return Ok(Value::Undefined);
    };

    Ok(match this.style_sheet_avm2() {
        Some(style_sheet) => Value::Object(Object::StyleSheetObject(style_sheet)),
        None => Value::Null,
    })
}

pub fn set_style_sheet<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let Some(this) = this
        .as_display_object()
        .and_then(|this| this.as_edit_text())
    else {
        return Ok(Value::Undefined);
    };

    let style_sheet = args.try_get_object(0).and_then(|o| o.as_style_sheet());

    this.set_style_sheet_avm2(activation.context, style_sheet);

    Ok(Value::Undefined)
}
