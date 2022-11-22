use chrono::{DateTime, FixedOffset, Local, Offset, TimeZone, Utc};

// For tests, we emulate being in Nepal with a local time of 2001-02-03 at 04:05:06.
// Nepal has a timezone offset of +5:45, and has never used DST.
// This makes it an ideal candidate for fixed tests.
const MOCK_TIME: bool = cfg!(any(test, feature = "deterministic"));

pub fn get_current_date_time() -> DateTime<Utc> {
    if MOCK_TIME {
        get_timezone()
            .with_ymd_and_hms(2001, 2, 3, 4, 5, 6)
            .single()
            .expect("Unambiguous mock time")
            .into()
    } else {
        Utc::now()
    }
}

pub fn get_timezone() -> FixedOffset {
    if MOCK_TIME {
        FixedOffset::east_opt(20700).expect("Unambiguous mock timezone")
    } else {
        Local::now().offset().fix()
    }
}
