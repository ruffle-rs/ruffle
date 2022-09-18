//! Tests running SWFs in a headless Ruffle instance.
//!
//! Trace output can be compared with correct output from the official Flash Player.

use approx::assert_relative_eq;
use regex::Regex;
use ruffle_core::backend::{
    log::LogBackend,
    navigator::{NullExecutor, NullNavigatorBackend},
    storage::{MemoryStorageBackend, StorageBackend},
};
use ruffle_core::context::UpdateContext;
use ruffle_core::events::MouseButton as RuffleMouseButton;
use ruffle_core::external::Value as ExternalValue;
use ruffle_core::external::{ExternalInterfaceMethod, ExternalInterfaceProvider};
use ruffle_core::limits::ExecutionLimit;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::{Player, PlayerBuilder, PlayerEvent, ViewportDimensions};
use ruffle_input_format::{AutomatedEvent, InputInjector, MouseButton as InputMouseButton};

#[cfg(feature = "imgtests")]
use ruffle_render_wgpu::backend::WgpuRenderBackend;
#[cfg(feature = "imgtests")]
use ruffle_render_wgpu::{target::TextureTarget, wgpu};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const RUN_IMG_TESTS: bool = cfg!(feature = "imgtests");

fn set_logger() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .is_test(true)
        .try_init();
}

type Error = Box<dyn std::error::Error>;

macro_rules! val_or_false {
    ($val:literal) => {
        $val
    };
    () => {
        false
    };
}

macro_rules! val_or_empty_slice {
    ($val:expr) => {
        $val
    };
    () => {
        &[]
    };
}

// This macro generates test cases for a given list of SWFs.
// If 'img' is true, then we will render an image of the final frame
// of the SWF, and compare it against a reference image on disk.
macro_rules! swf_tests {
    ($($(#[$attr:meta])* ($name:ident, $path:expr, $num_frames:literal $(, img = $img:literal)? $(, frame_time_sleep = $frame_time_sleep:literal)? ),)*) => {
        $(
        #[test]
        $(#[$attr])*
        fn $name() -> Result<(), Error> {
            set_logger();
            test_swf(
                concat!("tests/swfs/", $path, "/test.swf"),
                $num_frames,
                concat!("tests/swfs/", $path, "/input.json"),
                concat!("tests/swfs/", $path, "/output.txt"),
                val_or_false!($($img)?),
                val_or_false!($($frame_time_sleep)?),
            )
        }
        )*
    };
}

// This macro generates test cases for a given list of SWFs using `test_swf_approx`.
// If provided, `@num_patterns` must be a `&[Regex]`. Each regex in the slice is
// tested against the expected and actual - if it matches, then each capture
// group is treated as a floating-point value to be compared approximately.
// The rest of the string (outside of the capture groups) is compared exactly.
macro_rules! swf_tests_approx {
    ($($(#[$attr:meta])* ($name:ident, $path:expr, $num_frames:literal $(, @num_patterns = $num_patterns:expr)? $(, @img = $img:literal)? $(, $opt:ident = $val:expr)*),)*) => {
        $(
        #[test]
        $(#[$attr])*
        fn $name() -> Result<(), Error> {
            set_logger();
            test_swf_approx(
                concat!("tests/swfs/", $path, "/test.swf"),
                $num_frames,
                concat!("tests/swfs/", $path, "/input.json"),
                concat!("tests/swfs/", $path, "/output.txt"),
                val_or_empty_slice!($($num_patterns)?),
                val_or_false!($($img)?),
                |actual, expected| assert_relative_eq!(actual, expected $(, $opt = $val)*),
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
    (action_to_integer, "avm1/action_to_integer", 1),
    (add_swf4, "avm1/add_swf4", 1),
    (add_swf5, "avm1/add_swf5", 1),
    (add, "avm1/add", 1),
    (add2, "avm1/add2", 1),
    (add_property, "avm1/add_property", 1),
    (arguments, "avm1/arguments", 1),
    (array_call_method, "avm1/array_call_method", 1),
    (array_concat, "avm1/array_concat", 1),
    (array_constructor, "avm1/array_constructor", 1),
    (array_enumerate, "avm1/array_enumerate", 1),
    (array_length, "avm1/array_length", 1),
    (array_properties, "avm1/array_properties", 1),
    (array_prototyping, "avm1/array_prototyping", 1),
    (array_slice, "avm1/array_slice", 1),
    (array_sort, "avm1/array_sort", 1),
    (array_splice, "avm1/array_splice", 1),
    (array_trivial, "avm1/array_trivial", 1),
    (as_broadcaster_initialize, "avm1/as_broadcaster_initialize", 1),
    (as_broadcaster, "avm1/as_broadcaster", 1),
    (as_set_prop_flags, "avm1/as_set_prop_flags", 1),
    (as_set_prop_flags_version, "avm1/as_set_prop_flags_version", 1),
    (as_set_prop_flags_version_swf5, "avm1/as_set_prop_flags_version_swf5", 1),
    (as_set_prop_flags_version_swf6, "avm1/as_set_prop_flags_version_swf6", 1),
    (as_set_prop_flags_version_swf7, "avm1/as_set_prop_flags_version_swf7", 1),
    (as_set_prop_flags_version_swf8, "avm1/as_set_prop_flags_version_swf8", 1),
    (as_set_prop_flags_version_swf9, "avm1/as_set_prop_flags_version_swf9", 1),
    (as_transformed_flag, "avm1/as_transformed_flag", 3),
    (as1_constructor_v6, "avm1/as1_constructor_v6", 1),
    (as1_constructor_v7, "avm1/as1_constructor_v7", 1),
    (as2_bitand, "avm1/bitand", 1),
    (as2_bitor, "avm1/bitor", 1),
    (as2_bitxor, "avm1/bitxor", 1),
    (as2_oop, "avm1/as2_oop", 1),
    (as2_super_and_this_v6, "avm1/as2_super_and_this_v6", 1),
    (as2_super_and_this_v8, "avm1/as2_super_and_this_v8", 1),
    (as2_super_via_manual_prototype, "avm1/as2_super_via_manual_prototype", 1),
    (as3_add, "avm2/add", 1),
    (as3_agal_compiler, "avm2/agal_compiler", 1),
    (as3_application_domain, "avm2/application_domain", 1),
    (as3_array_access, "avm2/array_access", 1),
    (as3_array_concat, "avm2/array_concat", 1),
    (as3_array_constr, "avm2/array_constr", 1),
    (as3_array_delete, "avm2/array_delete", 1),
    (as3_array_enumeration_elements, "avm2/array_enumeration_elements", 1),
    (as3_array_enumeration, "avm2/array_enumeration", 1),
    (as3_array_every, "avm2/array_every", 1),
    (as3_array_filter, "avm2/array_filter", 1),
    (as3_array_foreach, "avm2/array_foreach", 1),
    (as3_array_hasownproperty, "avm2/array_hasownproperty", 1),
    (as3_array_holes, "avm2/array_holes", 1),
    (as3_array_indexof, "avm2/array_indexof", 1),
    (as3_array_join, "avm2/array_join", 1),
    (as3_array_lastindexof, "avm2/array_lastindexof", 1),
    (as3_array_length, "avm2/array_length", 1),
    (as3_array_literal, "avm2/array_literal", 1),
    (as3_array_map, "avm2/array_map", 1),
    (as3_array_pop, "avm2/array_pop", 1),
    (as3_array_push, "avm2/array_push", 1),
    (as3_array_reverse, "avm2/array_reverse", 1),
    (as3_array_shift, "avm2/array_shift", 1),
    (as3_array_slice, "avm2/array_slice", 1),
    (as3_array_some, "avm2/array_some", 1),
    (as3_array_sort, "avm2/array_sort", 1),
    (as3_array_sorton, "avm2/array_sorton", 1),
    (as3_array_splice, "avm2/array_splice", 1),
    (as3_array_storage, "avm2/array_storage", 1),
    (as3_array_tolocalestring, "avm2/array_tolocalestring", 1),
    (as3_array_tostring, "avm2/array_tostring", 1),
    (as3_array_unshift, "avm2/array_unshift", 1),
    (as3_array_valueof, "avm2/array_valueof", 1),
    (as3_astype, "avm2/astype", 1),
    (as3_astypelate, "avm2/astypelate", 1),
    (as3_bitand, "avm2/bitand", 1),
    (as3_bitmap_constr, "avm2/bitmap_constr", 1),
    (as3_bitmap_data, "avm2/bitmap_data", 1),
    (as3_bitmapdata_copypixels, "avm2/bitmapdata_copypixels", 2, img = true),
    #[ignore] (as3_bitmap_properties, "avm2/bitmap_properties", 1),
    (as3_bitmap_subclass, "avm2/bitmap_subclass", 1),
    #[cfg_attr(not(feature = "imgtests"), ignore)] (as3_bitmap_subclass_properties, "avm2/bitmap_subclass_properties", 1, img = true),
    (as3_bitmap_timeline, "avm2/bitmap_timeline", 1),
    #[cfg_attr(not(feature = "imgtests"), ignore)] (as3_bitmapdata_clone, "avm2/bitmapdata_clone", 1, img = true),
    (as3_bitmapdata_constr, "avm2/bitmapdata_constr", 1),
    (as3_bitmapdata_dispose, "avm2/bitmapdata_dispose", 1),
    // We need a render backend in order to call `BitmapData.draw`
    #[cfg_attr(not(feature = "imgtests"), ignore)] (as3_bitmapdata_draw, "avm2/bitmapdata_draw", 1, img = true),
    #[cfg_attr(not(feature = "imgtests"), ignore)] (as3_bitmapdata_opaque, "avm2/bitmapdata_opaque", 1, img = true),
    (as3_bitmapdata_zero_size, "avm2/bitmapdata_zero_size", 1),
    #[cfg_attr(not(feature = "imgtests"), ignore)] (as3_bitmapdata_embedded, "avm2/bitmapdata_embedded", 1, img = true),
    (as3_bitmapdata_fillrect, "avm2/bitmapdata_fillrect", 1),
    (as3_bitnot, "avm2/bitnot", 1),
    (as3_bitor, "avm2/bitor", 1),
    (as3_bitxor, "avm2/bitxor", 1),
    (as3_boolean_constr, "avm2/boolean_constr", 1),
    (as3_boolean_negation, "avm2/boolean_negation", 1),
    (as3_boolean_tostring, "avm2/boolean_tostring", 1),
    (as3_bytearray_readobject_amf0, "avm2/bytearray_readobject_amf0", 1),
    (as3_bytearray_readobject_amf3, "avm2/bytearray_readobject_amf3", 1),
    (as3_bytearray_writeobject, "avm2/bytearray_writeobject", 1),
    (as3_bytearray, "avm2/bytearray", 1),
    (as3_checkfilter, "avm2/checkfilter", 1),
    (as3_class_call, "avm2/class_call", 1),
    (as3_class_cast_call, "avm2/class_cast_call", 1),
    (as3_class_enumeration, "avm2/class_enumeration", 1),
    (as3_class_is, "avm2/class_is", 1),
    (as3_class_methods, "avm2/class_methods", 1),
    (as3_class_object_properties, "avm2/class_object_properties", 1),
    (as3_class_singleton, "avm2/class_singleton", 1),
    (as3_class_supercalls_mismatched, "avm2/class_supercalls_mismatched", 1),
    (as3_class_to_locale_string, "avm2/class_to_locale_string", 1),
    (as3_class_to_string, "avm2/class_to_string", 1),
    (as3_class_value_of, "avm2/class_value_of", 1),
    (as3_closures, "avm2/closures", 1),
    (as3_coerce_property, "avm2/coerce_property", 1),
    (as3_coerce_string, "avm2/coerce_string", 1),
    (as3_constructor_call, "avm2/constructor_call", 1),
    (as3_control_flow_bool, "avm2/control_flow_bool", 1),
    (as3_control_flow_stricteq, "avm2/control_flow_stricteq", 1),
    (as3_convert_boolean, "avm2/convert_boolean", 1),
    (as3_convert_integer, "avm2/convert_integer", 1),
    (as3_convert_number, "avm2/convert_number", 1),
    (as3_convert_uinteger, "avm2/convert_uinteger", 1),
    (as3_date_parse, "avm2/date_parse", 1),
    (as3_date, "avm2/date", 1),
    (as3_declocal_i, "avm2/declocal_i", 1),
    (as3_declocal, "avm2/declocal", 1),
    (as3_decrement_i, "avm2/decrement_i", 1),
    (as3_decrement, "avm2/decrement", 1),
    (as3_default_values, "avm2/default_values", 1),
    (as3_dictionary_access, "avm2/dictionary_access", 1),
    (as3_dictionary_delete, "avm2/dictionary_delete", 1),
    (as3_dictionary_foreach, "avm2/dictionary_foreach", 1),
    (as3_dictionary_hasownproperty, "avm2/dictionary_hasownproperty", 1),
    (as3_dictionary_in, "avm2/dictionary_in", 1),
    (as3_dictionary_namespaces, "avm2/dictionary_namespaces", 1),
    (as3_displayobject_alpha, "avm2/displayobject_alpha", 1),
    (as3_displayobject_blendmode, "avm2/displayobject_blendmode", 1, img = true),
    (as3_displayobject_hittestobject, "avm2/displayobject_hittestobject", 1),
    (as3_displayobject_hittestpoint, "avm2/displayobject_hittestpoint", 2),
    (as3_displayobject_mask, "avm2/displayobject_mask", 1, img = true),
    (as3_displayobject_name, "avm2/displayobject_name", 4),
    (as3_displayobject_parent, "avm2/displayobject_parent", 4),
    (as3_displayobject_root, "avm2/displayobject_root", 4),
    (as3_displayobject_visible, "avm2/displayobject_visible", 4),
    (as3_displayobject_x, "avm2/displayobject_x", 1),
    (as3_displayobject_y, "avm2/displayobject_y", 1),
    (as3_displayobjectcontainer_addchild_timelinepull0, "avm2/displayobjectcontainer_addchild_timelinepull0", 7),
    (as3_displayobjectcontainer_addchild_timelinepull1, "avm2/displayobjectcontainer_addchild_timelinepull1", 7),
    (as3_displayobjectcontainer_addchild_timelinepull2, "avm2/displayobjectcontainer_addchild_timelinepull2", 7),
    (as3_displayobjectcontainer_addchild, "avm2/displayobjectcontainer_addchild", 1),
    (as3_displayobjectcontainer_addchildat_timelinelock0, "avm2/displayobjectcontainer_addchildat_timelinelock0", 7),
    (as3_displayobjectcontainer_addchildat_timelinelock1, "avm2/displayobjectcontainer_addchildat_timelinelock1", 7),
    (as3_displayobjectcontainer_addchildat_timelinelock2, "avm2/displayobjectcontainer_addchildat_timelinelock2", 7),
    (as3_displayobjectcontainer_addchildat, "avm2/displayobjectcontainer_addchildat", 1),
    (as3_displayobjectcontainer_contains, "avm2/displayobjectcontainer_contains", 5),
    (as3_displayobjectcontainer_getchildat, "avm2/displayobjectcontainer_getchildat", 1),
    (as3_displayobjectcontainer_getchildbyname, "avm2/displayobjectcontainer_getchildbyname", 1),
    (as3_displayobjectcontainer_getchildindex, "avm2/displayobjectcontainer_getchildindex", 5),
    (as3_displayobjectcontainer_removechild_timelinemanip_remove1, "avm2/displayobjectcontainer_removechild_timelinemanip_remove1", 7),
    (as3_displayobjectcontainer_removechild, "avm2/displayobjectcontainer_removechild", 1),
    (as3_displayobjectcontainer_removechildat, "avm2/displayobjectcontainer_removechildat", 1),
    (as3_displayobjectcontainer_removechildren, "avm2/displayobjectcontainer_removechildren", 5),
    (as3_displayobjectcontainer_setchildindex, "avm2/displayobjectcontainer_setchildindex", 1),
    (as3_displayobjectcontainer_stopallmovieclips, "avm2/displayobjectcontainer_stopallmovieclips", 2),
    (as3_displayobjectcontainer_swapchildren, "avm2/displayobjectcontainer_swapchildren", 1),
    (as3_displayobjectcontainer_swapchildrenat, "avm2/displayobjectcontainer_swapchildrenat", 1),
    (as3_displayobjectcontainer_timelineinstance, "avm2/displayobjectcontainer_timelineinstance", 6),
    (as3_documentclass, "avm2/documentclass", 1),
    (as3_domain_memory, "avm2/domain_memory", 1),
    (as3_drag_drop, "avm2/drag_drop", 14),
    (as3_edittext_antialiastype, "avm2/edittext_antialiastype", 1),
    (as3_edittext_default_format, "avm2/edittext_default_format", 1),
    (as3_edittext_html_entity, "avm2/edittext_html_entity", 1),
    (as3_edittext_html_roundtrip, "avm2/edittext_html_roundtrip", 1),
    (as3_edittext_mouseenabled, "avm2/edittext_mouseenabled", 1),
    (as3_edittext_newline_stripping, "avm2/edittext_newline_stripping", 1),
    (as3_edittext_width_height, "avm2/edittext_width_height", 1),
    (as3_equals, "avm2/equals", 1),
    (as3_error_stack_trace, "avm2/error_stack_trace", 1),
    (as3_error_tostring, "avm2/error_tostring", 1),
    (as3_error_tostring_more, "avm2/error_tostring_more", 1),
    (as3_es3_inheritance, "avm2/es3_inheritance", 1),
    (as3_es4_inheritance, "avm2/es4_inheritance", 1),
    (as3_es4_interfaces, "avm2/es4_interfaces", 1),
    (as3_es4_method_binding, "avm2/es4_method_binding", 1),
    (as3_es4_oop_prototypes, "avm2/es4_oop_prototypes", 1),
    (as3_es4_protected_inheritance, "avm2/es4_protected_inheritance", 1),
    (as3_event_bubbles, "avm2/event_bubbles", 1),
    (as3_event_cancelable, "avm2/event_cancelable", 1),
    (as3_event_clone, "avm2/event_clone", 1),
    (as3_event_formattostring, "avm2/event_formattostring", 1),
    (as3_event_handler_exception, "avm2/event_handler_exception", 2),
    (as3_event_isdefaultprevented, "avm2/event_isdefaultprevented", 1),
    (as3_event_type, "avm2/event_type", 1),
    (as3_event_valueof_tostring, "avm2/event_valueof_tostring", 1),
    (as3_eventdispatcher_dispatchevent_cancel, "avm2/eventdispatcher_dispatchevent_cancel", 1),
    (as3_eventdispatcher_dispatchevent_handlerorder, "avm2/eventdispatcher_dispatchevent_handlerorder", 1),
    (as3_eventdispatcher_dispatchevent_this, "avm2/eventdispatcher_dispatchevent_this", 1),
    (as3_eventdispatcher_dispatchevent, "avm2/eventdispatcher_dispatchevent", 1),
    (as3_eventdispatcher_haseventlistener, "avm2/eventdispatcher_haseventlistener", 1),
    (as3_eventdispatcher_tostring, "avm2/eventdispatcher_tostring", 1),
    (as3_eventdispatcher_willtrigger, "avm2/eventdispatcher_willtrigger", 1),
    (as3_falsiness, "avm2/falsiness", 1),
    (as3_font_embedded, "avm2/font_embedded", 1),
    (as3_font_hasglyphs, "avm2/font_hasglyphs", 1),
    (as3_framelabel_constr, "avm2/framelabel_constr", 5),
    (as3_function_call_arguments, "avm2/function_call_arguments", 1),
    (as3_function_call_coercion, "avm2/function_call_coercion", 1),
    (as3_function_call_default, "avm2/function_call_default", 1),
    (as3_function_call_rest, "avm2/function_call_rest", 1),
    (as3_function_call_types, "avm2/function_call_types", 1),
    (as3_function_call_via_apply, "avm2/function_call_via_apply", 1),
    (as3_function_call_via_call, "avm2/function_call_via_call", 1),
    (as3_function_call, "avm2/function_call", 1),
    #[ignore] (as3_function_proto, "avm2/function_proto", 1),
    (as3_function_length, "avm2/function_length", 1),
    (as3_function_object, "avm2/function_object", 1),
    (as3_function_to_locale_string, "avm2/function_to_locale_string", 1),
    (as3_function_to_string, "avm2/function_to_string", 1),
    (as3_function_type, "avm2/function_type", 1),
    (as3_function_value_of, "avm2/function_value_of", 1),
    (as3_generate_random_bytes, "avm2/generate_random_bytes", 1),
    (as3_get_definition_by_name, "avm2/get_definition_by_name", 1),
    (as3_get_qualified_class_name, "avm2/get_qualified_class_name", 1),
    (as3_get_qualified_super_class_name, "avm2/get_qualified_super_class_name", 1),
    (as3_get_timer, "avm2/get_timer", 1),
    (as3_getouterscope, "avm2/getouterscope", 1),
    (as3_greaterequals, "avm2/greaterequals", 1),
    (as3_greaterthan, "avm2/greaterthan", 1),
    (as3_has_own_property, "avm2/has_own_property", 1),
    (as3_hasownproperty_namespaces, "avm2/hasownproperty_namespaces", 1),
    (as3_hello_world, "avm2/hello_world", 1),
    (as3_if_eq, "avm2/if_eq", 1),
    (as3_if_gt, "avm2/if_gt", 1),
    (as3_if_gte, "avm2/if_gte", 1),
    (as3_if_lt, "avm2/if_lt", 1),
    (as3_if_lte, "avm2/if_lte", 1),
    (as3_if_ne, "avm2/if_ne", 1),
    (as3_if_stricteq, "avm2/if_stricteq", 1),
    (as3_if_strictne, "avm2/if_strictne", 1),
    (as3_in, "avm2/in", 1),
    (as3_inclocal_i, "avm2/inclocal_i", 1),
    (as3_inclocal, "avm2/inclocal", 1),
    (as3_increment_i, "avm2/increment_i", 1),
    (as3_increment, "avm2/increment", 1),
    (as3_instanceof, "avm2/instanceof", 1),
    (as3_int_constr, "avm2/int_constr", 1),
    (as3_int_edge_cases, "avm2/int_edge_cases", 1),
    #[ignore] (as3_int_toexponential, "avm2/int_toexponential", 1), //Ignored because Flash Player has a print routine that adds extraneous zeros to things
    (as3_int_tofixed, "avm2/int_tofixed", 1),
    #[ignore] (as3_int_toprecision, "avm2/int_toprecision", 1), //Ignored because Flash Player has a print routine that adds extraneous zeros to things
    (as3_int_tostring, "avm2/int_tostring", 1),
    (as3_interactiveobject_enabled, "avm2/interactiveobject_enabled", 1),
    (as3_interface_namespaces, "avm2/interface_namespaces", 1),
    (as3_invalid_utf8, "avm2/invalid_utf8", 1),
    (as3_is_finite, "avm2/is_finite", 1),
    (as3_is_nan, "avm2/is_nan", 1),
    (as3_is_prototype_of, "avm2/is_prototype_of", 1),
    (as3_issue_5292, "avm2/issue_5292", 1),
    (as3_istype, "avm2/istype", 1),
    (as3_istypelate_coerce, "avm2/istypelate_coerce", 1),
    (as3_istypelate, "avm2/istypelate", 1),
    (as3_json_parse, "avm2/json_parse", 1),
    (as3_json_stringify, "avm2/json_stringify", 1),
    (as3_lazyinit, "avm2/lazyinit", 1),
    (as3_lessequals, "avm2/lessequals", 1),
    (as3_lessthan, "avm2/lessthan", 1),
    (as3_loader_events, "avm2/loader_events", 3, img = true),
    (as3_loader_loadbytes_events, "avm2/loader_loadbytes_events", 3, img = true),
    (as3_loaderinfo_events, "avm2/loaderinfo_events", 2),
    (as3_loaderinfo_properties, "avm2/loaderinfo_properties", 2),
    (as3_loaderinfo_root, "avm2/loaderinfo_root", 1),
    (as3_loaderinfo_quine, "avm2/loaderinfo_quine", 2),
    (as3_lshift, "avm2/lshift", 1),
    (as3_modulo, "avm2/modulo", 1),
    (as3_mouseevent_constr, "avm2/mouseevent_constr", 1),
    (as3_mouseevent_stagexy, "avm2/mouseevent_stagexy", 1),
    (as3_mouseevent_valueof_tostring, "avm2/mouseevent_valueof_tostring", 1),
    (as3_movieclip_child_property, "avm2/movieclip_child_property", 1),
    (as3_movieclip_constr, "avm2/movieclip_constr", 1),
    (as3_movieclip_currentlabels, "avm2/movieclip_currentlabels", 5),
    (as3_movieclip_currentscene, "avm2/movieclip_currentscene", 5),
    (as3_movieclip_dispatchevent_cancel, "avm2/movieclip_dispatchevent_cancel", 1),
    (as3_movieclip_dispatchevent_handlerorder, "avm2/movieclip_dispatchevent_handlerorder", 1),
    (as3_movieclip_dispatchevent_selfadd, "avm2/movieclip_dispatchevent_selfadd", 1),
    (as3_movieclip_dispatchevent_target, "avm2/movieclip_dispatchevent_target", 1),
    (as3_movieclip_dispatchevent, "avm2/movieclip_dispatchevent", 1),
    (as3_movieclip_displayevents_clickgoto, "avm2/movieclip_displayevents_clickgoto", 32),
    (as3_movieclip_displayevents_clickgoto2, "avm2/movieclip_displayevents_clickgoto2", 46),
    (as3_movieclip_displayevents_clickplay, "avm2/movieclip_displayevents_clickplay", 32),
    (as3_movieclip_displayevents_clicksymbol, "avm2/movieclip_displayevents_clicksymbol", 32),
    (as3_movieclip_displayevents_constructframegoto, "avm2/movieclip_displayevents_constructframegoto", 12),
    (as3_movieclip_displayevents_constructframeplay, "avm2/movieclip_displayevents_constructframeplay", 6),
    (as3_movieclip_displayevents_constructframesymbol, "avm2/movieclip_displayevents_constructframesymbol", 12),
    (as3_movieclip_displayevents_dblhandler, "avm2/movieclip_displayevents_dblhandler", 4),
    (as3_movieclip_displayevents_enterframegoto, "avm2/movieclip_displayevents_enterframegoto", 15),
    (as3_movieclip_displayevents_enterframeplay, "avm2/movieclip_displayevents_enterframeplay", 6),
    (as3_movieclip_displayevents_enterframesymbol, "avm2/movieclip_displayevents_enterframesymbol", 15),
    (as3_movieclip_displayevents_exitframegoto, "avm2/movieclip_displayevents_exitframegoto", 12),
    (as3_movieclip_displayevents_exitframeplay, "avm2/movieclip_displayevents_exitframeplay", 6),
    (as3_movieclip_displayevents_exitframesymbol, "avm2/movieclip_displayevents_exitframesymbol", 12),
    (as3_movieclip_displayevents_looping, "avm2/movieclip_displayevents_looping", 5),
    (as3_movieclip_displayevents_timeline, "avm2/movieclip_displayevents_timeline", 7),
    (as3_movieclip_displayevents_stopped, "avm2/movieclip_displayevents_stopped", 10),
    (as3_movieclip_displayevents, "avm2/movieclip_displayevents", 9),
    (as3_movieclip_drawrect, "avm2/movieclip_drawrect", 1),
    (as3_movieclip_goto_during_frame_script, "avm2/movieclip_goto_during_frame_script", 1),
    (as3_movieclip_gotoandplay, "avm2/movieclip_gotoandplay", 5),
    (as3_movieclip_gotoandstop, "avm2/movieclip_gotoandstop", 5),
    (as3_movieclip_gotoandstop_children, "avm2/movieclip_gotoandstop_children", 1),
    (as3_movieclip_gotoandstop_framescripts1, "avm2/movieclip_gotoandstop_framescripts1", 1),
    (as3_movieclip_gotoandstop_framescripts2, "avm2/movieclip_gotoandstop_framescripts2", 1),
    (as3_movieclip_gotoandstop_framescripts_self, "avm2/movieclip_gotoandstop_framescripts_self", 1),
    (as3_movieclip_gotoandstop_queueing, "avm2/movieclip_gotoandstop_queueing", 2),
    (as3_movieclip_next_frame, "avm2/movieclip_next_frame", 5),
    (as3_movieclip_next_scene, "avm2/movieclip_next_scene", 5),
    (as3_movieclip_play, "avm2/movieclip_play", 5),
    (as3_movieclip_prev_frame, "avm2/movieclip_prev_frame", 5),
    (as3_movieclip_prev_scene, "avm2/movieclip_prev_scene", 5),
    (as3_movieclip_properties, "avm2/movieclip_properties", 4),
    (as3_movieclip_scenes, "avm2/movieclip_scenes", 5),
    (as3_movieclip_soundtransform, "avm2/movieclip_soundtransform", 49),
    (as3_movieclip_stop, "avm2/movieclip_stop", 5),
    (as3_movieclip_symbol_constr, "avm2/movieclip_symbol_constr", 1),
    (as3_movieclip_willtrigger, "avm2/movieclip_willtrigger", 3),
    (as3_multiply, "avm2/multiply", 1),
    (as3_nan_scale, "avm2/nan_scale", 1),
    (as3_negate, "avm2/negate", 1),
    (as3_nonconflicting_declarations, "avm2/nonconflicting_declarations", 1),
    (as3_number_constr, "avm2/number_constr", 1),
    #[ignore] (as3_number_tostring, "avm2/number_tostring", 1), //Ignored because Flash Player adds extra x, W, and/or ° symbols randomly
    (as3_object_enumeration, "avm2/object_enumeration", 1),
    (as3_object_prototype, "avm2/object_prototype", 1),
    (as3_object_to_locale_string, "avm2/object_to_locale_string", 1),
    (as3_object_to_string, "avm2/object_to_string", 1),
    (as3_object_value_of, "avm2/object_value_of", 1),
    (as3_op_coerce_x, "avm2/op_coerce_x", 1),
    (as3_op_coerce, "avm2/op_coerce", 1),
    (as3_op_escxattr, "avm2/op_escxattr", 1),
    (as3_op_escxelem, "avm2/op_escxelem", 1),
    (as3_op_lookupswitch, "avm2/op_lookupswitch", 1),
    (as3_parse_int, "avm2/parse_int", 1),
    (as3_place_object_replace_2, "avm2/place_object_replace_2", 3),
    (as3_place_object_replace, "avm2/place_object_replace", 2),
    (as3_point, "avm2/point", 1),
    (as3_property_is_enumerable, "avm2/property_is_enumerable", 1),
    (as3_propertyisenumerable_namespaces, "avm2/propertyisenumerable_namespaces", 1),
    (as3_proxy_callproperty, "avm2/proxy_callproperty", 1),
    (as3_proxy_deleteproperty, "avm2/proxy_deleteproperty", 1),
    (as3_proxy_enumeration, "avm2/proxy_enumeration", 1),
    (as3_proxy_getproperty, "avm2/proxy_getproperty", 1),
    (as3_proxy_hasproperty, "avm2/proxy_hasproperty", 1),
    (as3_proxy_setproperty, "avm2/proxy_setproperty", 1),
    (as3_qname_constr_namespace, "avm2/qname_constr_namespace", 1),
    (as3_qname_constr, "avm2/qname_constr", 1),
    (as3_qname_tostring, "avm2/qname_tostring", 1),
    (as3_qname_valueof, "avm2/qname_valueof", 1),
    (as3_rectangle, "avm2/rectangle", 1),
    (as3_vector3d, "avm2/vector3d", 1),
    (as3_regexp_constr, "avm2/regexp_constr", 1),
    (as3_regexp_exec, "avm2/regexp_exec", 1),
    (as3_regexp_test, "avm2/regexp_test", 1),
    (as3_rshift, "avm2/rshift", 1),
    (as3_scene_constr, "avm2/scene_constr", 5),
    (as3_set_property_is_enumerable, "avm2/set_property_is_enumerable", 1),
    (as3_shape_drawrect, "avm2/shape_drawrect", 1),
    (as3_simplebutton_childevents_nested, "avm2/simplebutton_childevents_nested", 2),
    (as3_simplebutton_childevents, "avm2/simplebutton_childevents", 2),
    (as3_simplebutton_childprops, "avm2/simplebutton_childprops", 1),
    (as3_simplebutton_childshuffle, "avm2/simplebutton_childshuffle", 1),
    #[ignore] (as3_simplebutton_constr_childevents, "avm2/simplebutton_constr_childevents", 2), //Broken by other accuracy fixes
    (as3_simplebutton_constr_params, "avm2/simplebutton_constr_params", 1),
    (as3_simplebutton_constr, "avm2/simplebutton_constr", 2),
    (as3_simplebutton_mouseenabled, "avm2/simplebutton_mouseenabled", 1),
    (as3_simplebutton_soundtransform, "avm2/simplebutton_soundtransform", 49),
    (as3_simplebutton_structure, "avm2/simplebutton_structure", 2),
    (as3_simplebutton_symbolclass, "avm2/simplebutton_symbolclass", 3),
    (as3_sound_embeddedprops, "avm2/sound_embeddedprops", 1),
    (as3_sound_play, "avm2/sound_play", 1),
    (as3_sound_valueof, "avm2/sound_valueof", 1),
    #[ignore] (as3_soundchannel_position, "avm2/soundchannel_position", 75),
    #[ignore] (as3_soundchannel_soundcomplete, "avm2/soundchannel_soundcomplete", 25),
    (as3_soundchannel_soundtransform, "avm2/soundchannel_soundtransform", 49),
    (as3_soundchannel_stop, "avm2/soundchannel_stop", 4),
    (as3_soundmixer_buffertime, "avm2/soundmixer_buffertime", 1),
    (as3_soundmixer_soundtransform, "avm2/soundmixer_soundtransform", 49),
    (as3_soundmixer_stopall, "avm2/soundmixer_stopall", 4),
    (as3_soundtransform, "avm2/soundtransform", 1),
    (as3_stage_access, "avm2/stage_access", 1),
    (as3_stage_display_state, "avm2/stage_display_state", 1),
    (as3_stage_displayobject_properties, "avm2/stage_displayobject_properties", 1),
    (as3_stage_loaderinfo_properties, "avm2/stage_loaderinfo_properties", 2),
    (as3_stage_mouseenabled, "avm2/stage_mouseenabled", 1),
    (as3_stage_properties, "avm2/stage_properties", 1),
    (as3_stage3d_rotating_cube, "avm2/stage3d_rotating_cube", 40, img = true),
    (as3_stage3d_triangle, "avm2/stage3d_triangle", 1, img = true),
    (as3_static_text, "avm2/static_text", 1),
    (as3_stored_properties, "avm2/stored_properties", 1),
    (as3_strict_equality, "avm2/strict_equality", 1),
    (as3_string_case, "avm2/string_case", 1),
    (as3_string_char_at, "avm2/string_char_at", 1),
    (as3_string_char_code_at, "avm2/string_char_code_at", 1),
    (as3_string_concat_fromcharcode, "avm2/string_concat_fromcharcode", 1),
    (as3_string_constr, "avm2/string_constr", 1),
    (as3_string_indexof_lastindexof, "avm2/string_indexof_lastindexof", 1),
    (as3_string_length, "avm2/string_length", 1),
    (as3_string_locale_compare, "avm2/string_locale_compare", 1),
    (as3_string_match, "avm2/string_match", 1),
    (as3_string_replace, "avm2/string_replace", 1),
    (as3_string_search, "avm2/string_search", 1),
    (as3_swz, "avm2/swz", 10),
    (as3_try_catch, "avm2/try_catch", 1),
    (as3_try_catch_typed, "avm2/try_catch_typed", 1),
    (as3_string_slice_substr_substring, "avm2/string_slice_substr_substring", 1),
    (as3_string_split, "avm2/string_split", 1),
    (as3_subtract, "avm2/subtract", 1),
    (as3_symbol_class_binary_data, "avm2/symbol_class_binary_data", 1),
    (as3_textformat, "avm2/textformat", 1),
    (as3_throw, "avm2/throw", 1),
    (as3_timeline_scripts, "avm2/timeline_scripts", 3),
    (as3_trace, "avm2/trace", 1),
    (as3_truthiness, "avm2/truthiness", 1),
    (as3_typeof, "avm2/typeof", 1),
    (as3_uint_constr, "avm2/uint_constr", 1),
    #[ignore] (as3_uint_toexponential, "avm2/uint_toexponential", 1), //Ignored because Flash Player has a print routine that adds extraneous zeros to things
    (as3_uint_tofixed, "avm2/uint_tofixed", 1),
    #[ignore] (as3_uint_toprecision, "avm2/uint_toprecision", 1), //Ignored because Flash Player has a print routine that adds extraneous zeros to things
    (as3_uint_tostring, "avm2/uint_tostring", 1),
    (as3_unchecked_function, "avm2/unchecked_function", 1),
    (as3_url_loader, "avm2/url_loader", 1),
    (as3_url_vars, "avm2/url_vars", 1),
    (as3_urshift, "avm2/urshift", 1),
    (as3_vector_coercion, "avm2/vector_coercion", 1),
    (as3_vector_concat, "avm2/vector_concat", 1),
    (as3_vector_constr, "avm2/vector_constr", 1),
    (as3_vector_enumeration, "avm2/vector_enumeration", 1),
    (as3_vector_every, "avm2/vector_every", 1),
    (as3_vector_filter, "avm2/vector_filter", 1),
    (as3_vector_holes, "avm2/vector_holes", 1),
    (as3_vector_indexof, "avm2/vector_indexof", 1),
    (as3_vector_insertat, "avm2/vector_insertat", 1),
    (as3_vector_int_access, "avm2/vector_int_access", 1),
    (as3_vector_int_delete, "avm2/vector_int_delete", 1),
    (as3_vector_join, "avm2/vector_join", 1),
    (as3_vector_lastindexof, "avm2/vector_lastindexof", 1),
    (as3_vector_legacy, "avm2/vector_legacy", 1),
    (as3_vector_map, "avm2/vector_map", 1),
    (as3_vector_pushpop, "avm2/vector_pushpop", 1),
    (as3_vector_removeat, "avm2/vector_removeat", 1),
    (as3_vector_reverse, "avm2/vector_reverse", 1),
    (as3_vector_shiftunshift, "avm2/vector_shiftunshift", 1),
    (as3_vector_slice, "avm2/vector_slice", 1),
    (as3_vector_sort, "avm2/vector_sort", 1),
    (as3_vector_splice, "avm2/vector_splice", 1),
    (as3_vector_tostring, "avm2/vector_tostring", 1),
    (as3_virtual_properties, "avm2/virtual_properties", 1),
    (as3_with, "avm2/with", 1),
    (as3_escape, "avm2/escape", 1),
    (as3_escape_multi_byte, "avm2/escape_multi_byte", 1),
    (attach_movie, "avm1/attach_movie", 1),
    (bad_placeobject_clipaction, "avm1/bad_placeobject_clipaction", 2),
    (bad_swf_tag_past_eof, "avm1/bad_swf_tag_past_eof", 1),
    (bevel_filter, "avm1/bevel_filter", 1),
    (bitmap_data, "avm1/bitmap_data", 1),
    (bitmap_data_compare, "avm1/bitmap_data_compare", 1),
    (bitmap_data_copypixels, "avm1/bitmap_data_copypixels", 2, img = true),
    (bitmap_data_max_size_swf10, "avm1/bitmap_data_max_size_swf10", 1),
    (bitmap_data_max_size_swf9, "avm1/bitmap_data_max_size_swf9", 1),
    (bitmap_data_noise, "avm1/bitmap_data_noise", 1),
    (bitmap_filter, "avm1/bitmap_filter", 1),
    (biturshift, "avm1/biturshift", 1),
    (biturshift_swf8, "avm1/biturshift_swf8", 1),
    (blur_filter, "avm1/blur_filter", 1),
    (button_children, "avm1/button_children", 1),
    (button_order, "avm1/button_order", 2),
    (call_method_empty_name, "avm1/call_method_empty_name", 1),
    (call, "avm1/call", 2),
    (clip_events, "avm1/clip_events", 4),
    (closure_scope, "avm1/closure_scope", 1),
    (color_matrix_filter, "avm1/color_matrix_filter", 1),
    (color_transform, "avm1/color_transform", 1),
    (color, "avm1/color", 1, img = true),
    (conflicting_instance_names, "avm1/conflicting_instance_names", 6),
    (constructor_function, "avm1/constructor_function", 1),
    (context_menu_item, "avm1/context_menu_item", 1),
    (context_menu, "avm1/context_menu", 1),
    (convolution_filter, "avm1/convolution_filter", 1),
    (create_empty_movie_clip, "avm1/create_empty_movie_clip", 2),
    (cross_movie_root, "avm1/cross_movie_root", 5),
    (custom_clip_methods, "avm1/custom_clip_methods", 3),
    (date, "avm1/date", 1),
    (default_names, "avm1/default_names", 6),
    (define_function_case_sensitive, "avm1/define_function_case_sensitive", 2),
    (define_function2_preload_order, "avm1/define_function2_preload_order", 1),
    (define_function2_preload, "avm1/define_function2_preload", 1),
    (define_local, "avm1/define_local", 1),
    (define_local_with_paths, "avm1/define_local_with_paths", 1),
    (delete, "avm1/delete", 3),
    (displacement_map_filter, "avm1/displacement_map_filter", 1),
    (divide_swf4, "avm1/divide_swf4", 1),
    (do_init_action, "avm1/do_init_action", 3),
    (drag_drop, "avm1/drag_drop", 14),
    (drop_shadow_filter, "avm1/drop_shadow_filter", 1),
    (duplicate_movie_clip_drawing, "avm1/duplicate_movie_clip_drawing", 1),
    (duplicate_movie_clip, "avm1/duplicate_movie_clip", 1),
    (edittext_antialiastype, "avm1/edittext_antialiastype", 1),
    (edittext_default_format, "avm1/edittext_default_format", 1),
    (edittext_font_size, "avm1/edittext_font_size", 1),
    (edittext_html_entity, "avm1/edittext_html_entity", 1),
    (edittext_html_roundtrip, "avm1/edittext_html_roundtrip", 1),
    (edittext_leading, "avm1/edittext_leading", 1),
    (edittext_newline_stripping, "avm1/edittext_newline_stripping", 1),
    #[ignore] (edittext_newlines, "avm1/edittext_newlines", 1),
    (edittext_password, "avm1/edittext_password", 1),
    (edittext_scroll, "avm1/edittext_scroll", 1),
    (edittext_width_height, "avm1/edittext_width_height", 1),
    (empty_movieclip_can_attach_movies, "avm1/empty_movieclip_can_attach_movies", 1),
    (enumerate, "avm1/enumerate", 1),
    (equals_swf4, "avm1/equals_swf4", 1),
    (equals_swf4_alt, "avm1/equals_swf4_alt", 1),
    (equals_swf5, "avm1/equals_swf5", 1),
    (equals, "avm1/equals", 1),
    (equals2_swf5, "avm1/equals2_swf5", 1),
    (equals2_swf6, "avm1/equals2_swf6", 1),
    (equals2_swf7, "avm1/equals2_swf7", 1),
    (error, "avm1/error", 1),
    (escape, "avm1/escape", 1),
    (execution_order1, "avm1/execution_order1", 3),
    (execution_order2, "avm1/execution_order2", 15),
    (execution_order3, "avm1/execution_order3", 5),
    (execution_order4, "avm1/execution_order4", 4),
    (export_assets, "avm1/export_assets", 1),
    (extends_chain, "avm1/extends_chain", 1),
    (extends_native_type, "avm1/extends_native_type", 1),
    (function_as_function, "avm1/function_as_function", 1),
    (function_base_clip_removed, "avm1/function_base_clip_removed", 3),
    (function_base_clip, "avm1/function_base_clip", 2),
    (function_suppress_and_preload, "avm1/function_suppress_and_preload", 1),
    (funky_function_calls, "avm1/funky_function_calls", 1),
    (get_bytes_total, "avm1/get_bytes_total", 1),
    (getproperty_swf4, "avm1/getproperty_swf4", 1),
    (getproperty_swf5, "avm1/getproperty_swf5", 1),
    (getproperty, "avm1/getproperty", 1),
    (get_variable_in_scope, "avm1/get_variable_in_scope", 1),
    (global_array, "avm1/global_array", 1),
    (global_is_bare, "avm1/global_is_bare", 1),
    (glow_filter, "avm1/glow_filter", 1),
    (goto_advance1, "avm1/goto_advance1", 2),
    (goto_advance2, "avm1/goto_advance2", 2),
    (goto_both_ways1, "avm1/goto_both_ways1", 2),
    (goto_both_ways2, "avm1/goto_both_ways2", 3),
    (goto_execution_order, "avm1/goto_execution_order", 3),
    (goto_execution_order2, "avm1/goto_execution_order2", 2),
    (goto_frame_number, "avm1/goto_frame_number", 4),
    (goto_frame, "avm1/goto_frame", 3),
    (goto_frame2, "avm1/goto_frame2", 5),
    (goto_label, "avm1/goto_label", 4),
    (goto_methods, "avm1/goto_methods", 1),
    (goto_rewind1, "avm1/goto_rewind1", 4),
    (goto_rewind2, "avm1/goto_rewind2", 5),
    (goto_rewind3, "avm1/goto_rewind3", 2),
    (gradient_bevel_filter, "avm1/gradient_bevel_filter", 1),
    (gradient_glow_filter, "avm1/gradient_glow_filter", 1),
    (greater_swf6, "avm1/greater_swf6", 1),
    (greater_swf7, "avm1/greater_swf7", 1),
    (greaterthan_swf5, "avm1/greaterthan_swf5", 1),
    (greaterthan_swf8, "avm1/greaterthan_swf8", 1),
    (has_own_property, "avm1/has_own_property", 1),
    (infinite_recursion_function_in_setter, "avm1/infinite_recursion_function_in_setter", 1),
    (infinite_recursion_function, "avm1/infinite_recursion_function", 1),
    (infinite_recursion_virtual_property, "avm1/infinite_recursion_virtual_property", 1),
    (init_array_invalid, "avm1/init_array_invalid", 1),
    (init_object_invalid, "avm1/init_array_invalid", 1),
    (init_object_order, "avm1/init_object_order", 1),
    (is_finite, "avm1/is_finite", 1),
    (is_finite_swf6, "avm1/is_finite_swf6", 1),
    (is_prototype_of, "avm1/is_prototype_of", 1),
    (issue_1086, "avm1/issue_1086", 1),
    (issue_1104, "avm1/issue_1104", 3),
    (issue_1671, "avm1/issue_1671", 1),
    (issue_1906, "avm1/issue_1906", 2),
    (issue_2030, "avm1/issue_2030", 1),
    (issue_2084, "avm1/issue_2084", 2),
    (issue_2166, "avm1/issue_2166", 1),
    (issue_2870, "avm1/issue_2870", 10),
    (issue_3169, "avm1/issue_3169", 1),
    (issue_3446, "avm1/issue_3446", 1),
    (issue_3522, "avm1/issue_3522", 2),
    (issue_4377, "avm1/issue_4377", 1),
    (issue_710, "avm1/issue_710", 1),
    (issue_768, "avm1/issue_768", 1),
    (lessthan_swf4, "avm1/lessthan_swf4", 1),
    (lessthan_swf4_alt, "avm1/lessthan_swf4_alt", 1),
    (lessthan_swf5, "avm1/lessthan_swf5", 1),
    (lessthan, "avm1/lessthan", 1),
    (lessthan2_swf5, "avm1/lessthan2_swf5", 1),
    (lessthan2_swf6, "avm1/lessthan2_swf6", 1),
    (lessthan2_swf7, "avm1/lessthan2_swf7", 1),
    (load_vars, "avm1/load_vars", 2),
    (loadmovie_fail, "avm1/loadmovie_fail", 1),
    (loadmovie_method, "avm1/loadmovie_method", 2),
    (loadmovie_registerclass, "avm1/loadmovie_registerclass", 2),
    (loadmovie_replace_root, "avm1/loadmovie_replace_root", 3),
    (loadmovie, "avm1/loadmovie", 2),
    (loadmovienum, "avm1/loadmovienum", 2),
    (loadvariables_method, "avm1/loadvariables_method", 3),
    (loadvariables, "avm1/loadvariables", 3),
    (loadvariablesnum, "avm1/loadvariablesnum", 3),
    (logical_ops_swf4, "avm1/logical_ops_swf4", 1),
    (logical_ops_swf8, "avm1/logical_ops_swf8", 1),
    (looping, "avm1/looping", 6),
    (math_min_max, "avm1/math_min_max", 1),
    (matrix, "avm1/matrix", 1),
    (mcl_as_broadcaster, "avm1/mcl_as_broadcaster", 1),
    (mcl_getprogress, "avm1/mcl_getprogress", 6),
    (mcl_giftarget, "avm1/mcl_giftarget", 11),
    (mcl_jpgtarget, "avm1/mcl_jpgtarget", 11),
    (mcl_loadclip, "avm1/mcl_loadclip", 11),
    (mcl_mislabeled_target, "avm1/mcl_mislabeled_target", 11),
    (mcl_pngtarget, "avm1/mcl_pngtarget", 11),
    (mcl_unloadclip, "avm1/mcl_unloadclip", 11),
    (mouse_listeners, "avm1/mouse_listeners", 1),
    (mouse_events, "avm1/mouse_events", 8),
    (movieclip_depth_methods, "avm1/movieclip_depth_methods", 3),
    (movieclip_get_instance_at_depth, "avm1/movieclip_get_instance_at_depth", 1),
    (movieclip_hittest_shapeflag, "avm1/movieclip_hittest_shapeflag", 11),
    (movieclip_hittest, "avm1/movieclip_hittest", 1),
    (movieclip_init_object, "avm1/movieclip_init_object", 1),
    (movieclip_lockroot, "avm1/movieclip_lockroot", 10),
    (movieclip_prototype_extension, "avm1/movieclip_prototype_extension", 1),
    (nan_scale, "avm1/nan_scale", 1),
    (nested_textfields_in_buttons, "avm1/nested_textfields_in_buttons", 1),
    (new_method_wrap, "avm1/new_method_wrap", 1),
    (new_object_enumerate, "avm1/new_object_enumerate", 1),
    (new_object_wrap, "avm1/new_object_wrap", 1),
    (object_constructor, "avm1/object_constructor", 1),
    (object_function, "avm1/object_function", 1),
    (object_prototypes, "avm1/object_prototypes", 1),
    (object_string_coerce_swf5, "avm1/object_string_coerce_swf5", 1),
    (object_string_coerce_swf6, "avm1/object_string_coerce_swf6", 1),
    (on_construct, "avm1/on_construct", 1),
    (parse_float, "avm1/parse_float", 1),
    (parse_int, "avm1/parse_int", 1),
    (path_string, "avm1/path_string", 1),
    (point, "avm1/point", 1),
    (primitive_instanceof, "avm1/primitive_instanceof", 1),
    (primitive_type_globals, "avm1/primitive_type_globals", 1),
    (prototype_enumerate, "avm1/prototype_enumerate", 1),
    (prototype_properties, "avm1/prototype_properties", 1),
    (rectangle, "avm1/rectangle", 1),
    (recursive_prototypes, "avm1/recursive_prototypes", 2),
    (register_and_init_order, "avm1/register_and_init_order", 1),
    (register_class_return_value, "avm1/register_class_return_value", 1),
    (register_class_swf6, "avm1/register_class_swf6", 3),
    (register_class, "avm1/register_class", 3),
    (register_underflow, "avm1/register_underflow", 1),
    (remove_movie_clip, "avm1/remove_movie_clip", 2),
    (removed_base_clip_tell_target, "avm1/removed_base_clip_tell_target", 2),
    (removed_clip_halts_script, "avm1/removed_clip_halts_script", 13),
    (root_global_parent, "avm1/root_global_parent", 3),
    (selection, "avm1/selection", 1),
    (set_interval, "avm1/set_interval", 40),
    (set_variable_scope, "avm1/set_variable_scope", 1),
    (single_frame, "avm1/single_frame", 2),
    (slash_syntax, "avm1/slash_syntax", 2),
    (sound, "avm1/sound", 1),
    (stage_display_state, "avm1/stage_display_state", 1),
    (stage_object_children, "avm1/stage_object_children", 2),
    (stage_object_enumerate, "avm1/stage_object_enumerate", 1),
    (stage_object_properties_get_var, "avm1/stage_object_properties_get_var", 1),
    (stage_property_representation, "avm1/stage_property_representation", 1),
    (strictequals_swf6, "avm1/strictequals_swf6", 1),
    (strictly_equals, "avm1/strictly_equals", 1),
    (string_coercion, "avm1/string_coercion", 1),
    (string_methods_negative_args, "avm1/string_methods_negative_args", 1),
    (string_methods, "avm1/string_methods", 1),
    (string_ops_swf6, "avm1/string_ops_swf6", 1),
    (swf4_actions_bool, "avm1/swf4_actions_bool", 1),
    (swf4_bool, "avm1/swf4_bool", 1),
    (swf4_function_calls, "avm1/swf4_function_calls", 1),
    (swf5_encoding, "avm1/swf5_encoding", 1),
    (swf5_no_closure, "avm1/swf5_no_closure", 1),
    (swf5_to_6_cross_call, "avm1/swf5_to_6_cross_call", 2),
    (swf6_string_as_bool, "avm1/swf6_string_as_bool", 1),
    (swf6_case_insensitive, "avm1/swf6_case_insensitive", 1),
    (swf6_to_5_cross_call, "avm1/swf6_to_5_cross_call", 2),
    (swf7_case_sensitive, "avm1/swf7_case_sensitive", 1),
    (target_clip_removed, "avm1/target_clip_removed", 1),
    (target_clip_swf5, "avm1/target_clip_swf5", 2),
    (target_clip_swf6, "avm1/target_clip_swf6", 2),
    (target_path, "avm1/target_path", 1),
    (tell_target_invalid, "avm1/tell_target_invalid", 1),
    (tell_target, "avm1/tell_target", 3),
    (text_format, "avm1/text_format", 1),
    (textfield_background_color, "avm1/textfield_background_color", 1),
    (textfield_border_color, "avm1/textfield_border_color", 1),
    (textfield_properties, "avm1/textfield_properties", 1),
    #[ignore] (textfield_text, "avm1/textfield_text", 1),
    (textfield_variable, "avm1/textfield_variable", 8),
    (this_scoping, "avm1/this_scoping", 1),
    (timeline_function_def, "avm1/timeline_function_def", 3),
    (avm2_timer, "avm2/timer", 280, frame_time_sleep = true),
    (timer_run_actions, "avm1/timer_run_actions", 1),
    (trace, "avm1/trace", 1),
    (transform, "avm1/transform", 1),
    (try_catch_finally, "avm1/try_catch_finally", 1),
    (try_finally_simple, "avm1/try_finally_simple", 1),
    (typeof_globals, "avm1/typeof_globals", 1),
    (typeofs, "avm1/typeof", 1),
    (uncaught_exception_bubbled, "avm1/uncaught_exception_bubbled", 1),
    (uncaught_exception, "avm1/uncaught_exception", 1),
    (undefined_to_string_swf6, "avm1/undefined_to_string_swf6", 1),
    (unescape, "avm1/unescape", 1),
    (unload_clip_event, "avm1/unload_clip_event", 2),
    (unloadmovie_method, "avm1/unloadmovie_method", 11),
    (unloadmovie, "avm1/unloadmovie", 11),
    (unloadmovienum, "avm1/unloadmovienum", 11),
    (use_hand_cursor, "avm1/use_hand_cursor", 1),
    (variable_args, "avm1/variable_args", 1),
    (waitforframe, "avm1/waitforframe", 1),
    (watch_textfield, "avm1/watch_textfield", 1),
    (watch_virtual_property_proto, "avm1/watch_virtual_property_proto", 1),
    #[ignore] (watch_virtual_property, "avm1/watch_virtual_property", 1),
    (watch, "avm1/watch", 1),
    (with_return, "avm1/with_return", 1),
    (with, "avm1/with", 1),
    (xml_append_child_with_parent, "avm1/xml_append_child_with_parent", 1),
    (xml_append_child, "avm1/xml_append_child", 1),
    (xml_attributes_read, "avm1/xml_attributes_read", 1),
    (xml_cdata, "avm1/xml_cdata", 1),
    (xml_clone_expandos, "avm1/xml_clone_expandos", 1),
    (xml_first_last_child, "avm1/xml_first_last_child", 1),
    (xml_has_child_nodes, "avm1/xml_has_child_nodes", 1),
    (xml_idmap, "avm1/xml_idmap", 1),
    (xml_ignore_comments, "avm1/xml_ignore_comments", 1),
    (xml_ignore_white, "avm1/xml_ignore_white", 1),
    (xml_insert_before, "avm1/xml_insert_before", 1),
    (xml_inspect_createmethods, "avm1/xml_inspect_createmethods", 1),
    (xml_inspect_doctype, "avm1/xml_inspect_doctype", 1),
    (xml_inspect_parsexml, "avm1/xml_inspect_parsexml", 1),
    (xml_inspect_xmldecl, "avm1/xml_inspect_xmldecl", 1),
    (xml_load, "avm1/xml_load", 1),
    (xml_namespaces, "avm1/xml_namespaces", 1),
    (xml_parent_and_child, "avm1/xml_parent_and_child", 1),
    (xml_remove_node, "avm1/xml_remove_node", 1),
    (xml_reparenting, "avm1/xml_reparenting", 1),
    (xml_siblings, "avm1/xml_siblings", 1),
    (xml_to_string_comment, "avm1/xml_to_string_comment", 1),
    (xml_to_string, "avm1/xml_to_string", 1),
    (xml_unescaping, "avm1/xml_unescaping", 1),
    (xml, "avm1/xml", 1),
}

// TODO: These tests have some inaccuracies currently, so we use approx_eq to test that numeric values are close enough.
// Eventually we can hopefully make some of these match exactly (see #193).
// Some will probably always need to be approx. (if they rely on trig functions, etc.)
swf_tests_approx! {
    (as3_coerce_string_precision, "avm2/coerce_string_precision", 1, max_relative = 30.0 * f64::EPSILON),
    (as3_displayobject_height, "avm2/displayobject_height", 7, epsilon = 0.06), // TODO: height/width appears to be off by 1 twip sometimes
    (as3_displayobject_rotation, "avm2/displayobject_rotation", 1, epsilon = 0.0000000001),
    (as3_displayobject_scrollrect, "avm2/displayobject_scrollrect", 100, @num_patterns = &[
        Regex::new(r"\(a=(.+), b=(.+), c=(.+), d=(.+), tx=(.+), ty=(.+)\)").unwrap()
    ], @img = true, max_relative = f32::EPSILON as f64),
    (as3_displayobject_width, "avm2/displayobject_width", 7, epsilon = 0.06),
    (as3_displayobject_transform, "avm2/displayobject_transform", 1, @num_patterns = &[
        Regex::new(r"\(a=(.+), b=(.+), c=(.+), d=(.+), tx=(.+), ty=(.+)\)").unwrap()
    ], max_relative = f32::EPSILON as f64),
    (as3_divide, "avm2/divide", 1, epsilon = 0.0), // TODO: Discrepancy in float formatting.
    (as3_edittext_align, "avm2/edittext_align", 1, epsilon = 3.0),
    (as3_edittext_autosize, "avm2/edittext_autosize", 1, epsilon = 0.1),
    (as3_edittext_bullet, "avm2/edittext_bullet", 1, epsilon = 3.0),
    (as3_edittext_font_size, "avm2/edittext_font_size", 1, epsilon = 0.1),
    (as3_edittext_getlinemetrics, "avm2/edittext_getlinemetrics", 1, epsilon = 0.85),
    (as3_edittext_leading, "avm2/edittext_leading", 1, epsilon = 0.3),
    (as3_edittext_letter_spacing, "avm2/edittext_letter_spacing", 1, epsilon = 15.0), // TODO: Discrepancy in wrapping in letterSpacing = 0.1 test.
    (as3_edittext_margins, "avm2/edittext_margins", 1, epsilon = 5.0), // TODO: Discrepancy in wrapping.
    (as3_edittext_tab_stops, "avm2/edittext_tab_stops", 1, epsilon = 5.0),
    (as3_edittext_underline, "avm2/edittext_underline", 1, epsilon = 4.0),
    (as3_math, "avm2/math", 1, max_relative = 30.0 * f64::EPSILON),
    (as3_matrix, "avm2/matrix", 1, @num_patterns = &[
        Regex::new(r"\(a=(.+?), b=(.+?), c=(.+?), d=(.+?), tx=(.+?), ty=(.+?)\)").unwrap()
    ], max_relative = f32::EPSILON as f64),
    (as3_number_toexponential, "avm2/number_toexponential", 1, max_relative = 0.001),
    (as3_number_tofixed, "avm2/number_tofixed", 1, max_relative = 0.001),
    (as3_number_toprecision, "avm2/number_toprecision", 1, max_relative = 0.001),
    (as3_parse_float, "avm2/parse_float", 1, max_relative = 5.0 * f64::EPSILON),
    (as3_parse_float_swf10, "avm2/parse_float_swf10", 1, max_relative = 5.0 * f64::EPSILON),
    (edittext_align, "avm1/edittext_align", 1, epsilon = 3.0),
    (edittext_autosize, "avm1/edittext_autosize", 1, epsilon = 0.1),
    (edittext_bullet, "avm1/edittext_bullet", 1, epsilon = 3.0),
    (edittext_hscroll, "avm1/edittext_hscroll", 1, epsilon = 3.0),
    (edittext_letter_spacing, "avm1/edittext_letter_spacing", 1, epsilon = 15.0), // TODO: Discrepancy in wrapping in letterSpacing = 0.1 test.
    (edittext_margins, "avm1/edittext_margins", 1, epsilon = 5.0), // TODO: Discrepancy in wrapping.
    (edittext_tab_stops, "avm1/edittext_tab_stops", 1, epsilon = 5.0),
    (edittext_underline, "avm1/edittext_underline", 1, epsilon = 4.0),
    (gettextextent, "avm1/gettextextent", 1, epsilon = 30.0), // TODO: Flash Player breaks single words that are longer than the line, but we don't.
    (local_to_global, "avm1/local_to_global", 1, epsilon = 0.051),
    (movieclip_getbounds, "avm1/movieclip_getbounds", 1, epsilon = 0.051),
    (stage_object_properties_swf6, "avm1/stage_object_properties_swf6", 4, epsilon = 0.051),
    (stage_object_properties, "avm1/stage_object_properties", 6, epsilon = 0.051),
}

#[test]
fn external_interface_avm1() -> Result<(), Error> {
    set_logger();
    test_swf_with_hooks(
        "tests/swfs/avm1/external_interface/test.swf",
        1,
        "tests/swfs/avm1/external_interface/input.json",
        "tests/swfs/avm1/external_interface/output.txt",
        |player| {
            player
                .lock()
                .unwrap()
                .add_external_interface(Box::new(ExternalInterfaceTestProvider::new()));
            Ok(())
        },
        |player| {
            let mut player_locked = player.lock().unwrap();

            let parroted =
                player_locked.call_internal_interface("parrot", vec!["Hello World!".into()]);
            player_locked.log_backend().avm_trace(&format!(
                "After calling `parrot` with a string: {:?}",
                parroted
            ));

            let mut nested = BTreeMap::new();
            nested.insert(
                "list".to_string(),
                vec![
                    "string".into(),
                    100.into(),
                    false.into(),
                    ExternalValue::Object(BTreeMap::new()),
                ]
                .into(),
            );

            let mut root = BTreeMap::new();
            root.insert("number".to_string(), (-500.1).into());
            root.insert("string".to_string(), "A string!".into());
            root.insert("true".to_string(), true.into());
            root.insert("false".to_string(), false.into());
            root.insert("null".to_string(), ExternalValue::Null);
            root.insert("nested".to_string(), nested.into());
            let result = player_locked
                .call_internal_interface("callWith", vec!["trace".into(), root.into()]);
            player_locked.log_backend().avm_trace(&format!(
                "After calling `callWith` with a complex payload: {:?}",
                result
            ));
            Ok(())
        },
        false,
        false,
    )
}

#[test]
fn external_interface_avm2() -> Result<(), Error> {
    set_logger();
    test_swf_with_hooks(
        "tests/swfs/avm2/external_interface/test.swf",
        1,
        "tests/swfs/avm2/external_interface/input.json",
        "tests/swfs/avm2/external_interface/output.txt",
        |player| {
            player
                .lock()
                .unwrap()
                .add_external_interface(Box::new(ExternalInterfaceTestProvider::new()));
            Ok(())
        },
        |player| {
            let mut player_locked = player.lock().unwrap();

            let parroted =
                player_locked.call_internal_interface("parrot", vec!["Hello World!".into()]);
            player_locked.log_backend().avm_trace(&format!(
                "After calling `parrot` with a string: {:?}",
                parroted
            ));

            player_locked.call_internal_interface("freestanding", vec!["Hello World!".into()]);

            let root: ExternalValue = vec![
                "string".into(),
                100.into(),
                ExternalValue::Null,
                false.into(),
            ]
            .into();

            let result =
                player_locked.call_internal_interface("callWith", vec!["trace".into(), root]);
            player_locked.log_backend().avm_trace(&format!(
                "After calling `callWith` with a complex payload: {:?}",
                result
            ));
            Ok(())
        },
        false,
        false,
    )
}

#[test]
fn shared_object_avm1() -> Result<(), Error> {
    set_logger();
    // Test SharedObject persistence. Run an SWF that saves data
    // to a shared object twice and verify that the data is saved.
    let mut memory_storage_backend: Box<dyn StorageBackend> =
        Box::<MemoryStorageBackend>::default();

    // Initial run; no shared object data.
    test_swf_with_hooks(
        "tests/swfs/avm1/shared_object/test.swf",
        1,
        "tests/swfs/avm1/shared_object/input1.json",
        "tests/swfs/avm1/shared_object/output1.txt",
        |_player| Ok(()),
        |player| {
            // Save the storage backend for next run.
            let mut player = player.lock().unwrap();
            std::mem::swap(player.storage_mut(), &mut memory_storage_backend);
            Ok(())
        },
        false,
        false,
    )?;

    // Verify that the flash cookie matches the expected one
    let expected = std::fs::read("tests/swfs/avm1/shared_object/RuffleTest.sol")?;
    assert_eq!(
        expected,
        memory_storage_backend
            .get("localhost//RuffleTest")
            .unwrap_or_default()
    );

    // Re-run the SWF, verifying that the shared object persists.
    test_swf_with_hooks(
        "tests/swfs/avm1/shared_object/test.swf",
        1,
        "tests/swfs/avm1/shared_object/input2.json",
        "tests/swfs/avm1/shared_object/output2.txt",
        |player| {
            // Swap in the previous storage backend.
            let mut player = player.lock().unwrap();
            std::mem::swap(player.storage_mut(), &mut memory_storage_backend);
            Ok(())
        },
        |_player| Ok(()),
        false,
        false,
    )?;

    Ok(())
}

#[test]
fn shared_object_avm2() -> Result<(), Error> {
    set_logger();
    // Test SharedObject persistence. Run an SWF that saves data
    // to a shared object twice and verify that the data is saved.
    let mut memory_storage_backend: Box<dyn StorageBackend> =
        Box::<MemoryStorageBackend>::default();

    // Initial run; no shared object data.
    test_swf_with_hooks(
        "tests/swfs/avm2/shared_object/test.swf",
        1,
        "tests/swfs/avm2/shared_object/input1.json",
        "tests/swfs/avm2/shared_object/output1.txt",
        |_player| Ok(()),
        |player| {
            // Save the storage backend for next run.
            let mut player = player.lock().unwrap();
            std::mem::swap(player.storage_mut(), &mut memory_storage_backend);
            Ok(())
        },
        false,
        false,
    )?;

    // Verify that the flash cookie matches the expected one
    let expected = std::fs::read("tests/swfs/avm2/shared_object/RuffleTest.sol")?;
    assert_eq!(
        expected,
        memory_storage_backend
            .get("localhost//RuffleTest")
            .unwrap_or_default()
    );

    // Re-run the SWF, verifying that the shared object persists.
    test_swf_with_hooks(
        "tests/swfs/avm2/shared_object/test.swf",
        1,
        "tests/swfs/avm2/shared_object/input2.json",
        "tests/swfs/avm2/shared_object/output2.txt",
        |player| {
            // Swap in the previous storage backend.
            let mut player = player.lock().unwrap();
            std::mem::swap(player.storage_mut(), &mut memory_storage_backend);
            Ok(())
        },
        |_player| Ok(()),
        false,
        false,
    )?;

    Ok(())
}

#[test]
fn timeout_avm1() -> Result<(), Error> {
    set_logger();
    test_swf_with_hooks(
        "tests/swfs/avm1/timeout/test.swf",
        1,
        "tests/swfs/avm1/timeout/input.json",
        "tests/swfs/avm1/timeout/output.txt",
        |player| {
            player
                .lock()
                .unwrap()
                .set_max_execution_duration(Duration::from_secs(5));
            Ok(())
        },
        |_| Ok(()),
        false,
        false,
    )
}

#[test]
fn stage_scale_mode() -> Result<(), Error> {
    set_logger();
    test_swf_with_hooks(
        "tests/swfs/avm1/stage_scale_mode/test.swf",
        1,
        "tests/swfs/avm1/stage_scale_mode/input.json",
        "tests/swfs/avm1/stage_scale_mode/output.txt",
        |player| {
            // Simulate a large viewport to test stage size.
            player
                .lock()
                .unwrap()
                .set_viewport_dimensions(ViewportDimensions {
                    width: 900,
                    height: 900,
                    scale_factor: 1.0,
                });
            Ok(())
        },
        |_| Ok(()),
        false,
        false,
    )
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
fn test_swf(
    swf_path: &str,
    num_frames: u32,
    simulated_input_path: &str,
    expected_output_path: &str,
    check_img: bool,
    frame_time_sleep: bool,
) -> Result<(), Error> {
    test_swf_with_hooks(
        swf_path,
        num_frames,
        simulated_input_path,
        expected_output_path,
        |_| Ok(()),
        |_| Ok(()),
        check_img,
        frame_time_sleep,
    )
}

/// Loads an SWF and runs it through the Ruffle core for a number of frames.
/// Tests that the trace output matches the given expected output.
#[allow(clippy::too_many_arguments)]
fn test_swf_with_hooks(
    swf_path: &str,
    num_frames: u32,
    simulated_input_path: &str,
    expected_output_path: &str,
    before_start: impl FnOnce(Arc<Mutex<Player>>) -> Result<(), Error>,
    before_end: impl FnOnce(Arc<Mutex<Player>>) -> Result<(), Error>,
    check_img: bool,
    frame_time_sleep: bool,
) -> Result<(), Error> {
    let injector =
        InputInjector::from_file(simulated_input_path).unwrap_or_else(|_| InputInjector::empty());
    let mut expected_output = std::fs::read_to_string(expected_output_path)?.replace("\r\n", "\n");

    // Strip a trailing newline if it has one.
    if expected_output.ends_with('\n') {
        expected_output = expected_output[0..expected_output.len() - "\n".len()].to_string();
    }

    let trace_log = run_swf(
        swf_path,
        num_frames,
        before_start,
        injector,
        before_end,
        check_img,
        frame_time_sleep,
    )?;
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
    simulated_input_path: &str,
    expected_output_path: &str,
    num_patterns: &[Regex],
    check_img: bool,
    approx_assert_fn: impl Fn(f64, f64),
) -> Result<(), Error> {
    let injector =
        InputInjector::from_file(simulated_input_path).unwrap_or_else(|_| InputInjector::empty());
    let trace_log = run_swf(
        swf_path,
        num_frames,
        |_| Ok(()),
        injector,
        |_| Ok(()),
        check_img,
        false,
    )?;
    let mut expected_data = std::fs::read_to_string(expected_output_path)?;

    // Strip a trailing newline if it has one.
    if expected_data.ends_with('\n') {
        expected_data = expected_data[0..expected_data.len() - "\n".len()].to_string();
    }

    std::assert_eq!(
        trace_log.lines().count(),
        expected_data.lines().count(),
        "# of lines of output didn't match"
    );

    for (actual, expected) in trace_log.lines().zip(expected_data.lines()) {
        // If these are numbers, compare using approx_eq.
        if let (Ok(actual), Ok(expected)) = (actual.parse::<f64>(), expected.parse::<f64>()) {
            // NaNs should be able to pass in an approx test.
            if actual.is_nan() && expected.is_nan() {
                continue;
            }

            // TODO: Lower this epsilon as the accuracy of the properties improves.
            // if let Some(relative_epsilon) = relative_epsilon {
            //     assert_relative_eq!(
            //         actual,
            //         expected,
            //         epsilon = absolute_epsilon,
            //         max_relative = relative_epsilon
            //     );
            // } else {
            //     assert_abs_diff_eq!(actual, expected, epsilon = absolute_epsilon);
            // }
            approx_assert_fn(actual, expected);
        } else {
            let mut found = false;
            // Check each of the user-provided regexes for a match
            for pattern in num_patterns {
                if let (Some(actual_captures), Some(expected_captures)) =
                    (pattern.captures(actual), pattern.captures(expected))
                {
                    found = true;
                    std::assert_eq!(
                        actual_captures.len(),
                        expected_captures.len(),
                        "Differing numbers of regex captures"
                    );

                    // Each capture group (other than group 0, which is always the entire regex
                    // match) represents a floating-point value
                    for (actual_val, expected_val) in actual_captures
                        .iter()
                        .skip(1)
                        .zip(expected_captures.iter().skip(1))
                    {
                        let actual_num = actual_val
                            .expect("Missing capture gruop value for 'actual'")
                            .as_str()
                            .parse::<f64>()
                            .expect("Failed to parse 'actual' capture group as float");
                        let expected_num = expected_val
                            .expect("Missing capture gruop value for 'expected'")
                            .as_str()
                            .parse::<f64>()
                            .expect("Failed to parse 'expected' capture group as float");
                        approx_assert_fn(actual_num, expected_num);
                    }
                    let modified_actual = pattern.replace(actual, "");
                    let modified_expected = pattern.replace(expected, "");
                    assert_eq!(modified_actual, modified_expected);
                    break;
                }
            }
            if !found {
                assert_eq!(actual, expected);
            }
        }
    }
    Ok(())
}

/// Loads an SWF and runs it through the Ruffle core for a number of frames.
/// Tests that the trace output matches the given expected output.
fn run_swf(
    swf_path: &str,
    num_frames: u32,
    before_start: impl FnOnce(Arc<Mutex<Player>>) -> Result<(), Error>,
    mut injector: InputInjector,
    before_end: impl FnOnce(Arc<Mutex<Player>>) -> Result<(), Error>,
    #[allow(unused)] mut check_img: bool,
    frame_time_sleep: bool,
) -> Result<String, Error> {
    #[allow(unused_assignments)]
    {
        check_img &= RUN_IMG_TESTS;
    }

    let base_path = Path::new(swf_path).parent().unwrap();
    let mut executor = NullExecutor::new();
    let movie = SwfMovie::from_path(swf_path, None)?;
    let frame_time = 1000.0 / movie.frame_rate().to_f64();
    let frame_time_duration = Duration::from_millis(frame_time as u64);
    let trace_output = Rc::new(RefCell::new(Vec::new()));

    #[allow(unused_mut)]
    let mut builder = PlayerBuilder::new();

    #[cfg(feature = "imgtests")]
    if check_img {
        const BACKEND: wgpu::Backends = wgpu::Backends::PRIMARY;

        let instance = wgpu::Instance::new(BACKEND);

        let descriptors =
            futures::executor::block_on(WgpuRenderBackend::<TextureTarget>::build_descriptors(
                BACKEND,
                instance,
                None,
                Default::default(),
                None,
            ))?;

        let width = movie.width().to_pixels() as u32;
        let height = movie.height().to_pixels() as u32;

        let target = TextureTarget::new(&descriptors.device, (width, height))?;

        builder = builder
            .with_renderer(WgpuRenderBackend::new(Arc::new(descriptors), target)?)
            .with_viewport_dimensions(width, height, 1.0);
    };

    let player = builder
        .with_log(TestLogBackend::new(trace_output.clone()))
        .with_navigator(NullNavigatorBackend::with_base_path(base_path, &executor))
        .with_max_execution_duration(Duration::from_secs(300))
        .with_viewport_dimensions(
            movie.width().to_pixels() as u32,
            movie.height().to_pixels() as u32,
            1.0,
        )
        .with_movie(movie)
        .build();

    before_start(player.clone())?;

    for _ in 0..num_frames {
        // If requested, ensure that the 'expected' amount of
        // time actually elapses between frames. This is useful for
        // tests that call 'flash.utils.getTimer()' and use
        // 'setInterval'/'flash.utils.Timer'
        //
        // Note that when Ruffle actually runs frames, we can
        // execute frames faster than this in order to 'catch up'
        // if we've fallen behind. However, in order to make regression
        // tests deterministic, we always call 'update_timers' with
        // an elapsed time of 'frame_time'. By sleeping for 'frame_time_duration',
        // we ensure that the result of 'flash.utils.getTimer()' is consistent
        // with timer execution (timers will see an elapsed time of *at least*
        // the requested timer interval).
        if frame_time_sleep {
            std::thread::sleep(frame_time_duration);
        }

        while !player
            .lock()
            .unwrap()
            .preload(&mut ExecutionLimit::exhausted())
        {}

        player.lock().unwrap().run_frame();
        player.lock().unwrap().update_timers(frame_time);
        executor.run();

        injector.next(|evt, _btns_down| {
            player.lock().unwrap().handle_event(match evt {
                AutomatedEvent::MouseDown { pos, btn } => PlayerEvent::MouseDown {
                    x: pos.0,
                    y: pos.1,
                    button: match btn {
                        InputMouseButton::Left => RuffleMouseButton::Left,
                        InputMouseButton::Middle => RuffleMouseButton::Middle,
                        InputMouseButton::Right => RuffleMouseButton::Right,
                    },
                },
                AutomatedEvent::MouseMove { pos } => PlayerEvent::MouseMove { x: pos.0, y: pos.1 },
                AutomatedEvent::MouseUp { pos, btn } => PlayerEvent::MouseUp {
                    x: pos.0,
                    y: pos.1,
                    button: match btn {
                        InputMouseButton::Left => RuffleMouseButton::Left,
                        InputMouseButton::Middle => RuffleMouseButton::Middle,
                        InputMouseButton::Right => RuffleMouseButton::Right,
                    },
                },
                AutomatedEvent::Wait => unreachable!(),
            });
        });
        // Rendering has side-effects (such as processing 'DisplayObject.scrollRect' updates)
        player.lock().unwrap().render();
    }

    // Render the image to disk
    // FIXME: Determine how we want to compare against on on-disk image
    #[cfg(feature = "imgtests")]
    if check_img {
        let mut player_lock = player.lock().unwrap();
        player_lock.render();
        let renderer = player_lock
            .renderer_mut()
            .downcast_mut::<WgpuRenderBackend<TextureTarget>>()
            .unwrap();

        // Use straight alpha, since we want to save this as a PNG
        let actual_image = renderer
            .capture_frame(false)
            .expect("Failed to capture image");

        let info = renderer.descriptors().adapter.get_info();
        let suffix = format!("{}-{:?}", std::env::consts::OS, info.backend);

        let expected_image_path = base_path.join(format!("expected-{}.png", &suffix));
        let expected_image = image::open(&expected_image_path);

        let matches = match expected_image {
            Ok(img) => {
                img.as_rgba8().expect("Expected 8-bit RGBA image").as_raw() == actual_image.as_raw()
            }
            Err(e) => {
                eprintln!(
                    "Failed to open expected image {:?}: {e:?}",
                    &expected_image_path
                );
                false
            }
        };

        if !matches {
            let actual_image_path = base_path.join(format!("actual-{suffix}.png"));
            actual_image.save_with_format(&actual_image_path, image::ImageFormat::Png)?;
            panic!(
                "Test output does not match expected image - saved actual image to {:?}",
                actual_image_path
            );
        }
    }

    before_end(player)?;

    executor.run();

    let trace = trace_output.borrow().join("\n");
    Ok(trace)
}

struct TestLogBackend {
    trace_output: Rc<RefCell<Vec<String>>>,
}

impl TestLogBackend {
    pub fn new(trace_output: Rc<RefCell<Vec<String>>>) -> Self {
        Self { trace_output }
    }
}

impl LogBackend for TestLogBackend {
    fn avm_trace(&self, message: &str) {
        self.trace_output.borrow_mut().push(message.to_string());
    }
}

#[derive(Default)]
pub struct ExternalInterfaceTestProvider {}

impl ExternalInterfaceTestProvider {
    pub fn new() -> Self {
        Default::default()
    }
}

fn do_trace(context: &mut UpdateContext<'_, '_, '_>, args: &[ExternalValue]) -> ExternalValue {
    context.avm_trace(&format!("[ExternalInterface] trace: {args:?}"));
    "Traced!".into()
}

fn do_ping(context: &mut UpdateContext<'_, '_, '_>, _args: &[ExternalValue]) -> ExternalValue {
    context.avm_trace("[ExternalInterface] ping");
    "Pong!".into()
}

fn do_reentry(context: &mut UpdateContext<'_, '_, '_>, _args: &[ExternalValue]) -> ExternalValue {
    context.avm_trace("[ExternalInterface] starting reentry");
    if let Some(callback) = context.external_interface.get_callback("callWith") {
        callback.call(
            context,
            "callWith",
            vec!["trace".into(), "successful reentry!".into()],
        )
    } else {
        ExternalValue::Null
    }
}

impl ExternalInterfaceProvider for ExternalInterfaceTestProvider {
    fn get_method(&self, name: &str) -> Option<Box<dyn ExternalInterfaceMethod>> {
        match name {
            "trace" => Some(Box::new(do_trace)),
            "ping" => Some(Box::new(do_ping)),
            "reentry" => Some(Box::new(do_reentry)),
            _ => None,
        }
    }

    fn on_callback_available(&self, _name: &str) {}

    fn on_fs_command(&self, _command: &str, _args: &str) -> bool {
        false
    }
}
