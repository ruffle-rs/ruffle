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
use crate::display_object::{EditText, TDisplayObject, TextLine, TextLineLayout};
use crate::font::FontType;
use crate::html::{FormatSpans, TextFormat, lower_from_text_spans};
use crate::string::{WStr, WString};
use swf::Twips;

const TEXT_LINE_MAX_LINE_WIDTH: f64 = 1_000_000.0;

pub fn create_text_line<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    avm2_stub_method!(activation, "flash.text.TextBlock", "createTextLine");

    let previous_text_line = args.try_get_object(0);
    let width = args.get_f64(1);

    let content = this.get_slot(block_slots::_CONTENT);

    if matches!(content, Value::Null) {
        return Ok(Value::Null);
    }

    let text = content
        .call_method(element_methods::GET_TEXT, &[], activation)
        .unwrap_or_else(|_| istr!("").into());
    let text = match text {
        Value::Null => return Ok(Value::Null),
        v => v.coerce_to_string(activation)?,
    };

    let next_line_start = match previous_text_line {
        Some(prev) => {
            let begin = prev
                .get_slot(line_slots::_TEXT_BLOCK_BEGIN_INDEX)
                .coerce_to_i32(activation)? as usize;
            let raw_len = prev
                .get_slot(line_slots::_RAW_TEXT_LENGTH)
                .coerce_to_i32(activation)? as usize;
            begin + raw_len
        }
        None => 0,
    };

    if next_line_start >= text.len() {
        this.set_slot(
            block_slots::_TEXT_LINE_CREATION_RESULT,
            istr!("complete").into(),
            activation,
        )?;
        return Ok(Value::Null);
    }

    let Value::Object(content_obj) = content else {
        unreachable!("TextBlock content slot must be ContentElement");
    };
    let movie = activation.caller_movie_or_root();
    let fallback = EditText::new_fte(activation.context, movie.clone(), 0.0, 0.0, width, 15.0);
    fallback.set_text(text.as_wstr(), activation.context);
    let element_format = content_obj
        .get_slot(element_slots::_ELEMENT_FORMAT)
        .as_object();
    let format = apply_format(activation, fallback, text.as_wstr(), element_format)?;
    fallback.set_word_wrap(true, activation.context);

    let measured_text = fallback.measure_text(activation.context);
    fallback.set_height(activation.context, measured_text.1.to_pixels());

    let spans = FormatSpans::from_text(WString::from(text.as_wstr()), format);
    let requested_width = if width >= TEXT_LINE_MAX_LINE_WIDTH {
        None
    } else {
        Some(Twips::from_pixels(width))
    };
    let layout = lower_from_text_spans(
        &spans,
        activation.context,
        movie.clone(),
        requested_width,
        false,
        true,
        FontType::Device,
    );
    let html_line = layout
        .lines()
        .first()
        .cloned()
        .expect("TextLine layout must contain a line for nonempty text");
    let text_line_layout = TextLineLayout::new(html_line, WString::from(text.as_wstr()));
    let raw_text_length = text_line_layout.raw_text_length();

    let text_line = TextLine::new(activation.context, movie, text_line_layout, fallback);
    let class = activation.avm2().classes().textline;
    let instance = initialize_for_allocator(activation.context, text_line.into(), class);

    instance.set_slot(line_slots::_TEXT_BLOCK, this.into(), activation)?;
    instance.set_slot(line_slots::_SPECIFIED_WIDTH, args.get_value(1), activation)?;
    instance.set_slot(
        line_slots::_RAW_TEXT_LENGTH,
        Value::from_usize_lossy(raw_text_length),
        activation,
    )?;
    instance.set_slot(
        line_slots::_TEXT_BLOCK_BEGIN_INDEX,
        Value::Integer(next_line_start as i32),
        activation,
    )?;

    if let Some(prev) = previous_text_line {
        prev.set_slot(line_slots::_NEXT_LINE, instance.into(), activation)?;
        instance.set_slot(line_slots::_PREVIOUS_LINE, prev.into(), activation)?;
    } else {
        this.set_slot(block_slots::_FIRST_LINE, instance.into(), activation)?;
    }
    this.set_slot(block_slots::_LAST_LINE, instance.into(), activation)?;

    this.set_slot(
        block_slots::_TEXT_LINE_CREATION_RESULT,
        istr!("success").into(),
        activation,
    )?;

    Ok(instance.into())
}

fn apply_format<'gc>(
    activation: &mut Activation<'_, 'gc>,
    display_object: EditText<'gc>,
    text: &WStr,
    element_format: Option<Object<'gc>>,
) -> Result<TextFormat, Error<'gc>> {
    let (format, is_device_font) = text_format_from_element_format(activation, element_format)?;
    display_object.set_is_device_font(activation.context, is_device_font);
    display_object.set_text_format(0, text.len(), format.clone(), activation.context);
    display_object.set_new_text_format(format.clone());

    Ok(format)
}

fn text_format_from_element_format<'gc>(
    activation: &mut Activation<'_, 'gc>,
    element_format: Option<Object<'gc>>,
) -> Result<(TextFormat, bool), Error<'gc>> {
    let Some(element_format) = element_format else {
        return Ok((TextFormat::default(), true));
    };

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

    Ok((
        TextFormat {
            color: Some(swf::Color::from_rgb(color, 0xFF)),
            size: Some(size),
            font,
            bold,
            italic,
            ..TextFormat::default()
        },
        is_device_font,
    ))
}
