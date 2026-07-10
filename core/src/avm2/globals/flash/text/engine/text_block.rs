use ruffle_macros::istr;

use crate::avm2::Avm2StrRepresentable;
use crate::avm2::activation::Activation;
use crate::avm2::error::{Error, Error2004Type, make_error_2004, make_error_2008};
use crate::avm2::globals::flash::display::display_object::initialize_for_allocator;
use crate::avm2::globals::slots::flash_text_engine_group_element as group_slots;
use crate::avm2::globals::slots::flash_text_engine_text_justifier as justifier_slots;
use crate::avm2::globals::slots::flash_text_engine_text_line as line_slots;
use crate::avm2::object::{ElementFormatObject, Object, TObject, VectorObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::display_object::{EditText, LineMetrics, TDisplayObject, TextLine};
use crate::font::FontType;
use crate::fte::{FontLookupValue, FontPostureValue, FontWeightValue, KerningValue};
use crate::fte::{TextBaselineValue, TextLineValidity, TextRotationValue};
use crate::html::{FormatSpans, TextFormat, TextSpan, lower_from_text_spans};
use crate::string::{AvmString, WStr, WString};
use swf::Twips;

pub use crate::avm2::object::text_block_allocator;

pub fn get_apply_non_linear_font_scaling<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this.apply_non_linear_font_scaling().into())
}

pub fn set_apply_non_linear_font_scaling<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    this.set_apply_non_linear_font_scaling(args.get_bool(0));
    Ok(Value::Undefined)
}

pub fn get_baseline_font_description<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this
        .baseline_font_description()
        .map(Value::from)
        .unwrap_or(Value::Null))
}

pub fn set_baseline_font_description<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    let value = args.try_get_object(0);
    this.set_baseline_font_description(value, activation.gc());
    Ok(Value::Undefined)
}

pub fn get_baseline_font_size<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this.baseline_font_size().into())
}

pub fn set_baseline_font_size<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    let value = args.get_f64(0);
    if value < 0.0 {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    }
    this.set_baseline_font_size(value);
    Ok(Value::Undefined)
}

pub fn get_baseline_zero<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this.baseline_zero().as_avm2_str(activation).into())
}

pub fn set_baseline_zero<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    let value = args.get_string_non_null(activation, 0, "baselineZero")?;
    let value = TextBaselineValue::from_avm2_str(&value)
        .filter(|v| !matches!(v, TextBaselineValue::UseDominantBaseline))
        .ok_or_else(|| make_error_2008(activation, "baselineZero"))?;
    this.set_baseline_zero(value);
    Ok(Value::Undefined)
}

pub fn get_bidi_level<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this.bidi_level().into())
}

pub fn set_bidi_level<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    let value = args.get_i32(0);
    if value < 0 {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    }
    this.set_bidi_level(value);
    // Changing the bidi level invalidates every previously broken line.
    invalidate_all_lines(activation, this.first_line());
    Ok(Value::Undefined)
}

pub fn get_line_rotation<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this.line_rotation().as_avm2_str(activation).into())
}

pub fn set_line_rotation<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    let value = args.get_string_non_null(activation, 0, "lineRotation")?;
    let value = TextRotationValue::from_avm2_str(&value)
        .filter(|v| !matches!(v, TextRotationValue::Auto))
        .ok_or_else(|| make_error_2008(activation, "lineRotation"))?;
    this.set_line_rotation(value);
    Ok(Value::Undefined)
}

pub fn get_tab_stops<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this
        .tab_stops()
        .map(|v| Value::from(VectorObject::from_vector(v.storage().clone(), activation)))
        .unwrap_or(Value::Null))
}

pub fn set_tab_stops<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    let tab_stops = args
        .try_get_object(0)
        .and_then(|v| v.as_vector_object())
        .map(|v| VectorObject::from_vector(v.storage().clone(), activation));
    this.set_tab_stops(tab_stops, activation.gc());
    Ok(Value::Undefined)
}

pub fn get_text_justifier<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this
        .text_justifier()
        .map(Value::from)
        .unwrap_or(Value::Null))
}

pub fn set_text_justifier<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    this.set_text_justifier(
        args.get_object(activation, 0, "textJustifier")?,
        activation.gc(),
    );
    Ok(Value::Undefined)
}

pub fn get_content<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this.content().map(Value::from).unwrap_or(Value::Null))
}

pub fn set_content<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    this.set_content(args.try_get_object(0), activation.gc());
    // Changing the content invalidates every previously broken line.
    invalidate_all_lines(activation, this.first_line());
    Ok(Value::Undefined)
}

pub fn get_text_line_creation_result<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this
        .text_line_creation_result()
        .map(Value::from)
        .unwrap_or(Value::Null))
}

pub fn get_first_invalid_line<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    let mut current = this.first_line();
    while let Some(line) = current {
        if let Some(text_line) = line.as_display_object().and_then(|d| d.as_text_line())
            && TextLineValidity::parse(text_line.validity().as_wstr()) != TextLineValidity::Valid
        {
            return Ok(line.into());
        }
        current = line.get_slot(line_slots::_NEXT_LINE).as_object();
    }
    Ok(Value::Null)
}

pub fn get_first_line<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this.first_line().map(Value::from).unwrap_or(Value::Null))
}

pub fn get_last_line<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this.last_line().map(Value::from).unwrap_or(Value::Null))
}

/// `TextBlock.releaseLines(firstLine, lastLine)`: detach the given inclusive
/// range of lines from the block, relink the surrounding chain, and invalidate
/// every line that followed the released range.
pub fn release_lines<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let block = this.as_object().unwrap();
    let tbo = block.as_text_block_object().unwrap();

    // Both arguments must be non-null lines that belong to this block.
    let (Some(first), Some(last)) = (args.try_get_object(0), args.try_get_object(1)) else {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    };
    if first.get_slot(line_slots::_TEXT_BLOCK).as_object() != Some(block)
        || last.get_slot(line_slots::_TEXT_BLOCK).as_object() != Some(block)
    {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    }

    let before = first.get_slot(line_slots::_PREVIOUS_LINE).as_object();
    let after = last.get_slot(line_slots::_NEXT_LINE).as_object();

    // Sever the released [first..=last] range from the block.
    let invalid = AvmString::new_utf8(activation.gc(), "invalid");
    let mut current = Some(first);
    while let Some(line) = current {
        let next = line.get_slot(line_slots::_NEXT_LINE).as_object();
        line.set_slot(line_slots::_TEXT_BLOCK, Value::Null, activation)?;
        line.set_slot(line_slots::_NEXT_LINE, Value::Null, activation)?;
        line.set_slot(line_slots::_PREVIOUS_LINE, Value::Null, activation)?;
        if let Some(text_line) = line.as_display_object().and_then(|d| d.as_text_line())
            && TextLineValidity::parse(text_line.validity().as_wstr()) != TextLineValidity::Static
        {
            // A STATIC line (produced by a TextLineFactory) can't transition to
            // INVALID (#2008); leave it as-is.
            text_line.set_validity(invalid, activation.context);
        }
        if line == last {
            break;
        }
        current = next;
    }

    // Relink the surrounding chain and fix up the block's endpoints.
    if let Some(before) = before {
        before.set_slot(
            line_slots::_NEXT_LINE,
            after.map(Value::from).unwrap_or(Value::Null),
            activation,
        )?;
    }
    if let Some(after) = after {
        after.set_slot(
            line_slots::_PREVIOUS_LINE,
            before.map(Value::from).unwrap_or(Value::Null),
            activation,
        )?;
    }
    if tbo.first_line() == Some(first) {
        tbo.set_first_line(after, activation.gc());
    }
    if tbo.last_line() == Some(last) {
        tbo.set_last_line(before, activation.gc());
    }

    // Every line following the released range becomes invalid.
    invalidate_all_lines(activation, after);

    Ok(Value::Undefined)
}

/// The maximum value accepted for `createTextLine`'s `width` parameter,
/// mirroring `TextLine.MAX_LINE_WIDTH`.
const MAX_LINE_WIDTH: f64 = 1_000_000.0;

/// A run of same-formatted text gathered from the block's content elements.
struct SpanSpec {
    len: usize,
    format: TextFormat,
    is_device: bool,
}

/// Recursively flatten the content element tree into text plus format runs.
///
/// Group elements contribute their children in order; graphic elements
/// contribute a single U+FDEF placeholder character, like in Flash Player.
fn collect_spans<'gc>(element: Object<'gc>, text: &mut WString, specs: &mut Vec<SpanSpec>) {
    let Some(content_element) = element.as_content_element_object() else {
        return;
    };

    let class_name = element.instance_class().name().local_name();

    if class_name.as_wstr() == b"GroupElement" {
        let elements = element.get_slot(group_slots::_ELEMENTS);
        if let Some(vector_obj) = elements.as_object() {
            let children: Vec<Value<'gc>> = vector_obj
                .as_vector_storage()
                .map(|storage| storage.iter().collect())
                .unwrap_or_default();
            for child in children {
                if let Some(child) = child.as_object() {
                    collect_spans(child, text, specs);
                }
            }
        }
        return;
    }

    let (format, is_device) = text_format_for(content_element.element_format());

    if class_name.as_wstr() == b"GraphicElement" {
        // FP represents a graphic element as U+FDEF in the raw text.
        text.push(0xFDEF);
        specs.push(SpanSpec {
            len: 1,
            format,
            is_device,
        });
    } else if let Some(element_text) = content_element.text()
        && !element_text.is_empty()
    {
        text.push_str(element_text.as_wstr());
        specs.push(SpanSpec {
            len: element_text.len(),
            format,
            is_device,
        });
    }
}

/// Map an `ElementFormat` (and its `FontDescription`) to a `TextFormat`
/// usable by the core layout engine. The bool indicates device font lookup.
fn text_format_for(element_format: Option<ElementFormatObject<'_>>) -> (TextFormat, bool) {
    if let Some(ef) = element_format {
        let (font, bold, italic, is_device) = if let Some(fd) = ef.font_description() {
            (
                WString::from(fd.font_name().as_wstr()),
                fd.font_weight() == FontWeightValue::Bold,
                fd.font_posture() == FontPostureValue::Italic,
                fd.font_lookup() == FontLookupValue::Device,
            )
        } else {
            (WString::from_utf8("_serif"), false, false, true)
        };

        // TODO: Support more ElementFormat properties (alpha, baselineShift,
        // typographicCase, digitCase/digitWidth, ligatureLevel, breakOpportunity).
        let format = TextFormat {
            font: Some(font),
            size: Some(ef.font_size()),
            color: Some(ef.color()),
            bold: Some(bold),
            italic: Some(italic),
            kerning: Some(ef.kerning() != KerningValue::Off),
            letter_spacing: Some(ef.tracking_left() + ef.tracking_right()),
            ..TextFormat::default()
        };
        (format, is_device)
    } else {
        // FP throws an IllegalOperationError when breaking lines over an
        // element with a null ElementFormat; we're lenient and fall back
        // to a default device font instead.
        let format = TextFormat {
            font: Some(WString::from_utf8("_serif")),
            size: Some(12.0),
            ..TextFormat::default()
        };
        (format, true)
    }
}

fn is_break_char(c: u16) -> bool {
    matches!(c, 0x20 | 0x09 | 0x0A | 0x0D | 0x2028 | 0x2029)
}

fn is_newline_char(c: u16) -> bool {
    matches!(c, 0x0A | 0x0D | 0x2028 | 0x2029)
}

/// Mark every line of the chain starting at `start` as invalid and sever it
/// from the block. Used when re-breaking over existing lines.
fn sever_chain<'gc>(
    activation: &mut Activation<'_, 'gc>,
    start: Option<Object<'gc>>,
) -> Result<(), Error<'gc>> {
    let invalid = AvmString::new_utf8(activation.gc(), "invalid");
    let mut current = start;
    while let Some(line) = current {
        current = line.get_slot(line_slots::_NEXT_LINE).as_object();
        line.set_slot(line_slots::_NEXT_LINE, Value::Null, activation)?;
        line.set_slot(line_slots::_PREVIOUS_LINE, Value::Null, activation)?;
        line.set_slot(line_slots::_TEXT_BLOCK, Value::Null, activation)?;
        if let Some(text_line) = line
            .as_display_object()
            .and_then(|dobj| dobj.as_text_line())
        {
            text_line.set_validity(invalid, activation.context);
        }
    }
    Ok(())
}

/// Mark every still-valid line in the chain starting at `start` INVALID,
/// without severing it, as Flash does when the block's content or bidiLevel
/// changes. STATIC lines are left untouched (they no longer belong to the
/// block, and a STATIC -> INVALID transition would raise #2008).
fn invalidate_all_lines<'gc>(activation: &mut Activation<'_, 'gc>, start: Option<Object<'gc>>) {
    let invalid = AvmString::new_utf8(activation.gc(), "invalid");
    let mut current = start;
    while let Some(line) = current {
        current = line.get_slot(line_slots::_NEXT_LINE).as_object();
        if let Some(text_line) = line.as_display_object().and_then(|d| d.as_text_line()) {
            let validity = TextLineValidity::parse(text_line.validity().as_wstr());
            if matches!(
                validity,
                TextLineValidity::Valid | TextLineValidity::PossiblyInvalid
            ) {
                text_line.set_validity(invalid, activation.context);
            }
        }
    }
}

pub fn create_text_line<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let previous_line = args.try_get_object(0);
    let width = args.get_f64(1);
    // TODO: Support lineOffset (tab stop origin offset).
    let _line_offset = args.get_f64(2);
    let fit_something = args.get_bool(3);

    break_text_line(activation, this, None, previous_line, width, fit_something)
}

pub fn recreate_text_line<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let Some(text_line) = args.try_get_object(0) else {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    };
    let previous_line = args.try_get_object(1);
    let width = args.get_f64(2);
    // TODO: Support lineOffset (tab stop origin offset).
    let _line_offset = args.get_f64(3);
    let fit_something = args.get_bool(4);

    if previous_line == Some(text_line) {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    }

    break_text_line(
        activation,
        this,
        Some(text_line),
        previous_line,
        width,
        fit_something,
    )
}

/// The shared implementation of `createTextLine` and `recreateTextLine`:
/// break the next line out of the block's remaining text.
fn break_text_line<'gc>(
    activation: &mut Activation<'_, 'gc>,
    block: Object<'gc>,
    reuse: Option<Object<'gc>>,
    previous_line: Option<Object<'gc>>,
    width: f64,
    fit_something: bool,
) -> Result<Value<'gc>, Error<'gc>> {
    if width.is_nan() || width > MAX_LINE_WIDTH || (width < 0.0 && !fit_something) {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    }
    let width = width.max(0.0);
    let width_twips = Twips::from_pixels(width);

    // The previous line must belong to this block.
    if let Some(prev) = previous_line {
        let prev_block = prev.get_slot(line_slots::_TEXT_BLOCK);
        if prev_block.as_object() != Some(block) {
            return Err(make_error_2004(activation, Error2004Type::ArgumentError));
        }
    }

    let tbo = block
        .as_text_block_object()
        .expect("TextBlock is backed by a native TextBlockObject");

    let Some(content_obj) = tbo.content() else {
        return Ok(Value::Null);
    };

    // Flatten the content tree into text + format runs.
    let mut text = WString::new();
    let mut specs = Vec::new();
    collect_spans(content_obj, &mut text, &mut specs);

    let start = if let Some(prev) = previous_line {
        let begin = prev
            .get_slot(line_slots::_TEXT_BLOCK_BEGIN_INDEX)
            .coerce_to_i32(activation)? as usize;
        let raw_length = prev
            .get_slot(line_slots::_RAW_TEXT_LENGTH)
            .coerce_to_i32(activation)? as usize;
        begin + raw_length
    } else {
        0
    };

    if start >= text.len() {
        // All of the text has already been broken into lines.
        tbo.set_text_line_creation_result(Some(istr!("complete")), activation.gc());
        return Ok(Value::Null);
    }

    let remaining = &text[start..];

    // Build the format runs overlapping the remaining text, and find the
    // font lookup of the run the new line starts in.
    let mut remaining_spans: Vec<TextSpan> = Vec::new();
    let mut is_device = true;
    {
        let mut pos = 0;
        for spec in &specs {
            let span_start = pos;
            let span_end = pos + spec.len;
            pos = span_end;
            if span_end <= start {
                continue;
            }
            if remaining_spans.is_empty() {
                is_device = spec.is_device;
            }
            let overlap = span_end - span_start.max(start);
            remaining_spans.push(TextSpan::with_length_and_format(overlap, &spec.format));
        }
    }

    let font_type = if is_device {
        FontType::Device
    } else {
        FontType::EmbeddedCFF
    };

    let movie = activation.caller_movie_or_root();
    let format_spans = FormatSpans::from_str_and_spans(remaining, &remaining_spans);
    let layout = lower_from_text_spans(
        &format_spans,
        activation.context,
        movie.clone(),
        Some(width_twips),
        false,
        true,
        font_type,
        // This layout only locates the line break; justification is applied
        // later, on the fallback that renders the line.
        false,
    );

    let lines = layout.lines();
    let Some(first_line) = lines.first() else {
        tbo.set_text_line_creation_result(Some(istr!("complete")), activation.gc());
        return Ok(Value::Null);
    };

    // If not even a single atom fits in the requested width, FP returns
    // null and reports insufficientWidth (unless fitSomething is set).
    if !fit_something && first_line.len() <= 1 && first_line.bounds().width() > width_twips {
        tbo.set_text_line_creation_result(
            Some(AvmString::new_utf8(activation.gc(), "insufficientWidth")),
            activation.gc(),
        );
        return Ok(Value::Null);
    }

    // The new line consumes everything up to the start of the second laid
    // out line (this includes any trailing whitespace and line terminators).
    let consumed = if lines.len() >= 2 {
        lines[1].start()
    } else {
        remaining.len()
    };
    let consumed = consumed.clamp(1, remaining.len());

    // Decide whether this line is justified. TLF sets the paragraph's
    // justification on the block's `textJustifier` (a `SpaceJustifier` for
    // `textAlign="justify"`); its `lineJustification` is "all",
    // "allButLast"/"allButMandatoryBreak", or "unjustified"/absent. The last
    // line of the paragraph is the one that consumes the remaining text.
    let is_last_line = consumed >= remaining.len();
    let line_justification = tbo
        .text_justifier()
        .map(|j| j.get_slot(justifier_slots::_LINE_JUSTIFICATION))
        .map(|v| v.coerce_to_string(activation))
        .transpose()?;
    let should_justify = match line_justification {
        Some(ref s) => {
            let lj: &WStr = s;
            if lj == WStr::from_units(b"all") {
                true
            } else if lj == WStr::from_units(b"allButLast")
                || lj == WStr::from_units(b"allButMandatoryBreak")
            {
                !is_last_line
            } else {
                false
            }
        }
        None => false,
    };

    // An emergency break is a break in the middle of a word, forced by a
    // word too long for the requested width.
    let is_emergency = consumed < remaining.len()
        && !is_break_char(remaining.at(consumed - 1))
        && !is_break_char(remaining.at(consumed));

    // Line terminators are part of the line's raw text, but not displayed.
    let mut display_len = consumed;
    while display_len > 0 && is_newline_char(remaining.at(display_len - 1)) {
        display_len -= 1;
    }
    let line_text = &remaining[..display_len];

    // Re-breaking from the middle (or the start) of the block invalidates
    // all the lines that follow the break point.
    if let Some(prev) = previous_line {
        let stale = prev.get_slot(line_slots::_NEXT_LINE).as_object();
        sever_chain(activation, stale)?;
    } else {
        let stale = tbo.first_line();
        sever_chain(activation, stale)?;
    }

    // Create a new TextLine, or reuse the one passed to recreateTextLine.
    let (instance, text_line) = if let Some(reused) = reuse {
        let Some(text_line) = reused
            .as_display_object()
            .and_then(|dobj| dobj.as_text_line())
        else {
            return Err(make_error_2004(activation, Error2004Type::ArgumentError));
        };
        // TODO: FP also resets all DisplayObject properties (transform,
        // alpha, event listeners, children...) of the recycled line.
        (reused, text_line)
    } else {
        let class = activation.avm2().classes().textline;
        let fallback = EditText::new_fte(activation.context, movie.clone(), 0.0, 0.0, width, 15.0);
        let text_line = TextLine::new(activation.context, movie, fallback);
        let instance = initialize_for_allocator(activation.context, text_line.into(), class);
        (instance.into(), text_line)
    };

    let fallback = text_line.fallback();
    fallback.set_word_wrap(false, activation.context);
    fallback.set_is_device_font(activation.context, is_device);
    fallback.set_text(line_text, activation.context);

    // Apply the format runs covering this line.
    {
        let mut pos = 0;
        for spec in &specs {
            let span_start = pos;
            let span_end = pos + spec.len;
            pos = span_end;
            let seg_start = span_start.max(start);
            let seg_end = span_end.min(start + display_len);
            if seg_start >= seg_end {
                continue;
            }
            fallback.set_text_format(
                seg_start - start,
                seg_end - start,
                spec.format.clone(),
                activation.context,
            );
        }
    }

    // Justified paragraphs: switch the fallback to Justify alignment (which
    // fragments the line into per-word boxes) and force it to spread even
    // though the fallback holds only this one (otherwise "final") line.
    fallback.set_always_justify(should_justify, activation.context);
    if should_justify {
        fallback.set_text_format(
            0,
            display_len,
            TextFormat {
                align: Some(swf::TextAlign::Justify),
                ..Default::default()
            },
            activation.context,
        );
    }

    // Size the field to its contents and extract the line metrics. A justified
    // line fills the requested width; otherwise the field is sized to the
    // natural content width.
    let gutter = EditText::GUTTER.to_pixels();
    let (measured_width, measured_height) = fallback.measure_text(activation.context);
    let field_width = if should_justify {
        width_twips.to_pixels()
    } else {
        measured_width.to_pixels()
    };
    fallback.set_width(activation.context, field_width + gutter * 2.0);
    fallback.set_height(
        activation.context,
        measured_height.to_pixels() + gutter * 2.0,
    );

    let metrics = {
        let layout = fallback.layout();
        if let Some(line) = layout.lines().first() {
            let cell_ascent = line.ascent();
            let cell_descent = line.descent();
            // Report the typographic ascent/descent (OS/2 `sTypo*`) to Spark so
            // `TextLine` measurements match Flash Player's FTE; fall back to the
            // cell metrics when the font exposes no typographic ones. The glyphs
            // are still laid out and rendered against the cell metrics.
            let (ascent, descent) = line
                .typo_ascent_descent()
                .unwrap_or((cell_ascent, cell_descent));
            LineMetrics {
                ascent,
                descent,
                text_width: layout.text_size().width(),
                fallback_ascent: cell_ascent,
                fallback_descent: cell_descent,
            }
        } else {
            LineMetrics::default()
        }
    };
    text_line.set_metrics(metrics);
    text_line.set_validity(istr!("valid"), activation.context);

    instance.set_slot(line_slots::_TEXT_BLOCK, block.into(), activation)?;
    instance.set_slot(line_slots::_SPECIFIED_WIDTH, width.into(), activation)?;
    instance.set_slot(
        line_slots::_RAW_TEXT_LENGTH,
        Value::from_usize_lossy(consumed),
        activation,
    )?;
    instance.set_slot(
        line_slots::_TEXT_BLOCK_BEGIN_INDEX,
        Value::from_usize_lossy(start),
        activation,
    )?;
    instance.set_slot(line_slots::_NEXT_LINE, Value::Null, activation)?;

    // Wire the new line into the block's line chain.
    match previous_line {
        Some(prev) => {
            prev.set_slot(line_slots::_NEXT_LINE, instance.into(), activation)?;
            instance.set_slot(line_slots::_PREVIOUS_LINE, prev.into(), activation)?;
        }
        None => {
            instance.set_slot(line_slots::_PREVIOUS_LINE, Value::Null, activation)?;
            tbo.set_first_line(Some(instance), activation.gc());
        }
    }
    tbo.set_last_line(Some(instance), activation.gc());

    let result = if is_emergency {
        AvmString::new_utf8(activation.gc(), "emergency")
    } else {
        istr!("success")
    };
    tbo.set_text_line_creation_result(Some(result), activation.gc());

    Ok(instance.into())
}
