//! Natives for flash.text.engine.TextBlock.
//!
//! createTextLine / recreateTextLine lay out the next line of text with
//! Ruffle's shared layout engine (crate::html::lower_from_text_spans, which
//! also backs EditText) and bind the result to a new (or existing)
//! FteTextLine DisplayObject backing the AS3 TextLine instance.

use ruffle_macros::istr;

use crate::avm2::activation::Activation;
use crate::avm2::error::{Error, make_error_2175};
use crate::avm2::function::FunctionArgs;
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
use crate::display_object::{FteLine, FteTextLine};
use crate::font::FontType;
use crate::html::{FormatSpans, LayoutLine, TextFormat, lower_from_text_spans};
use crate::string::WString;
use swf::Twips;

/// Shared logic for determining the start position and remaining text for a text line.
fn resolve_text_content<'gc>(
    activation: &mut Activation<'_, 'gc>,
    text_block: Object<'gc>,
    previous_text_line: Option<Object<'gc>>,
) -> Result<Option<(usize, crate::string::AvmString<'gc>, Object<'gc>)>, Error<'gc>> {
    let content = text_block.get_slot(block_slots::_CONTENT);

    // An empty TextBlock (null content) produces no line; Flash Player
    // returns null here rather than throwing.
    if matches!(content, Value::Null) {
        return Ok(None);
    }

    let full_text = {
        let txt = content
            .call_method(element_methods::GET_TEXT, &[], activation)
            .unwrap_or_else(|_| istr!("").into());

        if matches!(txt, Value::Null) {
            return Ok(None);
        }
        txt.coerce_to_string(activation)
            .expect("Guaranteed by AS bindings")
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

/// Resolve a ContentElement's ElementFormat into a [TextFormat], plus its
/// trackingLeft and trackingRight returned separately: they are folded into
/// letter_spacing for layout, but the line edges need the split to trim the
/// outer tracking (see [crate::html::LayoutLine::trim_edge_tracking]).
fn format_from_content<'gc>(
    activation: &mut Activation<'_, 'gc>,
    content: Object<'gc>,
) -> Result<(TextFormat, f64, f64), Error<'gc>> {
    let mut format = TextFormat {
        font: Some(WString::from_utf8("_sans")),
        size: Some(12.0),
        color: Some(swf::Color::from_rgb(0, 0xff)),
        ..Default::default()
    };

    // Flash Player throws Error #2175 when a content element carries a null
    // ElementFormat; createTextLine never reaches layout. Confirmed by
    // running the case in the Flash Player debugger, which logs the thrown
    // #2175. There is no automated test: asserting the throw needs a SWF
    // that catches the error and traces it, and a try/catch around
    // createTextLine does not catch #2175 in Flash Player (the error stays
    // uncaught), so it cannot be turned into a comparable trace line.
    let Some(ef) = content.get_slot(element_slots::_ELEMENT_FORMAT).as_object() else {
        return Err(make_error_2175(activation));
    };

    let color = ef
        .get_slot(format_slots::_COLOR)
        .coerce_to_u32(activation)?;
    format.color = Some(swf::Color::from_rgb(color & 0xff_ffff, 0xff));

    let size = ef
        .get_slot(format_slots::_FONT_SIZE)
        .coerce_to_number(activation)?;
    format.size = Some(size);

    // FTE tracks both sides of every glyph; the layout engine has a single
    // letter_spacing, so fold the two sides together.
    let tracking_left = ef
        .get_slot(format_slots::_TRACKING_LEFT)
        .coerce_to_number(activation)?;
    let tracking_right = ef
        .get_slot(format_slots::_TRACKING_RIGHT)
        .coerce_to_number(activation)?;
    format.letter_spacing = Some(tracking_left + tracking_right);

    // FTE kerning is "on" / "off" / "auto"; the layout engine takes a bool.
    let kerning = ef
        .get_slot(format_slots::_KERNING)
        .coerce_to_string(activation)?;
    format.kerning = Some(kerning.to_utf8_lossy() != "off");

    // baselineShift is a pixel offset (positive shifts the run down),
    // applied at render time without affecting line metrics. The
    // "superscript"/"subscript" string forms coerce to NaN and are skipped.
    let baseline_shift = ef
        .get_slot(format_slots::_BASELINE_SHIFT)
        .coerce_to_number(activation)?;
    format.baseline_shift = baseline_shift.is_finite().then_some(baseline_shift);

    if let Value::Object(fd) = ef.get_slot(format_slots::_FONT_DESCRIPTION) {
        let name = fd
            .get_slot(font_desc_slots::_FONT_NAME)
            .coerce_to_string(activation)?;
        format.font = Some(WString::from(name.as_wstr()));

        // FontWeight ("normal"/"bold") and FontPosture ("normal"/"italic")
        // pick the bold/italic device-font variants.
        let weight = fd
            .get_slot(font_desc_slots::_FONT_WEIGHT)
            .coerce_to_string(activation)?;
        format.bold = Some(weight.to_utf8_lossy() == "bold");

        let posture = fd
            .get_slot(font_desc_slots::_FONT_POSTURE)
            .coerce_to_string(activation)?;
        format.italic = Some(posture.to_utf8_lossy() == "italic");
    }

    Ok((format, tracking_left, tracking_right))
}

/// Collect the styled leaf runs of a ContentElement tree in document
/// order. A GroupElement contributes its children's runs; any other
/// element contributes one run of its own text and ElementFormat.
fn collect_runs<'gc>(
    activation: &mut Activation<'_, 'gc>,
    content: Object<'gc>,
    out: &mut Vec<(WString, TextFormat, f64, f64)>,
) -> Result<(), Error<'gc>> {
    let is_group = content.instance_class().name().local_name().to_utf8_lossy() == "GroupElement";

    if is_group {
        let count_name = crate::string::AvmString::new_utf8(activation.gc(), "elementCount");
        let get_at_name = crate::string::AvmString::new_utf8(activation.gc(), "getElementAt");
        let count = Value::from(content)
            .get_public_property(count_name, activation)?
            .coerce_to_i32(activation)?;
        for i in 0..count {
            let child = Value::from(content).call_public_property(
                get_at_name,
                FunctionArgs::from_slice(&[Value::from(i)]),
                activation,
            )?;
            if let Some(child) = child.as_object() {
                collect_runs(activation, child, out)?;
            }
        }
    } else {
        let text = Value::from(content)
            .call_method(element_methods::GET_TEXT, &[], activation)
            .unwrap_or_else(|_| istr!("").into())
            .coerce_to_string(activation)?;
        let (format, tracking_left, tracking_right) = format_from_content(activation, content)?;
        let mut run_text = WString::from(text.as_wstr());

        // typographicCase uppercase/lowercase is a real case transform;
        // caps/smallCaps/capsAndSmallCaps are OpenType features a device
        // font lacks, so they are left alone. Only apply the transform when
        // it preserves UTF-16 length, keeping block char offsets aligned.
        if let Some(ef) = content.get_slot(element_slots::_ELEMENT_FORMAT).as_object() {
            let tc = ef
                .get_slot(format_slots::_TYPOGRAPHIC_CASE)
                .coerce_to_string(activation)?;
            let transformed = match tc.to_utf8_lossy().as_ref() {
                "uppercase" => Some(run_text.to_utf8_lossy().to_uppercase()),
                "lowercase" => Some(run_text.to_utf8_lossy().to_lowercase()),
                _ => None,
            };
            if let Some(transformed) = transformed {
                let transformed = WString::from_utf8(&transformed);
                if transformed.len() == run_text.len() {
                    run_text = transformed;
                }
            }
        }
        out.push((run_text, format, tracking_left, tracking_right));
    }
    Ok(())
}

/// Lay out the text starting at start of full_text and return the first
/// line, plus the text the layout was built from.
fn lay_out_first_line<'gc>(
    activation: &mut Activation<'_, 'gc>,
    full_text: crate::string::AvmString<'gc>,
    start: usize,
    content: Object<'gc>,
    width: f64,
) -> Result<Option<(LayoutLine<'gc>, WString, usize)>, Error<'gc>> {
    // A TextLine ends at the first hard break, U+2028 (line separator) or
    // U+2029 (paragraph separator); the shared engine does not break on
    // either, so clamp the line here. The break char is kept as the line's
    // last char, the way Flash counts it in rawTextLength.
    let tail = &full_text[start..];
    let para_len = match tail.iter().position(|u| u == 0x2028 || u == 0x2029) {
        Some(pos) => pos + 1,
        None => tail.len(),
    };
    let line_end = start + para_len;

    // Collect the styled runs of the content tree. A GroupElement gives each
    // child run its own ElementFormat (size/color/font), and a run may carry
    // case-transformed text, so the line is assembled from the runs rather
    // than sliced straight out of the block.
    let mut runs: Vec<(WString, TextFormat, f64, f64)> = Vec::new();
    collect_runs(activation, content, &mut runs)?;

    // Assemble this line's text and per-run format spans. Each run covers
    // [run_off, run_off + len) of the block; clip it to [start, line_end).
    // run_spans also carries each run's tracking, so the line's outer edge
    // tracking can be trimmed once the line extent is known.
    let mut remaining = WString::new();
    let mut run_spans: Vec<(usize, usize, &TextFormat, f64, f64)> = Vec::new();
    let mut run_off = 0usize;
    for (run_text, run_fmt, tracking_left, tracking_right) in &runs {
        let run_start = run_off;
        run_off += run_text.len();
        let lo = run_start.max(start);
        let hi = run_off.min(line_end);
        if lo < hi {
            run_spans.push((
                lo - start,
                hi - start,
                run_fmt,
                *tracking_left,
                *tracking_right,
            ));
            remaining.push_str(&run_text[lo - run_start..hi - run_start]);
        }
    }
    if runs.is_empty() {
        remaining = WString::from(&tail[..para_len]);
    }

    let base = match runs.first() {
        Some((_, fmt, ..)) => fmt.clone(),
        None => format_from_content(activation, content)?.0,
    };
    let mut spans = FormatSpans::from_text(remaining.clone(), base);
    for (from, to, fmt, ..) in &run_spans {
        spans.set_text_format(*from, *to, fmt);
    }

    // TextLine.MAX_LINE_WIDTH (1000000) means "unbounded".
    let requested_width = if width >= 1_000_000.0 {
        None
    } else {
        Some(Twips::from_pixels(width))
    };

    let movie = activation.caller_movie_or_root();
    let layout = lower_from_text_spans(
        &spans,
        activation.context,
        movie,
        requested_width,
        false,
        true,
        FontType::Device,
        // flash.text.engine kerns device fonts; ElementFormat.kerning applies
        // to them. Classic TextField does not.
        true,
    );

    let Some(mut first) = layout.lines().first().cloned() else {
        return Ok(None);
    };

    // Flash applies trackingLeft and trackingRight only to the interior gaps
    // of a line. The leading tracking before the first glyph and the trailing
    // tracking after the last glyph are excluded. The layout engine spaced
    // every glyph uniformly, so trim those two edges using the tracking of
    // the runs the line begins and ends in. Run spans never overlap, so each
    // edge matches at most one run.
    let (line_lo, line_hi) = (first.start(), first.end());
    let mut leading = 0.0;
    let mut trailing = 0.0;
    for &(from, to, _, tracking_left, tracking_right) in &run_spans {
        if from <= line_lo && line_lo < to {
            leading = tracking_left;
        }
        if from < line_hi && line_hi <= to {
            trailing = tracking_right;
        }
    }
    first.trim_edge_tracking(Twips::from_pixels(leading), Twips::from_pixels(trailing));

    Ok(Some((first, remaining, start)))
}

pub fn create_text_line<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let previous_text_line = args.try_get_object(0);
    let width = args.get_f64(1);

    let Some((start, full_text, content)) =
        resolve_text_content(activation, this, previous_text_line)?
    else {
        return Ok(Value::Null);
    };

    let Some((html_line, text, begin)) =
        lay_out_first_line(activation, full_text, start, content, width)?
    else {
        this.set_slot(
            block_slots::_TEXT_LINE_CREATION_RESULT,
            istr!("complete").into(),
            activation,
        )?;
        return Ok(Value::Null);
    };

    let fte_line = FteLine::new(html_line, text, begin);
    let raw_text_length = fte_line.raw_text_length();
    let movie = activation.caller_movie_or_root();

    let fte = FteTextLine::new(activation.context, movie, fte_line);
    let class = activation.avm2().classes().textline;
    let instance = initialize_for_allocator(activation.context, fte.into(), class);

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

pub fn recreate_text_line<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let text_line = args.get_object(activation, 0, "textLine")?;
    let previous_text_line = args.try_get_object(1);
    let raw_width = args.get_f64(2);

    let width = if raw_width >= 1_000_000.0 {
        text_line
            .get_slot(line_slots::_SPECIFIED_WIDTH)
            .coerce_to_number(activation)
            .unwrap_or(raw_width)
    } else {
        raw_width
    };

    let Some((start, full_text, content)) =
        resolve_text_content(activation, this, previous_text_line)?
    else {
        return Ok(text_line.into());
    };

    let Some((html_line, text, begin)) =
        lay_out_first_line(activation, full_text, start, content, width)?
    else {
        return Ok(text_line.into());
    };

    let fte_line = FteLine::new(html_line, text, begin);
    let raw_text_length = fte_line.raw_text_length();

    if let Some(fte) = text_line
        .as_display_object()
        .and_then(|d| d.as_fte_text_line())
    {
        fte.set_line(activation.context, fte_line);
    }

    text_line.set_slot(line_slots::_TEXT_BLOCK, this.into(), activation)?;
    text_line.set_slot(line_slots::_SPECIFIED_WIDTH, width.into(), activation)?;
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
    // Recreated lines must be valid. TLF (TextFlowLine.getTextLine in the
    // shipped SWZ) bails to a null return path when validity is "invalid",
    // which releaseLines would have set.
    {
        let valid_str = crate::string::AvmString::new_utf8(activation.gc(), "valid");
        text_line.set_slot(line_slots::_VALIDITY, valid_str.into(), activation)?;
    }

    this.set_slot(
        block_slots::_TEXT_LINE_CREATION_RESULT,
        istr!("success").into(),
        activation,
    )?;

    // Re-link the recreated line's previousLine. Leave nextLine alone:
    // recreate may target a middle line, so its forward link must not be
    // cleared here.
    if let Some(prev) = previous_text_line {
        prev.set_slot(line_slots::_NEXT_LINE, text_line.into(), activation)?;
        text_line.set_slot(line_slots::_PREVIOUS_LINE, prev.into(), activation)?;
    } else {
        text_line.set_slot(line_slots::_PREVIOUS_LINE, Value::Null, activation)?;
        this.set_slot(block_slots::_FIRST_LINE, text_line.into(), activation)?;
    }
    let current_last = this.get_slot(block_slots::_LAST_LINE).as_object();
    let should_set_last = match (previous_text_line, current_last) {
        (Some(prev), Some(last)) => prev.as_ptr() == last.as_ptr(),
        (None, None) => true,
        _ => false,
    };
    if should_set_last {
        this.set_slot(block_slots::_LAST_LINE, text_line.into(), activation)?;
    }

    Ok(text_line.into())
}
