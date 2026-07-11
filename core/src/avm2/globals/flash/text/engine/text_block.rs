use ruffle_common::avm_string::AvmString;
use ruffle_macros::istr;

use crate::avm2::Avm2StrRepresentable;
use crate::avm2::activation::Activation;
use crate::avm2::error::{Error, Error2004Type, make_error_2004, make_error_2008};
use crate::avm2::globals::flash::display::display_object::initialize_for_allocator;
use crate::avm2::globals::methods::flash_text_engine_content_element as element_methods;
use crate::avm2::object::{Object, TObject, VectorObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2_stub_method;
use crate::display_object::{EditText, TDisplayObject, TextLine};
use crate::fte::FontWeightValue;
use crate::fte::TextLineCreationResultValue;
use crate::fte::{FontLookupValue, TextBaselineValue};
use crate::fte::{FontPostureValue, TextRotationValue};
use crate::html::TextFormat;
use crate::string::WStr;

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
    _activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
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

pub fn create_text_line<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this_obj = this.as_object().unwrap();
    let block = this_obj.as_text_block_object().unwrap();

    avm2_stub_method!(activation, "flash.text.TextBlock", "createTextLine");

    let previous_text_line = args.try_get_object(0);
    let width = args.get_f64(1);

    let previous_position = if let Some(previous_text_line) = previous_text_line {
        previous_text_line
            .as_display_object()
            .unwrap()
            .as_text_line()
            .unwrap()
            .end_index() as usize
    } else {
        0
    };

    let content = block.content();
    let text = if let Some(content) = block.content() {
        get_text_from_content(content, activation)?
    } else {
        None
    };
    let text = text.unwrap_or_else(|| istr!("")).as_wstr();

    let next_position = next_line_break(text, previous_position);

    if text.is_empty() || next_position == text.len() && previous_position == next_position {
        // No more text.
        block.set_text_line_creation_result(Some(TextLineCreationResultValue::Complete));
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

    let element_format = if let Some(content) = content {
        Value::from(content)
            .call_method(element_methods::GET_ELEMENT_FORMAT, &[], activation)?
            .as_object()
    } else {
        None
    };
    apply_format(activation, fallback, element_format, line_index)?;

    let text_line = TextLine::new(activation.context, movie, fallback);
    let instance = initialize_for_allocator(activation.context, text_line.into(), class);
    let instance: Object<'gc> = instance.into();

    text_line.set_text_block(Some(block), activation.gc());
    text_line.set_specified_width(width);
    text_line.set_raw_text_length(text.len() as u32);
    text_line.set_begin_index(previous_position as u32);
    text_line.set_end_index(next_position as u32);

    use crate::avm2::globals::slots::flash_text_engine_text_line as line_slots;
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

    block.set_text_line_creation_result(Some(TextLineCreationResultValue::Success));
    block.set_first_line(Some(instance), activation.gc());

    Ok(instance.into())
}

fn get_text_from_content<'gc>(
    content: Object<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Option<AvmString<'gc>>, Error<'gc>> {
    let text = Value::from(content).call_method(element_methods::GET_TEXT, &[], activation)?;

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
    if let Some(ef) = element_format.and_then(|o| o.as_element_format_object()) {
        // TODO: Support more ElementFormat properties
        let (font, bold, italic, is_device_font) = if let Some(fd) = ef.font_description() {
            (
                Some(fd.font_name().as_wstr().into()),
                Some(fd.font_weight() == FontWeightValue::Bold),
                Some(fd.font_posture() == FontPostureValue::Italic),
                fd.font_lookup() == FontLookupValue::Device,
            )
        } else {
            (None, None, None, true)
        };

        let format = TextFormat {
            color: Some(ef.color()),
            size: Some(ef.font_size()),
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
