/// Date and time represented by milliseconds since the Unix epoch.
///
/// This intentionally stores the raw Flash date value rather than using
/// `chrono::DateTime`. Flash can represent dates outside of chrono's range,
/// and some construction paths can even produce values outside TimeClip's
/// +/- 100,000,000 day range.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
#[repr(transparent)]
pub(crate) struct Date(f64);

impl Date {
    pub(crate) const MS_PER_SECOND: i32 = 1_000;
    pub(crate) const SECONDS_PER_MINUTE: i32 = 60;
    pub(crate) const MS_PER_MINUTE: i32 = Self::MS_PER_SECOND * Self::SECONDS_PER_MINUTE;
    pub(crate) const MINUTES_PER_HOUR: i32 = 60;
    pub(crate) const MS_PER_HOUR: i32 = Self::MS_PER_MINUTE * Self::MINUTES_PER_HOUR;
    pub(crate) const HOURS_PER_DAY: i32 = 24;
    pub(crate) const MS_PER_DAY: i32 = Self::MS_PER_HOUR * Self::HOURS_PER_DAY;

    const MONTH_OFFSETS: [[u16; 13]; 2] = [
        [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365],
        [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335, 366],
    ];

    pub(crate) const EPOCH: Self = Self(0.0);
    pub(crate) const INVALID: Self = Self(f64::NAN);

    pub(crate) const fn from_millis_unclipped(value: f64) -> Self {
        Self(value)
    }

    pub(crate) const fn time(self) -> f64 {
        self.0
    }

    pub(crate) fn is_valid(self) -> bool {
        self.0.is_finite()
    }

    /// ECMA-262 TimeClip.
    pub(crate) fn clip(self) -> Self {
        const LIMIT: f64 = 100_000_000.0 * Date::MS_PER_DAY as f64;

        if !self.is_valid() || self.0.abs() > LIMIT {
            return Self::INVALID;
        }

        Self(self.0.trunc())
    }

    /// ECMA-262 Day.
    pub(crate) fn day(self) -> f64 {
        (self.0 / f64::from(Self::MS_PER_DAY)).floor()
    }

    /// ECMA-262 DayFromYear.
    pub(crate) fn day_from_year(year: f64) -> f64 {
        (365.0 * (year - 1970.0)) + ((year - 1969.0) / 4.0).floor()
            - ((year - 1901.0) / 100.0).floor()
            + ((year - 1601.0) / 400.0).floor()
    }

    /// ECMA-262 TimeFromYear.
    fn from_year(year: i32) -> Self {
        Self(f64::from(Self::MS_PER_DAY) * Self::day_from_year(f64::from(year)))
    }

    /// ECMA-262 YearFromTime.
    pub(crate) fn year(self) -> i32 {
        let day = self.day();

        let mut low = (day / if self < Self::EPOCH { 365.0 } else { 366.0 }).floor() as i32 + 1970;
        let mut high = (day / if self < Self::EPOCH { 366.0 } else { 365.0 }).ceil() as i32 + 1970;

        while low < high {
            let pivot = ((f64::from(low) + f64::from(high)) / 2.0) as i32;

            if Self::from_year(pivot) <= self {
                if Self::from_year(pivot + 1) > self {
                    return pivot;
                }

                low = pivot + 1;
            } else {
                high = pivot - 1;
            }
        }

        low
    }

    pub(crate) const fn is_leap_year(year: i32) -> bool {
        year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
    }

    fn in_leap_year(self) -> bool {
        Self::is_leap_year(self.year())
    }

    /// ECMA-262 DayWithinYear.
    pub(crate) fn day_within_year(self) -> i32 {
        (self.day() - Self::day_from_year(f64::from(self.year()))) as i32
    }

    /// ECMA-262 MonthFromTime.
    pub(crate) fn month(self) -> i32 {
        let day = self.day_within_year();
        let leap = usize::from(self.in_leap_year());

        for month in 0..11 {
            if day < i32::from(Self::MONTH_OFFSETS[leap][month + 1]) {
                return month as i32;
            }
        }

        11
    }

    /// ECMA-262 DateFromTime.
    pub(crate) fn date(self) -> i32 {
        let month = self.month();
        let leap = usize::from(self.in_leap_year());
        let month_offset = Self::MONTH_OFFSETS[leap][month as usize];

        self.day_within_year() - i32::from(month_offset) + 1
    }

    /// ECMA-262 WeekDay.
    pub(crate) fn week_day(self) -> i32 {
        (self.day() + 4.0).rem_euclid(7.0) as i32
    }

    /// ECMA-262 HourFromTime.
    ///
    /// The `+ 0.5` matches Flash's floating-point behavior. In particular,
    /// this reproduces the observed hour rollover at 8639999999999999.
    pub(crate) fn hours(self) -> i32 {
        (((self.0 + 0.5) / f64::from(Self::MS_PER_HOUR)).floor())
            .rem_euclid(f64::from(Self::HOURS_PER_DAY)) as i32
    }

    /// ECMA-262 MinFromTime.
    pub(crate) fn minutes(self) -> i32 {
        ((self.0 / f64::from(Self::MS_PER_MINUTE)).floor())
            .rem_euclid(f64::from(Self::MINUTES_PER_HOUR)) as i32
    }

    /// ECMA-262 SecFromTime.
    pub(crate) fn seconds(self) -> i32 {
        ((self.0 / f64::from(Self::MS_PER_SECOND)).floor())
            .rem_euclid(f64::from(Self::SECONDS_PER_MINUTE)) as i32
    }

    /// ECMA-262 msFromTime.
    pub(crate) fn milliseconds(self) -> i32 {
        self.0.rem_euclid(f64::from(Self::MS_PER_SECOND)) as i32
    }

    fn day_from_month(year: f64, month: f64) -> f64 {
        if !year.is_finite() || !month.is_finite() {
            return f64::NAN;
        }

        let year = year as i32;
        let month = month as i32;

        if !(0..12).contains(&month) {
            return f64::NAN;
        }

        let leap = usize::from(Self::is_leap_year(year));
        let month_offset = Self::MONTH_OFFSETS[leap][month as usize];

        Self::day_from_year(f64::from(year)) + f64::from(month_offset)
    }

    /// ECMA-262 MakeTime.
    pub(crate) fn make_time(hours: f64, minutes: f64, seconds: f64, milliseconds: f64) -> f64 {
        if !hours.is_finite()
            || !minutes.is_finite()
            || !seconds.is_finite()
            || !milliseconds.is_finite()
        {
            return f64::NAN;
        }

        let hours = hours.trunc();
        let minutes = minutes.trunc();
        let seconds = seconds.trunc();
        let milliseconds = milliseconds.trunc();

        hours * f64::from(Self::MS_PER_HOUR)
            + minutes * f64::from(Self::MS_PER_MINUTE)
            + seconds * f64::from(Self::MS_PER_SECOND)
            + milliseconds
    }

    /// ECMA-262 MakeDay.
    pub(crate) fn make_day(year: f64, month: f64, date: f64) -> f64 {
        if !year.is_finite() || !month.is_finite() || !date.is_finite() {
            return f64::NAN;
        }

        let mut year = year.trunc();
        let month = month.trunc();
        let date = date.trunc();

        year += (month / 12.0).floor();
        let month = month.rem_euclid(12.0);

        Self::day_from_month(year, month) + date - 1.0
    }

    /// ECMA-262 MakeDate.
    pub(crate) fn make_date(day: f64, time: f64) -> Self {
        Self(day * f64::from(Self::MS_PER_DAY) + time)
    }
}

#[cfg(test)]
mod tests {
    use super::Date;

    #[allow(clippy::too_many_arguments)]
    fn assert_fields(
        date: Date,
        year: i32,
        month: i32,
        day: i32,
        hour: i32,
        minute: i32,
        second: i32,
        millisecond: i32,
    ) {
        assert_eq!(date.year(), year);
        assert_eq!(date.month(), month);
        assert_eq!(date.date(), day);
        assert_eq!(date.hours(), hour);
        assert_eq!(date.minutes(), minute);
        assert_eq!(date.seconds(), second);
        assert_eq!(date.milliseconds(), millisecond);
    }

    #[test]
    fn time_clip_truncates_toward_zero() {
        assert_eq!(Date::from_millis_unclipped(5.9).clip().time(), 5.0);
        assert_eq!(Date::from_millis_unclipped(-5.9).clip().time(), -5.0);
        assert_eq!(Date::from_millis_unclipped(0.9).clip().time(), 0.0);
        assert_eq!(Date::from_millis_unclipped(-0.9).clip().time(), -0.0);
    }

    #[test]
    fn time_clip_range() {
        assert_eq!(
            Date::from_millis_unclipped(-8_640_000_000_000_000.0)
                .clip()
                .time(),
            -8_640_000_000_000_000.0
        );
        assert_eq!(
            Date::from_millis_unclipped(8_640_000_000_000_000.0)
                .clip()
                .time(),
            8_640_000_000_000_000.0
        );

        assert!(
            Date::from_millis_unclipped(-8_640_000_000_000_001.0)
                .clip()
                .time()
                .is_nan()
        );
        assert!(
            Date::from_millis_unclipped(8_640_000_000_000_001.0)
                .clip()
                .time()
                .is_nan()
        );
    }

    #[test]
    fn flash_date_range_fields() {
        assert_fields(
            Date::from_millis_unclipped(-8_640_000_000_000_000.0),
            -271821,
            3,
            20,
            0,
            0,
            0,
            0,
        );

        assert_fields(
            Date::from_millis_unclipped(8_210_266_876_800_000.0),
            262143,
            0,
            1,
            0,
            0,
            0,
            0,
        );

        assert_fields(
            Date::from_millis_unclipped(8_300_000_000_000_000.0),
            264986,
            6,
            12,
            19,
            33,
            20,
            0,
        );

        assert_fields(
            Date::from_millis_unclipped(8_640_000_000_000_000.0),
            275760,
            8,
            13,
            0,
            0,
            0,
            0,
        );
    }

    #[test]
    fn flash_extreme_hour_rounding_quirk() {
        assert_fields(
            Date::from_millis_unclipped(8_639_999_999_999_998.0),
            275760,
            8,
            12,
            23,
            59,
            59,
            998,
        );

        assert_fields(
            Date::from_millis_unclipped(8_639_999_999_999_999.0),
            275760,
            8,
            12,
            0,
            59,
            59,
            999,
        );
    }

    #[test]
    fn make_time_truncates_fields_toward_zero() {
        assert_eq!(Date::make_time(1.9, 2.9, 3.9, 4.9), 3_723_004.0);

        assert_eq!(Date::make_time(-1.9, -2.9, -3.9, -4.9), -3_723_004.0);
    }

    #[test]
    fn make_day_truncates_and_normalizes() {
        let day = Date::make_day(2000.0, -1.9, 1.0);
        let date = Date::make_date(day, 0.0);

        assert_fields(date, 1999, 11, 1, 0, 0, 0, 0);

        let day = Date::make_day(2000.0, 0.0, -1.9);
        let date = Date::make_date(day, 0.0);

        assert_fields(date, 1999, 11, 30, 0, 0, 0, 0);
    }
}
