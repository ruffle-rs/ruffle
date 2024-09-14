use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::property::Attribute;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, ScriptObject, TObject, Value};
use crate::context::GcContext;
use crate::display_object::{DisplayObject, TDisplayObject, TDisplayObjectContainer};
use crate::string::{AvmString, WStr, WString};
use gc_arena::Collect;
use std::str;

mod accessibility;
mod array;
pub(crate) mod as_broadcaster;
pub(crate) mod bevel_filter;
mod bitmap_data;
mod bitmap_filter;
pub(crate) mod blur_filter;
pub(crate) mod boolean;
pub(crate) mod button;
mod color;
pub(crate) mod color_matrix_filter;
pub(crate) mod color_transform;
pub(crate) mod context_menu;
pub(crate) mod context_menu_item;
pub(crate) mod convolution_filter;
pub(crate) mod date;
pub(crate) mod displacement_map_filter;
pub(crate) mod drop_shadow_filter;
pub(crate) mod error;
mod external_interface;
pub(crate) mod file_reference;
mod function;
pub(crate) mod glow_filter;
pub(crate) mod gradient_filter;
mod key;
mod load_vars;
pub(crate) mod local_connection;
mod math;
mod matrix;
pub(crate) mod mouse;
pub(crate) mod movie_clip;
mod movie_clip_loader;
pub(crate) mod netconnection;
pub(crate) mod netstream;
pub(crate) mod number;
mod object;
mod point;
mod rectangle;
mod selection;
pub(crate) mod shared_object;
pub(crate) mod sound;
mod stage;
pub(crate) mod string;
pub(crate) mod style_sheet;
pub(crate) mod system;
pub(crate) mod system_capabilities;
pub(crate) mod system_ime;
pub(crate) mod system_security;
pub(crate) mod text_field;
mod text_format;
pub(crate) mod transform;
mod video;
pub(crate) mod xml;
mod xml_node;
pub(crate) mod xml_socket;

const GLOBAL_DECLS: &[Declaration] = declare_properties! {
    "trace" => method(trace; DONT_ENUM);
    "isFinite" => method(is_finite; DONT_ENUM);
    "isNaN" => method(is_nan; DONT_ENUM);
    "parseInt" => method(parse_int; DONT_ENUM);
    "parseFloat" => method(parse_float; DONT_ENUM);
    "ASSetPropFlags" => method(object::as_set_prop_flags; DONT_ENUM);
    "clearInterval" => method(clear_interval; DONT_ENUM);
    "setInterval" => method(set_interval; DONT_ENUM);
    "clearTimeout" => method(clear_timeout; DONT_ENUM);
    "setTimeout" => method(set_timeout; DONT_ENUM);
    "updateAfterEvent" => method(update_after_event; DONT_ENUM);
    "escape" => method(escape; DONT_ENUM);
    "unescape" => method(unescape; DONT_ENUM);
    "NaN" => property(get_nan; DONT_ENUM);
    "Infinity" => property(get_infinity; DONT_ENUM);
};

pub fn trace<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Unlike `Action::Trace`, `_global.trace` always coerces
    // undefined to "" in SWF6 and below. It also doesn't log
    // anything outside of the Flash editor's trace window.
    let out = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;
    activation.context.avm_trace(&out.to_utf8_lossy());
    Ok(Value::Undefined)
}

pub fn is_finite<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(val) = args.get(0) {
        Ok(val.coerce_to_f64(activation)?.is_finite().into())
    } else {
        Ok(false.into())
    }
}

pub fn is_nan<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(val) = args.get(0) {
        Ok(val.coerce_to_f64(activation)?.is_nan().into())
    } else {
        Ok(true.into())
    }
}

pub fn parse_int<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // ECMA-262 violation: parseInt() == undefined // not NaN
    if args.is_empty() {
        return Ok(Value::Undefined);
    }

    let radix: Option<i32> = args
        .get(1)
        .map(|x| x.coerce_to_i32(activation))
        .transpose()?;
    let radix = match radix {
        Some(r @ 2..=36) => Some(r as u32),
        Some(_) => return Ok(f64::NAN.into()),
        None => None,
    };

    let string = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;
    let string = string.as_wstr();

    fn parse_sign(string: &WStr) -> Option<f64> {
        string.get(0).and_then(|c| match u8::try_from(c) {
            Ok(b'+') => Some(1.),
            Ok(b'-') => Some(-1.),
            _ => None,
        })
    }

    let (radix, ignore_sign, string) = {
        let has_sign = parse_sign(string).is_some();

        let off = if has_sign { 1 } else { 0 };
        let zero = string.get(off) == Some(b'0' as u16);
        let hex = if zero {
            let hex = string.get(off + 1);
            hex == Some(b'x' as u16) || hex == Some(b'X' as u16)
        } else {
            false
        };

        if hex {
            if has_sign {
                // Emulate bug: unless "0x" is a valid sequence of digits in a given radix, the
                // prefixes "+0x", "+0X", "-0x", "-0X" should result in NaN instead of 0.
                // Otherwise, the minus sign should be ignored.
                match radix {
                    None | Some(0..=33) => return Ok(f64::NAN.into()),
                    Some(radix) => (radix, true, string),
                }
            } else {
                // Auto-detect hexadecimal prefix "0x" and strip it.
                // Emulate bug: the prefix is stripped regardless of the radix.
                //   parseInt('0x100', 10) == 100  // not 0
                //   parseInt('0x100', 36) == 1296 // not 1540944
                // Emulate bug: the prefix is expected before the sign or spaces.
                //   parseInt("0x  -10") == -16 // not NaN
                //   parseInt("  -0x10") == NaN // not -16
                (radix.unwrap_or(16), false, &string[2..])
            }
        } else if zero
            && radix.is_none()
            && string[1..]
                .iter()
                .all(|c| c >= b'0' as u16 && c <= b'7' as u16)
        {
            // ECMA-262 violation: auto-detect octal numbers ("0", "+0" or "-0" prefixes).
            // An auto-detected octal number cannot contain leading spaces or extra trailing characters.
            (8, false, string)
        } else {
            (radix.unwrap_or(10), false, string)
        }
    };

    // Strip spaces.
    let string = string.trim_start_matches(b"\t\n\r ".as_ref());

    let (sign, string) = if let Some(sign) = parse_sign(string) {
        let sign = if ignore_sign { 1. } else { sign };
        (sign, &string[1..])
    } else {
        (1., string)
    };

    let mut empty = true;
    let mut result = 0.0f64;
    for chr in string {
        let digit = u8::try_from(chr)
            .ok()
            .and_then(|c| (c as char).to_digit(radix));
        if let Some(digit) = digit {
            result = result * radix as f64 + digit as f64;
            empty = false;
        } else {
            break;
        }
    }

    if empty {
        Ok(f64::NAN.into())
    } else {
        Ok(result.copysign(sign).into())
    }
}

pub fn get_infinity<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if activation.swf_version() > 4 {
        Ok(f64::INFINITY.into())
    } else {
        Ok(Value::Undefined)
    }
}

pub fn get_nan<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if activation.swf_version() > 4 {
        Ok(f64::NAN.into())
    } else {
        Ok(Value::Undefined)
    }
}

pub fn parse_float<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(value) = args.get(0) {
        let string = value.coerce_to_string(activation)?;
        Ok(crate::avm1::value::parse_float_impl(&string, false).into())
    } else {
        Ok(Value::Undefined)
    }
}

pub fn set_interval<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    create_timer(activation, this, args, false)
}

pub fn set_timeout<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    create_timer(activation, this, args, true)
}

pub fn create_timer<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
    is_timeout: bool,
) -> Result<Value<'gc>, Error<'gc>> {
    // `setInterval` was added in Flash Player 6 but is not version-gated.
    use crate::timer::TimerCallback;

    let (callback, interval) = match args.get(0) {
        Some(Value::Object(o)) if o.as_executable().is_some() => (
            TimerCallback::Avm1Function {
                func: *o,
                params: args.get(2..).unwrap_or_default().to_vec(),
            },
            args.get(1),
        ),
        Some(Value::Object(o)) => (
            TimerCallback::Avm1Method {
                this: *o,
                method_name: args
                    .get(1)
                    .unwrap_or(&Value::Undefined)
                    .coerce_to_string(activation)?,
                params: args.get(3..).map(|s| s.to_vec()).unwrap_or_default(),
            },
            args.get(2),
        ),
        _ => return Ok(Value::Undefined),
    };

    let interval = match interval.unwrap_or(&Value::Undefined) {
        Value::Undefined => return Ok(Value::Undefined),
        value => value.coerce_to_i32(activation)?,
    };

    // If `is_timeout` is true, then set a repeat count of 1.
    // Otherwise, set a repeat count of 0 (repeat indefinitely)
    //
    // We start the timer immediately
    let id = activation
        .context
        .timers
        .add_timer(callback, interval, is_timeout);

    Ok(id.into())
}

pub fn clear_interval<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let id = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_i32(activation)?;
    if !activation.context.timers.remove(id) {
        tracing::info!("clearInterval: Timer {} does not exist", id);
    }

    Ok(Value::Undefined)
}

pub fn clear_timeout<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let id = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_i32(activation)?;
    if !activation.context.timers.remove(id) {
        tracing::info!("clearTimeout: Timer {} does not exist", id);
    }

    Ok(Value::Undefined)
}

pub fn update_after_event<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    *activation.context.needs_render = true;

    Ok(Value::Undefined)
}

pub fn escape<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let s = if let Some(val) = args.get(0) {
        val.coerce_to_string(activation)?
    } else {
        return Ok(Value::Undefined);
    };

    let mut buffer = Vec::<u8>::new();
    // TODO: unpaired surrogates will be lost; this is incorrect:
    // - `\u{DC00}` should become "%ED%B0%80";
    // - `\u{DFFF}` should become "%ED%BF%BF".
    for c in s.to_utf8_lossy().bytes() {
        match c {
            // ECMA-262 violation: @*_+-./ are not unescaped chars.
            b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' => {
                buffer.push(c);
            }
            // ECMA-262 violation: Avm1 does not support unicode escapes.
            _ => {
                const DIGITS: &[u8; 16] = b"0123456789ABCDEF";
                buffer.extend([b'%', DIGITS[(c / 16) as usize], DIGITS[(c % 16) as usize]]);
            }
        };
    }
    Ok(AvmString::new(activation.context.gc_context, WString::from_buf(buffer)).into())
}

pub fn unescape<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let s = if let Some(val) = args.get(0) {
        val.coerce_to_string(activation)?
    } else {
        return Ok(Value::Undefined);
    };

    let s = s.to_utf8_lossy();
    let mut out_bytes = Vec::<u8>::with_capacity(s.len());

    let mut remain = 0;
    let mut hex_chars = Vec::<u8>::with_capacity(2);

    // TODO: unpaired surrogates will be lost; this is incorrect:
    // - "%ED%B0%80" should become `\u{DC00}`;
    // - "%ED%BF%BF" should become `\u{DFFF}`.
    for c in s.bytes() {
        match c {
            b'%' => {
                remain = 2;
            }
            b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' if remain > 0 => {
                remain -= 1;
                hex_chars.push(c);

                if remain == 0 {
                    if let Some(b) = str::from_utf8(&hex_chars)
                        .ok()
                        .and_then(|s| u8::from_str_radix(s, 16).ok())
                    {
                        out_bytes.push(b);
                    }
                    hex_chars.clear();
                }
            }
            _ if remain > 0 => {
                remain = 0;
                hex_chars.clear();
            }
            b'+' => {
                out_bytes.push(b' ');
            }
            _ => {
                out_bytes.push(c);
            }
        }
    }
    Ok(AvmString::new_utf8(
        activation.context.gc_context,
        String::from_utf8_lossy(&out_bytes),
    )
    .into())
}

/// This structure represents all system builtins that are used regardless of
/// whatever the hell happens to `_global`. These are, of course,
/// user-modifiable.
#[derive(Collect, Clone)]
#[collect(no_drop)]
pub struct SystemPrototypes<'gc> {
    pub button: Object<'gc>,
    pub object: Object<'gc>,
    pub object_constructor: Object<'gc>,
    pub function: Object<'gc>,
    pub movie_clip: Object<'gc>,
    pub sound: Object<'gc>,
    pub text_field: Object<'gc>,
    pub text_format: Object<'gc>,
    pub array: Object<'gc>,
    pub array_constructor: Object<'gc>,
    pub xml_node_constructor: Object<'gc>,
    pub xml_constructor: Object<'gc>,
    pub string: Object<'gc>,
    pub number: Object<'gc>,
    pub boolean: Object<'gc>,
    pub matrix: Object<'gc>,
    pub matrix_constructor: Object<'gc>,
    pub point: Object<'gc>,
    pub point_constructor: Object<'gc>,
    pub rectangle: Object<'gc>,
    pub rectangle_constructor: Object<'gc>,
    pub transform_constructor: Object<'gc>,
    pub shared_object_constructor: Object<'gc>,
    pub color_transform: Object<'gc>,
    pub color_transform_constructor: Object<'gc>,
    pub context_menu: Object<'gc>,
    pub context_menu_constructor: Object<'gc>,
    pub context_menu_item: Object<'gc>,
    pub context_menu_item_constructor: Object<'gc>,
    pub date_constructor: Object<'gc>,
    pub bitmap_data: Object<'gc>,
    pub video: Object<'gc>,
    pub video_constructor: Object<'gc>,
    pub blur_filter: Object<'gc>,
    pub bevel_filter: Object<'gc>,
    pub glow_filter: Object<'gc>,
    pub drop_shadow_filter: Object<'gc>,
    pub color_matrix_filter: Object<'gc>,
    pub displacement_map_filter: Object<'gc>,
    pub convolution_filter: Object<'gc>,
    pub gradient_bevel_filter: Object<'gc>,
    pub gradient_glow_filter: Object<'gc>,
}

/// Initialize default global scope and builtins for an AVM1 instance.
pub fn create_globals<'gc>(
    context: &mut GcContext<'_, 'gc>,
) -> (
    SystemPrototypes<'gc>,
    Object<'gc>,
    as_broadcaster::BroadcasterFunctions<'gc>,
) {
    let gc_context = context.gc_context;

    let object_proto = ScriptObject::new(gc_context, None).into();
    let function_proto = function::create_proto(context, object_proto);

    object::fill_proto(context, object_proto, function_proto);

    let button_proto = button::create_proto(context, object_proto, function_proto);

    let movie_clip_proto = movie_clip::create_proto(context, object_proto, function_proto);

    let sound_proto = sound::create_proto(context, object_proto, function_proto);

    let style_sheet_proto = style_sheet::create_proto(context, object_proto, function_proto);
    let text_field_proto = text_field::create_proto(context, object_proto, function_proto);
    let text_format_proto = text_format::create_proto(context, object_proto, function_proto);

    let array_proto = array::create_proto(context, object_proto, function_proto);

    let color_proto = color::create_proto(context, object_proto, function_proto);

    let error_proto = error::create_proto(context, object_proto, function_proto);

    let xmlnode_proto = xml_node::create_proto(context, object_proto, function_proto);

    let string_proto = string::create_proto(context, object_proto, function_proto);
    let number_proto = number::create_proto(context, object_proto, function_proto);
    let boolean_proto = boolean::create_proto(context, object_proto, function_proto);
    let load_vars_proto = load_vars::create_proto(context, object_proto, function_proto);
    let local_connection_proto =
        local_connection::create_proto(context, object_proto, function_proto);
    let matrix_proto = matrix::create_proto(context, object_proto, function_proto);
    let point_proto = point::create_proto(context, object_proto, function_proto);
    let rectangle_proto = rectangle::create_proto(context, object_proto, function_proto);
    let color_transform_proto =
        color_transform::create_proto(context, object_proto, function_proto);
    let external_interface_proto = external_interface::create_proto(context, object_proto);
    let selection_proto = selection::create_proto(context, object_proto);

    let (broadcaster_functions, as_broadcaster) =
        as_broadcaster::create(context, object_proto, function_proto);

    let movie_clip_loader_proto = movie_clip_loader::create_proto(
        context,
        object_proto,
        function_proto,
        array_proto,
        broadcaster_functions,
    );

    let movie_clip_loader = FunctionObject::constructor(
        gc_context,
        Executable::Native(movie_clip_loader::constructor),
        constructor_to_fn!(movie_clip_loader::constructor),
        function_proto,
        movie_clip_loader_proto,
    );

    let video_proto = video::create_proto(context, object_proto, function_proto);
    let netstream_proto = netstream::create_proto(context, object_proto, function_proto);
    let netconnection_proto = netconnection::create_proto(context, object_proto, function_proto);
    let xml_socket_proto = xml_socket::create_proto(context, object_proto, function_proto);

    //TODO: These need to be constructors and should also set `.prototype` on each one
    let object = object::create_object_object(context, object_proto, function_proto);

    let context_menu_proto = context_menu::create_proto(context, object_proto, function_proto);
    let context_menu_item_proto =
        context_menu_item::create_proto(context, object_proto, function_proto);

    let button = FunctionObject::constructor(
        gc_context,
        Executable::Native(button::constructor),
        constructor_to_fn!(button::constructor),
        function_proto,
        button_proto,
    );
    let color = FunctionObject::constructor(
        gc_context,
        Executable::Native(color::constructor),
        constructor_to_fn!(color::constructor),
        function_proto,
        color_proto,
    );
    let error = FunctionObject::constructor(
        gc_context,
        Executable::Native(error::constructor),
        constructor_to_fn!(error::constructor),
        function_proto,
        error_proto,
    );
    let function = FunctionObject::constructor(
        gc_context,
        Executable::Native(function::constructor),
        Executable::Native(function::function),
        function_proto,
        function_proto,
    );
    let load_vars = FunctionObject::constructor(
        gc_context,
        Executable::Native(load_vars::constructor),
        constructor_to_fn!(load_vars::constructor),
        function_proto,
        load_vars_proto,
    );
    let local_connection = FunctionObject::constructor(
        gc_context,
        Executable::Native(local_connection::constructor),
        constructor_to_fn!(local_connection::constructor),
        function_proto,
        local_connection_proto,
    );
    let movie_clip = FunctionObject::constructor(
        gc_context,
        Executable::Native(movie_clip::constructor),
        constructor_to_fn!(movie_clip::constructor),
        function_proto,
        movie_clip_proto,
    );

    let sound = FunctionObject::constructor(
        gc_context,
        Executable::Native(sound::constructor),
        constructor_to_fn!(sound::constructor),
        function_proto,
        sound_proto,
    );
    let style_sheet = FunctionObject::constructor(
        gc_context,
        Executable::Native(style_sheet::constructor),
        constructor_to_fn!(style_sheet::constructor),
        function_proto,
        style_sheet_proto,
    );
    let text_field = FunctionObject::constructor(
        gc_context,
        Executable::Native(text_field::constructor),
        constructor_to_fn!(text_field::constructor),
        function_proto,
        text_field_proto,
    );
    let text_format = FunctionObject::constructor(
        gc_context,
        Executable::Native(text_format::constructor),
        constructor_to_fn!(text_format::constructor),
        function_proto,
        text_format_proto,
    );
    let array = array::create_array_object(context, array_proto, function_proto);
    let xmlnode = FunctionObject::constructor(
        gc_context,
        Executable::Native(xml_node::constructor),
        constructor_to_fn!(xml_node::constructor),
        function_proto,
        xmlnode_proto,
    );
    let xml = xml::create_constructor(context, xmlnode_proto, function_proto);
    let string = string::create_string_object(context, string_proto, function_proto);
    let number = number::create_number_object(context, number_proto, function_proto);
    let boolean = boolean::create_boolean_object(context, boolean_proto, function_proto);
    let date = date::create_constructor(context, object_proto, function_proto);
    let netstream = netstream::create_class(context, netstream_proto, function_proto);
    let netconnection = netconnection::create_class(context, netconnection_proto, function_proto);
    let xml_socket = xml_socket::create_class(context, xml_socket_proto, function_proto);

    let flash = ScriptObject::new(gc_context, Some(object_proto));

    let geom = ScriptObject::new(gc_context, Some(object_proto));
    let filters = ScriptObject::new(gc_context, Some(object_proto));
    let display = ScriptObject::new(gc_context, Some(object_proto));
    let net = ScriptObject::new(gc_context, Some(object_proto));

    let matrix = matrix::create_matrix_object(context, matrix_proto, function_proto);
    let point = point::create_point_object(context, point_proto, function_proto);
    let rectangle = rectangle::create_rectangle_object(context, rectangle_proto, function_proto);
    let color_transform = FunctionObject::constructor(
        gc_context,
        Executable::Native(color_transform::constructor),
        constructor_to_fn!(color_transform::constructor),
        function_proto,
        color_transform_proto,
    );
    let transform = transform::create_constructor(context, object_proto, function_proto);
    let video = FunctionObject::constructor(
        gc_context,
        Executable::Native(video::constructor),
        constructor_to_fn!(video::constructor),
        function_proto,
        video_proto,
    );

    flash.define_value(gc_context, "geom", geom.into(), Attribute::empty());
    flash.define_value(gc_context, "filters", filters.into(), Attribute::empty());
    flash.define_value(gc_context, "display", display.into(), Attribute::empty());
    geom.define_value(gc_context, "Matrix", matrix.into(), Attribute::empty());
    geom.define_value(gc_context, "Point", point.into(), Attribute::empty());
    geom.define_value(
        gc_context,
        "Rectangle",
        rectangle.into(),
        Attribute::empty(),
    );
    geom.define_value(
        gc_context,
        "ColorTransform",
        color_transform.into(),
        Attribute::empty(),
    );
    geom.define_value(
        gc_context,
        "Transform",
        transform.into(),
        Attribute::empty(),
    );

    let bitmap_filter_proto = bitmap_filter::create_proto(context, object_proto, function_proto);
    let bitmap_filter = FunctionObject::constructor(
        gc_context,
        Executable::Native(bitmap_filter::constructor),
        constructor_to_fn!(bitmap_filter::constructor),
        function_proto,
        bitmap_filter_proto,
    );

    let blur_filter = blur_filter::create_proto(context, bitmap_filter_proto, function_proto);
    let blur_filter_constructor =
        blur_filter::create_constructor(context, blur_filter, function_proto);

    let bevel_filter = bevel_filter::create_proto(context, bitmap_filter_proto, function_proto);
    let bevel_filter_constructor =
        bevel_filter::create_constructor(context, bevel_filter, function_proto);

    let glow_filter = glow_filter::create_proto(context, bitmap_filter_proto, function_proto);
    let glow_filter_constructor =
        glow_filter::create_constructor(context, glow_filter, function_proto);

    let drop_shadow_filter =
        drop_shadow_filter::create_proto(context, bitmap_filter_proto, function_proto);
    let drop_shadow_filter_constructor =
        drop_shadow_filter::create_constructor(context, drop_shadow_filter, function_proto);

    let color_matrix_filter =
        color_matrix_filter::create_proto(context, bitmap_filter_proto, function_proto);
    let color_matrix_filter_constructor =
        color_matrix_filter::create_constructor(context, color_matrix_filter, function_proto);

    let displacement_map_filter =
        displacement_map_filter::create_proto(context, bitmap_filter_proto, function_proto);
    let displacement_map_filter_constructor = displacement_map_filter::create_constructor(
        context,
        displacement_map_filter,
        function_proto,
    );

    let convolution_filter =
        convolution_filter::create_proto(context, bitmap_filter_proto, function_proto);
    let convolution_filter_constructor =
        convolution_filter::create_constructor(context, convolution_filter, function_proto);

    let gradient_bevel_filter =
        gradient_filter::create_bevel_proto(context, bitmap_filter_proto, function_proto);
    let gradient_bevel_filter_constructor =
        gradient_filter::create_bevel_constructor(context, gradient_bevel_filter, function_proto);

    let gradient_glow_filter =
        gradient_filter::create_glow_proto(context, bitmap_filter_proto, function_proto);
    let gradient_glow_filter_constructor =
        gradient_filter::create_glow_constructor(context, gradient_glow_filter, function_proto);

    filters.define_value(
        gc_context,
        "BitmapFilter",
        bitmap_filter.into(),
        Attribute::empty(),
    );
    filters.define_value(
        gc_context,
        "BlurFilter",
        blur_filter_constructor.into(),
        Attribute::empty(),
    );
    filters.define_value(
        gc_context,
        "BevelFilter",
        bevel_filter_constructor.into(),
        Attribute::empty(),
    );
    filters.define_value(
        gc_context,
        "GlowFilter",
        glow_filter_constructor.into(),
        Attribute::empty(),
    );
    filters.define_value(
        gc_context,
        "DropShadowFilter",
        drop_shadow_filter_constructor.into(),
        Attribute::empty(),
    );
    filters.define_value(
        gc_context,
        "ColorMatrixFilter",
        color_matrix_filter_constructor.into(),
        Attribute::empty(),
    );
    filters.define_value(
        gc_context,
        "DisplacementMapFilter",
        displacement_map_filter_constructor.into(),
        Attribute::empty(),
    );
    filters.define_value(
        gc_context,
        "ConvolutionFilter",
        convolution_filter_constructor.into(),
        Attribute::empty(),
    );

    filters.define_value(
        gc_context,
        "GradientBevelFilter",
        gradient_bevel_filter_constructor.into(),
        Attribute::empty(),
    );
    filters.define_value(
        gc_context,
        "GradientGlowFilter",
        gradient_glow_filter_constructor.into(),
        Attribute::empty(),
    );

    let bitmap_data_proto = ScriptObject::new(context.gc_context, Some(object_proto));
    let bitmap_data = bitmap_data::create_constructor(context, bitmap_data_proto, function_proto);

    display.define_value(
        gc_context,
        "BitmapData",
        bitmap_data.into(),
        Attribute::empty(),
    );

    let external = ScriptObject::new(gc_context, Some(object_proto));
    let external_interface = external_interface::create_external_interface_object(
        context,
        external_interface_proto,
        function_proto,
    );

    flash.define_value(gc_context, "external", external.into(), Attribute::empty());
    external.define_value(
        gc_context,
        "ExternalInterface",
        external_interface.into(),
        Attribute::empty(),
    );

    flash.define_value(gc_context, "net", net.into(), Attribute::empty());

    let file_reference_obj = file_reference::create_constructor(
        context,
        object_proto,
        function_proto,
        array_proto,
        broadcaster_functions,
    );

    net.define_value(
        gc_context,
        "FileReference",
        file_reference_obj.into(),
        Attribute::empty(),
    );

    let globals = ScriptObject::new(gc_context, None);
    globals.define_value(
        gc_context,
        "AsBroadcaster",
        as_broadcaster.into(),
        Attribute::DONT_ENUM,
    );
    globals.define_value(gc_context, "flash", flash.into(), Attribute::DONT_ENUM);
    globals.define_value(gc_context, "Array", array.into(), Attribute::DONT_ENUM);
    globals.define_value(gc_context, "Button", button.into(), Attribute::DONT_ENUM);
    globals.define_value(gc_context, "Color", color.into(), Attribute::DONT_ENUM);
    globals.define_value(gc_context, "Error", error.into(), Attribute::DONT_ENUM);
    globals.define_value(gc_context, "Object", object.into(), Attribute::DONT_ENUM);
    globals.define_value(
        gc_context,
        "Function",
        function.into(),
        Attribute::DONT_ENUM,
    );
    globals.define_value(
        gc_context,
        "LoadVars",
        load_vars.into(),
        Attribute::DONT_ENUM,
    );
    globals.define_value(
        gc_context,
        "LocalConnection",
        local_connection.into(),
        Attribute::DONT_ENUM,
    );
    globals.define_value(
        gc_context,
        "MovieClip",
        movie_clip.into(),
        Attribute::DONT_ENUM,
    );
    globals.define_value(
        gc_context,
        "MovieClipLoader",
        movie_clip_loader.into(),
        Attribute::DONT_ENUM,
    );
    globals.define_value(gc_context, "Sound", sound.into(), Attribute::DONT_ENUM);
    globals.define_value(
        gc_context,
        "TextField",
        text_field.into(),
        Attribute::DONT_ENUM,
    );
    text_field.define_value(
        gc_context,
        "StyleSheet",
        style_sheet.into(),
        Attribute::DONT_ENUM | Attribute::VERSION_7,
    );
    globals.define_value(
        gc_context,
        "TextFormat",
        text_format.into(),
        Attribute::DONT_ENUM,
    );
    globals.define_value(gc_context, "XMLNode", xmlnode.into(), Attribute::DONT_ENUM);
    globals.define_value(gc_context, "XML", xml.into(), Attribute::DONT_ENUM);
    globals.define_value(gc_context, "String", string.into(), Attribute::DONT_ENUM);
    globals.define_value(gc_context, "Number", number.into(), Attribute::DONT_ENUM);
    globals.define_value(gc_context, "Boolean", boolean.into(), Attribute::DONT_ENUM);
    globals.define_value(gc_context, "Date", date.into(), Attribute::DONT_ENUM);

    let shared_object = shared_object::create_constructor(context, object_proto, function_proto);
    globals.define_value(
        gc_context,
        "SharedObject",
        shared_object.into(),
        Attribute::DONT_ENUM,
    );

    let context_menu = FunctionObject::constructor(
        gc_context,
        Executable::Native(context_menu::constructor),
        constructor_to_fn!(context_menu::constructor),
        function_proto,
        context_menu_proto,
    );
    globals.define_value(
        gc_context,
        "ContextMenu",
        context_menu.into(),
        Attribute::DONT_ENUM,
    );

    let selection = selection::create_selection_object(
        context,
        selection_proto,
        function_proto,
        broadcaster_functions,
        array_proto,
    );
    globals.define_value(
        gc_context,
        "Selection",
        selection.into(),
        Attribute::DONT_ENUM,
    );

    let context_menu_item = FunctionObject::constructor(
        gc_context,
        Executable::Native(context_menu_item::constructor),
        constructor_to_fn!(context_menu_item::constructor),
        function_proto,
        context_menu_item_proto,
    );
    globals.define_value(
        gc_context,
        "ContextMenuItem",
        context_menu_item.into(),
        Attribute::DONT_ENUM,
    );

    let system_security = system_security::create(context, object_proto, function_proto);
    let system_capabilities = system_capabilities::create(context, object_proto, function_proto);
    let system_ime = system_ime::create(
        context,
        object_proto,
        function_proto,
        broadcaster_functions,
        array_proto,
    );

    let system = system::create(
        context,
        object_proto,
        function_proto,
        system_security,
        system_capabilities,
        system_ime,
    );
    globals.define_value(gc_context, "System", system.into(), Attribute::DONT_ENUM);

    globals.define_value(
        gc_context,
        "Math",
        Value::Object(math::create(context, object_proto, function_proto)),
        Attribute::DONT_ENUM,
    );
    globals.define_value(
        gc_context,
        "Mouse",
        Value::Object(mouse::create_mouse_object(
            context,
            object_proto,
            function_proto,
            broadcaster_functions,
            array_proto,
        )),
        Attribute::DONT_ENUM,
    );
    globals.define_value(
        gc_context,
        "Key",
        Value::Object(key::create_key_object(
            context,
            object_proto,
            function_proto,
            broadcaster_functions,
            array_proto,
        )),
        Attribute::DONT_ENUM,
    );
    globals.define_value(
        gc_context,
        "Stage",
        Value::Object(stage::create_stage_object(
            context,
            object_proto,
            array_proto,
            function_proto,
            broadcaster_functions,
        )),
        Attribute::DONT_ENUM,
    );
    globals.define_value(
        gc_context,
        "Accessibility",
        Value::Object(accessibility::create_accessibility_object(
            context,
            object_proto,
            function_proto,
        )),
        Attribute::DONT_ENUM,
    );
    globals.define_value(
        gc_context,
        "NetStream",
        netstream.into(),
        Attribute::DONT_ENUM,
    );
    globals.define_value(
        gc_context,
        "NetConnection",
        netconnection.into(),
        Attribute::DONT_ENUM,
    );
    globals.define_value(
        gc_context,
        "XMLSocket",
        xml_socket.into(),
        Attribute::DONT_ENUM,
    );

    define_properties_on(GLOBAL_DECLS, context, globals, function_proto);

    (
        SystemPrototypes {
            button: button_proto,
            object: object_proto,
            object_constructor: object,
            function: function_proto,
            movie_clip: movie_clip_proto,
            sound: sound_proto,
            text_field: text_field_proto,
            text_format: text_format_proto,
            array: array_proto,
            array_constructor: array,
            xml_node_constructor: xmlnode,
            xml_constructor: xml,
            string: string_proto,
            number: number_proto,
            boolean: boolean_proto,
            matrix: matrix_proto,
            matrix_constructor: matrix,
            point: point_proto,
            point_constructor: point,
            rectangle: rectangle_proto,
            rectangle_constructor: rectangle,
            transform_constructor: transform,
            shared_object_constructor: shared_object,
            color_transform: color_transform_proto,
            color_transform_constructor: color_transform,
            context_menu: context_menu_proto,
            context_menu_constructor: context_menu,
            context_menu_item: context_menu_item_proto,
            context_menu_item_constructor: context_menu_item,
            date_constructor: date,
            bitmap_data: bitmap_data_proto.into(),
            video: video_proto,
            video_constructor: video,
            blur_filter,
            bevel_filter,
            glow_filter,
            drop_shadow_filter,
            color_matrix_filter,
            displacement_map_filter,
            convolution_filter,
            gradient_bevel_filter,
            gradient_glow_filter,
        },
        globals.into(),
        broadcaster_functions,
    )
}

/// Depths used/returned by ActionScript are offset by this amount from depths used inside the SWF/by the VM.
/// The depth of objects placed on the timeline in the Flash IDE start from 0 in the SWF,
/// but are negative when queried from MovieClip.getDepth().
/// Add this to convert from AS -> SWF depth.
pub const AVM_DEPTH_BIAS: i32 = 16384;

/// The maximum depth that the AVM will allow you to swap or attach clips to.
/// What is the derivation of this number...?
const AVM_MAX_DEPTH: i32 = 2_130_706_428;

/// The maximum depth that the AVM will allow you to remove clips from.
/// What is the derivation of this number...?
const AVM_MAX_REMOVE_DEPTH: i32 = 2_130_706_416;

fn get_depth<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(display_object) = this.as_display_object() {
        if activation.swf_version() >= 6 {
            let depth = display_object.depth().wrapping_sub(AVM_DEPTH_BIAS);
            return Ok(depth.into());
        }
    }
    Ok(Value::Undefined)
}

pub fn remove_display_object<'gc>(this: DisplayObject<'gc>, activation: &mut Activation<'_, 'gc>) {
    let depth = this.depth().wrapping_sub(0);
    // Can only remove positive depths (when offset by the AVM depth bias).
    // Generally this prevents you from removing non-dynamically created clips,
    // although you can get around it with swapDepths.
    // TODO: Figure out the derivation of this range.
    if depth >= AVM_DEPTH_BIAS && depth < AVM_MAX_REMOVE_DEPTH && !this.avm1_removed() {
        // Need a parent to remove from.
        if let Some(mut parent) = this.avm1_parent().and_then(|o| o.as_movie_clip()) {
            parent.remove_child(activation.context, this);
        }
    }
}

#[cfg(test)]
#[allow(clippy::unreadable_literal)]
mod tests {
    use super::*;

    fn setup<'gc>(activation: &mut Activation<'_, 'gc>) -> Object<'gc> {
        create_globals(&mut activation.context.borrow_gc()).1
    }

    test_method!(boolean_function, "Boolean", setup,
        [19] => {
            [true] => true,
            [false] => false,
            [10.0] => true,
            [-10.0] => true,
            [0.0] => false,
            [f64::INFINITY] => true,
            [f64::NAN] => false,
            [""] => false,
            ["Hello"] => true,
            [" "] => true,
            ["0"] => true,
            ["1"] => true,
            [Value::Undefined] => false,
            [Value::Null] => false,
            [] => Value::Undefined
        },
        [6] => {
            [true] => true,
            [false] => false,
            [10.0] => true,
            [-10.0] => true,
            [0.0] => false,
            [f64::INFINITY] => true,
            [f64::NAN] => false,
            [""] => false,
            ["Hello"] => false,
            [" "] => false,
            ["0"] => false,
            ["1"] => true,
            [Value::Undefined] => false,
            [Value::Null] => false,
            [] => Value::Undefined
        }
    );

    test_method!(is_nan_function, "isNaN", setup,
        [19] => {
            [true] => false,
            [false] => false,
            [10.0] => false,
            [-10.0] => false,
            [0.0] => false,
            [f64::INFINITY] => false,
            [f64::NAN] => true,
            [""] => true,
            ["Hello"] => true,
            [" "] => true,
            ["  5  "] => true,
            ["0"] => false,
            ["1"] => false,
            ["Infinity"] => true,
            ["100a"] => true,
            ["0x10"] => false,
            ["0xhello"] => true,
            ["0x1999999981ffffff"] => false,
            ["0xUIXUIDFKHJDF012345678"] => true,
            ["123e-1"] => false,
            [] => true
        }
    );

    test_method!(is_finite, "isFinite", setup,
        [19] => {
            [true] => true,
            [false] => true,
            [10.0] => true,
            [-10.0] => true,
            [0.0] => true,
            [f64::INFINITY] => false,
            [f64::NEG_INFINITY] => false,
            [f64::NAN] => false,
            [""] => false,
            ["Hello"] => false,
            [" "] => false,
            ["  5  "] => false,
            ["0"] => true,
            ["1"] => true,
            ["Infinity"] => false,
            ["-Infinity"] => false,
            ["100a"] => false,
            ["0x10"] => true,
            ["0xhello"] => false,
            ["0x1999999981ffffff"] => true,
            ["0xUIXUIDFKHJDF012345678"] => false,
            ["123e-1"] => true,
            [Value::Undefined] => false,
            [Value::Null] => false,
            [] => false
        }
    );

    test_method!(number_function, "Number", setup,
        [5, 6] => {
            [true] => 1.0,
            [false] => 0.0,
            [10.0] => 10.0,
            [-10.0] => -10.0,
            ["true"] => f64::NAN,
            ["false"] => f64::NAN,
            [1.0] => 1.0,
            [0.0] => 0.0,
            [0.000] => 0.0,
            ["0.000"] => 0.0,
            ["True"] => f64::NAN,
            ["False"] => f64::NAN,
            [f64::NAN] => f64::NAN,
            [f64::INFINITY] => f64::INFINITY,
            [f64::NEG_INFINITY] => f64::NEG_INFINITY,
            [" 12"] => 12.0,
            [" \t\r\n12"] => 12.0,
            ["\u{A0}12"] => f64::NAN,
            [" 0x12"] => f64::NAN,
            ["01.2"] => 1.2,
            [""] => f64::NAN,
            ["Hello"] => f64::NAN,
            [" "] => f64::NAN,
            ["  5  "] => f64::NAN,
            ["0"] => 0.0,
            ["1"] => 1.0,
            ["Infinity"] => f64::NAN,
            ["-Infinity"] => f64::NAN,
            ["inf"]  => f64::NAN,
            ["-inf"]  => f64::NAN,
            ["100a"] => f64::NAN,
            ["0xhello"] => f64::NAN,
            ["123e-1"] => 12.3,
            ["0xUIXUIDFKHJDF012345678"] => f64::NAN,
            [] => 0.0
        },
        [5] => {
            ["0x12"] => f64::NAN,
            ["0x10"] => f64::NAN,
            ["0x1999999981ffffff"] => f64::NAN,
            ["010"] => 10,
            ["-010"] => -10,
            ["+010"] => 10,
            [" 010"] => 10,
            [" -010"] => -10,
            [" +010"] => 10,
            ["037777777777"] => 37777777777.0,
            ["-037777777777"] => -37777777777.0
        },
        [6, 7] => {
            ["0x12"] => 18.0,
            ["0x10"] => 16.0,
            ["-0x10"] => f64::NAN,
            ["0x1999999981ffffff"] => -2113929217.0,
            ["010"] => 8,
            ["-010"] => -8,
            ["+010"] => 8,
            [" 010"] => 10,
            [" -010"] => -10,
            [" +010"] => 10,
            ["037777777777"] => -1,
            ["-037777777777"] => 1
        },
        [5, 6] => {
            [Value::Undefined] => 0.0,
            [Value::Null] => 0.0
        },
        [7] => {
            [Value::Undefined] => f64::NAN,
            [Value::Null] => f64::NAN
        }
    );
}
