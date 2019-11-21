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
    ($($(#[$attr:meta])* ($name:ident, $path:expr, $num_frames:literal),)*) => {
        $(
        #[test]
        $(#[$attr])*
        fn $name() -> Result<(), Error> {
            test_swf(
                concat!("tests/swfs/", $path, "/test.swf"),
                $num_frames,
                concat!("tests/swfs/", $path, "/output.txt"),
            )
        }
        )*
    };
}

// List of SWFs to test.
// Format: (test_name, test_folder, number_of_frames_to_run)
// The test folder is a relative to core/tests/swfs
// Inside the folder is expected to be "test.swf" and "output.txt" with the correct output.
swf_tests! {
    (do_init_action, "avm1/do_init_action", 3),
    (execution_order1, "avm1/execution_order1", 3),
    (execution_order2, "avm1/execution_order2", 15),
    (execution_order3, "avm1/execution_order3", 5),
    (single_frame, "avm1/single_frame", 2),
    (looping, "avm1/looping", 6),
    (goto_advance1, "avm1/goto_advance1", 10),
    (goto_advance2, "avm1/goto_advance2", 10),
    (goto_both_ways1, "avm1/goto_both_ways1", 10),
    (goto_both_ways2, "avm1/goto_both_ways2", 10),
    (goto_rewind1, "avm1/goto_rewind1", 10),
    (goto_rewind2, "avm1/goto_rewind2", 10),
    (goto_rewind3, "avm1/goto_rewind3", 10),
    (goto_execution_order, "avm1/goto_execution_order", 3),
    (greaterthan_swf5, "avm1/greaterthan_swf5", 1),
    (greaterthan_swf8, "avm1/greaterthan_swf8", 1),
    (strictly_equals, "avm1/strictly_equals", 1),
    (tell_target, "avm1/tell_target", 3),
    (typeofs, "avm1/typeof", 1),
    (typeof_globals, "avm1/typeof_globals", 1),
    (closure_scope, "avm1/closure_scope", 1),
    (variable_args, "avm1/variable_args", 1),
    (custom_clip_methods, "avm1/custom_clip_methods", 3),
    (delete, "avm1/delete", 3),
    (timeline_function_def, "avm1/timeline_function_def", 3),
    (root_global_parent, "avm1/root_global_parent", 3),
    (register_underflow, "avm1/register_underflow", 1),
    (object_prototypes, "avm1/object_prototypes", 1),
    (movieclip_prototype_extension, "avm1/movieclip_prototype_extension", 1),
    (recursive_prototypes, "avm1/recursive_prototypes", 1),
    (has_own_property, "avm1/has_own_property", 1),
    #[ignore] (extends_chain, "avm1/extends_chain", 1),
    (is_prototype_of, "avm1/is_prototype_of", 1),
    #[ignore] (string_coercion, "avm1/string_coercion", 1),
    (lessthan_swf6, "avm1/lessthan_swf6", 1),
    (lessthan_swf7, "avm1/lessthan_swf7", 1),
}

#[test]
fn test_prototype_enumerate() -> Result<(), Error> {
    let trace_log = run_swf("tests/swfs/avm1/prototype_enumerate/test.swf", 1)?;
    let mut actual: Vec<String> = trace_log.lines().map(|s| s.to_string()).collect();
    let mut expected = vec!["a", "b", "c", "d", "e"];

    actual.sort();
    expected.sort();

    assert_eq!(actual, expected);
    Ok(())
}

/// Loads an SWF and runs it through the Ruffle core for a number of frames.
/// Tests that the trace output matches the given expected output.
fn test_swf(swf_path: &str, num_frames: u32, expected_output_path: &str) -> Result<(), Error> {
    let expected_output = std::fs::read_to_string(expected_output_path)?.replace("\r\n", "\n");

    let trace_log = run_swf(swf_path, num_frames)?;
    if trace_log != expected_output {
        println!(
            "Ruffle output:\n{}\nExpected output:\n{}",
            trace_log, expected_output
        );
        panic!("Ruffle output did not match expected output.");
    }

    Ok(())
}

/// Loads an SWF and runs it through the Ruffle core for a number of frames.
/// Tests that the trace output matches the given expected output.
fn run_swf(swf_path: &str, num_frames: u32) -> Result<String, Error> {
    let _ = log::set_logger(&TRACE_LOGGER).map(|()| log::set_max_level(log::LevelFilter::Info));

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

    Ok(trace_log())
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
