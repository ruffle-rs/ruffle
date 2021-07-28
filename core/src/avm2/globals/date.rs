//! `Date` class

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{date_allocator, DateObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use chrono::{DateTime, Datelike, Duration, LocalResult, TimeZone, Timelike, Utc};
use gc_arena::{GcCell, MutationContext};
use num_traits::ToPrimitive;

enum YearType {
    Full,
    Adjust(Box<dyn Fn(i64) -> i64>),
}

impl YearType {
    fn adjust(&self, year: i64) -> i64 {
        match self {
            YearType::Full => year,
            YearType::Adjust(function) => function(year),
        }
    }
}

struct DateAdjustment<
    'builder,
    'activation_a: 'builder,
    'gc: 'activation_a,
    'gc_context: 'activation_a,
    T: TimeZone + 'builder,
> {
    activation: &'builder mut Activation<'activation_a, 'gc, 'gc_context>,
    year_type: YearType,
    timezone: &'builder T,
    year: Option<Option<f64>>,
    month: Option<Option<f64>>,
    day: Option<Option<f64>>,
    hour: Option<Option<f64>>,
    minute: Option<Option<f64>>,
    second: Option<Option<f64>>,
    millisecond: Option<Option<f64>>,
    ignore_next: bool,
}

impl<'builder, 'activation_a, 'gc, 'gc_context, T: TimeZone>
    DateAdjustment<'builder, 'activation_a, 'gc, 'gc_context, T>
{
    fn new(
        activation: &'builder mut Activation<'activation_a, 'gc, 'gc_context>,
        timezone: &'builder T,
    ) -> Self {
        Self {
            activation,
            timezone,
            year_type: YearType::Full,
            year: None,
            month: None,
            day: None,
            hour: None,
            minute: None,
            second: None,
            millisecond: None,
            ignore_next: false,
        }
    }

    fn adjust_year(&mut self, adjuster: impl Fn(i64) -> i64 + 'static) -> &mut Self {
        self.year_type = YearType::Adjust(Box::new(adjuster));
        self
    }

    #[allow(dead_code)]
    fn year(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error> {
        if !self.ignore_next {
            self.year = Some(if let Some(value) = value {
                Some(value.coerce_to_number(self.activation)?)
            } else {
                None
            });
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn year_or(&mut self, value: Option<&Value<'gc>>, default: f64) -> Result<&mut Self, Error> {
        if !self.ignore_next {
            self.year = Some(if let Some(value) = value {
                let value = value.coerce_to_number(self.activation)?;
                if value.is_finite() {
                    Some(value)
                } else {
                    Some(default)
                }
            } else {
                Some(default)
            });
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn year_opt(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error> {
        if !self.ignore_next {
            self.year = match value {
                Some(&Value::Undefined) | None => {
                    self.ignore_next = true;
                    None
                }
                Some(value) => Some(Some(value.coerce_to_number(self.activation)?)),
            };
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn month(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error> {
        if !self.ignore_next {
            self.month = Some(if let Some(value) = value {
                Some(value.coerce_to_number(self.activation)?)
            } else {
                None
            });
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn month_or(&mut self, value: Option<&Value<'gc>>, default: f64) -> Result<&mut Self, Error> {
        if !self.ignore_next {
            self.month = Some(if let Some(value) = value {
                let value = value.coerce_to_number(self.activation)?;
                if value.is_finite() {
                    Some(value)
                } else {
                    Some(default)
                }
            } else {
                Some(default)
            });
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn month_opt(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error> {
        if !self.ignore_next {
            self.month = match value {
                Some(&Value::Undefined) | None => {
                    self.ignore_next = true;
                    None
                }
                Some(value) => Some(Some(value.coerce_to_number(self.activation)?)),
            };
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn day(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error> {
        if !self.ignore_next {
            self.day = Some(if let Some(value) = value {
                Some(value.coerce_to_number(self.activation)?)
            } else {
                None
            });
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn day_or(&mut self, value: Option<&Value<'gc>>, default: f64) -> Result<&mut Self, Error> {
        if !self.ignore_next {
            self.day = Some(if let Some(value) = value {
                let value = value.coerce_to_number(self.activation)?;
                if value.is_finite() {
                    Some(value)
                } else {
                    Some(default)
                }
            } else {
                Some(default)
            });
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn day_opt(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error> {
        if !self.ignore_next {
            self.day = match value {
                Some(&Value::Undefined) | None => {
                    self.ignore_next = true;
                    None
                }
                Some(value) => Some(Some(value.coerce_to_number(self.activation)?)),
            };
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn hour(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error> {
        if !self.ignore_next {
            self.hour = Some(if let Some(value) = value {
                Some(value.coerce_to_number(self.activation)?)
            } else {
                None
            });
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn hour_or(&mut self, value: Option<&Value<'gc>>, default: f64) -> Result<&mut Self, Error> {
        if !self.ignore_next {
            self.hour = Some(if let Some(value) = value {
                let value = value.coerce_to_number(self.activation)?;
                if value.is_finite() {
                    Some(value)
                } else {
                    Some(default)
                }
            } else {
                Some(default)
            });
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn hour_opt(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error> {
        if !self.ignore_next {
            self.hour = match value {
                Some(&Value::Undefined) | None => {
                    self.ignore_next = true;
                    None
                }
                Some(value) => Some(Some(value.coerce_to_number(self.activation)?)),
            };
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn minute(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error> {
        if !self.ignore_next {
            self.minute = Some(if let Some(value) = value {
                Some(value.coerce_to_number(self.activation)?)
            } else {
                None
            });
        }
        Ok(self)
    }

    fn minute_or(&mut self, value: Option<&Value<'gc>>, default: f64) -> Result<&mut Self, Error> {
        if !self.ignore_next {
            self.minute = Some(if let Some(value) = value {
                let value = value.coerce_to_number(self.activation)?;
                if value.is_finite() {
                    Some(value)
                } else {
                    Some(default)
                }
            } else {
                Some(default)
            });
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn minute_opt(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error> {
        if !self.ignore_next {
            self.minute = match value {
                Some(&Value::Undefined) | None => {
                    self.ignore_next = true;
                    None
                }
                Some(value) => Some(Some(value.coerce_to_number(self.activation)?)),
            };
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn second(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error> {
        if !self.ignore_next {
            self.second = Some(if let Some(value) = value {
                Some(value.coerce_to_number(self.activation)?)
            } else {
                None
            });
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn second_or(&mut self, value: Option<&Value<'gc>>, default: f64) -> Result<&mut Self, Error> {
        if !self.ignore_next {
            self.second = Some(if let Some(value) = value {
                let value = value.coerce_to_number(self.activation)?;
                if value.is_finite() {
                    Some(value)
                } else {
                    Some(default)
                }
            } else {
                Some(default)
            });
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn second_opt(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error> {
        if !self.ignore_next {
            self.second = match value {
                Some(&Value::Undefined) | None => {
                    self.ignore_next = true;
                    None
                }
                Some(value) => Some(Some(value.coerce_to_number(self.activation)?)),
            };
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn millisecond(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error> {
        if !self.ignore_next {
            self.millisecond = Some(if let Some(value) = value {
                Some(value.coerce_to_number(self.activation)?)
            } else {
                None
            });
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn millisecond_or(
        &mut self,
        value: Option<&Value<'gc>>,
        default: f64,
    ) -> Result<&mut Self, Error> {
        if !self.ignore_next {
            self.millisecond = Some(if let Some(value) = value {
                let value = value.coerce_to_number(self.activation)?;
                if value.is_finite() {
                    Some(value)
                } else {
                    Some(default)
                }
            } else {
                Some(default)
            });
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn millisecond_opt(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error> {
        if !self.ignore_next {
            self.millisecond = match value {
                Some(&Value::Undefined) | None => {
                    self.ignore_next = true;
                    None
                }
                Some(value) => Some(Some(value.coerce_to_number(self.activation)?)),
            };
        }
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

    fn calculate(&mut self, current: DateObject<'gc>) -> Option<DateTime<Utc>> {
        if let Some(current) = current.date_time().map(|v| v.with_timezone(self.timezone)) {
            let month_rem = self
                .month
                .flatten()
                .map(|v| v as i64)
                .unwrap_or_default()
                .div_euclid(12);
            let month =
                self.check_mapped_value(self.month, |v| v.rem_euclid(12), current.month0())?;
            let year = self
                .check_mapped_value(self.year, |v| self.year_type.adjust(v), current.year())?
                .wrapping_add(month_rem) as i32;
            let day = self.check_value(self.day, current.day())?;
            let hour = self.check_value(self.hour, current.hour())?;
            let minute = self.check_value(self.minute, current.minute())?;
            let second = self.check_value(self.second, current.second())?;
            let millisecond =
                self.check_value(self.millisecond, current.timestamp_subsec_millis())?;

            let duration = Duration::days(day - 1)
                + Duration::hours(hour)
                + Duration::minutes(minute)
                + Duration::seconds(second)
                + Duration::milliseconds(millisecond);

            if let LocalResult::Single(Some(result)) = current
                .timezone()
                .ymd_opt(year, (month + 1) as u32, 1)
                .and_hms_opt(0, 0, 0)
                .map(|date| date.checked_add_signed(duration))
            {
                return Some(result.with_timezone(&Utc));
            }
        }

        None
    }

    fn apply(&mut self, object: DateObject<'gc>) -> f64 {
        let date = self.calculate(object);
        object.set_date_time(self.activation.context.gc_context, date);
        if let Some(date) = date {
            date.timestamp_millis() as f64
        } else {
            f64::NAN
        }
    }
}

/// Implements `Date`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;
        if let Some(date) = this.as_date_object() {
            let timestamp = args.get(0).unwrap_or(&Value::Undefined);
            if timestamp != &Value::Undefined {
                if args.len() > 1 {
                    let timezone = activation.context.locale.get_timezone();

                    // We need a starting value to adjust from.
                    date.set_date_time(
                        activation.context.gc_context,
                        Some(timezone.ymd(0, 1, 1).and_hms(0, 0, 0).into()),
                    );

                    DateAdjustment::new(activation, &timezone)
                        .year_opt(args.get(0))?
                        .month_opt(args.get(1))?
                        .day_opt(args.get(2))?
                        .hour_opt(args.get(3))?
                        .minute_opt(args.get(4))?
                        .second_opt(args.get(5))?
                        .millisecond_opt(args.get(6))?
                        .adjust_year(|year| if year < 100 { year + 1900 } else { year })
                        .apply(date);
                } else {
                    let timestamp = timestamp.coerce_to_number(activation)?;
                    if timestamp.is_finite() {
                        if let LocalResult::Single(time) =
                            Utc.timestamp_millis_opt(timestamp as i64)
                        {
                            date.set_date_time(activation.context.gc_context, Some(time))
                        } else {
                            date.set_date_time(activation.context.gc_context, None);
                        }
                    } else {
                        date.set_date_time(activation.context.gc_context, None);
                    }
                }
            } else {
                date.set_date_time(
                    activation.context.gc_context,
                    Some(activation.context.locale.get_current_date_time()),
                )
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Date`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `time` property's getter, and the `getTime` method.
pub fn time<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|this| this.as_date_object()) {
        if let Some(date) = this.date_time() {
            return Ok((date.timestamp_millis() as f64).into());
        } else {
            return Ok(f64::NAN.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `time` property's setter, and the `setTime` method.
pub fn set_time<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|this| this.as_date_object()) {
        let new_time = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_number(activation)?;
        if new_time.is_finite() {
            let time = Utc.timestamp_millis(new_time as i64);
            this.set_date_time(activation.context.gc_context, Some(time));
            return Ok((time.timestamp_millis() as f64).into());
        } else {
            this.set_date_time(activation.context.gc_context, None);
            return Ok(f64::NAN.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `seconds` property's getter, and the `getSeconds` method.
pub fn seconds<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|this| this.as_date_object()) {
        if let Some(date) = this
            .date_time()
            .map(|date| date.with_timezone(&activation.context.locale.get_timezone()))
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
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|this| this.as_date_object()) {
        let timezone = activation.context.locale.get_timezone();
        let timestamp = DateAdjustment::new(activation, &timezone)
            .second(args.get(0))?
            .apply(this);
        return Ok(timestamp.into());
    }
    Ok(Value::Undefined)
}

/// Implements `milliseconds` property's getter, and the `getMilliseconds` method.
pub fn milliseconds<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|this| this.as_date_object()) {
        if let Some(date) = this
            .date_time()
            .map(|date| date.with_timezone(&activation.context.locale.get_timezone()))
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
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|this| this.as_date_object()) {
        let timezone = activation.context.locale.get_timezone();
        let timestamp = DateAdjustment::new(activation, &timezone)
            .millisecond(args.get(0))?
            .apply(this);
        return Ok(timestamp.into());
    }
    Ok(Value::Undefined)
}

/// Implements `minutes` property's getter, and the `getMinutes` method.
pub fn minutes<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|this| this.as_date_object()) {
        if let Some(date) = this
            .date_time()
            .map(|date| date.with_timezone(&activation.context.locale.get_timezone()))
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
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|this| this.as_date_object()) {
        let timezone = activation.context.locale.get_timezone();
        let timestamp = DateAdjustment::new(activation, &timezone)
            .minute(args.get(0))?
            .apply(this);
        return Ok(timestamp.into());
    }
    Ok(Value::Undefined)
}

/// Implements `hour` property's getter, and the `getHours` method.
pub fn hours<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|this| this.as_date_object()) {
        if let Some(date) = this
            .date_time()
            .map(|date| date.with_timezone(&activation.context.locale.get_timezone()))
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
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|this| this.as_date_object()) {
        let timezone = activation.context.locale.get_timezone();
        let timestamp = DateAdjustment::new(activation, &timezone)
            .hour(args.get(0))?
            .apply(this);
        return Ok(timestamp.into());
    }
    Ok(Value::Undefined)
}

/// Implements `date` property's getter, and the `getDate` method.
pub fn date<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|this| this.as_date_object()) {
        if let Some(date) = this
            .date_time()
            .map(|date| date.with_timezone(&activation.context.locale.get_timezone()))
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
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|this| this.as_date_object()) {
        let timezone = activation.context.locale.get_timezone();
        let timestamp = DateAdjustment::new(activation, &timezone)
            .day(args.get(0))?
            .apply(this);
        return Ok(timestamp.into());
    }
    Ok(Value::Undefined)
}

/// Construct `Date`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::public(), "Date"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init, "<Date instance initializer>", mc),
        Method::from_builtin(class_init, "<Date class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);
    write.set_instance_allocator(date_allocator);

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
    ];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[
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
    ];
    write.define_public_builtin_instance_methods(mc, PUBLIC_INSTANCE_METHODS);

    class
}
