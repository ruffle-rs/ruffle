use crate::avm2::activation::Activation;
use crate::avm2::error::{Error, Error2006Type, make_error_2006, make_error_2008};
use crate::avm2::function::FunctionArgs;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::display_object::{EditText, TDisplayObject, TextLine};
use crate::fte::TextLineValidity;
use ruffle_macros::istr;
use ruffle_render::matrix::Matrix;
use swf::{Point, Rectangle};

/// The number of atoms in a line, mirroring `TextLine.atomCount`.
///
/// As in the ActionScript, this is currently approximated by the raw text
/// length (combining characters are not yet collapsed into single atoms).
fn atom_count(text_line: TextLine) -> i32 {
    text_line.raw_text_length() as i32
}

pub fn get_text_width<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let display_object = this.as_display_object().unwrap();
    let Some(text_line) = display_object.as_text_line() else {
        return Ok(0.0.into());
    };

    Ok(text_line.metrics().text_width.to_pixels().into())
}

pub fn get_ascent<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let display_object = this.as_display_object().unwrap();
    let Some(text_line) = display_object.as_text_line() else {
        return Ok(0.0.into());
    };

    Ok(text_line.metrics().ascent.to_pixels().into())
}

pub fn get_descent<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let display_object = this.as_display_object().unwrap();
    let Some(text_line) = display_object.as_text_line() else {
        return Ok(0.0.into());
    };

    Ok(text_line.metrics().descent.to_pixels().into())
}

pub fn get_atom_bounds<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let display_object = this.as_display_object().unwrap();
    let Some(text_line) = display_object.as_text_line() else {
        return Ok(Value::Null);
    };

    let index = args.get_i32(0);
    if index < 0 || index >= atom_count(text_line) {
        return Err(make_error_2006(activation, Error2006Type::RangeError));
    }

    let metrics = text_line.metrics();
    let fallback = text_line.fallback();
    let bounds = match fallback.char_bounds(index as usize) {
        // `char_bounds` is in the fallback's local space (inset by the
        // gutter); the line's coordinate origin is on the baseline, so apply
        // the same offset the line uses when rendering the fallback (the cell
        // ascent, where the glyphs actually sit). Kept consistent with
        // `get_atom_index_at_point`.
        Some(bounds) => {
            Matrix::translate(
                -EditText::GUTTER,
                -(EditText::GUTTER + metrics.fallback_ascent),
            ) * bounds
        }
        // The index is a valid atom (a trailing line terminator counted in
        // `rawTextLength`) with no displayed glyph. Flash Player reports a
        // zero-width box at the end of the line's text; the invalid default
        // rectangle would instead place a caret at a garbage coordinate far off
        // to the right.
        None => Rectangle {
            x_min: metrics.text_width,
            x_max: metrics.text_width,
            y_min: -metrics.fallback_ascent,
            y_max: metrics.fallback_descent,
        },
    };

    let x = bounds.x_min.to_pixels();
    let y = bounds.y_min.to_pixels();
    let width = bounds.width().to_pixels();
    let height = bounds.height().to_pixels();
    activation.avm2().classes().rectangle.construct(
        activation,
        &[x.into(), y.into(), width.into(), height.into()],
    )
}

pub fn get_atom_index_at_point<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let display_object = this.as_display_object().unwrap();
    let Some(text_line) = display_object.as_text_line() else {
        return Ok((-1).into());
    };

    // Flash Player passes the point in stage (global) coordinates.
    let stage_point = Point::from_pixels(args.get_f64(0), args.get_f64(1));

    // Stage -> line-local (origin on the baseline).
    let Some(local) = display_object.global_to_local(stage_point) else {
        return Ok((-1).into());
    };

    // Line-local -> the fallback EditText's local space, undoing the offset the
    // line applies when it renders the fallback (`(-gutter, -(gutter + ascent))`,
    // with the cell ascent). Kept consistent with `get_atom_bounds`.
    let gutter = EditText::GUTTER;
    let ascent = text_line.metrics().fallback_ascent;
    let fallback_point = Point::new(local.x + gutter, local.y + gutter + ascent);

    let index = text_line
        .fallback()
        .char_index_at_point(fallback_point)
        .map_or(-1, |i| i as i32);
    Ok(index.into())
}

pub fn get_atom_word_boundary_on_left<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let display_object = this.as_display_object().unwrap();
    let Some(text_line) = display_object.as_text_line() else {
        return Ok(false.into());
    };

    let index = args.get_i32(0);
    if index < 0 || index >= atom_count(text_line) {
        return Err(make_error_2006(activation, Error2006Type::RangeError));
    }
    if index == 0 {
        // The first atom of a line always begins a word.
        return Ok(true.into());
    }

    let fallback = text_line.fallback();
    let text = fallback.text();
    let index = index as usize;
    if index >= text.len() {
        return Ok(false.into());
    }

    // A word boundary is present on the left of an atom when the preceding
    // character is whitespace and the atom itself is not.
    let is_space = |c: u16| {
        matches!(
            c,
            0x20 | 0x09 | 0x0A | 0x0D | 0xA0 | 0x2028 | 0x2029 | 0x3000
        )
    };
    let prev = text.at(index - 1);
    let curr = text.at(index);
    Ok((is_space(prev) && !is_space(curr)).into())
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
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let display_object = this.as_display_object().unwrap();
    let Some(text_line) = display_object.as_text_line() else {
        return Ok(0.0.into());
    };

    let metrics = text_line.metrics();
    Ok((metrics.ascent + metrics.descent).to_pixels().into())
}
