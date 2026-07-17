use crate::avm2::activation::Activation;
use crate::avm2::error::{Error2004Type, make_error_2004, make_error_2008};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::{Avm2StrRepresentable, Error};
use crate::fte::TabAlignmentValue;

pub use crate::avm2::object::tab_stop_allocator;

pub fn get_alignment<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_tab_stop_object().unwrap();
    Ok(this.alignment().as_avm2_str(activation).into())
}

pub fn set_alignment<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_tab_stop_object().unwrap();
    let s = args.get_string_non_null(activation, 0, "alignment")?;
    let Some(value) = TabAlignmentValue::from_avm2_str(&s) else {
        return Err(make_error_2008(activation, "alignment"));
    };
    this.set_alignment(value);
    Ok(Value::Undefined)
}

pub fn get_position<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_tab_stop_object().unwrap();
    Ok(this.position().into())
}

pub fn set_position<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_tab_stop_object().unwrap();
    let value = args.get_f64(0);
    if value < 0.0 {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    }
    this.set_position(value);
    Ok(Value::Undefined)
}

pub fn get_decimal_alignment_token<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_tab_stop_object().unwrap();
    Ok(this.decimal_alignment_token().into())
}

pub fn set_decimal_alignment_token<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_tab_stop_object().unwrap();
    let value = args.get_string_non_null(activation, 0, "decimalAlignmentToken")?;
    this.set_decimal_alignment_token(value, activation.gc());
    Ok(Value::Undefined)
}
