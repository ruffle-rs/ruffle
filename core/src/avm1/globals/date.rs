use crate::avm1::clamp::Clamp;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::NativeObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Activation, Error, Object, ScriptObject, TObject, Value};
use crate::locale::{get_current_date_time, get_timezone};
use crate::string::{AvmString, StringContext};
use gc_arena::Gc;
use std::cell::Cell;
use std::fmt;

#[inline]
fn rem_euclid_i32(lhs: f64, rhs: i32) -> i32 {
    let result = (lhs % f64::from(rhs)).clamp_to_i32();
    if result < 0 {
        result + rhs
    } else {
        result
    }
}

/// Date and time, represented by milliseconds since epoch.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
#[repr(transparent)]
pub struct Date(f64);

impl Date {
    const MS_PER_SECOND: i32 = 1000;
    const SECONDS_PER_MINUTE: i32 = 60;
    const MS_PER_MINUTE: i32 = Self::MS_PER_SECOND * Self::SECONDS_PER_MINUTE;
    const MINUTES_PER_HOUR: i32 = 60;
    const MS_PER_HOUR: i32 = Self::MS_PER_MINUTE * Self::MINUTES_PER_HOUR;
    const HOURS_PER_DAY: i32 = 24;
    const MS_PER_DAY: i32 = Self::MS_PER_HOUR * Self::HOURS_PER_DAY;

    const MONTH_OFFSETS: [[u16; 13]; 2] = [
        [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365],
        [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335, 366], // Leap year
    ];

    const EPOCH: Self = Self(0.0);
    const INVALID: Self = Self(f64::NAN);

    /// Create from specified date and time.
    fn new(
        mut year: f64,
        month: f64,
        date: f64,
        hour: f64,
        minute: f64,
        second: f64,
        millisecond: f64,
    ) -> Self {
        if year < 100.0 {
            year += 1900.0;
        }
        Self::make_date(
            Self::make_day(year, month, date),
            Self::make_time(hour, minute, second, millisecond),
        )
    }

    /// Create from current date and time.
    fn now() -> Self {
        Self(get_current_date_time().timestamp_millis() as f64)
    }

    /// Get milliseconds since epoch.
    pub const fn time(&self) -> f64 {
        self.0
    }

    fn is_valid(&self) -> bool {
        self.0.is_finite()
    }

    /// ECMA-262 Day - Get days since epoch.
    fn day(&self) -> f64 {
        (self.0 / f64::from(Self::MS_PER_DAY)).floor()
    }

    /// ECMA-262 TimeWithinDay - Get milliseconds within day.
    fn time_within_day(&self, swf_version: u8) -> f64 {
        if swf_version > 7 {
            self.0.rem_euclid(Self::MS_PER_DAY.into())
        } else {
            self.0 % f64::from(Self::MS_PER_DAY)
        }
    }

    /// ECMA-262 DayFromYear - Return days passed since epoch to January 1st of `year`.
    fn day_from_year(year: f64) -> f64 {
        (365.0 * (year - 1970.0)) + ((year - 1969.0) / 4.0).floor()
            - ((year - 1901.0) / 100.0).floor()
            + ((year - 1601.0) / 400.0).floor()
    }

    /// ECMA-262 TimeFromYear - Return January 1st of `year`.
    fn from_year(year: i32) -> Self {
        Self(f64::from(Self::MS_PER_DAY) * Self::day_from_year(year.into()))
    }

    /// ECMA-262 YearFromTime - Get year.
    fn year(&self) -> i32 {
        let day = self.day();
        // Perform binary search to find the largest `year: i32` such that `Self::from_year(year) <= *self`.
        let mut low =
            ((day / if *self < Self::EPOCH { 365.0 } else { 366.0 }).floor()).clamp_to_i32() + 1970;
        let mut high =
            ((day / if *self < Self::EPOCH { 366.0 } else { 365.0 }).ceil()).clamp_to_i32() + 1970;
        while low < high {
            let pivot = ((f64::from(low) + f64::from(high)) / 2.0).clamp_to_i32();
            if Self::from_year(pivot) <= *self {
                if Self::from_year(pivot + 1) > *self {
                    return pivot;
                }
                low = pivot + 1;
            } else {
                debug_assert!(Self::from_year(pivot) > *self);
                high = pivot - 1;
            }
        }
        low
    }

    /// Determine whether or not `year` is a leap year (i.e. has 366 days instead of 365).
    const fn is_leap_year(year: i32) -> bool {
        year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
    }

    /// ECMA-262 InLeapYear
    fn in_leap_year(&self) -> bool {
        Self::is_leap_year(self.year())
    }

    /// ECMA-262 MonthFromTime - Get month (0-11).
    fn month(&self) -> i32 {
        let day = self.day_within_year();
        let in_leap_year = self.in_leap_year();
        for i in 0..11 {
            if day < Self::MONTH_OFFSETS[usize::from(in_leap_year)][i as usize + 1].into() {
                return i;
            }
        }
        11
    }

    /// ECMA-262 DayWithinYear - Get days within year (0-365).
    fn day_within_year(&self) -> i32 {
        (self.day() - Self::day_from_year(self.year().into())).clamp_to_i32()
    }

    /// ECMA-262 DateFromTime - Get days within month (1-31).
    fn date(&self) -> i32 {
        let month = self.month();
        let month_offset = Self::MONTH_OFFSETS[usize::from(self.in_leap_year())][month as usize];
        self.day_within_year() - i32::from(month_offset) + 1
    }

    /// ECMA-262 WeekDay - Get days within week (0-6).
    fn week_day(&self) -> i32 {
        rem_euclid_i32(self.day() + 4.0, 7)
    }

    /// ECMA-262 LocalTZA - Get local timezone adjustment in milliseconds.
    fn local_tza(&self, _is_utc: bool) -> i32 {
        // TODO: Honor `is_utc` flag.
        get_timezone().local_minus_utc() * Self::MS_PER_SECOND
    }

    /// ECMA-262 LocalTime - Convert from UTC to local timezone.
    fn local(self) -> Self {
        Self(self.0 + f64::from(self.local_tza(true)))
    }

    /// ECMA-262 UTC - Convert from local timezone to UTC.
    fn utc(self) -> Self {
        Self(self.0 - f64::from(self.local_tza(false)))
    }

    /// Get timezone offset in minutes.
    fn timezone_offset(&self) -> f64 {
        (self.0 - self.local().0) / f64::from(Self::MS_PER_MINUTE)
    }

    /// ECMA-262 HourFromTime - Get hours (0-23).
    fn hours(&self) -> i32 {
        rem_euclid_i32(
            ((self.0 + 0.5) / f64::from(Self::MS_PER_HOUR)).floor(),
            Self::HOURS_PER_DAY,
        )
    }

    /// ECMA-262 MinFromTime - Get minutes (0-59).
    fn minutes(&self) -> i32 {
        rem_euclid_i32(
            (self.0 / f64::from(Self::MS_PER_MINUTE)).floor(),
            Self::MINUTES_PER_HOUR,
        )
    }

    /// ECMA-262 SecFromTime - Get seconds (0-59).
    fn seconds(&self) -> i32 {
        rem_euclid_i32(
            (self.0 / f64::from(Self::MS_PER_SECOND)).floor(),
            Self::SECONDS_PER_MINUTE,
        )
    }

    /// ECMA-262 msFromTime - Get milliseconds (0-999).
    fn milliseconds(&self) -> i32 {
        rem_euclid_i32(self.0, Self::MS_PER_SECOND)
    }

    /// ECMA-262 MakeTime
    fn make_time(hours: f64, minutes: f64, seconds: f64, milliseconds: f64) -> f64 {
        let hours = hours.floor(); // TODO: Round towards zero.
        let minutes = minutes.floor(); // TODO: Round towards zero.
        let seconds = seconds.floor(); // TODO: Round towards zero.
        let milliseconds = milliseconds.floor(); // TODO: Round towards zero.

        hours * f64::from(Self::MS_PER_HOUR)
            + minutes * f64::from(Self::MS_PER_MINUTE)
            + seconds * f64::from(Self::MS_PER_SECOND)
            + milliseconds
    }

    fn day_from_month(year: f64, month: f64) -> f64 {
        let year = year.clamp_to_i32();
        let month = month.floor().clamp_to_i32();
        if !(0..12).contains(&month) {
            return f64::NAN;
        }
        let is_leap_year = Self::is_leap_year(year);
        let month_offset = Self::MONTH_OFFSETS[usize::from(is_leap_year)][month as usize];
        Self::day_from_year(year.into()) + f64::from(month_offset)
    }

    /// ECMA-262 MakeDay
    fn make_day(year: f64, month: f64, date: f64) -> f64 {
        let mut year = year.floor(); // TODO: Round towards zero.
        let month = month.floor(); // TODO: Round towards zero.
        let date = date.floor(); // TODO: Round towards zero.

        year += (month / 12.0).floor();
        let month = month.rem_euclid(12.0);

        Self::day_from_month(year, month) + date - 1.0
    }

    /// ECMA-262 MakeDate - Create from days since epoch and milliseconds within day.
    fn make_date(day: f64, time: f64) -> Self {
        Self(day * f64::from(Self::MS_PER_DAY) + time)
    }

    /// ECMA-262 TimeClip
    fn clip(self) -> Self {
        const LIMIT: f64 = 100_000_000.0 * Date::MS_PER_DAY as f64;
        if !self.is_valid() || self.0.abs() > LIMIT {
            return Self::INVALID;
        }

        Self(self.0.floor())
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.is_valid() {
            return write!(f, "Invalid Date");
        }

        const DAYS_OF_WEEK: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        const MONTHS: [&str; 12] = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];

        let timezone_offset = (-self.timezone_offset()).clamp_to_i32();
        write!(
            f,
            "{} {} {} {:02}:{:02}:{:02} GMT{}{:02}{:02} {}",
            DAYS_OF_WEEK[self.week_day() as usize],
            MONTHS[self.month() as usize],
            self.date(),
            self.hours(),
            self.minutes(),
            self.seconds(),
            if timezone_offset < 0 { '-' } else { '+' },
            timezone_offset.abs() / Self::MINUTES_PER_HOUR,
            timezone_offset.abs() % Self::MINUTES_PER_HOUR,
            self.year(),
        )
    }
}

macro_rules! date_method {
    ($index:literal) => {
        |activation, this, args| method(activation, this, args, $index)
    };
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "getFullYear" => method(date_method!(0); DONT_ENUM | DONT_DELETE);
    "getYear" => method(date_method!(1); DONT_ENUM | DONT_DELETE);
    "getMonth" => method(date_method!(2); DONT_ENUM | DONT_DELETE);
    "getDate" => method(date_method!(3); DONT_ENUM | DONT_DELETE);
    "getDay" => method(date_method!(4); DONT_ENUM | DONT_DELETE);
    "getHours" => method(date_method!(5); DONT_ENUM | DONT_DELETE);
    "getMinutes" => method(date_method!(6); DONT_ENUM | DONT_DELETE);
    "getSeconds" => method(date_method!(7); DONT_ENUM | DONT_DELETE);
    "getMilliseconds" => method(date_method!(8); DONT_ENUM | DONT_DELETE);
    "setFullYear" => method(date_method!(9); DONT_ENUM | DONT_DELETE);
    "setMonth" => method(date_method!(10); DONT_ENUM | DONT_DELETE);
    "setDate" => method(date_method!(11); DONT_ENUM | DONT_DELETE);
    "setHours" => method(date_method!(12); DONT_ENUM | DONT_DELETE);
    "setMinutes" => method(date_method!(13); DONT_ENUM | DONT_DELETE);
    "setSeconds" => method(date_method!(14); DONT_ENUM | DONT_DELETE);
    "setMilliseconds" => method(date_method!(15); DONT_ENUM | DONT_DELETE);
    "getTime" => method(date_method!(16); DONT_ENUM | DONT_DELETE);
    "valueOf" => method(date_method!(16); DONT_ENUM | DONT_DELETE);
    "setTime" => method(date_method!(17); DONT_ENUM | DONT_DELETE);
    "getTimezoneOffset" => method(date_method!(18); DONT_ENUM | DONT_DELETE);
    "toString" => method(date_method!(19); DONT_ENUM | DONT_DELETE);
    "setYear" => method(date_method!(20); DONT_ENUM | DONT_DELETE);
    "getUTCFullYear" => method(date_method!(128); DONT_ENUM | DONT_DELETE);
    "getUTCYear" => method(date_method!(129); DONT_ENUM | DONT_DELETE);
    "getUTCMonth" => method(date_method!(130); DONT_ENUM | DONT_DELETE);
    "getUTCDate" => method(date_method!(131); DONT_ENUM | DONT_DELETE);
    "getUTCDay" => method(date_method!(132); DONT_ENUM | DONT_DELETE);
    "getUTCHours" => method(date_method!(133); DONT_ENUM | DONT_DELETE);
    "getUTCMinutes" => method(date_method!(134); DONT_ENUM | DONT_DELETE);
    "getUTCSeconds" => method(date_method!(135); DONT_ENUM | DONT_DELETE);
    "getUTCMilliseconds" => method(date_method!(136); DONT_ENUM | DONT_DELETE);
    "setUTCFullYear" => method(date_method!(137); DONT_ENUM | DONT_DELETE);
    "setUTCMonth" => method(date_method!(138); DONT_ENUM | DONT_DELETE);
    "setUTCDate" => method(date_method!(139); DONT_ENUM | DONT_DELETE);
    "setUTCHours" => method(date_method!(140); DONT_ENUM | DONT_DELETE);
    "setUTCMinutes" => method(date_method!(141); DONT_ENUM | DONT_DELETE);
    "setUTCSeconds" => method(date_method!(142); DONT_ENUM | DONT_DELETE);
    "setUTCMilliseconds" => method(date_method!(143); DONT_ENUM | DONT_DELETE);
};

const OBJECT_DECLS: &[Declaration] = declare_properties! {
    "UTC" => method(date_method!(257); DONT_ENUM | DONT_DELETE | READ_ONLY);
};

/// ECMA-262 Date
fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[f64],
) -> Result<Value<'gc>, Error<'gc>> {
    let date = match args[..] {
        [] => {
            let date = Date::now();
            if activation.swf_version() > 7 {
                Date(date.time().round())
            } else {
                date
            }
        }
        [timestamp] => Date(timestamp),
        [year, month, ..] => {
            let date = args.get(2).copied().unwrap_or(1.0);
            let hour = args.get(3).copied().unwrap_or(0.0);
            let minute = args.get(4).copied().unwrap_or(0.0);
            let second = args.get(5).copied().unwrap_or(0.0);
            let millisecond = args.get(6).copied().unwrap_or(0.0);
            Date::new(year, month, date, hour, minute, second, millisecond).utc()
        }
    };
    this.set_native(
        activation.gc(),
        NativeObject::Date(Gc::new(activation.gc(), Cell::new(date))),
    );
    Ok(this.into())
}

/// `Date()` invoked without `new` returns current date and time as a string, as defined in ECMA-262.
fn function<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(AvmString::new_utf8(activation.gc(), Date::now().local().to_string()).into())
}

/// ECMA-262 Date.UTC
fn utc<'gc>(args: &[f64]) -> Result<Value<'gc>, Error<'gc>> {
    match args[..] {
        [] | [_] => Ok(Value::Undefined),
        [year, month, ..] => {
            let date = args.get(2).copied().unwrap_or(1.0);
            let hour = args.get(3).copied().unwrap_or(0.0);
            let minute = args.get(4).copied().unwrap_or(0.0);
            let second = args.get(5).copied().unwrap_or(0.0);
            let millisecond = args.get(6).copied().unwrap_or(0.0);
            let date = Date::new(year, month, date, hour, minute, second, millisecond);
            Ok(date.time().into())
        }
    }
}

fn method<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
    mut index: u16,
) -> Result<Value<'gc>, Error<'gc>> {
    const GET_FULL_YEAR: u16 = 0;
    const GET_YEAR: u16 = 1;
    const GET_MONTH: u16 = 2;
    const GET_DATE: u16 = 3;
    const GET_DAY: u16 = 4;
    const GET_HOURS: u16 = 5;
    const GET_MINUTES: u16 = 6;
    const GET_SECONDS: u16 = 7;
    const GET_MILLISECONDS: u16 = 8;
    const SET_FULL_YEAR: u16 = 9;
    const SET_MONTH: u16 = 10;
    const SET_DATE: u16 = 11;
    const SET_HOURS: u16 = 12;
    const SET_MINUTES: u16 = 13;
    const SET_SECONDS: u16 = 14;
    const SET_MILLISECONDS: u16 = 15;
    const GET_TIME: u16 = 16;
    const SET_TIME: u16 = 17;
    const GET_TIMEZONE_OFFSET: u16 = 18;
    const TO_STRING: u16 = 19;
    const SET_YEAR: u16 = 20;
    const CONSTRUCTOR: u16 = 256;
    const UTC: u16 = 257;

    let mut values = Vec::with_capacity(7);
    for arg in args.iter().take(7) {
        if arg == &Value::Undefined {
            break;
        }
        values.push(arg.coerce_to_f64(activation)?);
    }
    let args = values;

    match index {
        CONSTRUCTOR => return constructor(activation, this, &args),
        UTC => return utc(&args),
        _ => {}
    }

    let date_ref = match this.native() {
        NativeObject::Date(date) => date,
        _ => return Ok(Value::Undefined),
    };
    let date = date_ref.get();

    match index {
        GET_TIME => return Ok(date.time().into()),
        SET_TIME => {
            let timestamp = args.first().copied().unwrap_or(f64::NAN);
            let new_date = Date(timestamp).clip();
            date_ref.set(new_date);
            return Ok(new_date.time().into());
        }
        GET_TIMEZONE_OFFSET => return Ok(date.timezone_offset().into()),
        _ => {}
    }

    // Map method `index` to local counterpart.
    let is_utc = index >= 128;
    if is_utc {
        index -= 128;
    }

    // Return `NaN` for invalid dates.
    let is_get = matches!(
        index,
        GET_FULL_YEAR..=GET_MILLISECONDS | GET_TIME | GET_TIMEZONE_OFFSET
    );
    if is_get && date.time().is_nan() {
        return Ok(f64::NAN.into());
    }

    // Handle `setYear()` using `setFullYear()`.
    let is_set_year = index == SET_YEAR;
    if is_set_year {
        index = SET_FULL_YEAR;
    }

    let arg = |i: u16| {
        i.checked_sub(index)
            .and_then(|i| args.get(usize::from(i)))
            .copied()
            .or_else(|| (i == index).then_some(f64::NAN))
    };

    let date = if is_utc { date } else { date.local() };

    let set_date = |day: f64, time: f64| {
        let mut date = Date::make_date(day, time);
        if !is_utc {
            date = date.utc();
        }
        date = date.clip();
        date_ref.set(date);
        date.time()
    };

    Ok(match index {
        GET_FULL_YEAR => date.year().into(),
        GET_YEAR => (date.year() - 1900).into(),
        GET_MONTH => date.month().into(),
        GET_DATE => date.date().into(),
        GET_DAY => date.week_day().into(),
        GET_HOURS => date.hours().into(),
        GET_MINUTES => date.minutes().into(),
        GET_SECONDS => date.seconds().into(),
        GET_MILLISECONDS => date.milliseconds().into(),
        SET_FULL_YEAR..=SET_DATE => {
            let year = arg(SET_FULL_YEAR).map_or_else(
                || date.year().into(),
                |mut year| {
                    if is_set_year && year >= 0.0 && year <= 99.0 {
                        year += 1900.0;
                    }
                    year
                },
            );
            let mut month = arg(SET_MONTH).unwrap_or_else(|| date.month().into());
            if index == SET_MONTH && month.is_nan() {
                // `setMonth()` special case.
                month = 0.0;
            }
            let new_date = arg(SET_DATE).unwrap_or_else(|| {
                let new_date = date.date().into();
                if index == SET_MONTH && activation.swf_version() > 6 {
                    // TODO: `setMonth()` special case.
                }
                new_date
            });
            set_date(
                Date::make_day(year, month, new_date),
                date.time_within_day(activation.swf_version()),
            )
            .into()
        }
        SET_HOURS..=SET_MILLISECONDS => {
            let hours = arg(SET_HOURS).unwrap_or_else(|| date.hours().into());
            let mut minutes = arg(SET_MINUTES).unwrap_or_else(|| date.minutes().into());
            let mut seconds = arg(SET_SECONDS).unwrap_or_else(|| date.seconds().into());
            let mut milliseconds =
                arg(SET_MILLISECONDS).unwrap_or_else(|| date.milliseconds().into());
            if index == SET_MINUTES {
                // `setMinutes()` special case.
                minutes = minutes.clamp_to_i32().into();
                seconds = seconds.clamp_to_i32().into();
                milliseconds = milliseconds.clamp_to_i32().into();
            }
            set_date(
                date.day(),
                Date::make_time(hours, minutes, seconds, milliseconds),
            )
            .into()
        }
        TO_STRING => AvmString::new_utf8(activation.gc(), date.to_string()).into(),
        GET_TIME..=GET_TIMEZONE_OFFSET | SET_YEAR.. => unreachable!(), // Handled above.
    })
}

pub fn create_constructor<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let date_proto = ScriptObject::new(context.gc(), Some(proto));
    define_properties_on(PROTO_DECLS, context, date_proto, fn_proto);

    let date_constructor = FunctionObject::constructor(
        context.gc(),
        Executable::Native(date_method!(256)),
        Executable::Native(function),
        fn_proto,
        date_proto.into(),
    );
    let object = date_constructor.raw_script_object();
    define_properties_on(OBJECT_DECLS, context, object, fn_proto);

    date_constructor
}
