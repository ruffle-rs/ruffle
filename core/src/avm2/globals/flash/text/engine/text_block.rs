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
use crate::html::{FormatSpans, TextFormat, lower_from_text_spans_for_text_line};
use crate::string::{WStr, WString};
use swf::Twips;

const TEXT_LINE_MAX_LINE_WIDTH: f64 = 1_000_000.0;

fn format_from_content<'gc>(
    activation: &mut Activation<'_, 'gc>,
    content: Object<'gc>,
) -> Result<(TextFormat, f64, f64), Error<'gc>> {
    // Match the AS defaults from new ElementFormat() and new FontDescription().
    let mut format = TextFormat {
        font: Some(WString::from_utf8("_serif")),
        size: Some(12.0),
        color: Some(swf::Color::from_rgb(0, 0xff)),
        ..Default::default()
    };

    let mut tracking_left = 0.0;
    let mut tracking_right = 0.0;

    if let Some(ef) = content.get_slot(element_slots::_ELEMENT_FORMAT).as_object() {
        let color = ef
            .get_slot(format_slots::_COLOR)
            .coerce_to_u32(activation)?;
        format.color = Some(swf::Color::from_rgb(color & 0xff_ffff, 0xff));
        format.size = Some(
            ef.get_slot(format_slots::_FONT_SIZE)
                .coerce_to_number(activation)?,
        );
        tracking_left = ef
            .get_slot(format_slots::_TRACKING_LEFT)
            .coerce_to_number(activation)?;
        tracking_right = ef
            .get_slot(format_slots::_TRACKING_RIGHT)
            .coerce_to_number(activation)?;
        format.letter_spacing = Some(tracking_left + tracking_right);
        let kerning = ef
            .get_slot(format_slots::_KERNING)
            .coerce_to_string(activation)?;
        format.kerning = Some(kerning.to_utf8_lossy() != "off");
        let baseline_shift = ef
            .get_slot(format_slots::_BASELINE_SHIFT)
            .coerce_to_number(activation)?;
        format.baseline_shift = baseline_shift.is_finite().then_some(baseline_shift);
        if let Value::Object(fd) = ef.get_slot(format_slots::_FONT_DESCRIPTION) {
            let font_name = fd
                .get_slot(font_desc_slots::_FONT_NAME)
                .coerce_to_string(activation)?;
            let font_lookup = fd
                .get_slot(font_desc_slots::_FONT_LOOKUP)
                .coerce_to_string(activation)?;
            format.font_type = Some(match font_lookup.to_utf8_lossy().as_ref() {
                "embeddedCFF" => FontType::EmbeddedCFF,
                _ => FontType::Device,
            });
            format.font = Some(WString::from(font_name.as_wstr()));
            let weight = fd
                .get_slot(font_desc_slots::_FONT_WEIGHT)
                .coerce_to_string(activation)?;
            format.bold = Some(weight.to_utf8_lossy() == "bold");
            let posture = fd
                .get_slot(font_desc_slots::_FONT_POSTURE)
                .coerce_to_string(activation)?;
            format.italic = Some(posture.to_utf8_lossy() == "italic");
        }
    }

    Ok((format, tracking_left, tracking_right))
}

fn text_until_hard_break(text: &WStr, start: usize) -> WString {
    let tail = text.slice(start..).unwrap_or_else(WStr::empty);
    let mut len = tail.len();
    for (pos, unit) in tail.iter().enumerate() {
        if matches!(unit, 0x2028 | 0x2029) {
            len = pos + 1;
            break;
        }
    }
    WString::from(tail.slice(..len).unwrap_or_else(WStr::empty))
}

fn content_text<'gc>(
    activation: &mut Activation<'_, 'gc>,
    content: Value<'gc>,
) -> Result<Option<crate::string::AvmString<'gc>>, Error<'gc>> {
    let text = content.call_method(element_methods::GET_TEXT, &[], activation)?;
    Ok(match text {
        Value::Null => None,
        v => Some(v.coerce_to_string(activation)?),
    })
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

    let content = this.get_slot(block_slots::_CONTENT);

    if matches!(content, Value::Null) {
        return Ok(Value::Null);
    }

    let Some(text) = content_text(activation, content)? else {
        return Ok(Value::Null);
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

    let content_obj = content
        .as_object()
        .expect("TextBlock content slot must be null or ContentElement");
    let mut displayed_text = text_until_hard_break(text.as_wstr(), next_line_start);
    if let Some(ef) = content_obj
        .get_slot(element_slots::_ELEMENT_FORMAT)
        .as_object()
    {
        let typographic_case = ef
            .get_slot(format_slots::_TYPOGRAPHIC_CASE)
            .coerce_to_string(activation)?;
        let transformed = match typographic_case.to_utf8_lossy().as_ref() {
            "uppercase" => Some(displayed_text.to_utf8_lossy().to_uppercase()),
            "lowercase" => Some(displayed_text.to_utf8_lossy().to_lowercase()),
            _ => None,
        };
        if let Some(transformed) = transformed {
            let transformed = WString::from_utf8(&transformed);
            if transformed.len() == displayed_text.len() {
                displayed_text = transformed;
            }
        }
    }
    let displayed_text = displayed_text;
    let (format, tracking_left, tracking_right) = format_from_content(activation, content_obj)?;
    let spans = FormatSpans::from_text(displayed_text.clone(), format);
    let requested_width = if width >= TEXT_LINE_MAX_LINE_WIDTH {
        None
    } else {
        Some(Twips::from_pixels(width))
    };
    let movie = activation.caller_movie_or_root();
    let layout = lower_from_text_spans_for_text_line(
        &spans,
        activation.context,
        movie.clone(),
        requested_width,
        false,
        true,
        FontType::Device,
    );
    let Some(mut html_line) = layout.lines().first().cloned() else {
        return Ok(Value::Null);
    };
    html_line.trim_edge_tracking(
        Twips::from_pixels(tracking_left),
        Twips::from_pixels(tracking_right),
    );
    let text_line_layout = TextLineLayout::new(html_line, displayed_text, next_line_start);
    let raw_text_length = text_line_layout.raw_text_length();

    let fallback = EditText::new_fte(activation.context, movie.clone(), 0.0, 0.0, width, 15.0);
    fallback.set_text(text.as_wstr(), activation.context);
    let element_format = content_obj
        .get_slot(element_slots::_ELEMENT_FORMAT)
        .as_object();
    apply_format(activation, fallback, text.as_wstr(), element_format)?;

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

    display_object.set_height(activation.context, measured_text.1.to_pixels());

    Ok(())
}

pub fn recreate_text_line<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .expect("TextBlock native method receiver must be an object");
    let Some(text_line) = args.try_get_object(0) else {
        return Ok(Value::Null);
    };
    let previous_text_line = args.try_get_object(1);

    let content = this.get_slot(block_slots::_CONTENT);
    if matches!(content, Value::Null) {
        return Ok(Value::Null);
    }

    let Some(text) = content_text(activation, content)? else {
        return Ok(Value::Null);
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

    let content_obj = content
        .as_object()
        .expect("TextBlock content slot must be null or ContentElement");
    let mut displayed_text = text_until_hard_break(text.as_wstr(), next_line_start);
    if let Some(ef) = content_obj
        .get_slot(element_slots::_ELEMENT_FORMAT)
        .as_object()
    {
        let typographic_case = ef
            .get_slot(format_slots::_TYPOGRAPHIC_CASE)
            .coerce_to_string(activation)?;
        let transformed = match typographic_case.to_utf8_lossy().as_ref() {
            "uppercase" => Some(displayed_text.to_utf8_lossy().to_uppercase()),
            "lowercase" => Some(displayed_text.to_utf8_lossy().to_lowercase()),
            _ => None,
        };
        if let Some(transformed) = transformed {
            let transformed = WString::from_utf8(&transformed);
            if transformed.len() == displayed_text.len() {
                displayed_text = transformed;
            }
        }
    }
    let (format, tracking_left, tracking_right) = format_from_content(activation, content_obj)?;
    let spans = FormatSpans::from_text(displayed_text.clone(), format);
    let width = args.get_f64(2);
    let requested_width = if width >= 1_000_000.0 {
        None
    } else {
        Some(Twips::from_pixels(width))
    };
    let movie = activation.caller_movie_or_root();
    let layout = lower_from_text_spans_for_text_line(
        &spans,
        activation.context,
        movie.clone(),
        requested_width,
        false,
        true,
        FontType::Device,
    );
    let Some(mut html_line) = layout.lines().first().cloned() else {
        return Ok(Value::Null);
    };
    html_line.trim_edge_tracking(
        Twips::from_pixels(tracking_left),
        Twips::from_pixels(tracking_right),
    );
    let text_line_layout = TextLineLayout::new(html_line, displayed_text, next_line_start);
    let raw_text_length = text_line_layout.raw_text_length();

    let text_line_display = text_line
        .as_display_object()
        .expect("TextBlock.recreateTextLine target must be a display object")
        .as_text_line()
        .expect("TextBlock.recreateTextLine target must be a TextLine");
    let fallback = EditText::new_fte(activation.context, movie, 0.0, 0.0, width, 15.0);
    fallback.set_text(text.as_wstr(), activation.context);
    let element_format = content_obj
        .get_slot(element_slots::_ELEMENT_FORMAT)
        .as_object();
    apply_format(activation, fallback, text.as_wstr(), element_format)?;
    text_line_display.set_line(activation.context, text_line_layout, fallback);

    text_line.set_slot(line_slots::_TEXT_BLOCK, this.into(), activation)?;
    text_line.set_slot(line_slots::_SPECIFIED_WIDTH, width.into(), activation)?;
    text_line.set_slot(
        line_slots::_RAW_TEXT_LENGTH,
        Value::from_usize_lossy(raw_text_length),
        activation,
    )?;
    text_line.set_slot(
        line_slots::_TEXT_BLOCK_BEGIN_INDEX,
        Value::Integer(next_line_start as i32),
        activation,
    )?;
    this.set_slot(
        block_slots::_TEXT_LINE_CREATION_RESULT,
        istr!("success").into(),
        activation,
    )?;

    Ok(text_line.into())
}
