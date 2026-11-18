use crate::avm2::Avm2StrRepresentable;
use crate::avm2::activation::Activation;
use crate::avm2::error::{Error, Error2006Type, make_error_2006, make_error_2008};
use crate::avm2::function::FunctionArgs;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::display_object::{EditText, TDisplayObject};
use crate::fte::TextLineValidity;
use ruffle_macros::istr;

pub fn get_text_width<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let display_object = this.as_display_object().unwrap();
    let Some(text_line) = display_object.as_text_line() else {
        return Ok(0.0.into());
    };

    let measured_text = text_line.measure_text(activation.context);
    Ok(measured_text.0.to_pixels().into())
}

pub fn get_has_tabs<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_display_object()
        .unwrap()
        .as_text_line()
        .unwrap();

    Ok(this.has_tabs().into())
}

pub fn get_ascent<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let display_object = this.as_display_object().unwrap();
    let text_line = display_object.as_text_line().unwrap();

    Ok(text_line.metrics().0.into())
}

pub fn get_descent<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let display_object = this.as_display_object().unwrap();
    let text_line = display_object.as_text_line().unwrap();

    Ok(text_line.metrics().1.into())
}

pub fn get_validity<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let display_object = this.as_display_object().unwrap();
    let Some(text_line) = display_object.as_text_line() else {
        return Ok(Value::Undefined);
    };

    let validity = match text_line.validity() {
        TextLineValidity::Valid => istr!("valid"),
        TextLineValidity::Invalid => istr!("invalid"),
        TextLineValidity::Static => istr!("static"),
        TextLineValidity::PossiblyInvalid => istr!("possiblyInvalid"),
        TextLineValidity::UserInvalid(string) => string,
    };

    Ok(validity.into())
}

pub fn set_validity<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let display_object = this.as_display_object().unwrap();
    let Some(text_line) = display_object.as_text_line() else {
        return Ok(Value::Undefined);
    };

    let value = args.get_string_non_null(activation, 0, "validity")?;

    let previous_value = text_line.validity();
    let new_value = TextLineValidity::parse(value);

    let transition_allowed = match (previous_value, new_value) {
        (a, b) if a == b => true,
        (_, TextLineValidity::PossiblyInvalid) => false,
        (_, TextLineValidity::Static) => true,
        (TextLineValidity::Static, _) => false,
        (TextLineValidity::Invalid, _) => false,
        _ => true,
    };

    if !transition_allowed {
        return Err(make_error_2008(activation, "validity"));
    }

    text_line.set_validity(new_value, activation.gc());
    Ok(Value::Undefined)
}

pub fn get_text_block<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_display_object()
        .unwrap()
        .as_text_line()
        .unwrap();

    let block = this.text_block_from_script();

    Ok(block.map(Value::from).unwrap_or(Value::Null))
}

pub fn get_specified_width<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_display_object()
        .unwrap()
        .as_text_line()
        .unwrap();

    Ok(this.specified_width().into())
}

pub fn get_raw_text_length<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_display_object()
        .unwrap()
        .as_text_line()
        .unwrap();

    Ok(this.raw_text_length().into())
}

pub fn get_text_block_begin_index<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_display_object()
        .unwrap()
        .as_text_line()
        .unwrap();

    Ok(this.begin_index().into())
}

pub fn get_previous_line<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_display_object()
        .unwrap()
        .as_text_line()
        .unwrap();

    Ok(this
        .previous_line()
        .and_then(|line| line.object2())
        .map(Value::from)
        .unwrap_or(Value::Null))
}

pub fn get_next_line<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_display_object()
        .unwrap()
        .as_text_line()
        .unwrap();

    Ok(this
        .next_line()
        .and_then(|line| line.object2())
        .map(Value::from)
        .unwrap_or(Value::Null))
}

pub fn get_text_height<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let display_object = this.as_display_object().unwrap();
    let Some(text_line) = display_object.as_text_line() else {
        return Ok(0.0.into());
    };

    let measured_text = text_line.measure_text(activation.context);
    Ok(measured_text.1.to_pixels().into())
}

pub fn get_atom_bounds<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_display_object()
        .unwrap()
        .as_text_line()
        .unwrap();

    let index = args.get_i32(0);

    if index < 0 || index as u32 >= this.atom_count() {
        return Err(make_error_2006(activation, Error2006Type::RangeError));
    }

    let (ascent, descent) = this.metrics();
    let fallback = this.fallback();
    let is_terminator = fallback
        .text()
        .get(index as usize)
        .is_some_and(|c| matches!(c, 0x0A | 0x0D | 0x2028 | 0x2029));

    // TODO: Bidi text isn't reordered, so atoms of right-to-left text are
    // placed left to right.
    let bounds = fallback.char_bounds(index as usize);
    let (x, width) = match bounds {
        Some(b) if !is_terminator => (
            (b.x_min - EditText::GUTTER).to_pixels(),
            b.width().to_pixels(),
        ),
        // Flash places the line terminator after the text, as wide as a space.
        _ => (
            this.measure_text(activation.context).0.to_pixels(),
            this.space_advance().to_pixels(),
        ),
    };
    activation.avm2().classes().rectangle.construct(
        activation,
        &[
            x.into(),
            (-ascent).into(),
            width.into(),
            (ascent + descent).into(),
        ],
    )
}

pub fn get_baseline_zero<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_display_object()
        .unwrap()
        .as_text_line()
        .unwrap();

    Ok(this.baseline_zero().as_avm2_str(activation).into())
}

pub fn get_atom_count<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_display_object()
        .unwrap()
        .as_text_line()
        .unwrap();

    Ok(this.atom_count().into())
}
