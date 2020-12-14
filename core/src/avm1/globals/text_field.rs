use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::globals::display_object::{self, AVM_DEPTH_BIAS, AVM_MAX_REMOVE_DEPTH};
use crate::avm1::property::Attribute::*;
use crate::avm1::{AvmString, Object, ScriptObject, TObject, Value};
use crate::avm_error;
use crate::display_object::{AutoSizeMode, EditText, TDisplayObject, TDisplayObjectContainer};
use crate::html::TextFormat;
use enumset::EnumSet;
use gc_arena::MutationContext;

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

macro_rules! with_text_field_props {
    ($obj:ident, $gc:ident, $fn_proto:ident, $($name:literal => [$get:ident $(, $set:ident)*],)*) => {
        $(
            $obj.add_property(
                $gc,
                $name,
                with_text_field_props!(getter $gc, $fn_proto, $get),
                with_text_field_props!(setter $gc, $fn_proto, $($set),*),
                Default::default()
            );
        )*
    };

    (getter $gc:ident, $fn_proto:ident, $get:ident) => {
        FunctionObject::function(
            $gc,
            Executable::Native(
                |activation: &mut Activation<'_, 'gc, '_>, this, _args| -> Result<Value<'gc>, Error<'gc>> {
                    if let Some(display_object) = this.as_display_object() {
                        if let Some(edit_text) = display_object.as_edit_text() {
                            return $get(edit_text, activation);
                        }
                    }
                    Ok(Value::Undefined)
                } as crate::avm1::function::NativeFunction<'gc>
            ),
            Some($fn_proto),
            $fn_proto
        )
    };

    (setter $gc:ident, $fn_proto:ident, $set:ident) => {
        Some(FunctionObject::function(
            $gc,
            Executable::Native(
                |activation: &mut Activation<'_, 'gc, '_>, this, args| -> Result<Value<'gc>, Error<'gc>> {
                    if let Some(display_object) = this.as_display_object() {
                        if let Some(edit_text) = display_object.as_edit_text() {
                            let value = args
                                .get(0)
                                .unwrap_or(&Value::Undefined)
                                .clone();
                            $set(edit_text, activation, value)?;
                        }
                    }
                    Ok(Value::Undefined)
                } as crate::avm1::function::NativeFunction<'gc>
            ),
            Some($fn_proto),
            $fn_proto)
        )
    };

    (setter $gc:ident, $fn_proto:ident,) => {
        None
    };
}

/// Implements `TextField`
pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
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
        "replaceText" => replace_text,
        "removeTextField" => remove_text_field
    );

    with_text_field_props!(
        object, gc_context, fn_proto,
        "autoSize" => [auto_size, set_auto_size],
        "backgroundColor" => [background_color, set_background_color],
        "border" => [border, set_border],
        "borderColor" => [border_color, set_border_color],
        "embedFonts" => [embed_fonts, set_embed_fonts],
        "html" => [html, set_html],
        "htmlText" => [html_text, set_html_text],
        "length" => [length],
        "multiline" => [multiline, set_multiline],
        "selectable" => [selectable, set_selectable],
        "text" => [text, set_text],
        "textColor" => [text_color, set_text_color],
        "textHeight" => [text_height],
        "textWidth" => [text_width],
        "type" => [get_type, set_type],
        "variable" => [variable, set_variable],
        "wordWrap" => [word_wrap, set_word_wrap],
    );

    object.into()
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

fn remove_text_field<'gc>(
    text_field: EditText<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let depth = text_field.depth();

    if depth >= AVM_DEPTH_BIAS && depth < AVM_MAX_REMOVE_DEPTH {
        // Need a parent to remove from.
        let mut parent = if let Some(parent) = text_field.parent().and_then(|o| o.as_movie_clip()) {
            parent
        } else {
            return Ok(Value::Undefined);
        };

        parent.remove_child(&mut activation.context, text_field.into(), EnumSet::all());
    }

    Ok(Value::Undefined)
}

pub fn text<'gc>(
    this: EditText<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(AvmString::new(activation.context.gc_context, this.text()).into())
}

pub fn set_text<'gc>(
    this: EditText<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    if let Err(err) = this.set_text(
        value.coerce_to_string(activation)?.to_string(),
        &mut activation.context,
    ) {
        avm_error!(activation, "Error when setting TextField.text: {}", err);
    }
    this.propagate_text_binding(activation);

    Ok(())
}

pub fn html<'gc>(
    this: EditText<'gc>,
    _activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.is_html().into())
}

pub fn set_html<'gc>(
    this: EditText<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    let current_swf_version = activation.current_swf_version();
    this.set_is_html(&mut activation.context, value.as_bool(current_swf_version));
    Ok(())
}

pub fn text_color<'gc>(
    this: EditText<'gc>,
    _activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(color) = this.new_text_format().color {
        return Ok(color.to_rgb().into());
    }
    Ok(Value::Undefined)
}

pub fn set_text_color<'gc>(
    this: EditText<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    if let Ok(rgb) = value.coerce_to_u32(activation) {
        let tf = TextFormat {
            color: Some(swf::Color::from_rgb(rgb, 0xFF)),
            ..TextFormat::default()
        };
        this.set_text_format(0, this.text_length(), tf.clone(), &mut activation.context);
        this.set_new_text_format(tf, &mut activation.context);
    }
    Ok(())
}

pub fn html_text<'gc>(
    this: EditText<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Value<'gc>, Error<'gc>> {
    if let Ok(text) = this.html_text(&mut activation.context) {
        return Ok(AvmString::new(activation.context.gc_context, text).into());
    }

    Ok(Value::Undefined)
}

pub fn set_html_text<'gc>(
    this: EditText<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    let text = value.coerce_to_string(activation)?;
    let _ = this.set_html_text(text.to_string(), &mut activation.context);
    // Changing the htmlText does NOT update variable bindings (does not call EditText::propagate_text_binding).
    Ok(())
}

pub fn background_color<'gc>(
    this: EditText<'gc>,
    _activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.background_color().into())
}

pub fn set_background_color<'gc>(
    this: EditText<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    let rgb = value.coerce_to_u32(activation)?;
    this.set_background_color(activation.context.gc_context, rgb & 0xFFFFFF);
    Ok(())
}

pub fn border<'gc>(
    this: EditText<'gc>,
    _activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.has_border().into())
}

pub fn set_border<'gc>(
    this: EditText<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    let has_border = value.as_bool(activation.current_swf_version());
    this.set_has_border(activation.context.gc_context, has_border);
    Ok(())
}

pub fn border_color<'gc>(
    this: EditText<'gc>,
    _activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.border_color().into())
}

pub fn set_border_color<'gc>(
    this: EditText<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    let rgb = value.coerce_to_u32(activation)?;
    this.set_border_color(activation.context.gc_context, rgb & 0xFFFFFF);
    Ok(())
}

pub fn embed_fonts<'gc>(
    this: EditText<'gc>,
    _activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok((!this.is_device_font()).into())
}

pub fn set_embed_fonts<'gc>(
    this: EditText<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    let embed_fonts = value.as_bool(activation.current_swf_version());
    this.set_is_device_font(&mut activation.context, !embed_fonts);
    Ok(())
}

pub fn length<'gc>(
    this: EditText<'gc>,
    _activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok((this.text_length() as f64).into())
}

pub fn text_width<'gc>(
    this: EditText<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Value<'gc>, Error<'gc>> {
    let metrics = this.measure_text(&mut activation.context);
    Ok(metrics.0.to_pixels().into())
}

pub fn text_height<'gc>(
    this: EditText<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Value<'gc>, Error<'gc>> {
    let metrics = this.measure_text(&mut activation.context);
    Ok(metrics.1.to_pixels().into())
}

pub fn multiline<'gc>(
    this: EditText<'gc>,
    _activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.is_multiline().into())
}

pub fn set_multiline<'gc>(
    this: EditText<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    let is_multiline = value.as_bool(activation.current_swf_version());
    this.set_multiline(is_multiline, &mut activation.context);
    Ok(())
}

pub fn selectable<'gc>(
    this: EditText<'gc>,
    _activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.is_selectable().into())
}

pub fn set_selectable<'gc>(
    this: EditText<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    let set_selectable = value.as_bool(activation.current_swf_version());
    this.set_selectable(set_selectable, &mut activation.context);
    Ok(())
}

fn variable<'gc>(
    this: EditText<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(variable) = this.variable() {
        return Ok(AvmString::new(activation.context.gc_context, variable.to_string()).into());
    }

    // Unset `variable` returns null, not undefined
    Ok(Value::Null)
}

fn set_variable<'gc>(
    this: EditText<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    let variable = match value {
        Value::Undefined | Value::Null => None,
        v => Some(v.coerce_to_string(activation)?),
    };
    this.set_variable(variable.map(|v| v.to_string()), activation);
    Ok(())
}

pub fn word_wrap<'gc>(
    this: EditText<'gc>,
    _activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.is_word_wrap().into())
}

pub fn set_word_wrap<'gc>(
    this: EditText<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    let is_word_wrap = value.as_bool(activation.current_swf_version());
    this.set_word_wrap(is_word_wrap, &mut activation.context);
    Ok(())
}

pub fn auto_size<'gc>(
    this: EditText<'gc>,
    _activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(match this.autosize() {
        AutoSizeMode::None => "none".into(),
        AutoSizeMode::Left => "left".into(),
        AutoSizeMode::Center => "center".into(),
        AutoSizeMode::Right => "right".into(),
    })
}

pub fn set_auto_size<'gc>(
    this: EditText<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    this.set_autosize(
        match value {
            Value::String(s) if s == "left" => AutoSizeMode::Left,
            Value::String(s) if s == "center" => AutoSizeMode::Center,
            Value::String(s) if s == "right" => AutoSizeMode::Right,
            Value::Bool(true) => AutoSizeMode::Left,
            _ => AutoSizeMode::None,
        },
        &mut activation.context,
    );

    Ok(())
}

pub fn get_type<'gc>(
    this: EditText<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Value<'gc>, Error<'gc>> {
    let tf_type = match this.is_editable() {
        true => "input",
        false => "dynamic",
    };
    Ok(AvmString::new(activation.context.gc_context, tf_type).into())
}

pub fn set_type<'gc>(
    this: EditText<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    match value
        .coerce_to_string(activation)?
        .to_ascii_lowercase()
        .as_str()
    {
        "input" => this.set_editable(true, &mut activation.context),
        "dynamic" => this.set_editable(false, &mut activation.context),
        value => log::warn!("Invalid TextField.type: {}", value),
    };
    Ok(())
}
