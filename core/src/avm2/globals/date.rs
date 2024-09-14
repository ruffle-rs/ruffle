//! `Date` class

use crate::avm2::activation::Activation;
pub use crate::avm2::object::date_allocator;
use crate::avm2::object::{DateObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::locale::{get_current_date_time, get_timezone};
use crate::string::{utils as string_utils, AvmString, WStr};
use chrono::{DateTime, Datelike, Duration, FixedOffset, LocalResult, TimeZone, Timelike, Utc};
use num_traits::ToPrimitive;

struct DateAdjustment<'builder, 'activation_a: 'builder, 'gc: 'activation_a, T: TimeZone + 'builder>
{
    activation: &'builder mut Activation<'activation_a, 'gc>,
    timezone: &'builder T,
    year: Option<Option<f64>>,
    month: Option<Option<f64>>,
    day: Option<Option<f64>>,
    hour: Option<Option<f64>>,
    minute: Option<Option<f64>>,
    second: Option<Option<f64>>,
    millisecond: Option<Option<f64>>,
}

impl<'builder, 'activation_a, 'gc, T: TimeZone> DateAdjustment<'builder, 'activation_a, 'gc, T> {
    fn new(
        activation: &'builder mut Activation<'activation_a, 'gc>,
        timezone: &'builder T,
    ) -> Self {
        Self {
            activation,
            timezone,
            year: None,
            month: None,
            day: None,
            hour: None,
            minute: None,
            second: None,
            millisecond: None,
        }
    }

    fn map_year(&mut self, data_fn: impl Fn(f64) -> f64) -> &mut Self {
        if let Some(year) = self.year.flatten() {
            self.year = Some(Some(data_fn(year)));
        }
        self
    }

    fn year(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error<'gc>> {
        self.year = match value {
            Some(value) => Some(Some(value.coerce_to_number(self.activation)?)),
            None => None,
        };
        Ok(self)
    }

    fn month(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error<'gc>> {
        self.month = match value {
            Some(value) => Some(Some(value.coerce_to_number(self.activation)?)),
            None => None,
        };
        Ok(self)
    }

    fn day(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error<'gc>> {
        self.day = match value {
            Some(value) => Some(Some(value.coerce_to_number(self.activation)?)),
            None => None,
        };
        Ok(self)
    }

    fn hour(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error<'gc>> {
        self.hour = match value {
            Some(value) => Some(Some(value.coerce_to_number(self.activation)?)),
            None => None,
        };
        Ok(self)
    }

    fn minute(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error<'gc>> {
        self.minute = match value {
            Some(value) => Some(Some(value.coerce_to_number(self.activation)?)),
            None => None,
        };
        Ok(self)
    }

    fn second(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error<'gc>> {
        self.second = match value {
            Some(value) => Some(Some(value.coerce_to_number(self.activation)?)),
            None => None,
        };
        Ok(self)
    }

    fn millisecond(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error<'gc>> {
        self.millisecond = match value {
            Some(value) => Some(Some(value.coerce_to_number(self.activation)?)),
            None => None,
        };
        Ok(self)
    }

    fn check_value(
        &self,
        specified: Option<Option<f64>>,
        current: impl ToPrimitive,
    ) -> Option<i64> {
        match specified {
            Some(Some(value)) if value.is_finite() => Some(value as i64),
            Some(_) => None,
            None => current.to_i64(),
        }
    }

    fn check_mapped_value(
        &self,
        specified: Option<Option<f64>>,
        map: impl FnOnce(i64) -> i64,
        current: impl ToPrimitive,
    ) -> Option<i64> {
        match specified {
            Some(Some(value)) if value.is_finite() => Some(map(value as i64)),
            Some(_) => None,
            None => current.to_i64(),
        }
    }

    fn calculate(&mut self, current: DateTime<T>) -> Option<DateTime<Utc>> {
        let month_rem = self
            .month
            .flatten()
            .map(|v| v as i64)
            .unwrap_or_default()
            .div_euclid(12);
        let month = self.check_mapped_value(self.month, |v| v.rem_euclid(12), current.month0())?;
        let year = self
            .check_value(self.year, current.year())?
            .wrapping_add(month_rem) as i32;
        let day = self.check_value(self.day, current.day())?;
        let hour = self.check_value(self.hour, current.hour())?;
        let minute = self.check_value(self.minute, current.minute())?;
        let second = self.check_value(self.second, current.second())?;
        let millisecond = self.check_value(self.millisecond, current.timestamp_subsec_millis())?;

        let duration = Duration::try_days(day - 1)?
            + Duration::try_hours(hour)?
            + Duration::try_minutes(minute)?
            + Duration::try_seconds(second)?
            + Duration::try_milliseconds(millisecond)?;

        if let LocalResult::Single(Some(result)) = current
            .timezone()
            .with_ymd_and_hms(year, (month + 1) as u32, 1, 0, 0, 0)
            .map(|date| date.checked_add_signed(duration))
        {
            Some(result.with_timezone(&Utc))
        } else {
            None
        }
    }

    fn apply(&mut self, object: DateObject<'gc>) -> f64 {
        let date = if let Some(current) = object.date_time().map(|v| v.with_timezone(self.timezone))
        {
            self.calculate(current)
        } else {
            None
        };
        object.set_date_time(date);
        if let Some(date) = date {
            date.timestamp_millis() as f64
        } else {
            f64::NAN
        }
    }
}

fn get_arguments_array<'gc>(args: &[Value<'gc>]) -> Vec<Value<'gc>> {
    let object = args[0].as_object().unwrap();
    let array_storage = object.as_array_storage().unwrap();
    array_storage
        .iter()
        .map(|v| v.unwrap()) // Arguments should be array with no holes
        .collect()
}

/// Implements `Date`'s instance constructor.
pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();
    let arguments = get_arguments_array(args);

    let timestamp = arguments.get(0).unwrap_or(&Value::Undefined);
    if timestamp != &Value::Undefined {
        if arguments.len() > 1 {
            let timezone = get_timezone();

            // We need a starting value to adjust from.
            this.set_date_time(Some(
                timezone
                    .with_ymd_and_hms(0, 1, 1, 0, 0, 0)
                    .single()
                    .expect("Found ambiguous epoch time when constructing Date")
                    .into(),
            ));

            DateAdjustment::new(activation, &timezone)
                .year(arguments.get(0))?
                .month(arguments.get(1))?
                .day(arguments.get(2))?
                .hour(arguments.get(3))?
                .minute(arguments.get(4))?
                .second(arguments.get(5))?
                .millisecond(arguments.get(6))?
                .map_year(|year| if year < 100.0 { year + 1900.0 } else { year })
                .apply(this);
        } else {
            let timestamp = if let Value::String(date_str) = timestamp {
                parse_full_date(activation, *date_str).unwrap_or(f64::NAN)
            } else {
                timestamp.coerce_to_number(activation)?
            };
            if timestamp.is_finite() {
                if let LocalResult::Single(time) = Utc.timestamp_millis_opt(timestamp as i64) {
                    this.set_date_time(Some(time))
                }
            }
        }
    } else {
        this.set_date_time(Some(get_current_date_time()))
    }

    Ok(Value::Undefined)
}

pub fn call_handler<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation
        .avm2()
        .classes()
        .date
        .construct(activation, &[])?
        .into())
}

/// Implements `getTime` method.
pub fn get_time<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();
    this.value_of(activation.context.gc_context)
}

/// Implements `setTime` method.
pub fn set_time<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();

    let new_time = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_number(activation)?;
    if new_time.is_finite() {
        let time = Utc
            .timestamp_millis_opt(new_time as i64)
            .single()
            .expect("Found ambiguous timestamp for current time zone");
        this.set_date_time(Some(time));
        Ok((time.timestamp_millis() as f64).into())
    } else {
        this.set_date_time(None);
        Ok(f64::NAN.into())
    }
}

/// Implements the `getMilliseconds` method.
pub fn get_milliseconds<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();

    if let Some(date) = this
        .date_time()
        .map(|date| date.with_timezone(&get_timezone()))
    {
        Ok((date.timestamp_subsec_millis() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `setMilliseconds` method.
pub fn _set_milliseconds<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();
    let args = get_arguments_array(args);

    let timestamp = DateAdjustment::new(activation, &get_timezone())
        .millisecond(args.get(0))?
        .apply(this);
    Ok(timestamp.into())
}

/// Implements the `getSeconds` method.
pub fn get_seconds<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();

    if let Some(date) = this
        .date_time()
        .map(|date| date.with_timezone(&get_timezone()))
    {
        Ok((date.second() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements `setSeconds` method.
pub fn _set_seconds<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();
    let args = get_arguments_array(args);

    let timestamp = DateAdjustment::new(activation, &get_timezone())
        .second(args.get(0))?
        .millisecond(args.get(1))?
        .apply(this);
    Ok(timestamp.into())
}

/// Implements `getMinutes` method.
pub fn get_minutes<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();

    if let Some(date) = this
        .date_time()
        .map(|date| date.with_timezone(&get_timezone()))
    {
        Ok((date.minute() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `setMinutes` method.
pub fn _set_minutes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();
    let args = get_arguments_array(args);

    let timestamp = DateAdjustment::new(activation, &get_timezone())
        .minute(args.get(0))?
        .second(args.get(1))?
        .millisecond(args.get(2))?
        .apply(this);
    Ok(timestamp.into())
}

/// Implements the `getHours` method.
pub fn get_hours<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();

    if let Some(date) = this
        .date_time()
        .map(|date| date.with_timezone(&get_timezone()))
    {
        Ok((date.hour() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements `setHours` method.
pub fn _set_hours<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();
    let args = get_arguments_array(args);

    let timestamp = DateAdjustment::new(activation, &get_timezone())
        .hour(args.get(0))?
        .minute(args.get(1))?
        .second(args.get(2))?
        .millisecond(args.get(3))?
        .apply(this);
    Ok(timestamp.into())
}

/// Implements `getDate` method.
pub fn get_date<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();

    if let Some(date) = this
        .date_time()
        .map(|date| date.with_timezone(&get_timezone()))
    {
        Ok((date.day() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements `setDate` method.
pub fn _set_date<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();
    let args = get_arguments_array(args);

    let timestamp = DateAdjustment::new(activation, &get_timezone())
        .day(args.get(0))?
        .apply(this);
    Ok(timestamp.into())
}

/// Implements the `getMonth` method.
pub fn get_month<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();

    if let Some(date) = this
        .date_time()
        .map(|date| date.with_timezone(&get_timezone()))
    {
        Ok((date.month0() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `setMonth` method.
pub fn _set_month<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();
    let args = get_arguments_array(args);

    let timestamp = DateAdjustment::new(activation, &get_timezone())
        .month(args.get(0))?
        .day(args.get(1))?
        .apply(this);
    Ok(timestamp.into())
}

/// Implements the `getFullYear` method.
pub fn get_full_year<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();

    if let Some(date) = this
        .date_time()
        .map(|date| date.with_timezone(&get_timezone()))
    {
        Ok((date.year() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `setFullYear` method.
pub fn _set_full_year<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();
    let args = get_arguments_array(args);

    let timezone = get_timezone();
    if this.date_time().is_none() {
        this.set_date_time(Some(
            timezone
                .with_ymd_and_hms(0, 1, 1, 0, 0, 0)
                .single()
                .expect("Found ambiguous epoch time when constructing Date")
                .into(),
        ));
    }
    let timestamp = DateAdjustment::new(activation, &timezone)
        .year(args.get(0))?
        .month(args.get(1))?
        .day(args.get(2))?
        .apply(this);
    Ok(timestamp.into())
}

/// Implements the `getDay` method.
pub fn get_day<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();

    if let Some(date) = this
        .date_time()
        .map(|date| date.with_timezone(&get_timezone()))
    {
        Ok((date.weekday().num_days_from_sunday() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `getUTCMilliseconds` method.
pub fn get_utc_milliseconds<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();

    if let Some(date) = this.date_time() {
        Ok((date.timestamp_subsec_millis() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `setUTCMilliseconds` method.
pub fn _set_utc_milliseconds<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();
    let args = get_arguments_array(args);

    let timestamp = DateAdjustment::new(activation, &Utc)
        .millisecond(args.get(0))?
        .apply(this);
    Ok(timestamp.into())
}

/// Implements the `getUTCSeconds` method.
pub fn get_utc_seconds<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();

    if let Some(date) = this.date_time() {
        Ok((date.second() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `setUTCSeconds` method.
pub fn _set_utc_seconds<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();
    let args = get_arguments_array(args);

    let timestamp = DateAdjustment::new(activation, &Utc)
        .second(args.get(0))?
        .millisecond(args.get(1))?
        .apply(this);
    Ok(timestamp.into())
}

/// Implements the `getUTCMinutes` method.
pub fn get_utc_minutes<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();

    if let Some(date) = this.date_time() {
        Ok((date.minute() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `setUTCMinutes` method.
pub fn _set_utc_minutes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();
    let args = get_arguments_array(args);

    let timestamp = DateAdjustment::new(activation, &Utc)
        .minute(args.get(0))?
        .second(args.get(1))?
        .millisecond(args.get(2))?
        .apply(this);
    Ok(timestamp.into())
}

/// Implements the `getUTCHours` method.
pub fn get_utc_hours<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();

    if let Some(date) = this.date_time() {
        Ok((date.hour() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `setUTCHours` method.
pub fn _set_utc_hours<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();
    let args = get_arguments_array(args);

    let timestamp = DateAdjustment::new(activation, &Utc)
        .hour(args.get(0))?
        .minute(args.get(1))?
        .second(args.get(2))?
        .millisecond(args.get(3))?
        .apply(this);
    Ok(timestamp.into())
}

/// Implements the `getUTCDate` method.
pub fn get_utc_date<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();

    if let Some(date) = this.date_time() {
        Ok((date.day() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `setUTCDate` method.
pub fn _set_utc_date<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();
    let args = get_arguments_array(args);

    let timestamp = DateAdjustment::new(activation, &Utc)
        .day(args.get(0))?
        .apply(this);
    Ok(timestamp.into())
}

/// Implements the `getUTCMonth` method.
pub fn get_utc_month<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();

    if let Some(date) = this.date_time() {
        Ok((date.month0() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `setUTCMonth` method.
pub fn _set_utc_month<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();
    let args = get_arguments_array(args);

    let timestamp = DateAdjustment::new(activation, &Utc)
        .month(args.get(0))?
        .day(args.get(1))?
        .apply(this);
    Ok(timestamp.into())
}

/// Implements the `getUTCFullYear` method.
pub fn get_utc_full_year<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();

    if let Some(date) = this.date_time() {
        Ok((date.year() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `setUTCFullYear` method.
pub fn _set_utc_full_year<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();
    let args = get_arguments_array(args);

    if this.date_time().is_none() {
        this.set_date_time(Some(
            Utc.with_ymd_and_hms(0, 1, 1, 0, 0, 0)
                .single()
                .expect("Found ambiguous epoch time when constructing Date"),
        ));
    }
    let timestamp = DateAdjustment::new(activation, &Utc)
        .year(args.get(0))?
        .month(args.get(1))?
        .day(args.get(2))?
        .apply(this);
    Ok(timestamp.into())
}

/// Implements the `getUTCDay` method.
pub fn get_utc_day<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();

    if let Some(date) = this.date_time() {
        Ok((date.weekday().num_days_from_sunday() as f64).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `getTimezoneOffset` method.
pub fn get_timezone_offset<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();

    if let Some(date) = this
        .date_time()
        .map(|date| date.with_timezone(&get_timezone()))
    {
        let offset = date.offset().utc_minus_local() as f64;
        Ok((offset / 60.0).into())
    } else {
        Ok(f64::NAN.into())
    }
}

/// Implements the `UTC` class method.
pub fn utc<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let date = DateAdjustment::new(activation, &Utc)
        .year(args.get(0))?
        .month(args.get(1))?
        .day(args.get(2))?
        .hour(args.get(3))?
        .minute(args.get(4))?
        .second(args.get(5))?
        .millisecond(args.get(6))?
        .map_year(|year| if year < 100.0 { year + 1900.0 } else { year })
        .calculate(
            Utc.with_ymd_and_hms(0, 1, 1, 0, 0, 0)
                .single()
                .expect("Found ambiguous UTC time conversions"),
        );
    let millis = if let Some(date) = date {
        date.timestamp_millis() as f64
    } else {
        f64::NAN
    };

    Ok(millis.into())
}

/// Implements the `toString` method.
pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();

    if let Some(date) = this
        .date_time()
        .map(|date| date.with_timezone(&get_timezone()))
    {
        Ok(AvmString::new_utf8(
            activation.context.gc_context,
            date.format("%a %b %-d %T GMT%z %-Y").to_string(),
        )
        .into())
    } else {
        Ok("Invalid Date".into())
    }
}

/// Implements the `toUTCString` method.
pub fn to_utc_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();

    if let Some(date) = this.date_time() {
        Ok(AvmString::new_utf8(
            activation.context.gc_context,
            date.format("%a %b %-d %T %-Y UTC").to_string(),
        )
        .into())
    } else {
        Ok("Invalid Date".into())
    }
}

/// Implements the `toLocaleString` method.
pub fn to_locale_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();

    if let Some(date) = this
        .date_time()
        .map(|date| date.with_timezone(&get_timezone()))
    {
        Ok(AvmString::new_utf8(
            activation.context.gc_context,
            date.format("%a %b %-d %-Y %T %p").to_string(),
        )
        .into())
    } else {
        Ok("Invalid Date".into())
    }
}

/// Implements the `toTimeString` method.
pub fn to_time_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();

    if let Some(date) = this
        .date_time()
        .map(|date| date.with_timezone(&get_timezone()))
    {
        Ok(AvmString::new_utf8(
            activation.context.gc_context,
            date.format("%T GMT%z").to_string(),
        )
        .into())
    } else {
        Ok("Invalid Date".into())
    }
}

/// Implements the `toLocaleTimeString` method.
pub fn to_locale_time_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();

    if let Some(date) = this
        .date_time()
        .map(|date| date.with_timezone(&get_timezone()))
    {
        Ok(AvmString::new_utf8(
            activation.context.gc_context,
            date.format("%T %p").to_string(),
        )
        .into())
    } else {
        Ok("Invalid Date".into())
    }
}

/// Implements the `toDateString` & `toLocaleDateString` method.
pub fn to_date_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_date_object().unwrap();

    if let Some(date) = this
        .date_time()
        .map(|date| date.with_timezone(&get_timezone()))
    {
        Ok(AvmString::new_utf8(
            activation.context.gc_context,
            date.format("%a %b %-d %-Y").to_string(),
        )
        .into())
    } else {
        Ok("Invalid Date".into())
    }
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

    let timezone = get_timezone();
    let mut final_time = DateAdjustment::new(activation, &timezone);
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
            final_time.year = Some(Some(year as f64));
            final_time.month = Some(Some(month as f64));
            final_time.day = Some(Some(day as f64));
        } else if let Some((hours, minutes, seconds)) = parse_hms(item) {
            // Parse HH:MM:SS

            if final_time.hour.is_some()
                || final_time.minute.is_some()
                || final_time.second.is_some()
            {
                return None;
            }
            final_time.hour = Some(Some(hours as f64));
            final_time.minute = Some(Some(minutes as f64));
            final_time.second = Some(Some(seconds as f64));
        } else if DAYS.iter().any(|&d| d == item) {
            // Parse abbreviated weekname (Sun, Mon, etc...)
            // DO NOTHING
        } else if let Some(month) = parse_mon(item) {
            // Parse abbreviated month name (Jan, Feb, etc...)
            final_time.month = Some(Some(month as f64));
        } else if item.starts_with(WStr::from_units(b"GMT"))
            || item.starts_with(WStr::from_units(b"UTC"))
        {
            // Parse GMT-HHMM/GMT+HHMM or UTC-HHMM/UTC+HHMM

            if new_timezone.is_some() || item.len() != 8 {
                return None;
            }
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
                final_time.year = Some(Some(num as f64));
            // Otherwise, lets parse as a day
            } else {
                if final_time.day.is_some() {
                    return None;
                }
                final_time.day = Some(Some(num as f64))
            }
        } else {
            return None;
        }
    }
    // It is required that year, month, and day all have data.
    if final_time.year.is_none() || final_time.month.is_none() || final_time.day.is_none() {
        return None;
    }
    if let Some(timestamp) = final_time.calculate(
        new_timezone
            .unwrap_or(timezone)
            .with_ymd_and_hms(0, 1, 1, 0, 0, 0)
            .single()
            .expect(
                "Found ambiguous starting time when converting parsed dates into local timezone",
            ),
    ) {
        Some(timestamp.timestamp_millis() as f64)
    } else {
        None
    }
}

/// Implements the `parse` class method.
#[allow(clippy::question_mark)]
pub fn parse<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let date_str = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;

    Ok(parse_full_date(activation, date_str)
        .unwrap_or(f64::NAN)
        .into())
}
