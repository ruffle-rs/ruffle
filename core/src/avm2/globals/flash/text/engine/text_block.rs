use crate::avm2::Avm2;
use crate::avm2::Avm2StrRepresentable;
use crate::avm2::activation::Activation;
use crate::avm2::error::{Error, Error2004Type, make_error_2004, make_error_2008, make_error_2175};
use crate::avm2::function::FunctionArgs;
use crate::avm2::globals::flash::display::display_object::initialize_for_allocator;
use crate::avm2::globals::slots::flash_text_engine_text_justifier as justifier_slots;
use crate::avm2::object::{ContentElementObject, ElementData, TObject, VectorObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2_stub_setter;
use crate::display_object::{DisplayObject, EditText, LineMetrics, TDisplayObject, TextLine};
use crate::font::FontType;
use crate::fte::{
    FontLookupValue, KerningValue, TextBaselineValue, TextLineCreationResultValue,
    TextLineValidity, TextRotationValue,
};
use crate::html::{FormatSpans, TextFormat, TextSpan, lower_from_text_spans};
use crate::string::{WStr, WString};
use swf::Twips;

pub use crate::avm2::object::text_block_allocator;

pub fn get_apply_non_linear_font_scaling<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this.apply_non_linear_font_scaling().into())
}

pub fn set_apply_non_linear_font_scaling<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();

    avm2_stub_setter!(
        activation,
        "flash.text.engine.TextBlock",
        "applyNonLinearFontScaling"
    );

    this.set_apply_non_linear_font_scaling(args.get_bool(0));

    Ok(Value::Undefined)
}

pub fn get_baseline_font_description<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
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
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();

    avm2_stub_setter!(
        activation,
        "flash.text.engine.TextBlock",
        "baselineFontDescription"
    );

    let value = args
        .try_get_object(0)
        .map(|v| v.as_font_description_object().unwrap());

    this.set_baseline_font_description(value, activation.gc());

    Ok(Value::Undefined)
}

pub fn get_baseline_font_size<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this.baseline_font_size().into())
}

pub fn set_baseline_font_size<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    let value = args.get_f64(0);
    if value < 0.0 {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    }

    avm2_stub_setter!(
        activation,
        "flash.text.engine.TextBlock",
        "baselineFontSize"
    );

    this.set_baseline_font_size(value);

    Ok(Value::Undefined)
}

pub fn get_baseline_zero<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this.baseline_zero().as_avm2_str(activation).into())
}

pub fn set_baseline_zero<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();

    let value = args.get_string_non_null(activation, 0, "baselineZero")?;
    let value = TextBaselineValue::from_avm2_str(&value)
        .filter(|v| !matches!(v, TextBaselineValue::UseDominantBaseline))
        .ok_or_else(|| make_error_2008(activation, "baselineZero"))?;

    avm2_stub_setter!(activation, "flash.text.engine.TextBlock", "baselineZero");

    this.set_baseline_zero(value);

    Ok(Value::Undefined)
}

pub fn get_bidi_level<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this.bidi_level().into())
}

pub fn set_bidi_level<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    let value = args.get_i32(0);
    if value < 0 {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    }

    avm2_stub_setter!(activation, "flash.text.engine.TextBlock", "bidiLevel");

    // As with assigning `content`, changing the bidi level invalidates the
    // previously broken lines.
    for line in this.lines() {
        line.set_validity(TextLineValidity::Invalid, activation.gc());
    }

    this.set_bidi_level(value);

    Ok(Value::Undefined)
}

pub fn get_line_rotation<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this.line_rotation().as_avm2_str(activation).into())
}

pub fn set_line_rotation<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();

    let value = args.get_string_non_null(activation, 0, "lineRotation")?;
    let value = TextRotationValue::from_avm2_str(&value)
        .filter(|v| !matches!(v, TextRotationValue::Auto))
        .ok_or_else(|| make_error_2008(activation, "lineRotation"))?;

    avm2_stub_setter!(activation, "flash.text.engine.TextBlock", "lineRotation");

    this.set_line_rotation(value);

    Ok(Value::Undefined)
}

pub fn get_tab_stops<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
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
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    let tab_stops = args
        .try_get_object(0)
        .and_then(|v| v.as_vector_object())
        .map(|v| VectorObject::from_vector(v.storage().clone(), activation));

    avm2_stub_setter!(activation, "flash.text.engine.TextBlock", "tabStops");

    this.set_tab_stops(tab_stops, activation.gc());

    Ok(Value::Undefined)
}

pub fn get_text_justifier<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
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
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();

    let justifier = args.get_object(activation, 0, "textJustifier")?;

    avm2_stub_setter!(activation, "flash.text.engine.TextBlock", "textJustifier");

    this.set_text_justifier(justifier, activation.gc());

    Ok(Value::Undefined)
}

pub fn get_content<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this.content().map(Value::from).unwrap_or(Value::Null))
}

pub fn set_content<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();

    let content = args
        .try_get_object(0)
        .map(|v| v.as_content_element_object().unwrap());

    // Flash forbids setting a `TextBlock`'s content to a user-defined subclass
    // of `ContentElement`
    let is_invalid = content.is_some_and(|c| {
        let data = c.element_data();
        matches!(&*data, ElementData::Invalid)
    });
    if is_invalid {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    }

    this.set_content(content, activation.gc());

    Ok(Value::Undefined)
}

pub fn get_text_line_creation_result<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this
        .text_line_creation_result()
        .map(|v| v.as_avm2_str(activation).into())
        .unwrap_or(Value::Null))
}

pub fn get_first_invalid_line<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();

    // The first line in the chain whose validity is not "valid".
    let line = this
        .lines()
        .find(|l| !matches!(l.validity(), TextLineValidity::Valid))
        .map(|l| l.object2().expect("Already created"))
        .map(Value::from);

    Ok(line.unwrap_or(Value::Null))
}

pub fn get_first_line<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();

    let line = this
        .first_line()
        .map(|l| l.object2().expect("Already created"))
        .map(Value::from);

    Ok(line.unwrap_or(Value::Null))
}

pub fn get_last_line<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();

    let line = this
        .lines()
        .last()
        .map(|l| l.object2().expect("Already created"))
        .map(Value::from);

    Ok(line.unwrap_or(Value::Null))
}

pub fn release_lines<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();

    let line_1 = args
        .get_object(activation, 0, "firstLine")?
        .as_display_object()
        .and_then(|o| o.as_text_line())
        .expect("Guaranteed by AS signature");

    let line_2 = args
        .get_object(activation, 1, "lastLine")?
        .as_display_object()
        .and_then(|o| o.as_text_line())
        .expect("Guaranteed by AS signature");

    // Flash has some unexpected behavior for certain edge cases of this method:

    // 1. Callers can swap the order of parameters to this method without ever
    //    affecting the functionality of the method.
    // 2. If two text lines are in the same text block and are siblings to each
    //    other, this method can still fail with an error if they can't be
    //    reached by iterating forward on the linked list of lines of this text
    //    block. It's possible to enter this scenario by rebreaking a line in
    //    the middle of a block.
    // 3. This method will always throw an error if either line is from a
    //    different text block.

    // The simplest way I could reproduce this behavior was with the following
    // logic:

    // 1. Start iterating over the linked list of lines, starting at
    //    `this.first_line()`.
    // 2. When reaching a line that is either `line_1` or `line_2` for the first
    //    time, record its position in the iterator.
    // 3. When reaching a line that is either `line_1` or `line_2` for the
    //    second time, record its position in the iterator and stop iterating.
    // 4. If either iteration reached the end of the list, return an error, as
    //    that means that one of the lines was not present in the block.
    // 5. Use the two positions to release the lines between `line_1` and
    //    `line_2` in the linked list of lines.

    let matches_either_line =
        |l| DisplayObject::ptr_eq(l, line_1) || DisplayObject::ptr_eq(l, line_2);

    let Some(first_position) = this.lines().position(matches_either_line) else {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    };

    if DisplayObject::ptr_eq(line_1, line_2) {
        // Special case: the two lines are the same. NOTE: We only do this after
        // we ensure that the line is actually reachable from `this.first_line`.
        line_1.release(activation.gc());

        return Ok(Value::Undefined);
    }

    let lines_count = this
        .lines()
        .skip(first_position + 1)
        .position(matches_either_line)
        .map(|p| p + 2);

    let Some(lines_count) = lines_count else {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    };

    let lines_to_remove = this
        .lines()
        .skip(first_position)
        .take(lines_count)
        .collect::<Vec<_>>();

    // `collect` the iterator, as we're modifying the doubly-linked list at the
    // same time as we iterate over it

    for line in lines_to_remove {
        line.release(activation.gc());
    }

    Ok(Value::Undefined)
}

/// The number of pixels the requested line width may not exceed,
/// mirroring `TextLine.MAX_LINE_WIDTH`.
const MAX_LINE_WIDTH: f64 = 1_000_000.0;

pub fn do_create_text_line<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this_obj = this.as_object().unwrap();
    let block = this_obj.as_text_block_object().unwrap();

    let line_to_use = args
        .try_get_object(0)
        .and_then(|o| o.as_display_object())
        .and_then(|o| o.as_text_line());

    let previous_text_line = args
        .try_get_object(1)
        .and_then(|o| o.as_display_object())
        .and_then(|o| o.as_text_line());

    let width = args.get_f64(2);
    // TODO: Support lineOffset (tab stop origin offset).
    let _line_offset = args.get_f64(3);
    let fit_something = args.get_bool(4);

    if width.is_nan() || width > MAX_LINE_WIDTH || (width < 0.0 && !fit_something) {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    }
    let width = width.max(0.0);
    let width_twips = Twips::from_pixels(width);

    let content = block.content().expect("Guaranteed by AS checks");

    // Flatten the content tree into text plus format runs.
    let mut text = WString::new();
    let mut specs = Vec::new();
    if let Err(HandleContentError::NullElementFormat) =
        collect_spans(content, &mut text, &mut specs)
    {
        // For some reason, FP handles this error as it handles an uncaught
        // exception, and returns `null` from this method.
        let error = make_error_2175(activation);

        Avm2::uncaught_error(activation, None, error, "Error creating TextLine");

        return Ok(Value::Null);
    }

    let start = if let Some(previous_text_line) = previous_text_line {
        previous_text_line.end_index() as usize
    } else {
        0
    };

    if start >= text.len() {
        // All of the text has already been broken into lines. This also covers
        // the content shrinking after lines were created.
        block.set_text_line_creation_result(Some(TextLineCreationResultValue::Complete));
        return Ok(Value::Null);
    }

    // A line never extends past the next mandatory break: FP ends a paragraph
    // at LF, CR (merging a CR LF pair), U+2028 or U+2029. `hard_end` is the end
    // of this paragraph (terminator included); `content_len` drops the trailing
    // terminator, which is not displayed but still counts toward the line's
    // rawTextLength. The width-aware layout runs over the content only, so the
    // terminator is consumed by whichever line completes the paragraph.
    let hard_end = next_line_break(&text[start..], 0);
    let mut content_len = hard_end;
    while content_len > 0 && is_newline_char(text.at(start + content_len - 1)) {
        content_len -= 1;
    }
    let content = &text[start..start + content_len];

    // Build the format runs overlapping this paragraph's content, and find the
    // font lookup of the run the new line starts in.
    let mut content_spans: Vec<TextSpan> = Vec::new();
    let mut is_device = true;
    {
        let mut pos = 0;
        for spec in &specs {
            let span_start = pos;
            let span_end = pos + spec.len;
            pos = span_end;
            if span_end <= start || span_start >= start + content_len {
                continue;
            }
            if content_spans.is_empty() {
                is_device = spec.is_device;
            }
            let overlap = span_end.min(start + content_len) - span_start.max(start);
            content_spans.push(TextSpan::with_length_and_format(overlap, &spec.format));
        }
    }

    let font_type = if is_device {
        FontType::Device
    } else {
        FontType::EmbeddedCFF
    };

    // NOTE: Do not use `caller_movie_or_root()` here: `createTextLine` is
    // called through the AS wrapper in playerglobal, so the caller movie is
    // playerglobal itself, which is not flagged AS3 — a `TextLine` created
    // with it gets rejected by `addChild` with error #2180.
    let movie = activation.context.root_swf.clone();

    // Lay the content out at the requested width to find the soft wrap point,
    // if any. `is_last_line` is true when this line completes the paragraph
    // (nothing wrapped past it), in which case it also swallows the terminator.
    let (consumed, display_len, is_last_line, is_emergency) = if content_len == 0 {
        // Empty paragraph (a bare terminator).
        (hard_end, 0, true, false)
    } else {
        let format_spans = FormatSpans::from_str_and_spans(content, &content_spans);
        let layout = lower_from_text_spans(
            &format_spans,
            activation.context,
            movie,
            Some(width_twips),
            false,
            true,
            font_type,
            // This layout only locates the line break; justification is applied
            // later, on the fallback that renders the line.
            false,
        );
        let lines = layout.lines();

        // If not even a single atom fits in the requested width, FP returns
        // null and reports insufficientWidth (unless fitSomething is set).
        if let Some(first_line) = lines.first()
            && !fit_something
            && first_line.len() <= 1
            && first_line.bounds().width() > width_twips
        {
            block.set_text_line_creation_result(Some(
                TextLineCreationResultValue::InsufficientWidth,
            ));
            return Ok(Value::Null);
        }

        if lines.len() >= 2 {
            // Soft wrap inside the content: this line ends at the wrap point and
            // does not reach the terminator; the rest goes to later lines.
            let c = lines[1].start().clamp(1, content_len);
            let emergency = c < content_len
                && !is_break_char(content.at(c - 1))
                && !is_break_char(content.at(c));
            (c, c, false, emergency)
        } else {
            // The whole paragraph fits on one line: consume the terminator too.
            (hard_end, content_len, true, false)
        }
    };

    let line_text = &text[start..start + display_len];

    // Decide whether this line is justified. TLF sets the paragraph's
    // justification on the block's `textJustifier` (a `SpaceJustifier` for
    // `textAlign="justify"`); its `lineJustification` is "all",
    // "allButLast"/"allButMandatoryBreak", or "unjustified"/absent.
    let line_justification = block
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

    // NOTE: Re-breaking from the middle (or the start) of the block does NOT
    // invalidate or unlink the stale lines that follow the break point: FP
    // leaves their validity, their block link and their sibling links exactly
    // as they were (only `prev.nextLine` is repointed to the new line below).

    // `do_create_text_line` is called from both `TextBlock.recreateTextLine`
    // and `TextBlock.createTextLine`. The former passes this method an existing
    // `TextLine` to reuse, while the latter expects this method to create a new
    // `TextLine`.
    let text_line = if let Some(line) = line_to_use {
        // `TextLine.recreateTextLine` is the caller: completely reset the
        // properties of the passed line and use it.
        line.reset_properties(activation.gc());
        line
    } else {
        // `TextLine.createTextLine` is the caller: create a new `TextLine`.
        create_text_line(activation, width)
    };

    let text_line_instance = text_line.object2().expect("Already created the object2");
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
            // Flash Player reports typographic (OS/2 sTypo) metrics for FTE
            // lines; fall back to the hhea/cell metrics when the font provides
            // none. Glyph placement always uses the cell metrics.
            let (ascent, descent) = line
                .typo_ascent_descent()
                .unwrap_or((cell_ascent, cell_descent));
            LineMetrics {
                ascent,
                descent,
                fallback_ascent: cell_ascent,
                fallback_descent: cell_descent,
                text_width: layout.text_size().width(),
            }
        } else {
            LineMetrics::default()
        }
    };
    text_line.set_metrics(metrics);
    text_line.set_validity(TextLineValidity::Valid, activation.gc());

    text_line.set_text_block(Some(block), activation.gc());
    text_line.set_specified_width(width);
    text_line.set_raw_text_length(consumed as u32);
    text_line.set_begin_index(start as u32);
    text_line.set_end_index((start + consumed) as u32);

    // Wire the new line into the block's line chain.
    if let Some(previous_line) = previous_text_line {
        text_line.set_line_index(previous_line.line_index() + 1);
        text_line.set_previous_line(Some(previous_line), activation.gc());
        text_line.set_next_line(None, activation.gc());
        previous_line.set_next_line(Some(text_line), activation.gc());
    } else {
        text_line.set_line_index(0);
        text_line.set_previous_line(None, activation.gc());
        text_line.set_next_line(None, activation.gc());
        // If there's no previous line, then this is the first line.
        block.set_first_line(Some(text_line), activation.gc());
    }

    let result = if is_emergency {
        TextLineCreationResultValue::Emergency
    } else {
        TextLineCreationResultValue::Success
    };
    block.set_text_line_creation_result(Some(result));

    Ok(text_line_instance.into())
}

fn create_text_line<'gc>(activation: &mut Activation<'_, 'gc>, width: f64) -> TextLine<'gc> {
    let class = activation.avm2().classes().textline;

    // See the NOTE in `do_create_text_line` about the movie choice (#2180).
    let movie = activation.context.root_swf.clone();

    let fallback = EditText::new_fte(activation.context, movie.clone(), 0.0, 0.0, width, 15.0);
    let text_line = TextLine::new(activation.context, movie, fallback);
    initialize_for_allocator(activation.context, text_line.into(), class);

    text_line
}

enum HandleContentError {
    NullElementFormat,
}

/// A run of text sharing one format, produced by flattening the content
/// element tree.
struct SpanSpec {
    len: usize,
    format: TextFormat,
    is_device: bool,
}

/// Flatten the content element tree into raw text plus format runs.
/// `GroupElement` recurses over its children, and `GraphicElement`
/// contributes a U+FDEF placeholder, as in Flash Player.
fn collect_spans<'gc>(
    content: ContentElementObject<'gc>,
    text: &mut WString,
    specs: &mut Vec<SpanSpec>,
) -> Result<(), HandleContentError> {
    let data = content.element_data();

    match &*data {
        ElementData::Text {
            text: element_text, ..
        } => {
            // If `text` is `None`, FP just completely ignores the element. It
            // doesn't even check its `elementFormat`.
            if let Some(element_text) = element_text {
                let (format, is_device) = text_format_for(content)?;
                text.push_str(element_text);
                specs.push(SpanSpec {
                    len: element_text.len(),
                    format,
                    is_device,
                });
            }
        }
        ElementData::Group { elements } => {
            // TODO: The docs say GroupElement's format has some effects?
            for element in elements {
                collect_spans(*element, text, specs)?;
            }
        }
        ElementData::Graphic => {
            // FP represents a graphic element as U+FDEF in the raw text.
            let (format, is_device) = text_format_for(content)?;
            text.push(0xFDEF);
            specs.push(SpanSpec {
                len: 1,
                format,
                is_device,
            });
        }
        ElementData::Invalid => {
            unreachable!(
                "TextBlock and GroupElement prevent holding user subclasses of ContentElement"
            )
        }
    }

    Ok(())
}

/// Map an `ElementFormat` (and its `FontDescription`) to a `TextFormat`
/// usable by the core layout engine. The bool indicates device font lookup.
fn text_format_for<'gc>(
    content: ContentElementObject<'gc>,
) -> Result<(TextFormat, bool), HandleContentError> {
    let ef = content
        .element_format()
        .ok_or(HandleContentError::NullElementFormat)?;
    let fd = ef.font_description();

    // TODO: Support more ElementFormat properties (alpha, baselineShift,
    // typographicCase, digitCase/digitWidth, ligatureLevel, breakOpportunity).
    let format = TextFormat {
        kerning: Some(ef.kerning() != KerningValue::Off),
        letter_spacing: Some(ef.tracking_left() + ef.tracking_right()),
        ..ef.as_text_format()
    };
    let is_device = fd.font_lookup() == FontLookupValue::Device;

    Ok((format, is_device))
}

fn is_break_char(c: u16) -> bool {
    matches!(c, 0x20 | 0x09 | 0x0A | 0x0D | 0x2028 | 0x2029)
}

fn is_newline_char(c: u16) -> bool {
    matches!(c, 0x0A | 0x0D | 0x2028 | 0x2029)
}

/// The end (exclusive) of the line starting at `start`: right after the next
/// mandatory line terminator (LF, CR — merging a CR LF pair — U+2028 or
/// U+2029), or the end of the text.
fn next_line_break(text: &WStr, start: usize) -> usize {
    let remaining_text = &text[start..];
    let len = remaining_text
        .iter()
        .position(is_newline_char)
        // Include the separator.
        .map(|pos| {
            if remaining_text.get(pos) == Some(0x0D) && remaining_text.get(pos + 1) == Some(0x0A) {
                pos + 2
            } else {
                pos + 1
            }
        });

    if let Some(len) = len {
        start + len
    } else {
        text.len()
    }
}
