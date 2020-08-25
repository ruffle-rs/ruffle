use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::date_object::DateObject;
use crate::avm1::property::Attribute;
use crate::avm1::{AvmString, Object, TObject, Value};
use chrono::{DateTime, Datelike, Duration, FixedOffset, LocalResult, TimeZone, Timelike, Utc};
use enumset::EnumSet;
use gc_arena::{Collect, MutationContext};
use num_traits::ToPrimitive;
use std::f64::NAN;

macro_rules! implement_local_getters {
    ($gc_context: ident, $object:ident, $fn_proto: expr, $($name:expr => $fn:expr),*) => {
        $(
            $object.force_set_function(
                $name,
                |activation: &mut Activation<'_, 'gc, '_>, this, _args| -> Result<Value<'gc>, Error<'gc>> {
                    if let Some(this) = this.as_date_object() {
                        if let Some(date) = this.date_time() {
                            let local = date.with_timezone(&activation.context.locale.get_timezone());
                            Ok($fn(&local).into())
                        } else {
                            Ok(NAN.into())
                        }
                    } else {
                        Ok(Value::Undefined)
                    }
                } as crate::avm1::function::NativeFunction<'gc>,
                $gc_context,
                Attribute::DontDelete | Attribute::ReadOnly | Attribute::DontEnum,
                $fn_proto
            );
        )*
    };
}

macro_rules! implement_methods {
    ($gc_context: ident, $object:ident, $fn_proto: expr, $($name:expr => $fn:expr),*) => {
        $(
            $object.force_set_function(
                $name,
                |activation: &mut Activation<'_, 'gc, '_>, this, args| -> Result<Value<'gc>, Error<'gc>> {
                    if let Some(this) = this.as_date_object() {
                        $fn(activation, this, args)
                    } else {
                        Ok(Value::Undefined)
                    }
                } as crate::avm1::function::NativeFunction<'gc>,
                $gc_context,
                Attribute::DontDelete | Attribute::ReadOnly | Attribute::DontEnum,
                $fn_proto
            );
        )*
    };
}

macro_rules! implement_utc_getters {
    ($gc_context: ident, $object:ident, $fn_proto: expr, $($name:expr => $fn:expr),*) => {
        $(
            $object.force_set_function(
                $name,
                |_activation: &mut Activation<'_, 'gc, '_>, this, _args| -> Result<Value<'gc>, Error<'gc>> {
                    if let Some(this) = this.as_date_object() {
                        if let Some(date) = this.date_time() {
                            Ok($fn(&date).into())
                        } else {
                            Ok(NAN.into())
                        }
                    } else {
                        Ok(Value::Undefined)
                    }
                } as crate::avm1::function::NativeFunction<'gc>,
                $gc_context,
                Attribute::DontDelete | Attribute::ReadOnly | Attribute::DontEnum,
                $fn_proto
            );
        )*
    };
}

enum YearType {
    Full,
    Adjust(Box<dyn Fn(i64) -> i64>),
}

unsafe impl Collect for YearType {
    #[inline]
    fn needs_trace() -> bool {
        false
    }
}

impl YearType {
    fn adjust(&self, year: i64) -> i64 {
        match self {
            YearType::Full => year,
            YearType::Adjust(function) => function(year),
        }
    }
}

#[derive(Collect)]
#[collect(no_drop)]
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
    fn year(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error<'gc>> {
        if !self.ignore_next {
            self.year = Some(if let Some(value) = value {
                Some(value.coerce_to_f64(self.activation)?)
            } else {
                None
            });
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn year_or(
        &mut self,
        value: Option<&Value<'gc>>,
        default: f64,
    ) -> Result<&mut Self, Error<'gc>> {
        if !self.ignore_next {
            self.year = Some(if let Some(value) = value {
                let value = value.coerce_to_f64(self.activation)?;
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
    fn year_opt(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error<'gc>> {
        if !self.ignore_next {
            self.year = match value {
                Some(&Value::Undefined) | None => {
                    self.ignore_next = true;
                    None
                }
                Some(value) => Some(Some(value.coerce_to_f64(self.activation)?)),
            };
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn month(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error<'gc>> {
        if !self.ignore_next {
            self.month = Some(if let Some(value) = value {
                Some(value.coerce_to_f64(self.activation)?)
            } else {
                None
            });
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn month_or(
        &mut self,
        value: Option<&Value<'gc>>,
        default: f64,
    ) -> Result<&mut Self, Error<'gc>> {
        if !self.ignore_next {
            self.month = Some(if let Some(value) = value {
                let value = value.coerce_to_f64(self.activation)?;
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
    fn month_opt(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error<'gc>> {
        if !self.ignore_next {
            self.month = match value {
                Some(&Value::Undefined) | None => {
                    self.ignore_next = true;
                    None
                }
                Some(value) => Some(Some(value.coerce_to_f64(self.activation)?)),
            };
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn day(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error<'gc>> {
        if !self.ignore_next {
            self.day = Some(if let Some(value) = value {
                Some(value.coerce_to_f64(self.activation)?)
            } else {
                None
            });
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn day_or(
        &mut self,
        value: Option<&Value<'gc>>,
        default: f64,
    ) -> Result<&mut Self, Error<'gc>> {
        if !self.ignore_next {
            self.day = Some(if let Some(value) = value {
                let value = value.coerce_to_f64(self.activation)?;
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
    fn day_opt(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error<'gc>> {
        if !self.ignore_next {
            self.day = match value {
                Some(&Value::Undefined) | None => {
                    self.ignore_next = true;
                    None
                }
                Some(value) => Some(Some(value.coerce_to_f64(self.activation)?)),
            };
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn hour(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error<'gc>> {
        if !self.ignore_next {
            self.hour = Some(if let Some(value) = value {
                Some(value.coerce_to_f64(self.activation)?)
            } else {
                None
            });
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn hour_or(
        &mut self,
        value: Option<&Value<'gc>>,
        default: f64,
    ) -> Result<&mut Self, Error<'gc>> {
        if !self.ignore_next {
            self.hour = Some(if let Some(value) = value {
                let value = value.coerce_to_f64(self.activation)?;
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
    fn hour_opt(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error<'gc>> {
        if !self.ignore_next {
            self.hour = match value {
                Some(&Value::Undefined) | None => {
                    self.ignore_next = true;
                    None
                }
                Some(value) => Some(Some(value.coerce_to_f64(self.activation)?)),
            };
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn minute(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error<'gc>> {
        if !self.ignore_next {
            self.minute = Some(if let Some(value) = value {
                Some(value.coerce_to_f64(self.activation)?)
            } else {
                None
            });
        }
        Ok(self)
    }

    fn minute_or(
        &mut self,
        value: Option<&Value<'gc>>,
        default: f64,
    ) -> Result<&mut Self, Error<'gc>> {
        if !self.ignore_next {
            self.minute = Some(if let Some(value) = value {
                let value = value.coerce_to_f64(self.activation)?;
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
    fn minute_opt(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error<'gc>> {
        if !self.ignore_next {
            self.minute = match value {
                Some(&Value::Undefined) | None => {
                    self.ignore_next = true;
                    None
                }
                Some(value) => Some(Some(value.coerce_to_f64(self.activation)?)),
            };
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn second(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error<'gc>> {
        if !self.ignore_next {
            self.second = Some(if let Some(value) = value {
                Some(value.coerce_to_f64(self.activation)?)
            } else {
                None
            });
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn second_or(
        &mut self,
        value: Option<&Value<'gc>>,
        default: f64,
    ) -> Result<&mut Self, Error<'gc>> {
        if !self.ignore_next {
            self.second = Some(if let Some(value) = value {
                let value = value.coerce_to_f64(self.activation)?;
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
    fn second_opt(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error<'gc>> {
        if !self.ignore_next {
            self.second = match value {
                Some(&Value::Undefined) | None => {
                    self.ignore_next = true;
                    None
                }
                Some(value) => Some(Some(value.coerce_to_f64(self.activation)?)),
            };
        }
        Ok(self)
    }

    #[allow(dead_code)]
    fn millisecond(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error<'gc>> {
        if !self.ignore_next {
            self.millisecond = Some(if let Some(value) = value {
                Some(value.coerce_to_f64(self.activation)?)
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
    ) -> Result<&mut Self, Error<'gc>> {
        if !self.ignore_next {
            self.millisecond = Some(if let Some(value) = value {
                let value = value.coerce_to_f64(self.activation)?;
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
    fn millisecond_opt(&mut self, value: Option<&Value<'gc>>) -> Result<&mut Self, Error<'gc>> {
        if !self.ignore_next {
            self.millisecond = match value {
                Some(&Value::Undefined) | None => {
                    self.ignore_next = true;
                    None
                }
                Some(value) => Some(Some(value.coerce_to_f64(self.activation)?)),
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
            NAN
        }
    }
}

fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = if let Some(object) = this.as_date_object() {
        object
    } else {
        return Ok(Value::Undefined);
    };

    let timestamp = args.get(0).unwrap_or(&Value::Undefined);
    if timestamp != &Value::Undefined {
        if args.len() > 1 {
            let timezone = activation.context.locale.get_timezone();

            // We need a starting value to adjust from.
            this.set_date_time(
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
                .apply(this);
        } else if let LocalResult::Single(time) =
            Utc.timestamp_millis_opt(timestamp.coerce_to_f64(activation)? as i64)
        {
            this.set_date_time(activation.context.gc_context, Some(time))
        } else {
            this.set_date_time(activation.context.gc_context, None);
        }
    } else {
        this.set_date_time(
            activation.context.gc_context,
            Some(activation.context.locale.get_current_date_time()),
        )
    }

    Ok(Value::Undefined)
}

fn create_utc<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() < 2 {
        return Ok(Value::Undefined);
    }

    // We need a starting value to adjust from.
    let date = DateObject::with_date_time(
        activation.context.gc_context,
        Some(activation.context.avm1.prototypes.date),
        Some(Utc.ymd(0, 1, 1).and_hms(0, 0, 0)),
    );

    let timestamp = DateAdjustment::new(activation, &Utc)
        .year(args.get(0))?
        .month(args.get(1))?
        .day_opt(args.get(2))?
        .hour_opt(args.get(3))?
        .minute_opt(args.get(4))?
        .second_opt(args.get(5))?
        .millisecond_opt(args.get(6))?
        .adjust_year(|year| if year < 100 { year + 1900 } else { year })
        .apply(date);

    Ok(timestamp.into())
}

fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DateObject<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let date = this.date_time();

    if let Some(date) = date {
        let local = date.with_timezone(&activation.context.locale.get_timezone());
        Ok(AvmString::new(
            activation.context.gc_context,
            local.format("%a %b %-d %T GMT%z %-Y").to_string(),
        )
        .into())
    } else {
        Ok("Invalid Date".into())
    }
}

fn get_timezone_offset<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DateObject<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let date = if let Some(date) = this.date_time() {
        date.with_timezone(&activation.context.locale.get_timezone())
    } else {
        return Ok(NAN.into());
    };

    let seconds = date.offset().utc_minus_local() as f32;
    let minutes = seconds / 60.0;
    Ok(minutes.into())
}

fn set_date<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DateObject<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.is_empty() {
        this.set_date_time(activation.context.gc_context, None);
        Ok(NAN.into())
    } else {
        let timezone = activation.context.locale.get_timezone();
        let timestamp = DateAdjustment::new(activation, &timezone)
            .day(args.get(0))?
            .apply(this);
        Ok(timestamp.into())
    }
}

fn set_utc_date<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DateObject<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.is_empty() {
        this.set_date_time(activation.context.gc_context, None);
        Ok(NAN.into())
    } else {
        let timestamp = DateAdjustment::new(activation, &Utc)
            .day(args.get(0))?
            .apply(this);
        Ok(timestamp.into())
    }
}

fn set_year<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DateObject<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let timezone = activation.context.locale.get_timezone();
    let timestamp = DateAdjustment::new(activation, &timezone)
        .year(args.get(0))?
        .adjust_year(|year| {
            if year >= 0 && year < 100 {
                year + 1900
            } else {
                year
            }
        })
        .apply(this);
    Ok(timestamp.into())
}

fn set_hours<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DateObject<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let timezone = activation.context.locale.get_timezone();
    let timestamp = DateAdjustment::new(activation, &timezone)
        .hour(args.get(0))?
        .apply(this);
    Ok(timestamp.into())
}

fn set_utc_hours<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DateObject<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let timestamp = DateAdjustment::new(activation, &Utc)
        .hour(args.get(0))?
        .minute_opt(args.get(1))?
        .second_opt(args.get(2))?
        .millisecond_opt(args.get(3))?
        .apply(this);
    Ok(timestamp.into())
}

fn set_milliseconds<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DateObject<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let timezone = activation.context.locale.get_timezone();
    let timestamp = DateAdjustment::new(activation, &timezone)
        .millisecond(args.get(0))?
        .apply(this);
    Ok(timestamp.into())
}

fn set_utc_milliseconds<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DateObject<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let timestamp = DateAdjustment::new(activation, &Utc)
        .millisecond(args.get(0))?
        .apply(this);
    Ok(timestamp.into())
}

fn set_minutes<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DateObject<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let timezone = activation.context.locale.get_timezone();
    let timestamp = DateAdjustment::new(activation, &timezone)
        .minute_or(args.get(0), -2147483648.0)?
        .apply(this);
    Ok(timestamp.into())
}

fn set_utc_minutes<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DateObject<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let timestamp = DateAdjustment::new(activation, &Utc)
        .minute_or(args.get(0), -2147483648.0)?
        .second_opt(args.get(1))?
        .millisecond_opt(args.get(2))?
        .apply(this);
    Ok(timestamp.into())
}

fn set_month<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DateObject<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let timezone = activation.context.locale.get_timezone();
    let timestamp = DateAdjustment::new(activation, &timezone)
        .month_or(args.get(0), 0.0)?
        .day_opt(args.get(1))?
        .apply(this);
    Ok(timestamp.into())
}

fn set_utc_month<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DateObject<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let timestamp = DateAdjustment::new(activation, &Utc)
        .month_or(args.get(0), 0.0)?
        .day_opt(args.get(1))?
        .apply(this);
    Ok(timestamp.into())
}

fn set_seconds<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DateObject<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let timezone = activation.context.locale.get_timezone();
    let timestamp = DateAdjustment::new(activation, &timezone)
        .second(args.get(0))?
        .apply(this);
    Ok(timestamp.into())
}

fn set_utc_seconds<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DateObject<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let timestamp = DateAdjustment::new(activation, &Utc)
        .second(args.get(0))?
        .millisecond_opt(args.get(1))?
        .apply(this);
    Ok(timestamp.into())
}

fn set_time<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DateObject<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let new_time = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(activation)?;

    if new_time.is_finite() {
        let time = Utc.timestamp_millis(new_time as i64);
        this.set_date_time(activation.context.gc_context, Some(time));
        return Ok(time.timestamp_millis().into());
    }

    this.set_date_time(activation.context.gc_context, None);
    Ok(NAN.into())
}

fn set_full_year<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DateObject<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let timezone = activation.context.locale.get_timezone();
    let timestamp = DateAdjustment::new(activation, &timezone)
        .year(args.get(0))?
        .month_opt(args.get(1))?
        .day_opt(args.get(2))?
        .apply(this);
    Ok(timestamp.into())
}

fn set_utc_full_year<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DateObject<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let timestamp = DateAdjustment::new(activation, &Utc)
        .year(args.get(0))?
        .month_opt(args.get(1))?
        .day_opt(args.get(2))?
        .apply(this);
    Ok(timestamp.into())
}

pub fn create_date_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    date_proto: Object<'gc>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    let date = FunctionObject::function(
        gc_context,
        Executable::Native(constructor),
        fn_proto,
        date_proto,
    );
    let mut object = date.as_script_object().unwrap();

    object.force_set_function("UTC", create_utc, gc_context, EnumSet::empty(), fn_proto);

    date
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let date = DateObject::with_date_time(gc_context, Some(proto), None);
    let mut object = date.as_script_object().unwrap();

    implement_local_getters!(
        gc_context,
        object,
        Some(fn_proto),
        "getDate" => Datelike::day,
        "getDay" => |date: &DateTime<FixedOffset>| date.weekday().num_days_from_sunday(),
        "getFullYear" => Datelike::year,
        "getHours" => Timelike::hour,
        "getMilliseconds" => DateTime::timestamp_subsec_millis,
        "getMinutes" => Timelike::minute,
        "getMonth" => Datelike::month0,
        "getSeconds" => Timelike::second,
        "getYear" => |date: &DateTime<FixedOffset>| date.year() - 1900
    );

    implement_utc_getters!(
        gc_context,
        object,
        Some(fn_proto),
        "valueOf" => DateTime::timestamp_millis,
        "getTime" => DateTime::timestamp_millis,
        "getUTCDate" => Datelike::day,
        "getUTCDay" => |date: &DateTime<Utc>| date.weekday().num_days_from_sunday(),
        "getUTCFullYear" => Datelike::year,
        "getUTCHours" => Timelike::hour,
        "getUTCMilliseconds" => DateTime::timestamp_subsec_millis,
        "getUTCMinutes" => Timelike::minute,
        "getUTCMonth" => Datelike::month0,
        "getUTCSeconds" => Timelike::second,
        "getUTCYear" => |date: &DateTime<Utc>| date.year() - 1900
    );

    implement_methods!(
        gc_context,
        object,
        Some(fn_proto),
        "toString" => to_string,
        "getTimezoneOffset" => get_timezone_offset,
        "setDate" => set_date,
        "setUTCDate" => set_utc_date,
        "setYear" => set_year,
        "setFullYear" => set_full_year,
        "setUTCFullYear" => set_utc_full_year,
        "setHours" => set_hours,
        "setUTCHours" => set_utc_hours,
        "setMilliseconds" => set_milliseconds,
        "setUTCMilliseconds" => set_utc_milliseconds,
        "setMinutes" => set_minutes,
        "setUTCMinutes" => set_utc_minutes,
        "setMonth" => set_month,
        "setUTCMonth" => set_utc_month,
        "setSeconds" => set_seconds,
        "setUTCSeconds" => set_utc_seconds,
        "setTime" => set_time
    );

    date.into()
}
