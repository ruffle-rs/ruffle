//! `Date` class

use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::function::FunctionArgs;
use crate::avm2::object::DateObject;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::date::Date;
use crate::string::{AvmString, WStr, utils as string_utils};
use chrono::FixedOffset;
use ruffle_macros::istr;

pub use crate::avm2::object::date_allocator;

#[derive(Default)]
struct ParsedDateFields {
    year: Option<f64>,
    month: Option<f64>,
    day: Option<f64>,
    hour: Option<f64>,
    minute: Option<f64>,
    second: Option<f64>,
}

fn local_date(date: Date, timezone: FixedOffset) -> Date {
    if !date.is_valid() {
        return Date::INVALID;
    }

    Date::from_millis_unclipped(date.time() + f64::from(timezone.local_minus_utc()) * 1000.0)
}

#[allow(clippy::too_many_arguments)]
fn apply_local_adjustment<'gc>(
    activation: &mut Activation<'_, 'gc>,
    object: DateObject<'gc>,
    year: Option<Value<'gc>>,
    month: Option<Value<'gc>>,
    day: Option<Value<'gc>>,
    hour: Option<Value<'gc>>,
    minute: Option<Value<'gc>>,
    second: Option<Value<'gc>>,
    millisecond: Option<Value<'gc>>,
    recover_invalid: bool,
) -> Result<f64, Error<'gc>> {
    let timezone = activation.context.locale.get_timezone();
    let current = object.date();

    let base = if current.is_valid() {
        local_date(current, timezone)
    } else if recover_invalid {
        Date::from_millis_unclipped(0.0)
    } else {
        object.set_date(Date::INVALID);
        return Ok(f64::NAN);
    };

    let read = |value: Option<Value<'gc>>, fallback: f64, activation: &mut Activation<'_, 'gc>| {
        match value {
            Some(value) => value.coerce_to_number(activation),
            None => Ok(fallback),
        }
    };

    let year = read(year, base.year() as f64, activation)?;
    let month = read(month, base.month() as f64, activation)?;
    let day = read(day, base.date() as f64, activation)?;
    let hour = read(hour, base.hours() as f64, activation)?;
    let minute = read(minute, base.minutes() as f64, activation)?;
    let second = read(second, base.seconds() as f64, activation)?;
    let millisecond = read(millisecond, base.milliseconds() as f64, activation)?;

    let local = Date::make_date(
        Date::make_day(year, month, day),
        Date::make_time(hour, minute, second, millisecond),
    );

    let result =
        Date::from_millis_unclipped(local.time() - f64::from(timezone.local_minus_utc()) * 1000.0)
            .clip();

    object.set_date(result);
    Ok(result.time())
}

#[allow(clippy::too_many_arguments)]
fn apply_utc_adjustment<'gc>(
    activation: &mut Activation<'_, 'gc>,
    object: DateObject<'gc>,
    year: Option<Value<'gc>>,
    month: Option<Value<'gc>>,
    day: Option<Value<'gc>>,
    hour: Option<Value<'gc>>,
    minute: Option<Value<'gc>>,
    second: Option<Value<'gc>>,
    millisecond: Option<Value<'gc>>,
    recover_invalid: bool,
) -> Result<f64, Error<'gc>> {
    let current = object.date();

    let base = if current.is_valid() {
        current
    } else if recover_invalid {
        Date::from_millis_unclipped(0.0)
    } else {
        object.set_date(Date::INVALID);
        return Ok(f64::NAN);
    };

    let read = |value: Option<Value<'gc>>, fallback: f64, activation: &mut Activation<'_, 'gc>| {
        match value {
            Some(value) => value.coerce_to_number(activation),
            None => Ok(fallback),
        }
    };

    let year = read(year, base.year() as f64, activation)?;
    let month = read(month, base.month() as f64, activation)?;
    let day = read(day, base.date() as f64, activation)?;
    let hour = read(hour, base.hours() as f64, activation)?;
    let minute = read(minute, base.minutes() as f64, activation)?;
    let second = read(second, base.seconds() as f64, activation)?;
    let millisecond = read(millisecond, base.milliseconds() as f64, activation)?;

    let result = Date::make_date(
        Date::make_day(year, month, day),
        Date::make_time(hour, minute, second, millisecond),
    )
    .clip();

    object.set_date(result);
    Ok(result.time())
}

pub fn init_custom_prototype<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let this = this.as_class_object().unwrap();

    let prototype_date_object = DateObject::for_prototype(activation.context, this);

    this.link_prototype(activation.context, prototype_date_object);

    Ok(Value::Undefined)
}

/// Implements `Date`'s instance constructor.
pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_date_object().unwrap();

    let timestamp = args.get_optional(0).unwrap_or(Value::Undefined);
    if !matches!(timestamp, Value::Undefined) {
        if args.len() > 1 {
            let mut year = args.get_value(0).coerce_to_number(activation)?;
            let month = args.get_value(1).coerce_to_number(activation)?;
            let day = args
                .get_optional(2)
                .unwrap_or(Value::Integer(1))
                .coerce_to_number(activation)?;
            let hour = args
                .get_optional(3)
                .unwrap_or(Value::Integer(0))
                .coerce_to_number(activation)?;
            let minute = args
                .get_optional(4)
                .unwrap_or(Value::Integer(0))
                .coerce_to_number(activation)?;
            let second = args
                .get_optional(5)
                .unwrap_or(Value::Integer(0))
                .coerce_to_number(activation)?;
            let millisecond = args
                .get_optional(6)
                .unwrap_or(Value::Integer(0))
                .coerce_to_number(activation)?;

            if year < 100.0 {
                year += 1900.0;
            }

            let local_date = Date::make_date(
                Date::make_day(year, month, day),
                Date::make_time(hour, minute, second, millisecond),
            );

            let timezone = activation.context.locale.get_timezone();
            let date = Date::from_millis_unclipped(
                local_date.time() - f64::from(timezone.local_minus_utc()) * 1000.0,
            );

            this.set_date(date);
        } else {
            let timestamp = if let Value::String(date_str) = timestamp {
                parse_full_date(activation, date_str).unwrap_or(f64::NAN)
            } else {
                timestamp.coerce_to_number(activation)?
            };
            this.set_date(Date::from_millis_unclipped(timestamp).clip());
        }
    } else {
        {
            let now = activation.context.locale.get_current_date_time();
            this.set_date(Date::from_millis_unclipped(now.timestamp_millis() as f64));
        }
    }

    Ok(Value::Undefined)
}

pub fn call_handler<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    activation.avm2().classes().date.construct(activation, &[])
}

/// Implements `getTime` method.
pub fn get_time<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_date_object().unwrap();

    Ok(this.date().time().into())
}

/// Implements `setTime` method.
pub fn _set_time<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_date_object().unwrap();

    let new_date = Date::from_millis_unclipped(args.get_f64(0)).clip();
    this.set_date(new_date);
    Ok(new_date.time().into())
}

/// Implements the `getMilliseconds` method.
pub fn get_milliseconds<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_date_object().unwrap();

    let date = local_date(this.date(), activation.context.locale.get_timezone());

    if date.is_valid() {
        Ok((date.milliseconds() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `setMilliseconds` method.
pub fn _set_milliseconds<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_date_object().unwrap();
    Ok(apply_local_adjustment(
        activation,
        this,
        None,
        None,
        None,
        None,
        None,
        None,
        args.get_optional(0),
        false,
    )?
    .into())
}

/// Implements the `getSeconds` method.
pub fn get_seconds<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_date_object().unwrap();

    let date = local_date(this.date(), activation.context.locale.get_timezone());

    if date.is_valid() {
        Ok((date.seconds() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements `setSeconds` method.
pub fn _set_seconds<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_date_object().unwrap();
    Ok(apply_local_adjustment(
        activation,
        this,
        None,
        None,
        None,
        None,
        None,
        args.get_optional(0),
        args.get_optional(1),
        false,
    )?
    .into())
}

/// Implements `getMinutes` method.
pub fn get_minutes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_date_object().unwrap();

    let date = local_date(this.date(), activation.context.locale.get_timezone());

    if date.is_valid() {
        Ok((date.minutes() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `setMinutes` method.
pub fn _set_minutes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_date_object().unwrap();
    Ok(apply_local_adjustment(
        activation,
        this,
        None,
        None,
        None,
        None,
        args.get_optional(0),
        args.get_optional(1),
        args.get_optional(2),
        false,
    )?
    .into())
}

/// Implements the `getHours` method.
pub fn get_hours<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_date_object().unwrap();

    let date = local_date(this.date(), activation.context.locale.get_timezone());

    if date.is_valid() {
        Ok((date.hours() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements `setHours` method.
pub fn _set_hours<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_date_object().unwrap();
    Ok(apply_local_adjustment(
        activation,
        this,
        None,
        None,
        None,
        args.get_optional(0),
        args.get_optional(1),
        args.get_optional(2),
        args.get_optional(3),
        false,
    )?
    .into())
}

/// Implements `getDate` method.
pub fn get_date<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_date_object().unwrap();

    let date = local_date(this.date(), activation.context.locale.get_timezone());

    if date.is_valid() {
        Ok((date.date() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements `setDate` method.
pub fn _set_date<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_date_object().unwrap();
    Ok(apply_local_adjustment(
        activation,
        this,
        None,
        None,
        args.get_optional(0),
        None,
        None,
        None,
        None,
        false,
    )?
    .into())
}

/// Implements the `getMonth` method.
pub fn get_month<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_date_object().unwrap();

    let date = local_date(this.date(), activation.context.locale.get_timezone());

    if date.is_valid() {
        Ok((date.month() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `setMonth` method.
pub fn _set_month<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_date_object().unwrap();
    Ok(apply_local_adjustment(
        activation,
        this,
        None,
        args.get_optional(0),
        args.get_optional(1),
        None,
        None,
        None,
        None,
        false,
    )?
    .into())
}

/// Implements the `getFullYear` method.
pub fn get_full_year<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_date_object().unwrap();

    let date = local_date(this.date(), activation.context.locale.get_timezone());

    if date.is_valid() {
        Ok((date.year() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `setFullYear` method.
pub fn _set_full_year<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_date_object().unwrap();
    Ok(apply_local_adjustment(
        activation,
        this,
        args.get_optional(0),
        args.get_optional(1),
        args.get_optional(2),
        None,
        None,
        None,
        None,
        true,
    )?
    .into())
}

/// Implements the `getDay` method.
pub fn get_day<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_date_object().unwrap();

    let date = local_date(this.date(), activation.context.locale.get_timezone());

    if date.is_valid() {
        Ok((date.week_day() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `getUTCMilliseconds` method.
pub fn get_utc_milliseconds<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_date_object().unwrap();

    let date = this.date();

    if date.is_valid() {
        Ok((date.milliseconds() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `setUTCMilliseconds` method.
pub fn _set_utc_milliseconds<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_date_object().unwrap();

    let timestamp = apply_utc_adjustment(
        activation,
        this,
        None,
        None,
        None,
        None,
        None,
        None,
        args.get_optional(0),
        false,
    )?;
    Ok(timestamp.into())
}

/// Implements the `getUTCSeconds` method.
pub fn get_utc_seconds<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_date_object().unwrap();

    let date = this.date();

    if date.is_valid() {
        Ok((date.seconds() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `setUTCSeconds` method.
pub fn _set_utc_seconds<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_date_object().unwrap();

    let timestamp = apply_utc_adjustment(
        activation,
        this,
        None,
        None,
        None,
        None,
        None,
        args.get_optional(0),
        args.get_optional(1),
        false,
    )?;
    Ok(timestamp.into())
}

/// Implements the `getUTCMinutes` method.
pub fn get_utc_minutes<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_date_object().unwrap();

    let date = this.date();

    if date.is_valid() {
        Ok((date.minutes() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `setUTCMinutes` method.
pub fn _set_utc_minutes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_date_object().unwrap();

    let timestamp = apply_utc_adjustment(
        activation,
        this,
        None,
        None,
        None,
        None,
        args.get_optional(0),
        args.get_optional(1),
        args.get_optional(2),
        false,
    )?;
    Ok(timestamp.into())
}

/// Implements the `getUTCHours` method.
pub fn get_utc_hours<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_date_object().unwrap();

    let date = this.date();

    if date.is_valid() {
        Ok((date.hours() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `setUTCHours` method.
pub fn _set_utc_hours<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_date_object().unwrap();

    let timestamp = apply_utc_adjustment(
        activation,
        this,
        None,
        None,
        None,
        args.get_optional(0),
        args.get_optional(1),
        args.get_optional(2),
        args.get_optional(3),
        false,
    )?;
    Ok(timestamp.into())
}

/// Implements the `getUTCDate` method.
pub fn get_utc_date<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_date_object().unwrap();

    let date = this.date();

    if date.is_valid() {
        Ok((date.date() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `setUTCDate` method.
pub fn _set_utc_date<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_date_object().unwrap();

    let timestamp = apply_utc_adjustment(
        activation,
        this,
        None,
        None,
        args.get_optional(0),
        None,
        None,
        None,
        None,
        false,
    )?;
    Ok(timestamp.into())
}

/// Implements the `getUTCMonth` method.
pub fn get_utc_month<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_date_object().unwrap();

    let date = this.date();

    if date.is_valid() {
        Ok((date.month() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `setUTCMonth` method.
pub fn _set_utc_month<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_date_object().unwrap();

    let timestamp = apply_utc_adjustment(
        activation,
        this,
        None,
        args.get_optional(0),
        args.get_optional(1),
        None,
        None,
        None,
        None,
        false,
    )?;
    Ok(timestamp.into())
}

/// Implements the `getUTCFullYear` method.
pub fn get_utc_full_year<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_date_object().unwrap();

    let date = this.date();

    if date.is_valid() {
        Ok((date.year() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `setUTCFullYear` method.
pub fn _set_utc_full_year<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_date_object().unwrap();

    let timestamp = apply_utc_adjustment(
        activation,
        this,
        args.get_optional(0),
        args.get_optional(1),
        args.get_optional(2),
        None,
        None,
        None,
        None,
        true,
    )?;
    Ok(timestamp.into())
}

/// Implements the `getUTCDay` method.
pub fn get_utc_day<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_date_object().unwrap();

    let date = this.date();

    if date.is_valid() {
        Ok((date.week_day() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `getTimezoneOffset` method.
pub fn get_timezone_offset<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_date_object().unwrap();

    if this.date().is_valid() {
        let timezone = activation.context.locale.get_timezone();
        let offset = -f64::from(timezone.local_minus_utc()) / 60.0;
        Ok(offset.into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `UTC` class method.
pub fn utc<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let mut year = args.get_value(0).coerce_to_number(activation)?;
    let month = args.get_value(1).coerce_to_number(activation)?;
    let day = args.get_value(2).coerce_to_number(activation)?;
    let hour = args.get_value(3).coerce_to_number(activation)?;
    let minute = args.get_value(4).coerce_to_number(activation)?;
    let second = args.get_value(5).coerce_to_number(activation)?;
    let millisecond = args.get_value(6).coerce_to_number(activation)?;

    if year < 100.0 {
        year += 1900.0;
    }

    let date = Date::make_date(
        Date::make_day(year, month, day),
        Date::make_time(hour, minute, second, millisecond),
    );

    Ok(date.time().into())
}

const WEEKDAY_NAMES: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

fn format_ampm(hour: i32) -> &'static str {
    if hour < 12 { "AM" } else { "PM" }
}

fn format_timezone(timezone: FixedOffset) -> String {
    let seconds = timezone.local_minus_utc();
    let sign = if seconds >= 0 { '+' } else { '-' };
    let minutes = seconds.unsigned_abs() / 60;

    format!("{}{:02}{:02}", sign, minutes / 60, minutes % 60)
}

/// Implements the `toString` method.
pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let this = this.as_date_object().unwrap();
    let date = this.date();

    if !date.is_valid() {
        return Ok(istr!("Invalid Date").into());
    }

    let timezone = activation.context.locale.get_timezone();
    let local = local_date(date, timezone);
    let value = format!(
        "{} {} {} {:02}:{:02}:{:02} GMT{} {}",
        WEEKDAY_NAMES[local.week_day() as usize],
        MONTH_NAMES[local.month() as usize],
        local.date(),
        local.hours(),
        local.minutes(),
        local.seconds(),
        format_timezone(timezone),
        local.year(),
    );

    Ok(AvmString::new_utf8(activation.gc(), value).into())
}

/// Implements the `toUTCString` method.
pub fn to_utc_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let this = this.as_date_object().unwrap();
    let date = this.date();

    if !date.is_valid() {
        return Ok(istr!("Invalid Date").into());
    }

    let value = format!(
        "{} {} {} {:02}:{:02}:{:02} {} UTC",
        WEEKDAY_NAMES[date.week_day() as usize],
        MONTH_NAMES[date.month() as usize],
        date.date(),
        date.hours(),
        date.minutes(),
        date.seconds(),
        date.year(),
    );

    Ok(AvmString::new_utf8(activation.gc(), value).into())
}

/// Implements the `toLocaleString` method.
pub fn to_locale_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let this = this.as_date_object().unwrap();
    let date = this.date();

    if !date.is_valid() {
        return Ok(istr!("Invalid Date").into());
    }

    let local = local_date(date, activation.context.locale.get_timezone());
    let value = format!(
        "{} {} {} {} {:02}:{:02}:{:02} {}",
        WEEKDAY_NAMES[local.week_day() as usize],
        MONTH_NAMES[local.month() as usize],
        local.date(),
        local.year(),
        local.hours(),
        local.minutes(),
        local.seconds(),
        format_ampm(local.hours()),
    );

    Ok(AvmString::new_utf8(activation.gc(), value).into())
}

/// Implements the `toTimeString` method.
pub fn to_time_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let this = this.as_date_object().unwrap();
    let date = this.date();

    if !date.is_valid() {
        return Ok(istr!("Invalid Date").into());
    }

    let timezone = activation.context.locale.get_timezone();
    let local = local_date(date, timezone);
    let value = format!(
        "{:02}:{:02}:{:02} GMT{}",
        local.hours(),
        local.minutes(),
        local.seconds(),
        format_timezone(timezone),
    );

    Ok(AvmString::new_utf8(activation.gc(), value).into())
}

/// Implements the `toLocaleTimeString` method.
pub fn to_locale_time_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let this = this.as_date_object().unwrap();
    let date = this.date();

    if !date.is_valid() {
        return Ok(istr!("Invalid Date").into());
    }

    let local = local_date(date, activation.context.locale.get_timezone());
    let value = format!(
        "{:02}:{:02}:{:02} {}",
        local.hours(),
        local.minutes(),
        local.seconds(),
        format_ampm(local.hours()),
    );

    Ok(AvmString::new_utf8(activation.gc(), value).into())
}

/// Implements the `toDateString` & `toLocaleDateString` method.
pub fn to_date_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let this = this.as_date_object().unwrap();
    let date = this.date();

    if !date.is_valid() {
        return Ok(istr!("Invalid Date").into());
    }

    let local = local_date(date, activation.context.locale.get_timezone());
    let value = format!(
        "{} {} {} {}",
        WEEKDAY_NAMES[local.week_day() as usize],
        MONTH_NAMES[local.month() as usize],
        local.date(),
        local.year(),
    );

    Ok(AvmString::new_utf8(activation.gc(), value).into())
}

/// Parse a date, in any of the three formats: YYYY/MM/DD, MM/DD/YYYY, Mon/DD/YYYY.
/// The output will always be: (year, month, day), or None if format is invalid.
fn parse_date(item: &WStr) -> Option<(u32, u32, u32)> {
    let mut iter = item.split(b'/');
    let first = iter.next()?;
    let parsed = if first.len() == 4 {
        // If the first item in this date is 4 characters long, we parse as YYYY/MM/DD
        let month = iter.next()?;
        if month.len() != 2 {
            return None;
        }
        let day = iter.next()?;
        if day.len() != 2 {
            return None;
        }
        (
            first.parse::<u32>().ok()?,
            month.parse::<u32>().ok()?.checked_sub(1)?,
            day.parse::<u32>().ok()?,
        )
    } else if first.len() == 2 {
        // If the first item in this date is 2 characters long, we parse as MM/DD/YYYY
        let day = iter.next()?;
        if day.len() != 2 {
            return None;
        }
        let year = iter.next()?;
        if year.len() != 4 {
            return None;
        }
        (
            year.parse::<u32>().ok()?,
            first.parse::<u32>().ok()?.checked_sub(1)?,
            day.parse::<u32>().ok()?,
        )
    } else if first.len() == 3 {
        // If the first item in this date is 3 characters long, we parse as Mon/DD/YYYY

        // First lets parse the Month
        let month = parse_mon(first)?;
        let day = iter.next()?;
        if day.len() != 2 {
            return None;
        }
        let year = iter.next()?;
        if year.len() != 4 {
            return None;
        }
        (
            year.parse::<u32>().ok()?,
            month as u32,
            day.parse::<u32>().ok()?,
        )
    } else {
        return None;
    };
    if iter.next().is_some() {
        // the iterator should have been empty
        return None;
    }
    Some(parsed)
}

/// Convert a month abbreviation to a number.
fn parse_mon(item: &WStr) -> Option<usize> {
    const MONTHS: [&[u8]; 12] = [
        b"Jan", b"Feb", b"Mar", b"Apr", b"May", b"Jun", b"Jul", b"Aug", b"Sep", b"Oct", b"Nov",
        b"Dec",
    ];
    MONTHS.iter().position(|&x| x == item)
}

/// Parses HH:MM:SS. The output is always (hours, minutes, seconds), or None if format was invalid.
fn parse_hms(item: &WStr) -> Option<(u32, u32, u32)> {
    let mut iter = item.split(b':');
    let hours = iter.next()?;
    if hours.len() != 2 {
        return None;
    }
    let minutes = iter.next()?;
    if minutes.len() != 2 {
        return None;
    }
    let seconds = iter.next()?;
    if seconds.len() != 2 {
        return None;
    }
    if iter.next().is_some() {
        // the iterator should have been empty
        return None;
    }
    Some((
        hours.parse::<u32>().ok()?,
        minutes.parse::<u32>().ok()?,
        seconds.parse::<u32>().ok()?,
    ))
}

pub fn parse_full_date<'gc>(
    activation: &mut Activation<'_, 'gc>,
    date: AvmString<'gc>,
) -> Option<f64> {
    const DAYS: [&[u8]; 7] = [b"Sun", b"Mon", b"Tue", b"Wed", b"Thu", b"Fri", b"Sat"];

    let timezone = activation.context.locale.get_timezone();
    let mut final_time = ParsedDateFields::default();
    let mut new_timezone = None;
    // The Date parser is flash is super flexible, so we need to go through each item individually and parse it to match Flash.
    // NOTE: DateTime::parse_from_str is not flexible enough for this, so we need to parse manually.
    for item in date
        .split(string_utils::swf_is_whitespace)
        .filter(|s| !s.is_empty())
    {
        if let Some((year, month, day)) = parse_date(item) {
            // Parse YYYY/MM/DD, MM/DD/YYYY, Mon/DD/YYYY

            // First we check if the fields we are going to set have already been set, if they are, we return NaN.
            // The same logic applies for all other if/else branches.
            if final_time.year.is_some() || final_time.month.is_some() || final_time.day.is_some() {
                return None;
            }
            final_time.year = Some(year as f64);
            final_time.month = Some(month as f64);
            final_time.day = Some(day as f64);
        } else if let Some((hours, minutes, seconds)) = parse_hms(item) {
            // Parse HH:MM:SS

            if final_time.hour.is_some()
                || final_time.minute.is_some()
                || final_time.second.is_some()
            {
                return None;
            }
            final_time.hour = Some(hours as f64);
            final_time.minute = Some(minutes as f64);
            final_time.second = Some(seconds as f64);
        } else if DAYS.iter().any(|&d| d == item) {
            // Parse abbreviated weekname (Sun, Mon, etc...)
            // DO NOTHING
        } else if let Some(month) = parse_mon(item) {
            // Parse abbreviated month name (Jan, Feb, etc...)
            final_time.month = Some(month as f64);
        } else if item.starts_with(WStr::from_units(b"GMT"))
            || item.starts_with(WStr::from_units(b"UTC"))
        {
            // Parse GMT-HHMM/GMT+HHMM or UTC-HHMM/UTC+HHMM
            // Also handle standalone GMT/UTC (means UTC+0000)

            if new_timezone.is_some() {
                return None;
            }

            if item == b"GMT" || item == b"UTC" {
                new_timezone = Some(FixedOffset::east_opt(0).expect("UTC offset should be valid"));
            } else if item.len() == 8 {
                let (other, tzn) = item.split_at(4);
                if tzn.len() != 4 {
                    return None;
                }
                let (hours, minutes) = tzn.split_at(2);
                let hours = hours.parse::<u32>().ok()?;
                let minutes = minutes.parse::<u32>().ok()?;
                let sign = other.at(3);
                // NOTE: In real flash, invalid (out of bounds) timezones were allowed, but there isn't a way to construct these using FixedOffset.
                // Since it is insanely rare to ever parse a date with an invalid timezone, for now we just return an error.
                new_timezone = Some(if sign == b'-' as u16 {
                    FixedOffset::west_opt(((hours * 60 * 60) + minutes * 60) as i32)?
                } else if sign == b'+' as u16 {
                    FixedOffset::east_opt(((hours * 60 * 60) + minutes * 60) as i32)?
                } else {
                    return None;
                });
            } else {
                return None;
            }
        } else if let Ok(mut num) = item.parse::<u32>() {
            // Parse either a day or a year

            // If the number is greater than 70, lets parse as a year
            if num >= 70 {
                if final_time.year.is_some() {
                    return None;
                }
                // If the number is less than 100, we add 1900 to it.
                if num < 100 {
                    num += 1900;
                }
                final_time.year = Some(num as f64);
            // Otherwise, lets parse as a day
            } else {
                if final_time.day.is_some() {
                    return None;
                }
                final_time.day = Some(num as f64)
            }
        } else {
            return None;
        }
    }
    // It is required that year, month, and day all have data.
    if final_time.year.is_none() || final_time.month.is_none() || final_time.day.is_none() {
        return None;
    }
    let timezone = new_timezone.unwrap_or(timezone);
    let local = Date::make_date(
        Date::make_day(final_time.year?, final_time.month?, final_time.day?),
        Date::make_time(
            final_time.hour.unwrap_or(0.0),
            final_time.minute.unwrap_or(0.0),
            final_time.second.unwrap_or(0.0),
            0.0,
        ),
    );

    Some(local.time() - f64::from(timezone.local_minus_utc()) * 1000.0)
}

/// Implements the `parse` class method.
pub fn parse<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let date_str = args.get_value(0).coerce_to_string(activation)?;

    Ok(parse_full_date(activation, date_str)
        .unwrap_or(f64::NAN)
        .into())
}
