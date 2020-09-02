use chrono::{DateTime, FixedOffset, Local, Offset, Utc};
use ruffle_core::backend::locale::LocaleBackend;

pub struct WebLocaleBackend();

impl WebLocaleBackend {
    pub fn new() -> Self {
        Self()
    }
}

impl LocaleBackend for WebLocaleBackend {
    fn get_current_date_time(&self) -> DateTime<Utc> {
        Utc::now()
    }

    fn get_timezone(&self) -> FixedOffset {
        Local::now().offset().fix()
    }
}
