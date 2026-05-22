//! Natives for the flash.text.engine.TextLine class.
//!
//! Every metric (ascent, descent, textWidth, getAtomBounds and the rest) reads
//! the FteLine that TextBlock.createTextLine caches on the TextLine instance;
//! see text_block.rs. The line is laid out once and these getters only read it.

use crate::avm2::activation::Activation;
use crate::avm2::error::Error;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::display_object::{Atom, FteLine};
use std::cell::Ref;

/// The laid-out FteLine backing this, when this is a TextLine.
fn fte_line<'gc>(this: Value<'gc>) -> Option<Ref<'gc, FteLine<'gc>>> {
    let line = this.as_object()?.as_display_object()?.as_fte_text_line()?;
    Some(line.line())
}

/// The atom at the given index of the line, or None when it is out of range.
fn atom_at<'a>(line: &'a FteLine, index: i32) -> Option<&'a Atom> {
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
    let Some(line) = fte_line(this) else {
        return Ok(0.0.into());
    };
    Ok((line.text_width() as f64).into())
}

pub fn get_text_height<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let Some(line) = fte_line(this) else {
        return Ok(0.0.into());
    };
    Ok(((line.ascent() + line.descent()) as f64).into())
}

pub fn get_ascent<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let Some(line) = fte_line(this) else {
        return Ok(12.0.into());
    };
    Ok((line.ascent() as f64).into())
}

pub fn get_descent<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let Some(line) = fte_line(this) else {
        return Ok(3.0.into());
    };
    Ok((line.descent() as f64).into())
}

pub fn get_baseline_position<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let baseline = args.get_value(0).coerce_to_string(activation)?;
    let (ascent, descent) = match fte_line(this) {
        Some(line) => (line.ascent(), line.descent()),
        None => (12.0, 3.0),
    };
    let position: f32 = match baseline.to_utf8_lossy().as_ref() {
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
    let Some(line) = fte_line(this) else {
        return Ok(0.into());
    };
    Ok((line.atoms().len() as i32).into())
}

pub fn get_atom_bounds<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let index = args.get_i32(0);
    let (x, y, width, height) = atom_bounds(this, index);

    let class = activation.avm2().classes().rectangle;
    class.construct(
        activation,
        &[x.into(), y.into(), width.into(), height.into()],
    )
}

/// The x, y, width and height of the atom, all zero when it does not exist.
fn atom_bounds(this: Value<'_>, index: i32) -> (f64, f64, f64, f64) {
    let Some(line) = fte_line(this) else {
        return (0.0, 0.0, 0.0, 0.0);
    };
    let Some(atom) = atom_at(&line, index) else {
        return (0.0, 0.0, 0.0, 0.0);
    };
    (
        atom.x as f64,
        -line.ascent() as f64,
        atom.width as f64,
        (line.ascent() + line.descent()) as f64,
    )
}

pub fn get_atom_bidi_level<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let index = args.get_i32(0);
    let Some(line) = fte_line(this) else {
        return Ok(0.into());
    };
    let Some(atom) = atom_at(&line, index) else {
        return Ok(0.into());
    };
    Ok((atom.bidi_level as i32).into())
}

pub fn get_atom_center<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let index = args.get_i32(0);
    let Some(line) = fte_line(this) else {
        return Ok(0.0.into());
    };
    let Some(atom) = atom_at(&line, index) else {
        return Ok(0.0.into());
    };
    Ok(((atom.x + atom.width / 2.0) as f64).into())
}

pub fn get_atom_index_at_char_index<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let char_index = args.get_i32(0);
    let Some(line) = fte_line(this) else {
        return Ok((-1).into());
    };
    // charIndex is text-block-absolute. Every atom spans exactly one char, in
    // order, so the atom index is the offset of charIndex from the line's
    // first atom.
    let Some(first) = line.atoms().first() else {
        return Ok((-1).into());
    };
    if char_index < 0 || (char_index as usize) < first.char_start {
        return Ok((-1).into());
    }
    let offset = char_index as usize - first.char_start;
    if offset < line.atoms().len() {
        Ok((offset as i32).into())
    } else {
        Ok((-1).into())
    }
}

pub fn get_atom_text_block_begin_index<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let index = args.get_i32(0);
    let Some(line) = fte_line(this) else {
        return Ok((-1).into());
    };
    let Some(atom) = atom_at(&line, index) else {
        return Ok((-1).into());
    };
    Ok((atom.char_start as i32).into())
}

pub fn get_atom_text_block_end_index<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let index = args.get_i32(0);
    let Some(line) = fte_line(this) else {
        return Ok((-1).into());
    };
    let Some(atom) = atom_at(&line, index) else {
        return Ok((-1).into());
    };
    Ok((atom.char_end as i32).into())
}

pub fn get_atom_word_boundary_on_left<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let index = args.get_i32(0);
    let Some(line) = fte_line(this) else {
        return Ok(false.into());
    };
    let Some(atom) = atom_at(&line, index) else {
        return Ok(false.into());
    };
    Ok(atom.word_boundary_on_left.into())
}
