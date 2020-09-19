use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::property::Attribute::*;
use crate::avm1::{Object, ScriptObject, TObject, Value};
use enumset::EnumSet;
use gc_arena::Collect;
use gc_arena::MutationContext;
use rand::Rng;
use std::f64;

mod array;
pub(crate) mod as_broadcaster;
mod bitmap_filter;
mod blur_filter;
pub(crate) mod boolean;
pub(crate) mod button;
mod color;
mod color_transform;
pub(crate) mod context_menu;
pub(crate) mod context_menu_item;
mod date;
pub(crate) mod display_object;
pub(crate) mod error;
mod external_interface;
mod function;
mod key;
mod load_vars;
mod math;
mod matrix;
pub(crate) mod mouse;
pub(crate) mod movie_clip;
mod movie_clip_loader;
pub(crate) mod number;
mod object;
mod point;
mod rectangle;
pub(crate) mod shared_object;
mod sound;
mod stage;
pub(crate) mod string;
pub(crate) mod system;
pub(crate) mod system_capabilities;
pub(crate) mod system_ime;
pub(crate) mod system_security;
pub(crate) mod text_field;
mod text_format;
mod transform;
mod xml;

pub fn random<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    match args.get(0) {
        Some(Value::Number(max)) => {
            Ok(activation.context.rng.gen_range(0.0f64, max).floor().into())
        }
        _ => Ok(Value::Undefined), //TODO: Shouldn't this be an error condition?
    }
}

pub fn is_finite<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let ret = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(activation)?
        .is_finite();
    Ok(ret.into())
}

pub fn is_nan<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
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
    activation: &mut Activation<'_, 'gc, '_>,
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
    if let Some(radix) = radix {
        if radix < 2 || radix > 36 {
            return Ok(f64::NAN.into());
        }
    }

    let string = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;
    let mut string_s = string.as_bytes();

    let mut ignore_sign = false;
    let radix = match string_s {
        // Emulate bug: unless "0x" is a valid sequence of digits in a given radix, these prefixes
        // should result in NaN instead of 0. Otherwise, the minus sign should be ignored.
        [b'+', b'0', b'x', ..]
        | [b'+', b'0', b'X', ..]
        | [b'-', b'0', b'x', ..]
        | [b'-', b'0', b'X', ..] => {
            if radix.unwrap_or(0) <= 33 {
                return Ok(f64::NAN.into());
            } else {
                ignore_sign = true;
                radix.unwrap() // radix is present and is > 33
            }
        }

        // Auto-detect hexadecimal prefix and strip it.
        // Emulate bug: the prefix is stripped regardless of the radix.
        //   parseInt('0x100', 10) == 100  // not 0
        //   parseInt('0x100', 36) == 1296 // not 1540944
        // Emulate bug: the prefix is expected before the sign or spaces.
        //   parseInt("0x  -10") == -16 // not NaN
        //   parseInt("  -0x10") == NaN // not -16
        [b'0', b'x', rest @ ..] | [b'0', b'X', rest @ ..] => {
            string_s = rest;
            radix.unwrap_or(16)
        }

        // ECMA-262 violation: auto-detect octal numbers.
        // An auto-detected octal number cannot contain leading spaces or extra trailing characters.
        [b'0', rest @ ..] | [b'+', b'0', rest @ ..] | [b'-', b'0', rest @ ..]
            if radix.is_none() && rest.iter().all(|&x| b'0' <= x && x <= b'7') =>
        {
            8
        }

        _ => radix.unwrap_or(10),
    };

    // Strip spaces.
    while let Some(chr) = string_s.first() {
        if !b"\t\n\r ".contains(chr) {
            break;
        }
        string_s = &string_s[1..];
    }

    let (sign, string_s) = match string_s {
        [b'+', rest @ ..] => (1., rest),
        [b'-', rest @ ..] => (-1., rest),
        rest => (1., rest),
    };
    let sign = if ignore_sign { 1. } else { sign };

    let mut empty = true;
    let mut result = 0.0f64;
    for &chr in string_s {
        let digit = match chr {
            b'0'..=b'9' => chr as u32 - b'0' as u32,
            b'a'..=b'z' => chr as u32 - b'a' as u32 + 10,
            b'A'..=b'Z' => chr as u32 - b'A' as u32 + 10,
            _ => break,
        };
        if digit as i32 >= radix {
            break;
        }
        result = result * radix as f64 + digit as f64;
        empty = false;
    }

    if empty {
        Ok(f64::NAN.into())
    } else {
        Ok(result.copysign(sign).into())
    }
}

pub fn get_infinity<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if activation.current_swf_version() > 4 {
        Ok(f64::INFINITY.into())
    } else {
        Ok(Value::Undefined)
    }
}

pub fn get_nan<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if activation.current_swf_version() > 4 {
        Ok(f64::NAN.into())
    } else {
        Ok(Value::Undefined)
    }
}

pub fn set_interval<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,

    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    create_timer(activation, this, args, false)
}

pub fn set_timeout<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,

    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    create_timer(activation, this, args, true)
}

pub fn create_timer<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,

    _this: Object<'gc>,
    args: &[Value<'gc>],
    is_timeout: bool,
) -> Result<Value<'gc>, Error<'gc>> {
    // `setInterval` was added in Flash Player 6 but is not version-gated.
    use crate::avm1::timer::TimerCallback;
    let (callback, i) = match args.get(0) {
        Some(Value::Object(o)) if o.as_executable().is_some() => (TimerCallback::Function(*o), 1),
        Some(Value::Object(o)) => (
            TimerCallback::Method {
                this: *o,
                method_name: args
                    .get(1)
                    .unwrap_or(&Value::Undefined)
                    .coerce_to_string(activation)?
                    .to_string(),
            },
            2,
        ),
        _ => return Ok(Value::Undefined),
    };

    let interval = match args.get(i) {
        Some(Value::Undefined) | None => return Ok(Value::Undefined),
        Some(value) => value.coerce_to_i32(activation)?,
    };
    let params = if let Some(params) = args.get(i + 1..) {
        params.to_vec()
    } else {
        vec![]
    };

    let id = activation
        .context
        .timers
        .add_timer(callback, interval, params, is_timeout);

    Ok(id.into())
}

pub fn clear_interval<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,

    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let id = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_i32(activation)?;
    if !activation.context.timers.remove(id) {
        log::info!("clearInterval: Timer {} does not exist", id);
    }

    Ok(Value::Undefined)
}

pub fn update_after_event<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    *activation.context.needs_render = true;

    Ok(Value::Undefined)
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
    pub xml_node: Object<'gc>,
    pub string: Object<'gc>,
    pub number: Object<'gc>,
    pub boolean: Object<'gc>,
    pub matrix: Object<'gc>,
    pub matrix_constructor: Object<'gc>,
    pub point: Object<'gc>,
    pub point_constructor: Object<'gc>,
    pub rectangle: Object<'gc>,
    pub rectangle_constructor: Object<'gc>,
    pub transform: Object<'gc>,
    pub transform_constructor: Object<'gc>,
    pub shared_object: Object<'gc>,
    pub shared_object_constructor: Object<'gc>,
    pub color_transform: Object<'gc>,
    pub color_transform_constructor: Object<'gc>,
    pub context_menu: Object<'gc>,
    pub context_menu_constructor: Object<'gc>,
    pub context_menu_item: Object<'gc>,
    pub context_menu_item_constructor: Object<'gc>,
    pub bitmap_filter: Object<'gc>,
    pub bitmap_filter_constructor: Object<'gc>,
    pub blur_filter: Object<'gc>,
    pub blur_filter_constructor: Object<'gc>,
    pub date: Object<'gc>,
}

/// Initialize default global scope and builtins for an AVM1 instance.
pub fn create_globals<'gc>(
    gc_context: MutationContext<'gc, '_>,
) -> (
    SystemPrototypes<'gc>,
    Object<'gc>,
    as_broadcaster::BroadcasterFunctions<'gc>,
) {
    let object_proto = ScriptObject::object_cell(gc_context, None);
    let function_proto = function::create_proto(gc_context, object_proto);

    object::fill_proto(gc_context, object_proto, function_proto);

    let button_proto: Object<'gc> = button::create_proto(gc_context, object_proto, function_proto);

    let movie_clip_proto: Object<'gc> =
        movie_clip::create_proto(gc_context, object_proto, function_proto);

    let sound_proto: Object<'gc> = sound::create_proto(gc_context, object_proto, function_proto);

    let text_field_proto: Object<'gc> =
        text_field::create_proto(gc_context, object_proto, function_proto);
    let text_format_proto: Object<'gc> =
        text_format::create_proto(gc_context, object_proto, function_proto);

    let array_proto: Object<'gc> = array::create_proto(gc_context, object_proto, function_proto);

    let color_proto: Object<'gc> = color::create_proto(gc_context, object_proto, function_proto);

    let error_proto: Object<'gc> = error::create_proto(gc_context, object_proto, function_proto);

    let xmlnode_proto: Object<'gc> =
        xml::create_xmlnode_proto(gc_context, object_proto, function_proto);

    let xml_proto: Object<'gc> = xml::create_xml_proto(gc_context, xmlnode_proto, function_proto);

    let string_proto: Object<'gc> = string::create_proto(gc_context, object_proto, function_proto);
    let number_proto: Object<'gc> = number::create_proto(gc_context, object_proto, function_proto);
    let boolean_proto: Object<'gc> =
        boolean::create_proto(gc_context, object_proto, function_proto);
    let load_vars_proto: Object<'gc> =
        load_vars::create_proto(gc_context, object_proto, function_proto);
    let matrix_proto: Object<'gc> = matrix::create_proto(gc_context, object_proto, function_proto);
    let point_proto: Object<'gc> = point::create_proto(gc_context, object_proto, function_proto);
    let rectangle_proto: Object<'gc> =
        rectangle::create_proto(gc_context, object_proto, function_proto);
    let color_transform_proto: Object<'gc> =
        color_transform::create_proto(gc_context, object_proto, function_proto);
    let transform_proto: Object<'gc> =
        transform::create_proto(gc_context, object_proto, function_proto);
    let external_interface_proto: Object<'gc> =
        external_interface::create_proto(gc_context, object_proto);

    let (broadcaster_functions, as_broadcaster) =
        as_broadcaster::create(gc_context, Some(object_proto), function_proto);

    let movie_clip_loader_proto: Object<'gc> = movie_clip_loader::create_proto(
        gc_context,
        object_proto,
        function_proto,
        array_proto,
        broadcaster_functions,
    );

    let movie_clip_loader = FunctionObject::constructor(
        gc_context,
        Executable::Native(movie_clip_loader::constructor),
        Some(function_proto),
        movie_clip_loader_proto,
    );
    let date_proto: Object<'gc> = date::create_proto(gc_context, object_proto, function_proto);

    //TODO: These need to be constructors and should also set `.prototype` on each one
    let object = object::create_object_object(gc_context, object_proto, function_proto);

    let context_menu_proto = context_menu::create_proto(gc_context, object_proto, function_proto);
    let context_menu_item_proto =
        context_menu_item::create_proto(gc_context, object_proto, function_proto);

    let button = FunctionObject::constructor(
        gc_context,
        Executable::Native(button::constructor),
        Some(function_proto),
        button_proto,
    );
    let color = FunctionObject::constructor(
        gc_context,
        Executable::Native(color::constructor),
        Some(function_proto),
        color_proto,
    );
    let error = FunctionObject::constructor(
        gc_context,
        Executable::Native(error::constructor),
        Some(function_proto),
        error_proto,
    );
    let function = FunctionObject::function_and_constructor(
        gc_context,
        Executable::Native(function::function),
        Executable::Native(function::constructor),
        Some(function_proto),
        function_proto,
    );
    let load_vars = FunctionObject::constructor(
        gc_context,
        Executable::Native(load_vars::constructor),
        Some(function_proto),
        load_vars_proto,
    );
    let movie_clip = FunctionObject::constructor(
        gc_context,
        Executable::Native(movie_clip::constructor),
        Some(function_proto),
        movie_clip_proto,
    );

    let sound = FunctionObject::constructor(
        gc_context,
        Executable::Native(sound::constructor),
        Some(function_proto),
        sound_proto,
    );
    let text_field = FunctionObject::constructor(
        gc_context,
        Executable::Native(text_field::constructor),
        Some(function_proto),
        text_field_proto,
    );
    let text_format = FunctionObject::constructor(
        gc_context,
        Executable::Native(text_format::constructor),
        Some(function_proto),
        text_format_proto,
    );
    let array = array::create_array_object(gc_context, array_proto, Some(function_proto));
    let xmlnode = FunctionObject::constructor(
        gc_context,
        Executable::Native(xml::xmlnode_constructor),
        Some(function_proto),
        xmlnode_proto,
    );
    let xml = FunctionObject::constructor(
        gc_context,
        Executable::Native(xml::xml_constructor),
        Some(function_proto),
        xml_proto,
    );
    let string = string::create_string_object(gc_context, string_proto, Some(function_proto));
    let number = number::create_number_object(gc_context, number_proto, Some(function_proto));
    let boolean = boolean::create_boolean_object(gc_context, boolean_proto, Some(function_proto));
    let date = date::create_date_object(gc_context, date_proto, Some(function_proto));

    let flash = ScriptObject::object(gc_context, Some(object_proto));

    let geom = ScriptObject::object(gc_context, Some(object_proto));
    let filters = ScriptObject::object(gc_context, Some(object_proto));

    let matrix = matrix::create_matrix_object(gc_context, matrix_proto, Some(function_proto));
    let point = point::create_point_object(gc_context, point_proto, Some(function_proto));
    let rectangle =
        rectangle::create_rectangle_object(gc_context, rectangle_proto, Some(function_proto));
    let color_transform = FunctionObject::function(
        gc_context,
        Executable::Native(color_transform::constructor),
        Some(function_proto),
        color_transform_proto,
    );
    let transform = FunctionObject::function(
        gc_context,
        Executable::Native(transform::constructor),
        Some(function_proto),
        transform_proto,
    );

    flash.define_value(gc_context, "geom", geom.into(), EnumSet::empty());
    flash.define_value(gc_context, "filters", filters.into(), EnumSet::empty());
    geom.define_value(gc_context, "Matrix", matrix.into(), EnumSet::empty());
    geom.define_value(gc_context, "Point", point.into(), EnumSet::empty());
    geom.define_value(gc_context, "Rectangle", rectangle.into(), EnumSet::empty());
    geom.define_value(
        gc_context,
        "ColorTransform",
        color_transform.into(),
        EnumSet::empty(),
    );
    geom.define_value(gc_context, "Transform", transform.into(), EnumSet::empty());

    let bitmap_filter_proto =
        bitmap_filter::create_proto(gc_context, object_proto, Some(function_proto));
    let bitmap_filter = FunctionObject::constructor(
        gc_context,
        Executable::Native(bitmap_filter::constructor),
        Some(function_proto),
        bitmap_filter_proto,
    );

    let blur_filter_proto =
        blur_filter::create_proto(gc_context, bitmap_filter_proto, function_proto);
    let blur_filter = FunctionObject::constructor(
        gc_context,
        Executable::Native(blur_filter::constructor),
        Some(function_proto),
        blur_filter_proto,
    );

    filters.define_value(
        gc_context,
        "BitmapFilter",
        bitmap_filter.into(),
        EnumSet::empty(),
    );
    filters.define_value(
        gc_context,
        "BlurFilter",
        blur_filter.into(),
        EnumSet::empty(),
    );

    let external = ScriptObject::object(gc_context, Some(object_proto));
    let external_interface = external_interface::create_external_interface_object(
        gc_context,
        external_interface_proto,
        function_proto,
    );

    flash.define_value(gc_context, "external", external.into(), EnumSet::empty());
    external.define_value(
        gc_context,
        "ExternalInterface",
        external_interface.into(),
        EnumSet::empty(),
    );

    let mut globals = ScriptObject::bare_object(gc_context);
    globals.define_value(
        gc_context,
        "AsBroadcaster",
        as_broadcaster.into(),
        DontEnum.into(),
    );
    globals.define_value(gc_context, "flash", flash.into(), DontEnum.into());
    globals.define_value(gc_context, "Array", array.into(), DontEnum.into());
    globals.define_value(gc_context, "Button", button.into(), DontEnum.into());
    globals.define_value(gc_context, "Color", color.into(), DontEnum.into());
    globals.define_value(gc_context, "Error", error.into(), DontEnum.into());
    globals.define_value(gc_context, "Object", object.into(), DontEnum.into());
    globals.define_value(gc_context, "Function", function.into(), DontEnum.into());
    globals.define_value(gc_context, "LoadVars", load_vars.into(), DontEnum.into());
    globals.define_value(gc_context, "MovieClip", movie_clip.into(), DontEnum.into());
    globals.define_value(
        gc_context,
        "MovieClipLoader",
        movie_clip_loader.into(),
        DontEnum.into(),
    );
    globals.define_value(gc_context, "Sound", sound.into(), DontEnum.into());
    globals.define_value(gc_context, "TextField", text_field.into(), DontEnum.into());
    globals.define_value(
        gc_context,
        "TextFormat",
        text_format.into(),
        DontEnum.into(),
    );
    globals.define_value(gc_context, "XMLNode", xmlnode.into(), DontEnum.into());
    globals.define_value(gc_context, "XML", xml.into(), DontEnum.into());
    globals.define_value(gc_context, "String", string.into(), DontEnum.into());
    globals.define_value(gc_context, "Number", number.into(), DontEnum.into());
    globals.define_value(gc_context, "Boolean", boolean.into(), DontEnum.into());
    globals.define_value(gc_context, "Date", date.into(), DontEnum.into());

    let shared_object_proto = shared_object::create_proto(gc_context, object_proto, function_proto);

    let shared_obj = shared_object::create_shared_object_object(
        gc_context,
        shared_object_proto,
        Some(function_proto),
    );
    globals.define_value(
        gc_context,
        "SharedObject",
        shared_obj.into(),
        DontEnum.into(),
    );

    let context_menu = FunctionObject::constructor(
        gc_context,
        Executable::Native(context_menu::constructor),
        Some(function_proto),
        context_menu_proto,
    );
    globals.define_value(
        gc_context,
        "ContextMenu",
        context_menu.into(),
        DontEnum.into(),
    );

    let context_menu_item = FunctionObject::constructor(
        gc_context,
        Executable::Native(context_menu_item::constructor),
        Some(function_proto),
        context_menu_item_proto,
    );
    globals.define_value(
        gc_context,
        "ContextMenuItem",
        context_menu_item.into(),
        DontEnum.into(),
    );

    let system_security = system_security::create(gc_context, Some(object_proto), function_proto);
    let system_capabilities =
        system_capabilities::create(gc_context, Some(object_proto), function_proto);
    let system_ime = system_ime::create(
        gc_context,
        Some(object_proto),
        Some(function_proto),
        broadcaster_functions,
        array_proto,
    );

    let system = system::create(
        gc_context,
        Some(object_proto),
        function_proto,
        system_security,
        system_capabilities,
        system_ime,
    );
    globals.define_value(gc_context, "System", system.into(), DontEnum.into());

    globals.define_value(
        gc_context,
        "Math",
        Value::Object(math::create(
            gc_context,
            Some(object_proto),
            Some(function_proto),
        )),
        DontEnum.into(),
    );
    globals.define_value(
        gc_context,
        "Mouse",
        Value::Object(mouse::create_mouse_object(
            gc_context,
            Some(object_proto),
            Some(function_proto),
            broadcaster_functions,
            array_proto,
        )),
        DontEnum.into(),
    );
    globals.define_value(
        gc_context,
        "Key",
        Value::Object(key::create_key_object(
            gc_context,
            Some(object_proto),
            Some(function_proto),
            broadcaster_functions,
            array_proto,
        )),
        DontEnum.into(),
    );
    globals.define_value(
        gc_context,
        "Stage",
        Value::Object(stage::create_stage_object(
            gc_context,
            Some(object_proto),
            Some(array_proto),
            function_proto,
            broadcaster_functions,
        )),
        DontEnum.into(),
    );
    globals.force_set_function(
        "isFinite",
        is_finite,
        gc_context,
        DontEnum,
        Some(function_proto),
    );
    globals.force_set_function("isNaN", is_nan, gc_context, DontEnum, Some(function_proto));
    globals.force_set_function(
        "parseInt",
        parse_int,
        gc_context,
        DontEnum,
        Some(function_proto),
    );
    globals.force_set_function("random", random, gc_context, DontEnum, Some(function_proto));
    globals.force_set_function(
        "ASSetPropFlags",
        object::as_set_prop_flags,
        gc_context,
        DontEnum,
        Some(function_proto),
    );
    globals.force_set_function(
        "clearInterval",
        clear_interval,
        gc_context,
        DontEnum,
        Some(function_proto),
    );
    globals.force_set_function(
        "setInterval",
        set_interval,
        gc_context,
        DontEnum,
        Some(function_proto),
    );
    globals.force_set_function(
        "setTimeout",
        set_timeout,
        gc_context,
        DontEnum,
        Some(function_proto),
    );
    globals.force_set_function(
        "updateAfterEvent",
        update_after_event,
        gc_context,
        DontEnum,
        Some(function_proto),
    );

    globals.add_property(
        gc_context,
        "NaN",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_nan),
            Some(function_proto),
            function_proto,
        ),
        None,
        DontEnum.into(),
    );
    globals.add_property(
        gc_context,
        "Infinity",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_infinity),
            Some(function_proto),
            function_proto,
        ),
        None,
        DontEnum.into(),
    );

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
            xml_node: xmlnode_proto,
            string: string_proto,
            number: number_proto,
            boolean: boolean_proto,
            matrix: matrix_proto,
            matrix_constructor: matrix,
            point: point_proto,
            point_constructor: point,
            rectangle: rectangle_proto,
            rectangle_constructor: rectangle,
            transform: transform_proto,
            transform_constructor: transform,
            shared_object: shared_object_proto,
            shared_object_constructor: shared_obj,
            color_transform: color_transform_proto,
            color_transform_constructor: color_transform,
            context_menu: context_menu_proto,
            context_menu_constructor: context_menu,
            context_menu_item: context_menu_item_proto,
            context_menu_item_constructor: context_menu_item,
            bitmap_filter: bitmap_filter_proto,
            bitmap_filter_constructor: bitmap_filter,
            blur_filter: blur_filter_proto,
            blur_filter_constructor: blur_filter,
            date: date_proto,
        },
        globals.into(),
        broadcaster_functions,
    )
}

#[cfg(test)]
#[allow(clippy::unreadable_literal)]
mod tests {
    use super::*;

    fn setup<'gc>(activation: &mut Activation<'_, 'gc, '_>) -> Object<'gc> {
        create_globals(activation.context.gc_context).1
    }

    test_method!(boolean_function, "Boolean", setup,
        [19] => {
            [true] => true,
            [false] => false,
            [10.0] => true,
            [-10.0] => true,
            [0.0] => false,
            [std::f64::INFINITY] => true,
            [std::f64::NAN] => false,
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
            [std::f64::INFINITY] => true,
            [std::f64::NAN] => false,
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
            [std::f64::INFINITY] => false,
            [std::f64::NAN] => true,
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
            [std::f64::INFINITY] => false,
            [std::f64::NEG_INFINITY] => false,
            [std::f64::NAN] => false,
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
            ["true"] => std::f64::NAN,
            ["false"] => std::f64::NAN,
            [1.0] => 1.0,
            [0.0] => 0.0,
            [0.000] => 0.0,
            ["0.000"] => 0.0,
            ["True"] => std::f64::NAN,
            ["False"] => std::f64::NAN,
            [std::f64::NAN] => std::f64::NAN,
            [std::f64::INFINITY] => std::f64::INFINITY,
            [std::f64::NEG_INFINITY] => std::f64::NEG_INFINITY,
            [" 12"] => 12.0,
            [" \t\r\n12"] => 12.0,
            ["\u{A0}12"] => std::f64::NAN,
            [" 0x12"] => std::f64::NAN,
            ["01.2"] => 1.2,
            [""] => std::f64::NAN,
            ["Hello"] => std::f64::NAN,
            [" "] => std::f64::NAN,
            ["  5  "] => std::f64::NAN,
            ["0"] => 0.0,
            ["1"] => 1.0,
            ["Infinity"] => std::f64::NAN,
            ["100a"] => std::f64::NAN,
            ["0xhello"] => std::f64::NAN,
            ["123e-1"] => 12.3,
            ["0xUIXUIDFKHJDF012345678"] => std::f64::NAN,
            [] => 0.0
        },
        [5] => {
            ["0x12"] => std::f64::NAN,
            ["0x10"] => std::f64::NAN,
            ["0x1999999981ffffff"] => std::f64::NAN,
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
            ["-0x10"] => std::f64::NAN,
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
            [Value::Undefined] => std::f64::NAN,
            [Value::Null] => std::f64::NAN
        }
    );
}
