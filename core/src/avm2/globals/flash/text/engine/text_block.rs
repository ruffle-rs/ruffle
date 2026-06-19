use ruffle_common::avm_string::AvmString;
use ruffle_macros::istr;

use crate::avm2::activation::Activation;
use crate::avm2::error::Error;
use crate::avm2::globals::flash::display::display_object::initialize_for_allocator;
use crate::avm2::globals::methods::flash_text_engine_content_element as element_methods;
use crate::avm2::globals::slots::flash_text_engine_content_element as element_slots;
use crate::avm2::globals::slots::flash_text_engine_element_format as format_slots;
use crate::avm2::globals::slots::flash_text_engine_font_description as font_desc_slots;
use crate::avm2::globals::slots::flash_text_engine_text_block as block_slots;
use crate::avm2::globals::slots::flash_text_engine_text_line as line_slots;
use crate::avm2::object::{Object, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2_stub_method;
use crate::display_object::{EditText, TDisplayObject, TextLine};
use crate::html::TextFormat;
use crate::string::WStr;

pub fn create_text_line<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    avm2_stub_method!(activation, "flash.text.TextBlock", "createTextLine");

    let previous_text_line = args.try_get_object(0);
    let width = args.get_f64(1);

    let previous_position = if let Some(previous_text_line) = previous_text_line {
        previous_text_line.get_slot(line_slots::_END_INDEX).as_u32() as usize
    } else {
        0
    };

    let content = this.get_slot(block_slots::_CONTENT);
    let text = get_text_from_content(content, activation)?
        .unwrap_or_else(|| istr!(""))
        .as_wstr();

    let next_position = next_line_break(text, previous_position);

    if text.is_empty() || next_position == text.len() && previous_position == next_position {
        // No more text.
        this.set_slot(
            block_slots::_TEXT_LINE_CREATION_RESULT,
            istr!("complete").into(),
            activation,
        )?;
        return Ok(Value::Null);
    }

    let line_index = if let Some(previous_text_line) = previous_text_line {
        previous_text_line
            .get_slot(line_slots::_LINE_INDEX)
            .as_u32()
            + 1
    } else {
        0
    };

    let subtext = &text[previous_position..next_position];

    let class = activation.avm2().classes().textline;
    let movie = activation.caller_movie_or_root();

    let fallback = EditText::new_fte(activation.context, movie.clone(), 0.0, 0.0, width, 15.0);
    fallback.set_text(subtext, activation.context);

    // FIXME: This needs to use `intrinsic_bounds` to measure the width
    // of the provided text, and set the width of the EditText to that.
    // Some games depend on this (e.g. Realm Grinder).

    let content_obj = content.as_object().unwrap();
    let element_format = content_obj
        .get_slot(element_slots::_ELEMENT_FORMAT)
        .as_object();
    apply_format(activation, fallback, element_format, line_index)?;

    let text_line = TextLine::new(activation.context, movie, fallback);
    let instance = initialize_for_allocator(activation.context, text_line.into(), class);

    instance.set_slot(line_slots::_TEXT_BLOCK, this.into(), activation)?;
    instance.set_slot(line_slots::_SPECIFIED_WIDTH, args.get_value(1), activation)?;
    instance.set_slot(
        line_slots::_RAW_TEXT_LENGTH,
        Value::from_usize_lossy(text.len()),
        activation,
    )?;
    instance.set_slot(
        line_slots::_BEGIN_INDEX,
        Value::from_usize_lossy(previous_position),
        activation,
    )?;
    instance.set_slot(
        line_slots::_END_INDEX,
        Value::from_usize_lossy(next_position),
        activation,
    )?;
    instance.set_slot(line_slots::_LINE_INDEX, line_index.into(), activation)?;

    if let Some(previous_text_line) = previous_text_line {
        instance.set_slot(
            line_slots::_PREVIOUS_LINE,
            previous_text_line.into(),
            activation,
        )?;
        previous_text_line.set_slot(
            //
            line_slots::_NEXT_LINE,
            instance.into(),
            activation,
        )?;
    }

    this.set_slot(
        block_slots::_TEXT_LINE_CREATION_RESULT,
        istr!("success").into(),
        activation,
    )?;
    this.set_slot(block_slots::_FIRST_LINE, instance.into(), activation)?;

    Ok(instance.into())
}

fn get_text_from_content<'gc>(
    content: Value<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Option<AvmString<'gc>>, Error<'gc>> {
    if matches!(content, Value::Null) {
        return Ok(None);
    }

    let text = content.call_method(element_methods::GET_TEXT, &[], activation)?;

    if matches!(text, Value::Null) {
        return Ok(None);
    }

    let text = text
        .coerce_to_string(activation)
        .expect("Guaranteed by AS bindings");

    if text.is_empty() {
        return Ok(None);
    }

    Ok(Some(text))
}

fn next_line_break(text: &WStr, start: usize) -> usize {
    let len = text[start..]
        .iter()
        .position(|ch| ch == b'\n' as u16)
        // Include the newline.
        .map(|pos| pos + 1);

    if let Some(len) = len {
        start + len
    } else {
        text.len()
    }
}

fn apply_format<'gc>(
    activation: &mut Activation<'_, 'gc>,
    edit_text: EditText<'gc>,
    element_format: Option<Object<'gc>>,
    line_index: u32,
) -> Result<(), Error<'gc>> {
    if let Some(element_format) = element_format {
        // TODO: Support more ElementFormat properties
        let color = element_format
            .get_slot(format_slots::_COLOR)
            .coerce_to_u32(activation)?;
        let size = element_format
            .get_slot(format_slots::_FONT_SIZE)
            .coerce_to_number(activation)?;

        let (font, bold, italic, is_device_font) = if let Value::Object(font_description) =
            element_format.get_slot(format_slots::_FONT_DESCRIPTION)
        {
            (
                Some(
                    font_description
                        .get_slot(font_desc_slots::_FONT_NAME)
                        .coerce_to_string(activation)?
                        .as_wstr()
                        .into(),
                ),
                Some(
                    &font_description
                        .get_slot(font_desc_slots::_FONT_WEIGHT)
                        .coerce_to_string(activation)?
                        == b"bold",
                ),
                Some(
                    &font_description
                        .get_slot(font_desc_slots::_FONT_POSTURE)
                        .coerce_to_string(activation)?
                        == b"italic",
                ),
                &font_description
                    .get_slot(font_desc_slots::_FONT_LOOKUP)
                    .coerce_to_string(activation)?
                    == b"device",
            )
        } else {
            (None, None, None, true)
        };

        let format = TextFormat {
            color: Some(swf::Color::from_rgb(color, 0xFF)),
            size: Some(size),
            font,
            bold,
            italic,
            ..TextFormat::default()
        };

        edit_text.set_is_device_font(activation.context, is_device_font);
        edit_text.set_text_format(
            0,
            edit_text.text_length(),
            format.clone(),
            activation.context,
        );
        edit_text.set_new_text_format(format);
    } else {
        edit_text.set_is_device_font(activation.context, true);
    }

    edit_text.set_word_wrap(true, activation.context);

    let measured_text = edit_text.measure_text(activation.context);

    edit_text.set_height(activation.context, measured_text.1.to_pixels());
    edit_text.set_y(measured_text.1 * line_index as i32);

    Ok(())
}
