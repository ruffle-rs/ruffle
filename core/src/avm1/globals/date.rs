use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::date_object::DateObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, TObject, Value};
use crate::locale::{get_current_date_time, get_timezone};
use crate::string::AvmString;
use chrono::{DateTime, Datelike, Duration, LocalResult, TimeZone, Timelike, Utc};
use gc_arena::{Collect, MutationContext};
use num_traits::ToPrimitive;

macro_rules! local_getter {
    ($fn:expr) => {
        |_activation, this, _args| {
            if let Some(this) = this.as_date_object() {
                if let Some(date) = this.date_time() {
                    let local = date.with_timezone(&get_timezone());
                    Ok($fn(&local).into())
                } else {
                    Ok(f64::NAN.into())
                }
            } else {
                Ok(Value::Undefined)
            }
        }
    };
}

macro_rules! utc_getter {
    ($fn:expr) => {
        |_activation, this, _args| {
            if let Some(this) = this.as_date_object() {
                if let Some(date) = this.date_time() {
                    Ok($fn(&date).into())
                } else {
                    Ok(f64::NAN.into())
                }
            } else {
                Ok(Value::Undefined)
            }
        }
    };
}

macro_rules! setter {
    ($fn:expr) => {
        |activation, this, args| {
            if let Some(this) = this.as_date_object() {
                $fn(activation, this, args)
            } else {
                Ok(Value::Undefined)
            }
        }
    };
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "getDay" => method(local_getter!(days_from_sunday); DONT_ENUM | DONT_DELETE);
    "getFullYear" => method(local_getter!(Datelike::year); DONT_ENUM | DONT_DELETE);
    "getDate" => method(local_getter!(Datelike::day); DONT_ENUM | DONT_DELETE);
    "getHours" => method(local_getter!(Timelike::hour); DONT_ENUM | DONT_DELETE);
    "getMilliseconds" => method(local_getter!(DateTime::timestamp_subsec_millis); DONT_ENUM | DONT_DELETE);
    "getMinutes" => method(local_getter!(Timelike::minute); DONT_ENUM | DONT_DELETE);
    "getMonth" => method(local_getter!(Datelike::month0); DONT_ENUM | DONT_DELETE);
    "getSeconds" => method(local_getter!(Timelike::second); DONT_ENUM | DONT_DELETE);
    "getYear" => method(local_getter!(year_1900_based); DONT_ENUM | DONT_DELETE);
    "valueOf" => method(utc_getter!(timestamp_millis_f64); DONT_ENUM | DONT_DELETE);
    "getTime" => method(utc_getter!(timestamp_millis_f64); DONT_ENUM | DONT_DELETE);
    "getUTCDate" => method(utc_getter!(Datelike::day); DONT_ENUM | DONT_DELETE);
    "getUTCDay" => method(utc_getter!(days_from_sunday); DONT_ENUM | DONT_DELETE);
    "getUTCFullYear" => method(utc_getter!(Datelike::year); DONT_ENUM | DONT_DELETE);
    "getUTCHours" => method(utc_getter!(Timelike::hour); DONT_ENUM | DONT_DELETE);
    "getUTCMilliseconds" => method(utc_getter!(DateTime::timestamp_subsec_millis); DONT_ENUM | DONT_DELETE);
    "getUTCMinutes" => method(utc_getter!(Timelike::minute); DONT_ENUM | DONT_DELETE);
    "getUTCMonth" => method(utc_getter!(Datelike::month0); DONT_ENUM | DONT_DELETE);
    "getUTCSeconds" => method(utc_getter!(Timelike::second); DONT_ENUM | DONT_DELETE);
    "getUTCYear" => method(utc_getter!(year_1900_based); DONT_ENUM | DONT_DELETE);
    "toString" => method(setter!(to_string); DONT_ENUM | DONT_DELETE);
    "getTimezoneOffset" => method(setter!(get_timezone_offset); DONT_ENUM | DONT_DELETE);
    "setDate" => method(setter!(set_date); DONT_ENUM | DONT_DELETE);
    "setUTCDate" => method(setter!(set_utc_date); DONT_ENUM | DONT_DELETE);
    "setYear" => method(setter!(set_year); DONT_ENUM | DONT_DELETE);
    "setFullYear" => method(setter!(set_full_year); DONT_ENUM | DONT_DELETE);
    "setUTCFullYear" => method(setter!(set_utc_full_year); DONT_ENUM | DONT_DELETE);
    "setHours" => method(setter!(set_hours); DONT_ENUM | DONT_DELETE);
    "setUTCHours" => method(setter!(set_utc_hours); DONT_ENUM | DONT_DELETE);
    "setMilliseconds" => method(setter!(set_milliseconds); DONT_ENUM | DONT_DELETE);
    "setUTCMilliseconds" => method(setter!(set_utc_milliseconds); DONT_ENUM | DONT_DELETE);
    "setMinutes" => method(setter!(set_minutes); DONT_ENUM | DONT_DELETE);
    "setUTCMinutes" => method(setter!(set_utc_minutes); DONT_ENUM | DONT_DELETE);
    "setMonth" => method(setter!(set_month); DONT_ENUM | DONT_DELETE);
    "setUTCMonth" => method(setter!(set_utc_month); DONT_ENUM | DONT_DELETE);
    "setSeconds" => method(setter!(set_seconds); DONT_ENUM | DONT_DELETE);
    "setUTCSeconds" => method(setter!(set_utc_seconds); DONT_ENUM | DONT_DELETE);
    "setTime" => method(setter!(set_time); DONT_ENUM | DONT_DELETE);
};

const OBJECT_DECLS: &[Declaration] = declare_properties! {
    "UTC" => method(create_utc; DONT_ENUM | DONT_DELETE | READ_ONLY);
};

fn days_from_sunday<T: Datelike>(date: &T) -> u32 {
    date.weekday().num_days_from_sunday()
}

fn year_1900_based<T: Datelike>(date: &T) -> i32 {
    date.year() - 1900
}

fn timestamp_millis_f64<T: TimeZone>(date: &DateTime<T>) -> f64 {
    date.timestamp_millis() as f64
}

#[derive(Collect)]
#[collect(require_static)]
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
            f64::NAN
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
        return Ok(this.into());
    };

    let timestamp = args.get(0).unwrap_or(&Value::Undefined);
    if timestamp != &Value::Undefined {
        if args.len() > 1 {
            let timezone = get_timezone();

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
        } else {
            let timestamp = timestamp.coerce_to_f64(activation)?;
            if timestamp.is_finite() {
                if let LocalResult::Single(time) = Utc.timestamp_millis_opt(timestamp as i64) {
                    this.set_date_time(activation.context.gc_context, Some(time))
                } else {
                    this.set_date_time(activation.context.gc_context, None);
                }
            } else {
                this.set_date_time(activation.context.gc_context, None);
            }
        }
    } else {
        this.set_date_time(activation.context.gc_context, Some(get_current_date_time()))
    }

    Ok(this.into())
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
        Some(activation.context.avm1.prototypes().date),
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
        let local = date.with_timezone(&get_timezone());
        Ok(AvmString::new_utf8(
            activation.context.gc_context,
            local.format("%a %b %-d %T GMT%z %-Y").to_string(),
        )
        .into())
    } else {
        Ok("Invalid Date".into())
    }
}

fn get_timezone_offset<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: DateObject<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let date = if let Some(date) = this.date_time() {
        date.with_timezone(&get_timezone())
    } else {
        return Ok(f64::NAN.into());
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
        Ok(f64::NAN.into())
    } else {
        let timestamp = DateAdjustment::new(activation, &get_timezone())
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
        Ok(f64::NAN.into())
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
    let timestamp = DateAdjustment::new(activation, &get_timezone())
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
    let timestamp = DateAdjustment::new(activation, &get_timezone())
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
    let timestamp = DateAdjustment::new(activation, &get_timezone())
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
    let timestamp = DateAdjustment::new(activation, &get_timezone())
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
    let timestamp = DateAdjustment::new(activation, &get_timezone())
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
    let timestamp = DateAdjustment::new(activation, &get_timezone())
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
        return Ok((time.timestamp_millis() as f64).into());
    }

    this.set_date_time(activation.context.gc_context, None);
    Ok(f64::NAN.into())
}

fn set_full_year<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DateObject<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let timestamp = DateAdjustment::new(activation, &get_timezone())
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
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let date = FunctionObject::function(
        gc_context,
        Executable::Native(constructor),
        Some(fn_proto),
        date_proto,
    );
    let object = date.as_script_object().unwrap();
    define_properties_on(OBJECT_DECLS, gc_context, object, fn_proto);
    date
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let date = DateObject::with_date_time(gc_context, Some(proto), None);
    let object = date.as_script_object().unwrap();
    define_properties_on(PROTO_DECLS, gc_context, object, fn_proto);
    date.into()
}
