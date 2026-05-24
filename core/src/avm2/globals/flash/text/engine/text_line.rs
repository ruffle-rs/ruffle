use crate::avm2::activation::Activation;
use crate::avm2::error::Error;
use crate::avm2::value::Value;
use crate::avm2_stub_getter;
use crate::display_object::TextLineLayout;
use std::cell::Ref;

fn text_line_layout<'gc>(this: Value<'gc>) -> Option<Ref<'gc, TextLineLayout<'gc>>> {
    let line = this.as_object()?.as_display_object()?.as_text_line()?;
    Some(line.line())
}

pub fn get_text_width<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(line) = text_line_layout(this) {
        avm2_stub_getter!(activation, "flash.text.engine.TextLine", "textWidth");
        return Ok((line.text_width() as f64).into());
    }

    let this = this.as_object().unwrap();

    let display_object = this.as_display_object().unwrap();
    if let Some(text_line) = display_object.as_text_line() {
        avm2_stub_getter!(activation, "flash.text.engine.TextLine", "textWidth");
        if let Some(measured_text) = text_line.measure_text(activation.context) {
            return Ok(measured_text.0.to_pixels().into());
        }
        return Ok(0.0.into());
    }
    let Some(edit_text) = display_object.as_edit_text() else {
        return Ok(0.0.into());
    };

    let measured_text = edit_text.measure_text(activation.context);
    Ok(measured_text.0.to_pixels().into())
}

pub fn get_text_height<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(line) = text_line_layout(this) {
        avm2_stub_getter!(activation, "flash.text.engine.TextLine", "textHeight");
        return Ok(((line.ascent() + line.descent()) as f64).into());
    }

    let this = this.as_object().unwrap();

    let display_object = this.as_display_object().unwrap();
    if let Some(text_line) = display_object.as_text_line() {
        avm2_stub_getter!(activation, "flash.text.engine.TextLine", "textHeight");
        if let Some(measured_text) = text_line.measure_text(activation.context) {
            return Ok(measured_text.1.to_pixels().into());
        }
        return Ok(0.0.into());
    }
    let Some(edit_text) = display_object.as_edit_text() else {
        return Ok(0.0.into());
    };

    let measured_text = edit_text.measure_text(activation.context);
    Ok(measured_text.1.to_pixels().into())
}

pub fn get_ascent<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let Some(line) = text_line_layout(this) else {
        avm2_stub_getter!(activation, "flash.text.engine.TextLine", "ascent");
        return Ok(12.0.into());
    };
    avm2_stub_getter!(activation, "flash.text.engine.TextLine", "ascent");
    Ok((line.ascent() as f64).into())
}

pub fn get_descent<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let Some(line) = text_line_layout(this) else {
        avm2_stub_getter!(activation, "flash.text.engine.TextLine", "descent");
        return Ok(3.0.into());
    };
    avm2_stub_getter!(activation, "flash.text.engine.TextLine", "descent");
    Ok((line.descent() as f64).into())
}
