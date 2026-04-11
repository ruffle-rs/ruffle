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
use crate::display_object::{EditText, TDisplayObject};
use crate::html::TextFormat;
use crate::string::WStr;

/// Shared logic for determining the start position and remaining text for a text line.
fn resolve_text_content<'gc>(
    activation: &mut Activation<'_, 'gc>,
    text_block: Object<'gc>,
    previous_text_line: Option<Object<'gc>>,
) -> Result<Option<(usize, crate::string::AvmString<'gc>, Object<'gc>)>, Error<'gc>> {
    let content = text_block.get_slot(block_slots::_CONTENT);

    if matches!(content, Value::Null) {
        return Ok(None);
    }

    // TODO: GraphicElement?
    let full_text = {
        let txt = content
            .call_method(element_methods::GET_TEXT, &[], activation)
            .unwrap_or_else(|_| istr!("").into());

        if matches!(txt, Value::Null) {
            // FP returns a null TextLine when `o` is null- note that
            // `o` is already coerced to a String because of the AS bindings.
            return Ok(None);
        } else {
            txt.coerce_to_string(activation)
                .expect("Guaranteed by AS bindings")
        }
    };

    let start = match previous_text_line {
        Some(prev) => {
            let prev_begin = prev
                .get_slot(line_slots::_TEXT_BLOCK_BEGIN_INDEX)
                .coerce_to_i32(activation)? as usize;
            let prev_len = prev
                .get_slot(line_slots::_RAW_TEXT_LENGTH)
                .coerce_to_i32(activation)? as usize;
            prev_begin + prev_len
        }
        None => 0,
    };

    if start >= full_text.len() {
        text_block.set_slot(
            block_slots::_TEXT_LINE_CREATION_RESULT,
            istr!("complete").into(),
            activation,
        )?;
        return Ok(None);
    }

    let content_obj = content.as_object().unwrap();
    Ok(Some((start, full_text, content_obj)))
}

/// Sets up an EditText display object with the remaining text, applies formatting,
/// measures the first layout line, then truncates to just that line's text.
/// Returns the number of characters that fit on the first layout line.
fn layout_text_line<'gc>(
    activation: &mut Activation<'_, 'gc>,
    display_object: EditText<'gc>,
    remaining_text: &WStr,
    content: Object<'gc>,
) -> Result<usize, Error<'gc>> {
    let element_format = content.get_slot(element_slots::_ELEMENT_FORMAT).as_object();

    // First pass: lay out all remaining text with word-wrap to find line breaks.
    display_object.set_text(remaining_text, activation.context);
    apply_format(activation, display_object, remaining_text, element_format)?;

    let raw_text_length = {
        let layout = display_object.layout();
        let lines = layout.lines();
        if let Some(first_line) = lines.first() {
            first_line.end() - first_line.start()
        } else {
            remaining_text.len()
        }
    };

    // Second pass: truncate to only the first line's text so the TextLine
    // doesn't render all remaining text (which would cause overlapping).
    let first_line_text = &remaining_text[..raw_text_length];
    display_object.set_text(first_line_text, activation.context);
    apply_format(activation, display_object, first_line_text, element_format)?;

    Ok(raw_text_length)
}

pub fn create_text_line<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    avm2_stub_method!(activation, "flash.text.TextBlock", "createTextLine");

    let previous_text_line = args.try_get_object(0);
    let width = args.get_f64(1);

    let Some((start, full_text, content)) =
        resolve_text_content(activation, this, previous_text_line)?
    else {
        return Ok(Value::Null);
    };

    let remaining_text = &full_text[start..];

    let class = activation.avm2().classes().textline;
    let movie = activation.caller_movie_or_root();

    // FIXME: TextLine should be its own DisplayObject
    let display_object: EditText =
        EditText::new_fte(activation.context, movie, 0.0, 0.0, width, 15.0);

    let raw_text_length = layout_text_line(activation, display_object, remaining_text, content)?;

    let instance = initialize_for_allocator(activation.context, display_object.into(), class);

    instance.set_slot(line_slots::_TEXT_BLOCK, this.into(), activation)?;
    instance.set_slot(line_slots::_SPECIFIED_WIDTH, args.get_value(1), activation)?;
    instance.set_slot(
        line_slots::_RAW_TEXT_LENGTH,
        (raw_text_length as i32).into(),
        activation,
    )?;
    instance.set_slot(
        line_slots::_TEXT_BLOCK_BEGIN_INDEX,
        (start as i32).into(),
        activation,
    )?;

    this.set_slot(
        block_slots::_TEXT_LINE_CREATION_RESULT,
        istr!("success").into(),
        activation,
    )?;

    if previous_text_line.is_none() {
        this.set_slot(block_slots::_FIRST_LINE, instance.into(), activation)?;
    }

    Ok(instance.into())
}

pub fn recreate_text_line<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    avm2_stub_method!(activation, "flash.text.TextBlock", "recreateTextLine");

    let text_line = args.get_object(activation, 0, "textLine")?;
    let previous_text_line = args.try_get_object(1);
    let width = args.get_f64(2);

    let Some((start, full_text, content)) =
        resolve_text_content(activation, this, previous_text_line)?
    else {
        return Ok(Value::Null);
    };

    let remaining_text = &full_text[start..];

    let display_object = text_line
        .as_display_object()
        .and_then(|d| d.as_edit_text())
        .unwrap();

    // Update the EditText width for the new layout
    display_object.set_width(activation.context, width);

    let raw_text_length = layout_text_line(activation, display_object, remaining_text, content)?;

    text_line.set_slot(line_slots::_TEXT_BLOCK, this.into(), activation)?;
    text_line.set_slot(line_slots::_SPECIFIED_WIDTH, args.get_value(2), activation)?;
    text_line.set_slot(
        line_slots::_RAW_TEXT_LENGTH,
        (raw_text_length as i32).into(),
        activation,
    )?;
    text_line.set_slot(
        line_slots::_TEXT_BLOCK_BEGIN_INDEX,
        (start as i32).into(),
        activation,
    )?;

    this.set_slot(
        block_slots::_TEXT_LINE_CREATION_RESULT,
        istr!("success").into(),
        activation,
    )?;

    if previous_text_line.is_none() {
        this.set_slot(block_slots::_FIRST_LINE, text_line.into(), activation)?;
    }

    Ok(text_line.into())
}

fn apply_format<'gc>(
    activation: &mut Activation<'_, 'gc>,
    display_object: EditText<'gc>,
    text: &WStr,
    element_format: Option<Object<'gc>>,
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

        display_object.set_is_device_font(activation.context, is_device_font);
        display_object.set_text_format(0, text.len(), format.clone(), activation.context);
        display_object.set_new_text_format(format);
    } else {
        display_object.set_is_device_font(activation.context, true);
    }

    display_object.set_word_wrap(true, activation.context);

    let measured_text = display_object.measure_text(activation.context);

    display_object.set_height(
        activation.context,
        measured_text.1.to_pixels() + EditText::GUTTER.to_pixels(),
    );

    Ok(())
}
