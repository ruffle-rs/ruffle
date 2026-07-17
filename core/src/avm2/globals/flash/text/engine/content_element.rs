use crate::avm2::Avm2StrRepresentable;
use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::error::make_error_2008;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::fte::TextRotationValue;
use crate::{avm2_stub_getter, avm2_stub_setter};

pub use crate::avm2::object::content_element_allocator;

pub fn get_text<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_content_element_object()
        .unwrap();
    Ok(match this.text() {
        Some(s) => s.into(),
        None => Value::Null,
    })
}

pub fn get_element_format<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_content_element_object()
        .unwrap();
    Ok(match this.element_format() {
        Some(ef) => ef.into(),
        None => Value::Null,
    })
}

pub fn set_element_format<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_content_element_object()
        .unwrap();
    let ef = args
        .try_get_object(0)
        .and_then(|o| o.as_element_format_object());
    this.set_element_format(ef, activation.gc());
    Ok(Value::Undefined)
}

pub fn get_text_block<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.text.engine.ContentElement", "textBlock");
    Ok(Value::Null)
}

pub fn get_text_block_begin_index<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(
        activation,
        "flash.text.engine.ContentElement",
        "textBlockBeginIndex"
    );
    Ok((-1).into())
}

pub fn get_group_element<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(
        activation,
        "flash.text.engine.ContentElement",
        "groupElement"
    );
    Ok(Value::Null)
}

pub fn get_event_mirror<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(
        activation,
        "flash.text.engine.ContentElement",
        "eventMirror"
    );
    let this = this
        .as_object()
        .unwrap()
        .as_content_element_object()
        .unwrap();
    Ok(this.event_mirror().map(|v| v.into()).unwrap_or(Value::Null))
}

pub fn set_event_mirror<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_setter!(
        activation,
        "flash.text.engine.ContentElement",
        "eventMirror"
    );
    let this = this
        .as_object()
        .unwrap()
        .as_content_element_object()
        .unwrap();
    this.set_event_mirror(args.try_get_object(0), activation.gc());
    Ok(Value::Undefined)
}

pub fn get_text_rotation<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(
        activation,
        "flash.text.engine.ContentElement",
        "textRotation"
    );
    let this = this
        .as_object()
        .unwrap()
        .as_content_element_object()
        .unwrap();
    Ok(this.text_rotation().as_avm2_str(activation).into())
}

pub fn set_text_rotation<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_setter!(
        activation,
        "flash.text.engine.ContentElement",
        "textRotation"
    );
    let this = this
        .as_object()
        .unwrap()
        .as_content_element_object()
        .unwrap();
    let s = args.get_string_non_null(activation, 0, "textRotation")?;
    this.set_text_rotation(
        TextRotationValue::from_avm2_str(&s)
            .filter(|&v| v != TextRotationValue::Auto)
            .ok_or_else(|| make_error_2008(activation, "textRotation"))?,
    );
    Ok(Value::Undefined)
}

pub fn get_raw_text<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.text.engine.ContentElement", "rawText");
    get_text(activation, this, args)
}
