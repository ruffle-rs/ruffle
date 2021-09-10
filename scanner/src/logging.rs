//! Utilities and mock objects needed for log output capture

use log::{Level, LevelFilter, Log, Metadata, Record};
use ruffle_core::backend::log::LogBackend;
use std::cell::RefCell;
use std::rc::Rc;

/// Log backend that specifically discards AVM trace output
pub struct ScanLogBackend();

impl ScanLogBackend {
    pub fn new() -> Self {
        Self()
    }
}

impl LogBackend for ScanLogBackend {
    fn avm_trace(&self, _message: &str) {}
}

thread_local! {
    /// Thread local log buffer.
    pub static LOCAL_LOGGER: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
}

/// `log` backend (not to be confused with Ruffle's notion of a log backend)
/// that only logs errors to a thread-local area.
pub struct ThreadLocalScanLogger();

static GLOBAL_LOGGER: ThreadLocalScanLogger = ThreadLocalScanLogger();

impl ThreadLocalScanLogger {
    pub fn init() {
        log::set_logger(&GLOBAL_LOGGER)
            .map(|()| log::set_max_level(LevelFilter::Info))
            .unwrap();
    }
}

impl Log for ThreadLocalScanLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() == Level::Error
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            LOCAL_LOGGER.with(|log_buffer| {
                log_buffer.borrow_mut().push(format!("{}", record.args()));
            })
        }
    }

    fn flush(&self) {}
}
