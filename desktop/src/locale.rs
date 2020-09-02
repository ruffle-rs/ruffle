use ruffle_core::backend::locale::LocaleBackend;
use ruffle_core::chrono::{DateTime, FixedOffset, Local, Offset, Utc};

pub struct DesktopLocaleBackend();

impl DesktopLocaleBackend {
    pub fn new() -> Self {
        Self()
    }
}

impl LocaleBackend for DesktopLocaleBackend {
    fn get_current_date_time(&self) -> DateTime<Utc> {
        Utc::now()
    }

    fn get_timezone(&self) -> FixedOffset {
        Local::now().offset().fix()
    }
}
