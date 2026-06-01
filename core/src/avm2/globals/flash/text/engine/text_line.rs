use crate::avm2::activation::Activation;
use crate::avm2::error::Error;
use crate::avm2::value::Value;
use crate::display_object::TextLineLayout;
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
