use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::globals::display_object;
use crate::avm1::property::Attribute::*;
use crate::avm1::{AvmString, Object, ScriptObject, TObject, Value};
use crate::avm_error;
use crate::display_object::{AutoSizeMode, EditText, TDisplayObject};
use crate::html::TextFormat;
use gc_arena::MutationContext;

/// Implements `TextField`
pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

pub fn get_text<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(display_object) = this.as_display_object() {
        if let Some(text_field) = display_object.as_edit_text() {
            return Ok(AvmString::new(activation.context.gc_context, text_field.text()).into());
        }
    }
    Ok(Value::Undefined)
}

pub fn set_text<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(display_object) = this.as_display_object() {
        if let Some(text_field) = display_object.as_edit_text() {
            if let Some(value) = args.get(0) {
                if let Err(err) = text_field.set_text(
                    value.coerce_to_string(activation)?.to_string(),
                    &mut activation.context,
                ) {
                    avm_error!(activation, "Error when setting TextField.text: {}", err);
                }
                text_field.propagate_text_binding(activation);
            }
        }
    }
    Ok(Value::Undefined)
}

pub fn get_html<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(display_object) = this.as_display_object() {
        if let Some(text_field) = display_object.as_edit_text() {
            return Ok(text_field.is_html().into());
        }
    }
    Ok(Value::Undefined)
}

pub fn set_html<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(display_object) = this.as_display_object() {
        if let Some(text_field) = display_object.as_edit_text() {
            if let Some(value) = args.get(0) {
                let current_swf_version = activation.current_swf_version();
                text_field.set_is_html(&mut activation.context, value.as_bool(current_swf_version));
            }
        }
    }
    Ok(Value::Undefined)
}

pub fn get_html_text<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(display_object) = this.as_display_object() {
        if let Some(text_field) = display_object.as_edit_text() {
            if let Ok(text) = text_field.html_text(&mut activation.context) {
                return Ok(AvmString::new(activation.context.gc_context, text).into());
            }
        }
    }
    Ok(Value::Undefined)
}

pub fn set_html_text<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(display_object) = this.as_display_object() {
        if let Some(text_field) = display_object.as_edit_text() {
            let text = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_string(activation)?;
            let _ = text_field.set_html_text(text.to_string(), &mut activation.context);
            // Changing the htmlText does NOT update variable bindings (does not call EditText::propagate_text_binding).
        }
    }
    Ok(Value::Undefined)
}

pub fn get_border<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(display_object) = this.as_display_object() {
        if let Some(text_field) = display_object.as_edit_text() {
            return Ok(text_field.has_border().into());
        }
    }

    Ok(Value::Undefined)
}

pub fn set_border<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(display_object) = this.as_display_object() {
        if let Some(text_field) = display_object.as_edit_text() {
            if let Some(value) = args.get(0) {
                let has_border = value.as_bool(activation.current_swf_version());
                text_field.set_has_border(activation.context.gc_context, has_border);
            }
        }
    }
    Ok(Value::Undefined)
}

pub fn get_embed_fonts<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(display_object) = this.as_display_object() {
        if let Some(text_field) = display_object.as_edit_text() {
            return Ok((!text_field.is_device_font()).into());
        }
    }

    Ok(Value::Undefined)
}

pub fn set_embed_fonts<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(display_object) = this.as_display_object() {
        if let Some(text_field) = display_object.as_edit_text() {
            if let Some(value) = args.get(0) {
                let embed_fonts = value.as_bool(activation.current_swf_version());
                text_field.set_is_device_font(&mut activation.context, !embed_fonts);
            }
        }
    }
    Ok(Value::Undefined)
}

pub fn get_length<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(display_object) = this.as_display_object() {
        if let Some(text_field) = display_object.as_edit_text() {
            return Ok((text_field.text_length() as f64).into());
        }
    }
    Ok(Value::Undefined)
}

macro_rules! with_text_field {
    ( $gc_context: ident, $object:ident, $fn_proto: expr, $($name:expr => $fn:expr),* ) => {{
        $(
            $object.force_set_function(
                $name,
                |activation: &mut Activation<'_, 'gc, '_>, this, args| -> Result<Value<'gc>, Error<'gc>> {
                    if let Some(display_object) = this.as_display_object() {
                        if let Some(text_field) = display_object.as_edit_text() {
                            return $fn(text_field, activation, args);
                        }
                    }
                    Ok(Value::Undefined)
                } as crate::avm1::function::NativeFunction<'gc>,
                $gc_context,
                DontDelete | ReadOnly | DontEnum,
                $fn_proto
            );
        )*
    }};
}

pub fn text_width<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(etext) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_edit_text())
    {
        let metrics = etext.measure_text(&mut activation.context);

        return Ok(metrics.0.to_pixels().into());
    }

    Ok(Value::Undefined)
}

pub fn text_height<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(etext) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_edit_text())
    {
        let metrics = etext.measure_text(&mut activation.context);

        return Ok(metrics.1.to_pixels().into());
    }

    Ok(Value::Undefined)
}

pub fn multiline<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(etext) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_edit_text())
    {
        return Ok(etext.is_multiline().into());
    }

    Ok(Value::Undefined)
}

pub fn set_multiline<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let is_multiline = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Undefined)
        .as_bool(activation.current_swf_version());

    if let Some(etext) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_edit_text())
    {
        etext.set_multiline(is_multiline, &mut activation.context);
    }

    Ok(Value::Undefined)
}

fn variable<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(etext) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_edit_text())
    {
        if let Some(variable) = etext.variable() {
            return Ok(AvmString::new(activation.context.gc_context, variable.to_string()).into());
        }
    }

    // Unset `variable` returns null, not undefined
    Ok(Value::Null)
}

fn set_variable<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let variable = match args.get(0) {
        None | Some(Value::Undefined) | Some(Value::Null) => None,
        Some(v) => Some(v.coerce_to_string(activation)?),
    };

    if let Some(etext) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_edit_text())
    {
        etext.set_variable(variable.map(|v| v.to_string()), activation);
    }

    Ok(Value::Undefined)
}

pub fn word_wrap<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(etext) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_edit_text())
    {
        return Ok(etext.is_word_wrap().into());
    }

    Ok(Value::Undefined)
}

pub fn set_word_wrap<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let is_word_wrap = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Undefined)
        .as_bool(activation.current_swf_version());

    if let Some(etext) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_edit_text())
    {
        etext.set_word_wrap(is_word_wrap, &mut activation.context);
    }

    Ok(Value::Undefined)
}

pub fn auto_size<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(etext) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_edit_text())
    {
        return Ok(match etext.autosize() {
            AutoSizeMode::None => "none".into(),
            AutoSizeMode::Left => "left".into(),
            AutoSizeMode::Center => "center".into(),
            AutoSizeMode::Right => "right".into(),
        });
    }

    Ok(Value::Undefined)
}

pub fn set_auto_size<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(etext) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_edit_text())
    {
        etext.set_autosize(
            match args.get(0).cloned().unwrap_or(Value::Undefined) {
                Value::String(s) if s == "left" => AutoSizeMode::Left,
                Value::String(s) if s == "center" => AutoSizeMode::Center,
                Value::String(s) if s == "right" => AutoSizeMode::Right,
                Value::Bool(true) => AutoSizeMode::Left,
                _ => AutoSizeMode::None,
            },
            &mut activation.context,
        );
    }

    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let mut object = ScriptObject::object(gc_context, Some(proto));

    display_object::define_display_object_proto(gc_context, object, fn_proto);

    with_text_field!(
        gc_context,
        object,
        Some(fn_proto),
        "getNewTextFormat" => get_new_text_format,
        "setNewTextFormat" => set_new_text_format,
        "getTextFormat" => get_text_format,
        "setTextFormat" => set_text_format,
        "replaceText" => replace_text
    );

    object.into()
}

pub fn attach_virtual_properties<'gc>(
    gc_context: MutationContext<'gc, '_>,
    object: Object<'gc>,
    fn_proto: Object<'gc>,
) {
    object.add_property(
        gc_context,
        "text",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_text),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_text),
            Some(fn_proto),
            fn_proto,
        )),
        ReadOnly.into(),
    );
    object.add_property(
        gc_context,
        "html",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_html),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_html),
            Some(fn_proto),
            fn_proto,
        )),
        ReadOnly.into(),
    );
    object.add_property(
        gc_context,
        "htmlText",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_html_text),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_html_text),
            Some(fn_proto),
            fn_proto,
        )),
        ReadOnly.into(),
    );
    object.add_property(
        gc_context,
        "length",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_length),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        ReadOnly.into(),
    );
    object.add_property(
        gc_context,
        "textWidth",
        FunctionObject::function(
            gc_context,
            Executable::Native(text_width),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        ReadOnly.into(),
    );
    object.add_property(
        gc_context,
        "textHeight",
        FunctionObject::function(
            gc_context,
            Executable::Native(text_height),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        ReadOnly.into(),
    );
    object.add_property(
        gc_context,
        "multiline",
        FunctionObject::function(
            gc_context,
            Executable::Native(multiline),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_multiline),
            Some(fn_proto),
            fn_proto,
        )),
        ReadOnly.into(),
    );
    object.add_property(
        gc_context,
        "variable",
        FunctionObject::function(
            gc_context,
            Executable::Native(variable),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_variable),
            Some(fn_proto),
            fn_proto,
        )),
        DontDelete | ReadOnly | DontEnum,
    );
    object.add_property(
        gc_context,
        "wordWrap",
        FunctionObject::function(
            gc_context,
            Executable::Native(word_wrap),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_word_wrap),
            Some(fn_proto),
            fn_proto,
        )),
        ReadOnly.into(),
    );
    object.add_property(
        gc_context,
        "autoSize",
        FunctionObject::function(
            gc_context,
            Executable::Native(auto_size),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_auto_size),
            Some(fn_proto),
            fn_proto,
        )),
        ReadOnly.into(),
    );
    object.add_property(
        gc_context,
        "border",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_border),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_border),
            Some(fn_proto),
            fn_proto,
        )),
        ReadOnly.into(),
    );
    object.add_property(
        gc_context,
        "embedFonts",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_embed_fonts),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_embed_fonts),
            Some(fn_proto),
            fn_proto,
        )),
        ReadOnly.into(),
    );
}

fn get_new_text_format<'gc>(
    text_field: EditText<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let tf = text_field.new_text_format();

    Ok(tf.as_avm1_object(activation)?.into())
}

fn set_new_text_format<'gc>(
    text_field: EditText<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let tf = args.get(0).cloned().unwrap_or(Value::Undefined);

    if let Value::Object(tf) = tf {
        let tf_parsed = TextFormat::from_avm1_object(tf, activation)?;
        text_field.set_new_text_format(tf_parsed, &mut activation.context);
    }

    Ok(Value::Undefined)
}

fn get_text_format<'gc>(
    text_field: EditText<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let (from, to) = match (args.get(0), args.get(1)) {
        (Some(f), Some(t)) => (
            f.coerce_to_f64(activation)? as usize,
            t.coerce_to_f64(activation)? as usize,
        ),
        (Some(f), None) => {
            let v = f.coerce_to_f64(activation)? as usize;
            (v, v.saturating_add(1))
        }
        _ => (0, text_field.text_length()),
    };

    Ok(text_field
        .text_format(from, to)
        .as_avm1_object(activation)?
        .into())
}

fn set_text_format<'gc>(
    text_field: EditText<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let tf = args.last().cloned().unwrap_or(Value::Undefined);

    if let Value::Object(tf) = tf {
        let tf_parsed = TextFormat::from_avm1_object(tf, activation)?;

        let (from, to) = match (args.get(0), args.get(1)) {
            (Some(f), Some(t)) if args.len() > 2 => (
                f.coerce_to_f64(activation)? as usize,
                t.coerce_to_f64(activation)? as usize,
            ),
            (Some(f), _) if args.len() > 1 => {
                let v = f.coerce_to_f64(activation)? as usize;
                (v, v.saturating_add(1))
            }
            _ => (0, text_field.text_length()),
        };

        text_field.set_text_format(from, to, tf_parsed, &mut activation.context);
    }

    Ok(Value::Undefined)
}

fn replace_text<'gc>(
    text_field: EditText<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let from = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_f64(activation)?;
    let to = args
        .get(1)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_f64(activation)?;
    let text = args
        .get(2)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_string(activation)?
        .to_string();

    text_field.replace_text(from as usize, to as usize, &text, &mut activation.context);

    Ok(Value::Undefined)
}
