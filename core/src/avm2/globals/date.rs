//! `Date` class

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::{date_allocator, DateObject, FunctionObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::QName;
use crate::locale::{get_current_date_time, get_timezone};
use crate::string::{utils as string_utils, AvmString, WStr};
use chrono::{DateTime, Datelike, Duration, FixedOffset, LocalResult, TimeZone, Timelike, Utc};
use num_traits::ToPrimitive;

// All of these methods will be defined as both
// AS3 instance methods and methods on the `Date` class prototype.
const PUBLIC_INSTANCE_AND_PROTO_METHODS: &[(&str, NativeMethodImpl)] = &[
    ("getTime", time),
    ("setTime", set_time),
    ("getMilliseconds", milliseconds),
    ("setMilliseconds", set_milliseconds),
    ("getSeconds", seconds),
    ("setSeconds", set_seconds),
    ("getMinutes", minutes),
    ("setMinutes", set_minutes),
    ("getHours", hours),
    ("setHours", set_hours),
    ("getDate", date),
    ("setDate", set_date),
    ("getMonth", month),
    ("setMonth", set_month),
    ("getFullYear", full_year),
    ("setFullYear", set_full_year),
    ("getDay", day),
    ("getUTCMilliseconds", milliseconds_utc),
    ("setUTCMilliseconds", set_milliseconds_utc),
    ("getUTCSeconds", seconds_utc),
    ("setUTCSeconds", set_seconds_utc),
    ("getUTCMinutes", minutes_utc),
    ("setUTCMinutes", set_minutes_utc),
    ("getUTCHours", hours_utc),
    ("setUTCHours", set_hours_utc),
    ("getUTCDate", date_utc),
    ("setUTCDate", set_date_utc),
    ("getUTCMonth", month_utc),
    ("setUTCMonth", set_month_utc),
    ("getUTCFullYear", full_year_utc),
    ("setUTCFullYear", set_full_year_utc),
    ("getUTCDay", day_utc),
    ("getTimezoneOffset", timezone_offset),
    ("valueOf", time),
    ("toString", to_string),
    ("toUTCString", to_utc_string),
    ("toLocaleString", to_locale_string),
    ("toTimeString", to_time_string),
    ("toLocaleTimeString", to_locale_time_string),
    ("toDateString", to_date_string),
    ("toLocaleDateString", to_date_string),
];

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

/// Implements `Date`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    activation.super_init(this, &[])?;
    if let Some(date) = this.as_date_object() {
        let timestamp = args.get(0).unwrap_or(&Value::Undefined);
        if timestamp != &Value::Undefined {
            if args.len() > 1 {
                let timezone = get_timezone();

                // We need a starting value to adjust from.
                date.set_date_time(Some(
                    timezone
                        .with_ymd_and_hms(0, 1, 1, 0, 0, 0)
                        .single()
                        .expect("Found ambiguous epoch time when constructing Date")
                        .into(),
                ));

                DateAdjustment::new(activation, &timezone)
                    .year(args.get(0))?
                    .month(args.get(1))?
                    .day(args.get(2))?
                    .hour(args.get(3))?
                    .minute(args.get(4))?
                    .second(args.get(5))?
                    .millisecond(args.get(6))?
                    .map_year(|year| if year < 100.0 { year + 1900.0 } else { year })
                    .apply(date);
            } else {
                let timestamp = if let Value::String(date_str) = timestamp {
                    parse_full_date(activation, *date_str).unwrap_or(f64::NAN)
                } else {
                    timestamp.coerce_to_number(activation)?
                };
                if timestamp.is_finite() {
                    if let LocalResult::Single(time) = Utc.timestamp_millis_opt(timestamp as i64) {
                        date.set_date_time(Some(time))
                    }
                }
            }
        } else {
            date.set_date_time(Some(get_current_date_time()))
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Date`'s class constructor.
pub fn class_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let scope = activation.create_scopechain();
    let gc_context = activation.context.gc_context;
    let this_class = this.as_class_object().unwrap();
    let date_proto = this_class.prototype();

    for (name, method) in PUBLIC_INSTANCE_AND_PROTO_METHODS {
        date_proto.set_string_property_local(
            *name,
            FunctionObject::from_method(
                activation,
                Method::from_builtin(*method, name, gc_context),
                scope,
                None,
                Some(this_class),
            )
            .into(),
            activation,
        )?;
        date_proto.set_local_property_is_enumerable(gc_context, (*name).into(), false);
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

/// Implements `time` property's getter, and the `getTime` method. This will also be used for `valueOf`.
pub fn time<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        return this.value_of(activation.context.gc_context);
    }

    Ok(Value::Undefined)
}

/// Implements `time` property's setter, and the `setTime` method.
pub fn set_time<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
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
            return Ok((time.timestamp_millis() as f64).into());
        } else {
            this.set_date_time(None);
            return Ok(f64::NAN.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `milliseconds` property's getter, and the `getMilliseconds` method.
pub fn milliseconds<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        if let Some(date) = this
            .date_time()
            .map(|date| date.with_timezone(&get_timezone()))
        {
            return Ok((date.timestamp_subsec_millis() as f64).into());
        } else {
            return Ok(f64::NAN.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `milliseconds` property's setter, and the `setMilliseconds` method.
pub fn set_milliseconds<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        let timestamp = DateAdjustment::new(activation, &get_timezone())
            .millisecond(args.get(0))?
            .apply(this);
        return Ok(timestamp.into());
    }
    Ok(Value::Undefined)
}

/// Implements `seconds` property's getter, and the `getSeconds` method.
pub fn seconds<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        if let Some(date) = this
            .date_time()
            .map(|date| date.with_timezone(&get_timezone()))
        {
            return Ok((date.second() as f64).into());
        } else {
            return Ok(f64::NAN.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `seconds` property's setter, and the `setSeconds` method.
pub fn set_seconds<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        let timestamp = DateAdjustment::new(activation, &get_timezone())
            .second(args.get(0))?
            .millisecond(args.get(1))?
            .apply(this);
        return Ok(timestamp.into());
    }
    Ok(Value::Undefined)
}

/// Implements `minutes` property's getter, and the `getMinutes` method.
pub fn minutes<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        if let Some(date) = this
            .date_time()
            .map(|date| date.with_timezone(&get_timezone()))
        {
            return Ok((date.minute() as f64).into());
        } else {
            return Ok(f64::NAN.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `minutes` property's setter, and the `setMinutes` method.
pub fn set_minutes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        let timestamp = DateAdjustment::new(activation, &get_timezone())
            .minute(args.get(0))?
            .second(args.get(1))?
            .millisecond(args.get(2))?
            .apply(this);
        return Ok(timestamp.into());
    }
    Ok(Value::Undefined)
}

/// Implements `hour` property's getter, and the `getHours` method.
pub fn hours<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        if let Some(date) = this
            .date_time()
            .map(|date| date.with_timezone(&get_timezone()))
        {
            return Ok((date.hour() as f64).into());
        } else {
            return Ok(f64::NAN.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `hours` property's setter, and the `setHours` method.
pub fn set_hours<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        let timestamp = DateAdjustment::new(activation, &get_timezone())
            .hour(args.get(0))?
            .minute(args.get(1))?
            .second(args.get(2))?
            .millisecond(args.get(3))?
            .apply(this);
        return Ok(timestamp.into());
    }
    Ok(Value::Undefined)
}

/// Implements `date` property's getter, and the `getDate` method.
pub fn date<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        if let Some(date) = this
            .date_time()
            .map(|date| date.with_timezone(&get_timezone()))
        {
            return Ok((date.day() as f64).into());
        } else {
            return Ok(f64::NAN.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `date` property's setter, and the `setDate` method.
pub fn set_date<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        let timestamp = DateAdjustment::new(activation, &get_timezone())
            .day(args.get(0))?
            .apply(this);
        return Ok(timestamp.into());
    }
    Ok(Value::Undefined)
}

/// Implements `month` property's getter, and the `getMonth` method.
pub fn month<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        if let Some(date) = this
            .date_time()
            .map(|date| date.with_timezone(&get_timezone()))
        {
            return Ok((date.month0() as f64).into());
        } else {
            return Ok(f64::NAN.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `month` property's setter, and the `setMonth` method.
pub fn set_month<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        let timestamp = DateAdjustment::new(activation, &get_timezone())
            .month(args.get(0))?
            .day(args.get(1))?
            .apply(this);
        return Ok(timestamp.into());
    }
    Ok(Value::Undefined)
}

/// Implements `fullYear` property's getter, and the `getFullYear` method.
pub fn full_year<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        if let Some(date) = this
            .date_time()
            .map(|date| date.with_timezone(&get_timezone()))
        {
            return Ok((date.year() as f64).into());
        } else {
            return Ok(f64::NAN.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `fullYear` property's setter, and the `setFullYear` method.
pub fn set_full_year<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
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
        return Ok(timestamp.into());
    }
    Ok(Value::Undefined)
}

/// Implements `day` property's getter, and the `getDay` method.
pub fn day<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        if let Some(date) = this
            .date_time()
            .map(|date| date.with_timezone(&get_timezone()))
        {
            return Ok((date.weekday().num_days_from_sunday() as f64).into());
        } else {
            return Ok(f64::NAN.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `millisecondsUTC` property's getter, and the `getUTCMilliseconds` method.
pub fn milliseconds_utc<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        if let Some(date) = this.date_time() {
            return Ok((date.timestamp_subsec_millis() as f64).into());
        } else {
            return Ok(f64::NAN.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `millisecondsUTC` property's setter, and the `setUTCMilliseconds` method.
pub fn set_milliseconds_utc<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        let timestamp = DateAdjustment::new(activation, &Utc)
            .millisecond(args.get(0))?
            .apply(this);
        return Ok(timestamp.into());
    }
    Ok(Value::Undefined)
}

/// Implements `secondsUTC` property's getter, and the `getUTCSeconds` method.
pub fn seconds_utc<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        if let Some(date) = this.date_time() {
            return Ok((date.second() as f64).into());
        } else {
            return Ok(f64::NAN.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `secondsUTC` property's setter, and the `setUTCSeconds` method.
pub fn set_seconds_utc<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        let timestamp = DateAdjustment::new(activation, &Utc)
            .second(args.get(0))?
            .millisecond(args.get(1))?
            .apply(this);
        return Ok(timestamp.into());
    }
    Ok(Value::Undefined)
}

/// Implements `minutesUTC` property's getter, and the `getUTCMinutes` method.
pub fn minutes_utc<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        if let Some(date) = this.date_time() {
            return Ok((date.minute() as f64).into());
        } else {
            return Ok(f64::NAN.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `minutesUTC` property's setter, and the `setUTCMinutes` method.
pub fn set_minutes_utc<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        let timestamp = DateAdjustment::new(activation, &Utc)
            .minute(args.get(0))?
            .second(args.get(1))?
            .millisecond(args.get(2))?
            .apply(this);
        return Ok(timestamp.into());
    }
    Ok(Value::Undefined)
}

/// Implements `hourUTC` property's getter, and the `getUTCHours` method.
pub fn hours_utc<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        if let Some(date) = this.date_time() {
            return Ok((date.hour() as f64).into());
        } else {
            return Ok(f64::NAN.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `hoursUTC` property's setter, and the `setUTCHours` method.
pub fn set_hours_utc<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        let timestamp = DateAdjustment::new(activation, &Utc)
            .hour(args.get(0))?
            .minute(args.get(1))?
            .second(args.get(2))?
            .millisecond(args.get(3))?
            .apply(this);
        return Ok(timestamp.into());
    }
    Ok(Value::Undefined)
}

/// Implements `dateUTC` property's getter, and the `getUTCDate` method.
pub fn date_utc<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        if let Some(date) = this.date_time() {
            return Ok((date.day() as f64).into());
        } else {
            return Ok(f64::NAN.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `dateUTC` property's setter, and the `setUTCDate` method.
pub fn set_date_utc<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        let timestamp = DateAdjustment::new(activation, &Utc)
            .day(args.get(0))?
            .apply(this);
        return Ok(timestamp.into());
    }
    Ok(Value::Undefined)
}

/// Implements `monthUTC` property's getter, and the `getUTCMonth` method.
pub fn month_utc<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        if let Some(date) = this.date_time() {
            return Ok((date.month0() as f64).into());
        } else {
            return Ok(f64::NAN.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `monthUTC` property's setter, and the `setUTCMonth` method.
pub fn set_month_utc<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        let timestamp = DateAdjustment::new(activation, &Utc)
            .month(args.get(0))?
            .day(args.get(1))?
            .apply(this);
        return Ok(timestamp.into());
    }
    Ok(Value::Undefined)
}

/// Implements `fullYearUTC` property's getter, and the `getUTCFullYear` method.
pub fn full_year_utc<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        if let Some(date) = this.date_time() {
            return Ok((date.year() as f64).into());
        } else {
            return Ok(f64::NAN.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `fullYearUTC` property's setter, and the `setUTCFullYear` method.
pub fn set_full_year_utc<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
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
        return Ok(timestamp.into());
    }
    Ok(Value::Undefined)
}

/// Implements `dayUTC` property's getter, and the `getUTCDay` method.
pub fn day_utc<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        if let Some(date) = this.date_time() {
            return Ok((date.weekday().num_days_from_sunday() as f64).into());
        } else {
            return Ok(f64::NAN.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `timezoneOffset` property's getter, and the `getTimezoneOffset` method.
pub fn timezone_offset<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        if let Some(date) = this
            .date_time()
            .map(|date| date.with_timezone(&get_timezone()))
        {
            let offset = date.offset().utc_minus_local() as f64;
            return Ok((offset / 60.0).into());
        } else {
            return Ok(f64::NAN.into());
        }
    }

    Ok(Value::Undefined)
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
    if let Some(this) = this.as_date_object() {
        if let Some(date) = this
            .date_time()
            .map(|date| date.with_timezone(&get_timezone()))
        {
            return Ok(AvmString::new_utf8(
                activation.context.gc_context,
                date.format("%a %b %-d %T GMT%z %-Y").to_string(),
            )
            .into());
        } else {
            return Ok("Invalid Date".into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements the `toUTCString` method.
pub fn to_utc_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        if let Some(date) = this.date_time() {
            return Ok(AvmString::new_utf8(
                activation.context.gc_context,
                date.format("%a %b %-d %T %-Y UTC").to_string(),
            )
            .into());
        } else {
            return Ok("Invalid Date".into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements the `toLocaleString` method.
pub fn to_locale_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        if let Some(date) = this
            .date_time()
            .map(|date| date.with_timezone(&get_timezone()))
        {
            return Ok(AvmString::new_utf8(
                activation.context.gc_context,
                date.format("%a %b %-d %-Y %T %p").to_string(),
            )
            .into());
        } else {
            return Ok("Invalid Date".into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements the `toTimeString` method.
pub fn to_time_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        if let Some(date) = this
            .date_time()
            .map(|date| date.with_timezone(&get_timezone()))
        {
            return Ok(AvmString::new_utf8(
                activation.context.gc_context,
                date.format("%T GMT%z").to_string(),
            )
            .into());
        } else {
            return Ok("Invalid Date".into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements the `toLocaleTimeString` method.
pub fn to_locale_time_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        if let Some(date) = this
            .date_time()
            .map(|date| date.with_timezone(&get_timezone()))
        {
            return Ok(AvmString::new_utf8(
                activation.context.gc_context,
                date.format("%T %p").to_string(),
            )
            .into());
        } else {
            return Ok("Invalid Date".into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements the `toDateString` & `toLocaleDateString` method.
pub fn to_date_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_date_object() {
        if let Some(date) = this
            .date_time()
            .map(|date| date.with_timezone(&get_timezone()))
        {
            return Ok(AvmString::new_utf8(
                activation.context.gc_context,
                date.format("%a %b %-d %-Y").to_string(),
            )
            .into());
        } else {
            return Ok("Invalid Date".into());
        }
    }

    Ok(Value::Undefined)
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

/// Construct `Date`'s class.
pub fn create_class<'gc>(activation: &mut Activation<'_, 'gc>) -> Class<'gc> {
    let mc = activation.context.gc_context;
    let class = Class::new(
        QName::new(activation.avm2().public_namespace_base_version, "Date"),
        Some(activation.avm2().classes().object.inner_class_definition()),
        Method::from_builtin(instance_init, "<Date instance initializer>", mc),
        Method::from_builtin(class_init, "<Date class initializer>", mc),
        activation.avm2().classes().class.inner_class_definition(),
        mc,
    );

    class.set_instance_allocator(mc, date_allocator);
    class.set_call_handler(
        mc,
        Method::from_builtin(call_handler, "<Date call handler>", mc),
    );

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("time", Some(time), Some(set_time)),
        ("milliseconds", Some(milliseconds), Some(set_milliseconds)),
        ("seconds", Some(seconds), Some(set_seconds)),
        ("minutes", Some(minutes), Some(set_minutes)),
        ("hours", Some(hours), Some(set_hours)),
        ("date", Some(date), Some(set_date)),
        ("month", Some(month), Some(set_month)),
        ("fullYear", Some(full_year), Some(set_full_year)),
        (
            "millisecondsUTC",
            Some(milliseconds_utc),
            Some(set_milliseconds_utc),
        ),
        ("day", Some(day), None),
        ("secondsUTC", Some(seconds_utc), Some(set_seconds_utc)),
        ("minutesUTC", Some(minutes_utc), Some(set_minutes_utc)),
        ("hoursUTC", Some(hours_utc), Some(set_hours_utc)),
        ("dateUTC", Some(date_utc), Some(set_date_utc)),
        ("monthUTC", Some(month_utc), Some(set_month_utc)),
        ("fullYearUTC", Some(full_year_utc), Some(set_full_year_utc)),
        ("dayUTC", Some(day_utc), None),
        ("timezoneOffset", Some(timezone_offset), None),
    ];
    class.define_builtin_instance_properties(
        mc,
        activation.avm2().public_namespace_base_version,
        PUBLIC_INSTANCE_PROPERTIES,
    );
    class.define_builtin_instance_methods(
        mc,
        activation.avm2().as3_namespace,
        PUBLIC_INSTANCE_AND_PROTO_METHODS,
    );

    const PUBLIC_CLASS_METHODS: &[(&str, NativeMethodImpl)] = &[("UTC", utc), ("parse", parse)];

    class.define_builtin_class_methods(
        mc,
        activation.avm2().public_namespace_base_version,
        PUBLIC_CLASS_METHODS,
    );

    const CLASS_CONSTANTS_INT: &[(&str, i32)] = &[("length", 7)];

    class.define_constant_int_class_traits(
        activation.avm2().public_namespace_base_version,
        CLASS_CONSTANTS_INT,
        activation,
    );

    class.mark_traits_loaded(activation.context.gc_context);
    class
        .init_vtable(activation.context)
        .expect("Native class's vtable should initialize");

    let c_class = class.c_class().expect("Class::new returns an i_class");

    c_class.mark_traits_loaded(activation.context.gc_context);
    c_class
        .init_vtable(activation.context)
        .expect("Native class's vtable should initialize");

    class
}
