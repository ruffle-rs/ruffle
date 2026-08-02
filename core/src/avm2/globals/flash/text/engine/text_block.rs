use crate::avm2::Avm2;
use crate::avm2::Avm2StrRepresentable;
use crate::avm2::activation::Activation;
use crate::avm2::error::{Error, Error2004Type, make_error_2004, make_error_2008, make_error_2175};
use crate::avm2::globals::flash::display::display_object::initialize_for_allocator;
use crate::avm2::object::{ContentElementObject, ElementData, VectorObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::display_object::{EditText, TDisplayObject, TextLine};
use crate::fte::{TextBaselineValue, TextLineCreationResultValue, TextRotationValue};
use crate::html::{FormatSpans, TextFormat, TextSpan};
use crate::string::{WStr, WString};
use crate::{avm2_stub_getter, avm2_stub_setter};

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
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
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

    avm2_stub_setter!(activation, "flash.text.engine.TextBlock", "baselineZero");

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

    avm2_stub_setter!(activation, "flash.text.engine.TextBlock", "bidiLevel");

    this.set_bidi_level(value);

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

    avm2_stub_setter!(activation, "flash.text.engine.TextBlock", "lineRotation");

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

    avm2_stub_setter!(activation, "flash.text.engine.TextBlock", "tabStops");

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

    let justifier = args.get_object(activation, 0, "textJustifier")?;

    avm2_stub_setter!(activation, "flash.text.engine.TextBlock", "textJustifier");

    this.set_text_justifier(justifier, activation.gc());

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
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this
        .text_line_creation_result()
        .map(|v| v.as_avm2_str(activation).into())
        .unwrap_or(Value::Null))
}

pub fn get_first_invalid_line<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(
        activation,
        "flash.text.engine.TextBlock",
        "firstInvalidLine"
    );

    Ok(Value::Null)
}

pub fn get_first_line<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();

    let line = this
        .first_line()
        .map(|l| l.object2().expect("Already created"))
        .map(Value::from);

    Ok(line.unwrap_or(Value::Null))
}

pub fn do_create_text_line<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
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

    let content = block.content().expect("Guaranteed by AS checks");

    let previous_position = if let Some(previous_text_line) = previous_text_line {
        previous_text_line.end_index() as usize
    } else {
        0
    };

    let spans = match get_spans_from_content(content) {
        Ok(spans) => spans,
        Err(HandleContentError::NullElementFormat) => {
            // For some reason, FP handles this error as it handles an uncaught
            // exception, and returns `null` from this method.
            let error = make_error_2175(activation);

            Avm2::uncaught_error(activation, None, error, "Error creating TextLine");

            return Ok(Value::Null);
        }
    };

    // TODO don't discard the formatting
    let text = spans.text();

    if previous_position > text.len() {
        // This can happen when the content is changed after creating a TextLine.
        block.set_text_line_creation_result(Some(TextLineCreationResultValue::Complete));
        return Ok(Value::Null);
    }

    let next_position = next_line_break(text, previous_position);

    if text.is_empty() || next_position == text.len() && previous_position == next_position {
        // No more text.
        block.set_text_line_creation_result(Some(TextLineCreationResultValue::Complete));
        return Ok(Value::Null);
    }

    let line_index = if let Some(previous_text_line) = previous_text_line {
        previous_text_line.line_index() + 1
    } else {
        0
    };

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

    let subtext = &text[previous_position..next_position];

    fallback.set_text(subtext, activation.context);

    let format = spans
        .span(0)
        .expect("Non-empty text should have at least one span")
        .get_text_format();

    apply_format(activation, fallback, format, line_index);

    // FIXME: This needs to use `intrinsic_bounds` to measure the width
    // of the provided text, and set the width of the EditText to that.
    // Some games depend on this (e.g. Realm Grinder).

    text_line.set_text_block(Some(block), activation.gc());
    text_line.set_specified_width(width);
    text_line.set_raw_text_length(text.len() as u32);
    text_line.set_begin_index(previous_position as u32);
    text_line.set_end_index(next_position as u32);
    text_line.set_line_index(line_index);

    if let Some(previous_line) = previous_text_line {
        text_line.set_previous_line(Some(previous_line), activation.gc());
        previous_line.set_next_line(Some(text_line), activation.gc());
    } else {
        // If there's no previous line, then this is the first line.
        block.set_first_line(Some(text_line), activation.gc());
    }

    block.set_text_line_creation_result(Some(TextLineCreationResultValue::Success));

    Ok(text_line_instance.into())
}

fn create_text_line<'gc>(activation: &mut Activation<'_, 'gc>, width: f64) -> TextLine<'gc> {
    let class = activation.avm2().classes().textline;

    // TODO should we use the caller's movie instead? Does it matter?
    let movie = activation.context.root_swf.clone();

    let fallback = EditText::new_fte(activation.context, movie.clone(), 0.0, 0.0, width, 15.0);
    let text_line = TextLine::new(activation.context, movie, fallback);
    initialize_for_allocator(activation.context, text_line.into(), class);

    text_line
}

enum HandleContentError {
    NullElementFormat,
}

fn get_spans_from_content<'gc>(
    content: ContentElementObject<'gc>,
) -> Result<FormatSpans, HandleContentError> {
    let mut all_text = WString::new();
    let mut all_spans = Vec::new();

    handle_content_element(content, &mut all_text, &mut all_spans)?;

    let mut spans = FormatSpans::from_str_and_spans(&all_text, &all_spans);
    spans.normalize();
    Ok(spans)
}

fn handle_content_element<'gc>(
    content: ContentElementObject<'gc>,
    all_text: &mut WString,
    all_spans: &mut Vec<TextSpan>,
) -> Result<(), HandleContentError> {
    let data = content.element_data();
    let format = content.element_format();

    match &*data {
        ElementData::Text { text } => {
            // If `text` is `None`, FP just completely ignores the element. It
            // doesn't even check its `elementFormat`.
            if let Some(text) = text {
                all_text.push_str(text);

                let format = format.ok_or(HandleContentError::NullElementFormat)?;
                let text_format = format.as_text_format();

                let span = TextSpan::with_length_and_format(text.len(), &text_format);
                all_spans.push(span);
            }
        }
        ElementData::Group { elements } => {
            // TODO: The docs say GroupElement's format has some effects?
            for element in elements {
                handle_content_element(*element, all_text, all_spans)?;
            }
        }
        ElementData::Graphic => {
            // TODO
        }
        ElementData::Invalid => {
            unreachable!(
                "TextBlock and GroupElement prevent holding user subclasses of ContentElement"
            )
        }
    }

    Ok(())
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
    format: TextFormat,
    line_index: u32,
) {
    // TODO: Handle device font/non-device font properly
    edit_text.set_is_device_font(activation.context, false);
    edit_text.set_text_format(
        0,
        edit_text.text_length(),
        format.clone(),
        activation.context,
    );
    edit_text.set_new_text_format(format);

    edit_text.set_word_wrap(true, activation.context);

    let measured_text = edit_text.measure_text(activation.context);

    edit_text.set_height(activation.context, measured_text.1.to_pixels());
    edit_text.set_y(measured_text.1 * line_index as i32);
}
