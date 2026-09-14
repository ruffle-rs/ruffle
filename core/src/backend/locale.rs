use chrono::{DateTime, FixedOffset, Local, Offset, TimeZone, Utc};

/// Provides locale-specific environment data.
pub trait LocaleBackend {
    fn get_current_date_time(&self) -> DateTime<Utc>;

    fn get_timezone(&self) -> FixedOffset;
}

/// Reads the current date/time and timezone from the host system.
pub struct DefaultLocaleBackend {}

impl DefaultLocaleBackend {
    pub fn new() -> Self {
        Self {}
    }
}

impl LocaleBackend for DefaultLocaleBackend {
    fn get_current_date_time(&self) -> DateTime<Utc> {
        Utc::now()
    }

    fn get_timezone(&self) -> FixedOffset {
        Local::now().offset().fix()
    }
}

impl Default for DefaultLocaleBackend {
    fn default() -> Self {
        DefaultLocaleBackend::new()
    }
}

/// A `LocaleBackend` with a fixed date/time and timezone, for deterministic
/// execution.
pub struct DeterministicLocaleBackend {
    date_time: DateTime<FixedOffset>,
}

impl DeterministicLocaleBackend {
    pub fn new(date_time: DateTime<FixedOffset>) -> Self {
        Self { date_time }
    }
}

impl LocaleBackend for DeterministicLocaleBackend {
    fn get_current_date_time(&self) -> DateTime<Utc> {
        self.date_time.into()
    }

    fn get_timezone(&self) -> FixedOffset {
        *self.date_time.offset()
    }
}

impl Default for DeterministicLocaleBackend {
    fn default() -> Self {
        // Emulates being in Nepal with a local time of 2001-02-03 at 04:05:06.
        // Nepal has a timezone offset of +5:45, and has never used DST.
        // This makes it an ideal candidate for fixed tests.
        Self::new(
            FixedOffset::east_opt(20700)
                .expect("Unambiguous mock timezone")
                .with_ymd_and_hms(2001, 2, 3, 4, 5, 6)
                .single()
                .expect("Unambiguous mock time"),
        )
    }
}
