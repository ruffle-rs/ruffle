use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::DeclContext;
use crate::avm1::{Object, Value};
use crate::display_object::{DisplayObject, TDisplayObject, TDisplayObjectContainer};
use crate::string::{AvmString, StringContext, WStr, WString};
use gc_arena::Collect;
use std::str;

mod accessibility;
pub(super) mod array;
pub(crate) mod as_broadcaster;
mod asnative;
pub(crate) mod bevel_filter;
mod bitmap_data;
mod bitmap_filter;
pub(crate) mod blur_filter;
pub(crate) mod boolean;
pub(crate) mod button;
mod camera;
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
mod file_reference_list;
mod function;
pub(crate) mod glow_filter;
pub(crate) mod gradient_filter;
mod key;
mod load_vars;
pub(crate) mod local_connection;
mod math;
mod matrix;
mod microphone;
pub(crate) mod mouse;
pub(crate) mod movie_clip;
mod movie_clip_loader;
pub(crate) mod netconnection;
pub(crate) mod netstream;
pub(crate) mod number;
mod object;
mod point;
mod print_job;
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
mod system_product;
pub(crate) mod system_security;
pub(crate) mod text_field;
mod text_format;
mod text_renderer;
mod text_snapshot;
pub(crate) mod transform;
mod video;
pub(crate) mod xml;
mod xml_node;
pub(crate) mod xml_socket;

mod method {
    pub const ESCAPE: u16 = 0;
    pub const UNESCAPE: u16 = 1;
    pub const PARSE_INT: u16 = 2;
    pub const PARSE_FLOAT: u16 = 3;
    pub const TRACE: u16 = 4;
}

fn method<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
    index: u16,
) -> Result<Value<'gc>, Error<'gc>> {
    use method::*;

    match index {
        ESCAPE => escape(activation, args),
        UNESCAPE => unescape(activation, args),
        PARSE_INT => parse_int(activation, args),
        PARSE_FLOAT => parse_float(activation, args),
        TRACE => trace(activation, args),
        _ => Ok(Value::Undefined),
    }
}

pub fn trace<'gc>(
    activation: &mut Activation<'_, 'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Unlike `Action::Trace`, `_global.trace` always coerces
    // undefined to "" in SWF6 and below. It also doesn't log
    // anything outside of the Flash editor's trace window.
    // Ruffle does not respect the latter behavior, and will treat it the same as an `Action::Trace`.
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
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // ECMA-262 violation: parseInt() == undefined // not NaN
    let Some(string) = args.get(0) else {
        return Ok(Value::Undefined);
    };

    parse_int_internal(activation, string, args.get(1))
}

pub fn parse_int_internal<'gc>(
    activation: &mut Activation<'_, 'gc>,
    string: &Value<'gc>,
    radix: Option<&Value<'gc>>,
) -> Result<Value<'gc>, Error<'gc>> {
    let radix: Option<i32> = radix.map(|x| x.coerce_to_i32(activation)).transpose()?;
    let radix = match radix {
        Some(r @ 2..=36) => Some(r as u32),
        Some(_) => return Ok(f64::NAN.into()),
        None => None,
    };

    let string = string.coerce_to_string(activation)?;
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
        Some(Value::Object(o)) if o.as_function().is_some() => (
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
    Ok(AvmString::new(activation.gc(), WString::from_buf(buffer)).into())
}

pub fn unescape<'gc>(
    activation: &mut Activation<'_, 'gc>,
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
    Ok(AvmString::new_utf8(activation.gc(), String::from_utf8_lossy(&out_bytes)).into())
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
    pub text_field: Object<'gc>,
    pub text_format: Object<'gc>,
    pub array: Object<'gc>,
    pub array_constructor: Object<'gc>,
    pub xml_node_constructor: Object<'gc>,
    pub xml_constructor: Object<'gc>,
    pub matrix_constructor: Object<'gc>,
    pub point_constructor: Object<'gc>,
    pub rectangle: Object<'gc>,
    pub rectangle_constructor: Object<'gc>,
    pub transform_constructor: Object<'gc>,
    pub shared_object_constructor: Object<'gc>,
    pub color_transform_constructor: Object<'gc>,
    pub context_menu_constructor: Object<'gc>,
    pub context_menu_item_constructor: Object<'gc>,
    pub date_constructor: Object<'gc>,
    pub bitmap_data: Object<'gc>,
    pub video: Object<'gc>,
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
    context: &mut StringContext<'gc>,
) -> (
    SystemPrototypes<'gc>,
    Object<'gc>,
    as_broadcaster::BroadcasterFunctions<'gc>,
) {
    let context = {
        let object_proto = Object::new_without_proto(context.gc());
        &mut DeclContext {
            object_proto,
            fn_proto: Object::new(context, Some(object_proto)),
            strings: context,
        }
    };

    let object = object::create_class(context);
    let function = function::create_class(context);
    let (broadcaster_fns, as_broadcaster) = as_broadcaster::create_class(context, object.proto);

    let flash = Object::new(context.strings, Some(object.proto));
    let external = Object::new(context.strings, Some(object.proto));
    let geom = Object::new(context.strings, Some(object.proto));
    let filters = Object::new(context.strings, Some(object.proto));
    let display = Object::new(context.strings, Some(object.proto));
    let net = Object::new(context.strings, Some(object.proto));
    let text = Object::new(context.strings, Some(object.proto));

    let button = button::create_class(context, object.proto);
    let movie_clip = movie_clip::create_class(context, object.proto);
    let sound = sound::create_class(context, object.proto);
    let style_sheet = style_sheet::create_class(context, object.proto);
    let text_field = text_field::create_class(context, object.proto);
    let text_format = text_format::create_class(context, object.proto);
    let array = array::create_class(context, object.proto);
    let color = color::create_class(context, object.proto);
    let error = error::create_class(context, object.proto);
    let xmlnode = xml_node::create_class(context, object.proto);
    let string = string::create_class(context, object.proto);
    let number = number::create_class(context, object.proto);
    let boolean = boolean::create_class(context, object.proto);
    let load_vars = load_vars::create_class(context, object.proto);
    let local_connection = local_connection::create_class(context, object.proto);
    let matrix = matrix::create_class(context, object.proto);
    let point = point::create_class(context, object.proto);
    let rectangle = rectangle::create_class(context, object.proto);
    let color_transform = color_transform::create_class(context, object.proto);
    let external_interface = external_interface::create_class(context, object.proto);
    let movie_clip_loader =
        movie_clip_loader::create_class(context, object.proto, broadcaster_fns, array.proto);
    let video = video::create_class(context, object.proto);
    let netstream = netstream::create_class(context, object.proto);
    let netconnection = netconnection::create_class(context, object.proto);
    let xml_socket = xml_socket::create_class(context, object.proto);
    let context_menu = context_menu::create_class(context, object.proto);
    let context_menu_item = context_menu_item::create_class(context, object.proto);
    let xml = xml::create_class(context, xmlnode.proto);
    let date = date::create_class(context, object.proto);
    let transform = transform::create_class(context, object.proto);
    let bitmap_filter = bitmap_filter::create_class(context, object.proto);
    let blur_filter = blur_filter::create_class(context, bitmap_filter.proto);
    let bevel_filter = bevel_filter::create_class(context, bitmap_filter.proto);
    let glow_filter = glow_filter::create_class(context, bitmap_filter.proto);
    let drop_shadow_filter = drop_shadow_filter::create_class(context, bitmap_filter.proto);
    let color_matrix_filter = color_matrix_filter::create_class(context, bitmap_filter.proto);
    let displacement_map_filter =
        displacement_map_filter::create_class(context, bitmap_filter.proto);
    let convolution_filter = convolution_filter::create_class(context, bitmap_filter.proto);
    let gradient_bevel_filter = gradient_filter::create_bevel_class(context, bitmap_filter.proto);
    let gradient_glow_filter = gradient_filter::create_glow_class(context, bitmap_filter.proto);
    let bitmap_data = bitmap_data::create_class(context, object.proto);
    let file_reference =
        file_reference::create_class(context, object.proto, broadcaster_fns, array.proto);
    let file_reference_list = file_reference_list::create_class(context, object.proto);
    let shared_object = shared_object::create_class(context, object.proto);
    let selection = selection::create(context, broadcaster_fns, array.proto);
    let camera = camera::create_class(context, object.proto);
    let microphone = microphone::create_class(context, object.proto);
    let print_job = print_job::create_class(context, object.proto);
    let text_snapshot = text_snapshot::create_class(context, object.proto);

    let system = system::create(context);
    let system_security = system_security::create(context);
    let system_capabilities = system_capabilities::create(context);
    let system_ime = system_ime::create(context, broadcaster_fns, array.proto);
    let system_product = system_product::create_class(context, object.proto);

    let math = math::create(context);
    let mouse = mouse::create(context, broadcaster_fns, array.proto);
    let key = key::create(context, broadcaster_fns, array.proto);
    let stage = stage::create(context, broadcaster_fns, array.proto);
    let accessibility = accessibility::create(context);

    let text_renderer = text_renderer::create_class(context, object.proto);

    // Top-level
    let globals = Object::new_without_proto(context.gc());
    let decls = declare_properties! {
        // ASnative doesn't seem to have an ASnative index (searched in `ASnative(0..10000, 0..10000)`).
        "ASnative" => method(asnative::asnative; DONT_ENUM);
        // TODO: ASconstructor
        "Object" => object(object.constr; DONT_ENUM);
        "Function" => object(function.constr; DONT_ENUM | VERSION_6);
        // TODO: enableDebugConsole - is this only present in the debugger version of FP?
        "NaN" => property(get_nan; DONT_ENUM);
        "Infinity" => property(get_infinity; DONT_ENUM);

        // Starting from here, FP defines these through its embedded `playerglobals.swf`
        "MovieClip" => object(movie_clip.constr; DONT_ENUM);
        "XMLSocket" => object(xml_socket.constr; DONT_ENUM);
        "AsBroadcaster" => object(as_broadcaster.constr; DONT_ENUM);
        "Color" => object(color.constr; DONT_ENUM);
        "NetConnection" => object(netconnection.constr; DONT_ENUM);
        "NetStream" => object(netstream.constr; DONT_ENUM);
        "Camera" => object(camera.constr; DONT_ENUM);
        "Microphone" => object(microphone.constr; DONT_ENUM);
        "SharedObject" => object(shared_object.constr; DONT_ENUM);
        "ContextMenuItem" => object(context_menu_item.constr; DONT_ENUM);
        "ContextMenu" => object(context_menu.constr; DONT_ENUM);
        "Error" => object(error.constr; DONT_ENUM);
        // TODO: AsSetupError
        // TODO: AssetCache
        // TODO: RemoteLSOUsage

        "ASSetPropFlags" => method(object::as_set_prop_flags; DONT_ENUM); // TODO: (1, 0)
        // TODO: ASSetNative - (4, 0)
        // TODO: ASSetAccessor - (4, 1)

        use fn method;
        "escape" => method(ESCAPE; DONT_ENUM);
        "unescape" => method(UNESCAPE; DONT_ENUM);
        "parseInt" => method(PARSE_INT; DONT_ENUM);
        "parseFloat" => method(PARSE_FLOAT; DONT_ENUM);
        "trace" => method(TRACE; DONT_ENUM);

        use default;
        "updateAfterEvent" => method(update_after_event; DONT_ENUM); // TODO: (9, 0)
        "isNaN" => method(is_nan; DONT_ENUM); // TODO: (200, 18)
        "isFinite" => method(is_finite; DONT_ENUM); // TODO: (200, 19)
        "setInterval" => method(set_interval; DONT_ENUM); // TODO: (250, 0)
        "clearTimeout" => method(clear_interval; DONT_ENUM); // TODO: (250, 1)
        // FIXME: this should the **same** function object as `clearTimeout`, not a copy
        "clearInterval" => method(clear_interval; DONT_ENUM); // TODO: (250, 1)
        "setTimeout" => method(set_timeout; DONT_ENUM); // TODO: (250, 2)
        // TODO: showRedrawRegions - (1021, 1)
        // TODO: addRequestHeader
        // TODO: clearRequestHeaders

        "Number" => object(number.constr; DONT_ENUM);
        "Boolean" => object(boolean.constr; DONT_ENUM);
        "Date" => object(date.constr; DONT_ENUM);
        "String" => object(string.constr; DONT_ENUM);
        "Array" => object(array.constr; DONT_ENUM);
        "Math" => object(math; DONT_ENUM);
        "Sound" => object(sound.constr; DONT_ENUM);
        "XMLNode" => object(xmlnode.constr; DONT_ENUM);
        "XML" => object(xml.constr; DONT_ENUM);
        "LoadVars" => object(load_vars.constr; DONT_ENUM);
        "Selection" => object(selection; DONT_ENUM);
        "Mouse" => object(mouse; DONT_ENUM);
        "Key" => object(key; DONT_ENUM);
        "Button" => object(button.constr; DONT_ENUM);
        "TextField" => object(text_field.constr; DONT_ENUM);
        "TextFormat" => object(text_format.constr; DONT_ENUM);
        "Stage" => object(stage; DONT_ENUM);
        "Video" => object(video.constr; DONT_ENUM);
        "Accessibility" => object(accessibility; DONT_ENUM);
        "System" => object(system; DONT_ENUM);
        "flash" => object(flash; DONT_ENUM | VERSION_8);
        "textRenderer" => object(text_renderer.constr);
        "LocalConnection" => object(local_connection.constr; DONT_ENUM);
        "MovieClipLoader" => object(movie_clip_loader.constr; DONT_ENUM);
        "PrintJob" => object(print_job.constr; DONT_ENUM);
        "TextSnapshot" => object(text_snapshot.constr; DONT_ENUM);
    };
    context.define_properties_on(globals, decls);

    // flash
    let decls = declare_properties! {
        "text" => object(text);
        "display" => object(display);
        "filters" => object(filters);
        "geom" => object(geom);
        "net" => object(net);
        "external" => object(external);
    };
    context.define_properties_on(flash, decls);

    // flash.display
    let decls = declare_properties! {
        "BitmapData" => object(bitmap_data.constr);
    };
    context.define_properties_on(display, decls);

    // flash.external
    let decls = declare_properties! {
        "ExternalInterface" => object(external_interface.constr);
    };
    context.define_properties_on(external, decls);

    // flash.filters
    let decls = declare_properties! {
        "BitmapFilter" => object(bitmap_filter.constr);
        "DropShadowFilter" => object(drop_shadow_filter.constr);
        "BlurFilter" => object(blur_filter.constr);
        "GlowFilter" => object(glow_filter.constr);
        "BevelFilter" => object(bevel_filter.constr);
        "GradientGlowFilter" => object(gradient_glow_filter.constr);
        "GradientBevelFilter" => object(gradient_bevel_filter.constr);
        "ConvolutionFilter" => object(convolution_filter.constr);
        "ColorMatrixFilter" => object(color_matrix_filter.constr);
        "DisplacementMapFilter" => object(displacement_map_filter.constr);
    };
    context.define_properties_on(filters, decls);

    // flash.geom
    let decls = declare_properties! {
        "Rectangle" => object(rectangle.constr);
        "Point" => object(point.constr);
        "Matrix" => object(matrix.constr);
        "ColorTransform" => object(color_transform.constr);
        "Transform" => object(transform.constr);
    };
    context.define_properties_on(geom, decls);

    // flash.net
    let decls = declare_properties! {
        "FileReference" => object(file_reference.constr);
        "FileReferenceList" => object(file_reference_list.constr);
    };
    context.define_properties_on(net, decls);

    // flash.text
    let decls = declare_properties! {
        "TextRenderer" => object(text_renderer.constr);
    };
    context.define_properties_on(text, decls);

    // System
    let decls = declare_properties! {
        "capabilities" => object(system_capabilities);
        "Product" => object(system_product.constr);
        "security" => object(system_security);
        "IME" => object(system_ime);
    };
    context.define_properties_on(system, decls);

    // TextField
    let decls = declare_properties! {
        "StyleSheet" => object(style_sheet.constr; DONT_ENUM | VERSION_7);
    };
    context.define_properties_on(text_field.constr, decls);

    (
        SystemPrototypes {
            button: button.proto,
            object: object.proto,
            object_constructor: object.constr,
            function: function.proto,
            movie_clip: movie_clip.proto,
            text_field: text_field.proto,
            text_format: text_format.proto,
            array: array.proto,
            array_constructor: array.constr,
            xml_node_constructor: xmlnode.constr,
            xml_constructor: xml.constr,
            matrix_constructor: matrix.constr,
            point_constructor: point.constr,
            rectangle: rectangle.proto,
            rectangle_constructor: rectangle.constr,
            transform_constructor: transform.constr,
            shared_object_constructor: shared_object.constr,
            color_transform_constructor: color_transform.constr,
            context_menu_constructor: context_menu.constr,
            context_menu_item_constructor: context_menu_item.constr,
            date_constructor: date.constr,
            bitmap_data: bitmap_data.proto,
            video: video.proto,
            blur_filter: blur_filter.proto,
            bevel_filter: bevel_filter.proto,
            glow_filter: glow_filter.proto,
            drop_shadow_filter: drop_shadow_filter.proto,
            color_matrix_filter: color_matrix_filter.proto,
            displacement_map_filter: displacement_map_filter.proto,
            convolution_filter: convolution_filter.proto,
            gradient_bevel_filter: gradient_bevel_filter.proto,
            gradient_glow_filter: gradient_glow_filter.proto,
        },
        globals,
        broadcaster_fns,
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
