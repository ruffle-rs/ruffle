use crate::avm2::activation::Activation;
use crate::avm2::error::Error;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::display_object::{Atom, TextLineLayout};
use std::cell::Ref;

fn text_line_layout<'gc>(this: Value<'gc>) -> Ref<'gc, TextLineLayout<'gc>> {
    this.as_object()
        .expect("TextLine native getter receiver must be an object")
        .as_display_object()
        .expect("TextLine native getter receiver must be a display object")
        .as_text_line()
        .expect("TextLine native getter receiver must be a TextLine")
        .line()
}

fn atom_at<'a>(line: &'a TextLineLayout, index: i32) -> Option<&'a Atom> {
    if index < 0 {
        return None;
    }
    line.atoms().get(index as usize)
}

pub fn get_text_width<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let line = text_line_layout(this);
    Ok((line.text_width() as f64).into())
}

pub fn get_text_height<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let line = text_line_layout(this);
    Ok(((line.ascent() + line.descent()) as f64).into())
}

pub fn get_ascent<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let line = text_line_layout(this);
    Ok((line.ascent() as f64).into())
}

pub fn get_descent<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let line = text_line_layout(this);
    Ok((line.descent() as f64).into())
}

pub fn get_baseline_position<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let baseline = args.get_value(0).coerce_to_string(activation)?;
    let line = text_line_layout(this);
    let (ascent, descent) = (line.ascent(), line.descent());
    let position = match baseline.to_utf8_lossy().as_ref() {
        "roman" => 0.0,
        "ascent" => -ascent,
        "descent" => descent,
        "ideographicTop" => -ascent,
        "ideographicCenter" => (descent - ascent) / 2.0,
        "ideographicBottom" => descent,
        _ => 0.0,
    };
    Ok((position as f64).into())
}

pub fn get_atom_count<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let line = text_line_layout(this);
    Ok((line.atoms().len() as i32).into())
}

pub fn get_atom_index_at_char_index<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let char_index = args.get_i32(0);
    let line = text_line_layout(this);
    let atom = line
        .atoms()
        .iter()
        .position(|atom| atom.char_start as i32 <= char_index && char_index < atom.char_end as i32)
        .map(|index| index as i32)
        .unwrap_or(-1);
    Ok(atom.into())
}

pub fn get_atom_bounds<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let (x, y, width, height) = match fte_line(this) {
        Some(line) => match atom_at(&line, args.get_i32(0)) {
            Some(atom) => (
                atom.x as f64,
                -line.ascent() as f64,
                atom.width as f64,
                (line.ascent() + line.descent()) as f64,
            ),
            None => (0.0, 0.0, 0.0, 0.0),
        },
        None => (0.0, 0.0, 0.0, 0.0),
    };

    activation.avm2().classes().rectangle.construct(
        activation,
        &[x.into(), y.into(), width.into(), height.into()],
    )
}

pub fn get_atom_center<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let Some(line) = fte_line(this) else {
        return Ok(0.0.into());
    };
    let Some(atom) = atom_at(&line, args.get_i32(0)) else {
        return Ok(0.0.into());
    };
    Ok(((atom.x + atom.width / 2.0) as f64).into())
}

pub fn get_atom_text_block_begin_index<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let line = text_line_layout(this);
    let Some(atom) = atom_at(&line, args.get_i32(0)) else {
        return Ok((-1).into());
    };
    Ok((atom.char_start as i32).into())
}

pub fn get_atom_text_block_end_index<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let line = text_line_layout(this);
    let Some(atom) = atom_at(&line, args.get_i32(0)) else {
        return Ok((-1).into());
    };
    Ok((atom.char_end as i32).into())
}
