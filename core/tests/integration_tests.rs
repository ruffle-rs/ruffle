//! Tests running SWFs in a headless Ruffle instance.
//!
//! Trace output can be compared with correct output from the official Flash Payer.

use log::{Metadata, Record};
use ruffle_core::backend::{
    audio::NullAudioBackend, navigator::NullNavigatorBackend, render::NullRenderer,
};
use ruffle_core::Player;
use std::cell::RefCell;

type Error = Box<dyn std::error::Error>;

// This macro generates test cases for a given list of SWFs.
macro_rules! swf_tests {
    ($(($name:ident, $path:expr, $num_frames:literal),)*) => {
    $(
        #[test]
        fn $name() -> Result<(), Error> {
            test_swf(
                concat!("tests/swfs/", $path, "/test.swf"),
                $num_frames,
                concat!("tests/swfs/", $path, "/output.txt"),
            )
        }
    )*
    }
}

// List of SWFs to test.
// Format: (test_name, test_folder, number_of_frames_to_run)
// The test folder is a relative to core/tests/swfs
// Inside the folder is expected to be "test.swf" and "output.txt" with the correct output.
swf_tests! {
    (single_frame, "avm1/single_frame", 2),
    (looping, "avm1/looping", 6),
    (goto_advance1, "avm1/goto_advance1", 10),
    (goto_advance2, "avm1/goto_advance2", 10),
    (goto_both_ways1, "avm1/goto_both_ways1", 10),
    (goto_both_ways2, "avm1/goto_both_ways2", 10),
    (goto_rewind1, "avm1/goto_rewind1", 10),
    (goto_rewind2, "avm1/goto_rewind2", 10),
    (goto_rewind3, "avm1/goto_rewind3", 10),
    (tell_target, "avm1/tell_target", 3),
}

/// Loads an SWF and runs it through the Ruffle core for a number of frames.
/// Tests that the trace output matches the given expected output.
fn test_swf(swf_path: &str, num_frames: u32, expected_output_path: &str) -> Result<(), Error> {
    let _ = log::set_logger(&TRACE_LOGGER).map(|()| log::set_max_level(log::LevelFilter::Info));
    let expected_output = std::fs::read_to_string(expected_output_path)?.replace("\r\n", "\n");

    let swf_data = std::fs::read(swf_path)?;
    let mut player = Player::new(
        NullRenderer,
        NullAudioBackend::new(),
        NullNavigatorBackend::new(),
        swf_data,
    )?;

    for _ in 0..num_frames {
        player.run_frame();
    }

    assert_eq!(trace_log(), expected_output);

    Ok(())
}

thread_local! {
    static TRACE_LOG: RefCell<String> = RefCell::new(String::new());
}

static TRACE_LOGGER: TraceLogger = TraceLogger;

/// `TraceLogger` captures output from AVM trace actions into a String.
struct TraceLogger;

fn trace_log() -> String {
    TRACE_LOG.with(|log| log.borrow().clone())
}

impl log::Log for TraceLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.target() == "avm_trace"
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            TRACE_LOG.with(|log| log.borrow_mut().push_str(&format!("{}\n", record.args())));
        }
    }

    fn flush(&self) {}
}
