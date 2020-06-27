//! Tests running SWFs in a headless Ruffle instance.
//!
//! Trace output can be compared with correct output from the official Flash Payer.

use approx::assert_abs_diff_eq;
use log::{Metadata, Record};
use ruffle_core::backend::navigator::{NullExecutor, NullNavigatorBackend};
use ruffle_core::backend::storage::MemoryStorageBackend;
use ruffle_core::backend::{
    audio::NullAudioBackend, input::NullInputBackend, render::NullRenderer,
};
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::Player;
use std::cell::RefCell;
use std::path::Path;
use std::sync::Arc;

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
    (as_broadcaster, "avm1/as_broadcaster", 1),
    (as_broadcaster_initialize, "avm1/as_broadcaster_initialize", 1),
    (attach_movie, "avm1/attach_movie", 1),
    (function_base_clip, "avm1/function_base_clip", 2),
    (call, "avm1/call", 2),
    (color, "avm1/color", 1),
    (clip_events, "avm1/clip_events", 4),
    (create_empty_movie_clip, "avm1/create_empty_movie_clip", 2),
    (empty_movieclip_can_attach_movies, "avm1/empty_movieclip_can_attach_movies", 1),
    (duplicate_movie_clip, "avm1/duplicate_movie_clip", 1),
    (mouse_listeners, "avm1/mouse_listeners", 1),
    (do_init_action, "avm1/do_init_action", 3),
    (execution_order1, "avm1/execution_order1", 3),
    (execution_order2, "avm1/execution_order2", 15),
    (execution_order3, "avm1/execution_order3", 5),
    (single_frame, "avm1/single_frame", 2),
    (looping, "avm1/looping", 6),
    (matrix, "avm1/matrix", 1),
    (point, "avm1/point", 1),
    (rectangle, "avm1/rectangle", 1),
    (goto_advance1, "avm1/goto_advance1", 2),
    (goto_advance2, "avm1/goto_advance2", 2),
    (goto_both_ways1, "avm1/goto_both_ways1", 2),
    (goto_both_ways2, "avm1/goto_both_ways2", 3),
    (goto_frame, "avm1/goto_frame", 3),
    (goto_frame2, "avm1/goto_frame2", 5),
    (goto_frame_number, "avm1/goto_frame_number", 4),
    (goto_label, "avm1/goto_label", 4),
    (goto_methods, "avm1/goto_methods", 1),
    (goto_rewind1, "avm1/goto_rewind1", 4),
    (goto_rewind2, "avm1/goto_rewind2", 5),
    (goto_rewind3, "avm1/goto_rewind3", 2),
    (goto_execution_order, "avm1/goto_execution_order", 3),
    (goto_execution_order2, "avm1/goto_execution_order2", 2),
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
    (default_names, "avm1/default_names", 6),
    (array_trivial, "avm1/array_trivial", 1),
    (array_concat, "avm1/array_concat", 1),
    (array_slice, "avm1/array_slice", 1),
    (array_splice, "avm1/array_splice", 1),
    (array_properties, "avm1/array_properties", 1),
    (array_prototyping, "avm1/array_prototyping", 1),
    (array_vs_object_length, "avm1/array_vs_object_length", 1),
    (array_sort, "avm1/array_sort", 1),
    (array_enumerate, "avm1/array_enumerate", 1),
    (timeline_function_def, "avm1/timeline_function_def", 3),
    (root_global_parent, "avm1/root_global_parent", 3),
    (register_underflow, "avm1/register_underflow", 1),
    (object_prototypes, "avm1/object_prototypes", 1),
    (movieclip_prototype_extension, "avm1/movieclip_prototype_extension", 1),
    (movieclip_hittest, "avm1/movieclip_hittest", 1),
    #[ignore] (textfield_text, "avm1/textfield_text", 1),
    (recursive_prototypes, "avm1/recursive_prototypes", 2),
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
    (get_variable_in_scope, "avm1/get_variable_in_scope", 1),
    (movieclip_init_object, "avm1/movieclip_init_object", 1),
    (greater_swf6, "avm1/greater_swf6", 1),
    (greater_swf7, "avm1/greater_swf7", 1),
    (equals_swf4, "avm1/equals_swf4", 1),
    (equals2_swf5, "avm1/equals2_swf5", 1),
    (equals2_swf6, "avm1/equals2_swf6", 1),
    (equals2_swf7, "avm1/equals2_swf7", 1),
    (register_class, "avm1/register_class", 1),
    (register_and_init_order, "avm1/register_and_init_order", 1),
    (on_construct, "avm1/on_construct", 1),
    (set_variable_scope, "avm1/set_variable_scope", 1),
    (slash_syntax, "avm1/slash_syntax", 2),
    (strictequals_swf6, "avm1/strictequals_swf6", 1),
    (string_methods, "avm1/string_methods", 1),
    (target_path, "avm1/target_path", 1),
    (global_is_bare, "avm1/global_is_bare", 1),
    (primitive_type_globals, "avm1/primitive_type_globals", 1),
    (primitive_instanceof, "avm1/primitive_instanceof", 1),
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
    (define_function2_preload, "avm1/define_function2_preload", 1),
    (define_function2_preload_order, "avm1/define_function2_preload_order", 1),
    (mcl_as_broadcaster, "avm1/mcl_as_broadcaster", 1),
    (uncaught_exception, "avm1/uncaught_exception", 1),
    (uncaught_exception_bubbled, "avm1/uncaught_exception_bubbled", 1),
    (try_catch_finally, "avm1/try_catch_finally", 1),
    (try_finally_simple, "avm1/try_finally_simple", 1),
    (loadmovie, "avm1/loadmovie", 2),
    (loadmovienum, "avm1/loadmovienum", 2),
    (loadmovie_registerclass, "avm1/loadmovie_registerclass", 2),
    (loadmovie_method, "avm1/loadmovie_method", 2),
    (unloadmovie, "avm1/unloadmovie", 11),
    (unloadmovienum, "avm1/unloadmovienum", 11),
    (unloadmovie_method, "avm1/unloadmovie_method", 11),
    (mcl_loadclip, "avm1/mcl_loadclip", 11),
    (mcl_unloadclip, "avm1/mcl_unloadclip", 11),
    (mcl_getprogress, "avm1/mcl_getprogress", 6),
    (load_vars, "avm1/load_vars", 2),
    (loadvariables, "avm1/loadvariables", 3),
    (loadvariablesnum, "avm1/loadvariablesnum", 3),
    (loadvariables_method, "avm1/loadvariables_method", 3),
    (xml_load, "avm1/xml_load", 1),
    (with_return, "avm1/with_return", 1),
    (watch, "avm1/watch", 1),
    #[ignore] (watch_virtual_property, "avm1/watch_virtual_property", 1),
    (cross_movie_root, "avm1/cross_movie_root", 5),
    (roots_and_levels, "avm1/roots_and_levels", 1),
    (swf6_case_insensitive, "avm1/swf6_case_insensitive", 1),
    (swf7_case_sensitive, "avm1/swf7_case_sensitive", 1),
    (prototype_enumerate, "avm1/prototype_enumerate", 1),
    (stage_object_enumerate, "avm1/stage_object_enumerate", 1),
    (new_object_enumerate, "avm1/new_object_enumerate", 1),
    (as2_super_and_this_v6, "avm1/as2_super_and_this_v6", 1),
    (as2_super_and_this_v8, "avm1/as2_super_and_this_v8", 1),
    (as2_super_via_manual_prototype, "avm1/as2_super_via_manual_prototype", 1),
    (as1_constructor_v6, "avm1/as1_constructor_v6", 1),
    (as1_constructor_v7, "avm1/as1_constructor_v7", 1),
    (issue_710, "avm1/issue_710", 1),
    (infinite_recursion_function, "avm1/infinite_recursion_function", 1),
    (infinite_recursion_function_in_setter, "avm1/infinite_recursion_function_in_setter", 1),
    (infinite_recursion_virtual_property, "avm1/infinite_recursion_virtual_property", 1),
    (edittext_font_size, "avm1/edittext_font_size", 1),
    (edittext_default_format, "avm1/edittext_default_format", 1),
    (edittext_leading, "avm1/edittext_leading", 1),
    #[ignore] (edittext_newlines, "avm1/edittext_newlines", 1),
    (edittext_html_entity, "avm1/edittext_html_entity", 1),
    #[ignore] (edittext_html_roundtrip, "avm1/edittext_html_roundtrip", 1),
    (define_local, "avm1/define_local", 1),
    (textfield_variable, "avm1/textfield_variable", 8),
    (error, "avm1/error", 1),
    (color_transform, "avm1/color_transform", 1),
    (with, "avm1/with", 1),
    (arguments, "avm1/arguments", 1),
    (prototype_properties, "avm1/prototype_properties", 1),
    (stage_object_properties_get_var, "avm1/stage_object_properties_get_var", 1),
    (set_interval, "avm1/set_interval", 20),
    (context_menu, "avm1/context_menu", 1),
    (context_menu_item, "avm1/context_menu_item", 1),
    (constructor_function, "avm1/constructor_function", 1),
    (global_array, "avm1/global_array", 1),
    (array_constructor, "avm1/array_constructor", 1),
    (array_apply, "avm1/array_constructor", 1),
    (object_function, "avm1/object_function", 1),
    (as3_hello_world, "avm2/hello_world", 1),
    (as3_function_call, "avm2/function_call", 1),
    (as3_function_call_via_call, "avm2/function_call_via_call", 1),
    (as3_constructor_call, "avm2/constructor_call", 1),
    (as3_class_methods, "avm2/class_methods", 1),
    (as3_es3_inheritance, "avm2/es3_inheritance", 1),
    (as3_es4_inheritance, "avm2/es4_inheritance", 1),
    (as3_stored_properties, "avm2/stored_properties", 1),
    (as3_virtual_properties, "avm2/virtual_properties", 1),
    (as3_es4_oop_prototypes, "avm2/es4_oop_prototypes", 1),
    (as3_es4_method_binding, "avm2/es4_method_binding", 1),
    (as3_control_flow_bool, "avm2/control_flow_bool", 1),
    (as3_control_flow_stricteq, "avm2/control_flow_stricteq", 1),
    (as3_object_enumeration, "avm2/object_enumeration", 1),
    (as3_class_enumeration, "avm2/class_enumeration", 1),
    (as3_is_prototype_of, "avm2/is_prototype_of", 1),
    (as3_has_own_property, "avm2/has_own_property", 1),
    (as3_property_is_enumerable, "avm2/property_is_enumerable", 1),
    (as3_set_property_is_enumerable, "avm2/set_property_is_enumerable", 1),
    (as3_object_to_string, "avm2/object_to_string", 1),
    (as3_function_to_string, "avm2/function_to_string", 1),
    (as3_class_to_string, "avm2/class_to_string", 1),
    (as3_object_to_locale_string, "avm2/object_to_locale_string", 1),
    (as3_function_to_locale_string, "avm2/function_to_locale_string", 1),
    (as3_class_to_locale_string, "avm2/class_to_locale_string", 1),
    (as3_object_value_of, "avm2/object_value_of", 1),
    (as3_function_value_of, "avm2/function_value_of", 1),
    (as3_class_value_of, "avm2/class_value_of", 1),
    (as3_if_stricteq, "avm2/if_stricteq", 1),
    (as3_if_strictne, "avm2/if_strictne", 1),
    (as3_strict_equality, "avm2/strict_equality", 1),
    (as3_es4_interfaces, "avm2/es4_interfaces", 1),
    (as3_istype, "avm2/istype", 1),
    (as3_instanceof, "avm2/instanceof", 1),
    (as3_truthiness, "avm2/truthiness", 1),
    (as3_falsiness, "avm2/falsiness", 1),
    (as3_boolean_negation, "avm2/boolean_negation", 1),
    (as3_convert_boolean, "avm2/convert_boolean", 1),
    (as3_convert_number, "avm2/convert_number", 1),
    (as3_convert_integer, "avm2/convert_integer", 1),
    (as3_convert_uinteger, "avm2/convert_uinteger", 1),
}

// TODO: These tests have some inaccuracies currently, so we use approx_eq to test that numeric values are close enough.
// Eventually we can hopefully make some of these match exactly (see #193).
// Some will probably always need to be approx. (if they rely on trig functions, etc.)
swf_tests_approx! {
    (local_to_global, "avm1/local_to_global", 1, 0.051),
    (stage_object_properties, "avm1/stage_object_properties", 4, 0.051),
    (stage_object_properties_swf6, "avm1/stage_object_properties_swf6", 4, 0.051),
    (movieclip_getbounds, "avm1/movieclip_getbounds", 1, 0.051),
    (edittext_letter_spacing, "avm1/edittext_letter_spacing", 1, 15.0), // TODO: Discrepancy in wrapping in letterSpacing = 0.1 test.
    (edittext_align, "avm1/edittext_align", 1, 3.0),
    (edittext_margins, "avm1/edittext_margins", 1, 5.0), // TODO: Discrepancy in wrapping.
    (edittext_tab_stops, "avm1/edittext_tab_stops", 1, 5.0),
    (edittext_bullet, "avm1/edittext_bullet", 1, 3.0),
    (edittext_underline, "avm1/edittext_underline", 1, 4.0),
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

    let base_path = Path::new(swf_path).parent().unwrap();
    let (mut executor, channel) = NullExecutor::new();
    let movie = SwfMovie::from_path(swf_path)?;
    let frame_time = 1000.0 / movie.header().frame_rate as f64;
    let player = Player::new(
        Box::new(NullRenderer),
        Box::new(NullAudioBackend::new()),
        Box::new(NullNavigatorBackend::with_base_path(base_path, channel)),
        Box::new(NullInputBackend::new()),
        Box::new(MemoryStorageBackend::default()),
    )?;
    player.lock().unwrap().set_root_movie(Arc::new(movie));

    for _ in 0..num_frames {
        player.lock().unwrap().run_frame();
        player.lock().unwrap().update_timers(frame_time);
        executor.poll_all().unwrap();
    }

    executor.block_all().unwrap();

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
