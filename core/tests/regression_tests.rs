//! Tests running SWFs in a headless Ruffle instance.
//!
//! Trace output can be compared with correct output from the official Flash Payer.

use approx::assert_abs_diff_eq;
use log::{Metadata, Record};
use ruffle_core::backend::{
    audio::NullAudioBackend, input::NullInputBackend, navigator::NullNavigatorBackend,
    render::NullRenderer,
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

// This macro generates test cases for a given list of SWFs using `test_swf_approx`.
macro_rules! swf_tests_approx {
    ($($(#[$attr:meta])* ($name:ident, $path:expr, $num_frames:literal, $epsilon:literal),)*) => {
        $(
        #[test]
        $(#[$attr])*
        fn $name() -> Result<(), Error> {
            test_swf_approx(
                concat!("tests/swfs/", $path, "/test.swf"),
                $num_frames,
                concat!("tests/swfs/", $path, "/output.txt"),
                $epsilon
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
    (add_property, "avm1/add_property", 1),
    (as_transformed_flag, "avm1/as_transformed_flag", 3),
    (attach_movie, "avm1/attach_movie", 1),
    (function_base_clip, "avm1/function_base_clip", 2),
    (call, "avm1/call", 2),
    (color, "avm1/color", 1),
    (clip_events, "avm1/clip_events", 4),
    (create_empty_movie_clip, "avm1/create_empty_movie_clip", 2),
    (duplicate_movie_clip, "avm1/duplicate_movie_clip", 1),
    (mouse_listeners, "avm1/mouse_listeners", 1),
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
    (goto_frame, "avm1/goto_frame", 8),
    (goto_frame2, "avm1/goto_frame2", 20),
    (goto_label, "avm1/goto_label", 8),
    (goto_methods, "avm1/goto_methods", 1),
    (goto_rewind1, "avm1/goto_rewind1", 10),
    (goto_rewind2, "avm1/goto_rewind2", 10),
    (goto_rewind3, "avm1/goto_rewind3", 10),
    (goto_execution_order, "avm1/goto_execution_order", 3),
    (goto_execution_order2, "avm1/goto_execution_order2", 3),
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
    (array_trivial, "avm1/array_trivial", 1),
    (array_concat, "avm1/array_concat", 1),
    (array_slice, "avm1/array_slice", 1),
    (array_splice, "avm1/array_splice", 1),
    (array_properties, "avm1/array_properties", 1),
    (array_prototyping, "avm1/array_prototyping", 1),
    (array_vs_object_length, "avm1/array_vs_object_length", 1),
    (timeline_function_def, "avm1/timeline_function_def", 3),
    (root_global_parent, "avm1/root_global_parent", 3),
    (register_underflow, "avm1/register_underflow", 1),
    (object_prototypes, "avm1/object_prototypes", 1),
    (movieclip_prototype_extension, "avm1/movieclip_prototype_extension", 1),
    (movieclip_hittest, "avm1/movieclip_hittest", 1),
    #[ignore] (textfield_text, "avm1/textfield_text", 1),
    (recursive_prototypes, "avm1/recursive_prototypes", 1),
    (stage_object_children, "avm1/stage_object_children", 2),
    (has_own_property, "avm1/has_own_property", 1),
    (extends_chain, "avm1/extends_chain", 1),
    (is_prototype_of, "avm1/is_prototype_of", 1),
    #[ignore] (string_coercion, "avm1/string_coercion", 1),
    (lessthan_swf4, "avm1/lessthan_swf4", 1),
    (lessthan2_swf5, "avm1/lessthan2_swf5", 1),
    (lessthan2_swf6, "avm1/lessthan2_swf6", 1),
    (lessthan2_swf7, "avm1/lessthan2_swf7", 1),
    (logical_ops_swf4, "avm1/logical_ops_swf4", 1),
    (logical_ops_swf8, "avm1/logical_ops_swf8", 1),
    (movieclip_depth_methods, "avm1/movieclip_depth_methods", 3),
    (greater_swf6, "avm1/greater_swf6", 1),
    (greater_swf7, "avm1/greater_swf7", 1),
    (equals_swf4, "avm1/equals_swf4", 1),
    (equals2_swf5, "avm1/equals2_swf5", 1),
    (equals2_swf6, "avm1/equals2_swf6", 1),
    (equals2_swf7, "avm1/equals2_swf7", 1),
    (set_variable_scope, "avm1/set_variable_scope", 1),
    (slash_syntax, "avm1/slash_syntax", 2),
    (strictequals_swf6, "avm1/strictequals_swf6", 1),
    (string_methods, "avm1/string_methods", 1),
    (target_path, "avm1/target_path", 1),
    (global_is_bare, "avm1/global_is_bare", 1),
    (primitive_type_globals, "avm1/primitive_type_globals", 1),
    (as2_oop, "avm1/as2_oop", 1),
    (xml, "avm1/xml", 1),
    (xml_namespaces, "avm1/xml_namespaces", 1),
    (xml_node_namespaceuri, "avm1/xml_node_namespaceuri", 1),
    (xml_node_weirdnamespace, "avm1/xml_node_weirdnamespace", 1),
    (xml_clone_expandos, "avm1/xml_clone_expandos", 1),
    (xml_has_child_nodes, "avm1/xml_has_child_nodes", 1),
    (xml_first_last_child, "avm1/xml_first_last_child", 1),
    (xml_parent_and_child, "avm1/xml_parent_and_child", 1),
    (xml_siblings, "avm1/xml_siblings", 1),
    (xml_attributes_read, "avm1/xml_attributes_read", 1),
    (xml_append_child, "avm1/xml_append_child", 1),
    (xml_append_child_with_parent, "avm1/xml_append_child_with_parent", 1),
    (xml_remove_node, "avm1/xml_remove_node", 1),
    (xml_insert_before, "avm1/xml_insert_before", 1),
    (xml_to_string, "avm1/xml_to_string", 1),
    (xml_to_string_comment, "avm1/xml_to_string_comment", 1),
    (xml_idmap, "avm1/xml_idmap", 1),
    (xml_inspect_doctype, "avm1/xml_inspect_doctype", 1),
    #[ignore] (xml_inspect_xmldecl, "avm1/xml_inspect_xmldecl", 1),
    (xml_inspect_createmethods, "avm1/xml_inspect_createmethods", 1),
    (xml_inspect_parsexml, "avm1/xml_inspect_parsexml", 1),
    (funky_function_calls, "avm1/funky_function_calls", 1),
    (undefined_to_string_swf6, "avm1/undefined_to_string_swf6", 1),
}

// TODO: These tests have some inaccuracies currently, so we use approx_eq to test that numeric values are close enough.
// Eventually we can hopefully make these match exactly (see #193).
swf_tests_approx! {
    (local_to_global, "avm1/local_to_global", 1, 0.4),
    (stage_object_properties, "avm1/stage_object_properties", 4, 0.051),
    (stage_object_properties_swf6, "avm1/stage_object_properties_swf6", 4, 0.051),
}

#[test]
fn test_prototype_enumerate() -> Result<(), Error> {
    let trace_log = run_swf("tests/swfs/avm1/prototype_enumerate/test.swf", 1)?;
    let mut actual: Vec<String> = trace_log.lines().map(|s| s.to_string()).collect();
    let mut expected = vec!["a", "b", "c", "d", "e"];

    actual.sort();
    expected.sort();

    assert_eq!(actual, expected, "ruffle output != flash player output");
    Ok(())
}

#[test]
fn test_stage_object_enumerate() -> Result<(), Error> {
    let trace_log = run_swf("tests/swfs/avm1/stage_object_enumerate/test.swf", 1)?;
    let mut actual: Vec<String> = trace_log.lines().map(|s| s.to_string()).collect();
    let mut expected = vec!["clip1", "clip2", "clip3", "foo"];

    actual.sort();
    expected.sort();

    assert_eq!(actual, expected, "ruffle output != flash player output");
    Ok(())
}

/// Wrapper around string slice that makes debug output `{:?}` to print string same way as `{}`.
/// Used in different `assert*!` macros in combination with `pretty_assertions` crate to make
/// test failures to show nice diffs.
/// Courtesy of https://github.com/colin-kiegel/rust-pretty-assertions/issues/24
#[derive(PartialEq, Eq)]
#[doc(hidden)]
pub struct PrettyString<'a>(pub &'a str);

/// Make diff to display string as multi-line string
impl<'a> std::fmt::Debug for PrettyString<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

macro_rules! assert_eq {
    ($left:expr, $right:expr) => {
        pretty_assertions::assert_eq!(PrettyString($left.as_ref()), PrettyString($right.as_ref()));
    };
    ($left:expr, $right:expr, $message:expr) => {
        pretty_assertions::assert_eq!(
            PrettyString($left.as_ref()),
            PrettyString($right.as_ref()),
            $message
        );
    };
}

/// Loads an SWF and runs it through the Ruffle core for a number of frames.
/// Tests that the trace output matches the given expected output.
fn test_swf(swf_path: &str, num_frames: u32, expected_output_path: &str) -> Result<(), Error> {
    let expected_output = std::fs::read_to_string(expected_output_path)?.replace("\r\n", "\n");

    let trace_log = run_swf(swf_path, num_frames)?;
    assert_eq!(
        trace_log, expected_output,
        "ruffle output != flash player output"
    );

    Ok(())
}

/// Loads an SWF and runs it through the Ruffle core for a number of frames.
/// Tests that the trace output matches the given expected output.
/// If a line has a floating point value, it will be compared approxinmately using the given epsilon.
fn test_swf_approx(
    swf_path: &str,
    num_frames: u32,
    expected_output_path: &str,
    epsilon: f64,
) -> Result<(), Error> {
    let trace_log = run_swf(swf_path, num_frames)?;
    let expected_data = std::fs::read_to_string(expected_output_path)?;
    std::assert_eq!(
        trace_log.lines().count(),
        expected_data.lines().count(),
        "# of lines of output didn't match"
    );

    for (actual, expected) in trace_log.lines().zip(expected_data.lines()) {
        // If these are numbers, compare using approx_eq.
        if let (Ok(actual), Ok(expected)) = (actual.parse::<f64>(), expected.parse::<f64>()) {
            // TODO: Lower this epsilon as the accuracy of the properties improves.
            assert_abs_diff_eq!(actual, expected, epsilon = epsilon);
        } else {
            assert_eq!(actual, expected);
        }
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
        NullInputBackend::new(),
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
