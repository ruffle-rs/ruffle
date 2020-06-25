use crate::avm1::function::Executable;
use crate::avm1::globals::display_object;
use crate::avm1::property::Attribute::*;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, ScriptObject, TObject, UpdateContext, Value};
use crate::display_object::{AutoSizeMode, EditText, TDisplayObject};
use crate::html::TextFormat;
use crate::xml::XMLDocument;
use gc_arena::MutationContext;

/// Implements `TextField`
pub fn constructor<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(Value::Undefined.into())
}

pub fn get_text<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(display_object) = this.as_display_object() {
        if let Some(text_field) = display_object.as_edit_text() {
            return Ok(text_field.text().into());
        }
    }
    Ok(Value::Undefined.into())
}

pub fn set_text<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(display_object) = this.as_display_object() {
        if let Some(text_field) = display_object.as_edit_text() {
            if let Some(value) = args.get(0) {
                if let Err(err) =
                    text_field.set_text(value.coerce_to_string(avm, context)?.to_string(), context)
                {
                    log::error!("Error when setting TextField.text: {}", err);
                }
            }
        }
    }
    Ok(Value::Undefined.into())
}

pub fn get_html<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(display_object) = this.as_display_object() {
        if let Some(_text_field) = display_object.as_edit_text() {
            return Ok(true.into());
        }
    }
    Ok(Value::Undefined.into())
}

pub fn set_html<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(display_object) = this.as_display_object() {
        if let Some(_text_field) = display_object.as_edit_text() {
            if let Some(_value) = args.get(0) {
                //TODO: Do something with this bool value
            }
        }
    }
    Ok(Value::Undefined.into())
}

pub fn get_html_text<'gc>(
    _avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(display_object) = this.as_display_object() {
        if let Some(text_field) = display_object.as_edit_text() {
            let html_tree = text_field.html_tree(context).as_node();
            let html_string_result = html_tree.into_string(&mut |_node| true);

            if let Err(err) = &html_string_result {
                log::warn!(
                    "Serialization error when reading TextField.htmlText: {}",
                    err
                );
            }

            return Ok(html_string_result.unwrap_or_else(|_| "".to_string()).into());
        }
    }
    Ok(Value::Undefined.into())
}

pub fn set_html_text<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(display_object) = this.as_display_object() {
        if let Some(mut text_field) = display_object.as_edit_text() {
            if let Some(value) = args.get(0) {
                let html_string = value
                    .clone()
                    .coerce_to_string(avm, context)?
                    .into_owned()
                    .replace("<sbr>", "\n")
                    .replace("<br>", "\n");
                let document = XMLDocument::new(context.gc_context);

                if let Err(err) =
                    document
                        .as_node()
                        .replace_with_str(context.gc_context, &html_string, false)
                {
                    log::warn!("Parsing error when setting TextField.htmlText: {}", err);
                }

                text_field.set_html_tree(document, context);
            }
        }
    }
    Ok(Value::Undefined.into())
}

pub fn get_border<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(display_object) = this.as_display_object() {
        if let Some(text_field) = display_object.as_edit_text() {
            return Ok(text_field.has_border().into());
        }
    }

    Ok(Value::Undefined.into())
}

pub fn set_border<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(display_object) = this.as_display_object() {
        if let Some(text_field) = display_object.as_edit_text() {
            if let Some(value) = args.get(0) {
                let has_border = value.as_bool(avm.current_swf_version());
                text_field.set_has_border(context.gc_context, has_border);
            }
        }
    }
    Ok(Value::Undefined.into())
}

pub fn get_embed_fonts<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(display_object) = this.as_display_object() {
        if let Some(text_field) = display_object.as_edit_text() {
            return Ok((!text_field.is_device_font()).into());
        }
    }

    Ok(Value::Undefined.into())
}

pub fn set_embed_fonts<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(display_object) = this.as_display_object() {
        if let Some(text_field) = display_object.as_edit_text() {
            if let Some(value) = args.get(0) {
                let embed_fonts = value.as_bool(avm.current_swf_version());
                text_field.set_is_device_font(context, embed_fonts);
            }
        }
    }
    Ok(Value::Undefined.into())
}

pub fn get_length<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(display_object) = this.as_display_object() {
        if let Some(text_field) = display_object.as_edit_text() {
            return Ok((text_field.text_length() as f64).into());
        }
    }
    Ok(Value::Undefined.into())
}

macro_rules! with_text_field {
    ( $gc_context: ident, $object:ident, $fn_proto: expr, $($name:expr => $fn:expr),* ) => {{
        $(
            $object.force_set_function(
                $name,
                |avm, context: &mut UpdateContext<'_, 'gc, '_>, this, args| -> Result<ReturnValue<'gc>, Error> {
                    if let Some(display_object) = this.as_display_object() {
                        if let Some(text_field) = display_object.as_edit_text() {
                            return $fn(text_field, avm, context, args);
                        }
                    }
                    Ok(Value::Undefined.into())
                } as crate::avm1::function::NativeFunction<'gc>,
                $gc_context,
                DontDelete | ReadOnly | DontEnum,
                $fn_proto
            );
        )*
    }};
}

pub fn text_width<'gc>(
    _avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(etext) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_edit_text())
    {
        let metrics = etext.measure_text(context);

        return Ok(metrics.0.to_pixels().into());
    }

    Ok(Value::Undefined.into())
}

pub fn text_height<'gc>(
    _avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(etext) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_edit_text())
    {
        let metrics = etext.measure_text(context);

        return Ok(metrics.1.to_pixels().into());
    }

    Ok(Value::Undefined.into())
}

pub fn multiline<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(etext) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_edit_text())
    {
        return Ok(etext.is_multiline().into());
    }

    Ok(Value::Undefined.into())
}

pub fn set_multiline<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let is_multiline = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Undefined)
        .as_bool(avm.current_swf_version());

    if let Some(etext) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_edit_text())
    {
        etext.set_multiline(is_multiline, context);
    }

    Ok(Value::Undefined.into())
}

pub fn word_wrap<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(etext) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_edit_text())
    {
        return Ok(etext.is_word_wrap().into());
    }

    Ok(Value::Undefined.into())
}

pub fn set_word_wrap<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let is_word_wrap = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Undefined)
        .as_bool(avm.current_swf_version());

    if let Some(etext) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_edit_text())
    {
        etext.set_word_wrap(is_word_wrap, context);
    }

    Ok(Value::Undefined.into())
}

pub fn auto_size<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(etext) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_edit_text())
    {
        return Ok(match etext.autosize() {
            AutoSizeMode::None => "none".to_string().into(),
            AutoSizeMode::Left => "left".to_string().into(),
            AutoSizeMode::Center => "center".to_string().into(),
            AutoSizeMode::Right => "right".to_string().into(),
        });
    }

    Ok(Value::Undefined.into())
}

pub fn set_auto_size<'gc>(
    _avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
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
            context,
        );
    }

    Ok(Value::Undefined.into())
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

pub fn attach_virtual_properties<'gc>(gc_context: MutationContext<'gc, '_>, object: Object<'gc>) {
    object.add_property(
        gc_context,
        "text",
        Executable::Native(get_text),
        Some(Executable::Native(set_text)),
        ReadOnly.into(),
    );
    object.add_property(
        gc_context,
        "html",
        Executable::Native(get_html),
        Some(Executable::Native(set_html)),
        ReadOnly.into(),
    );
    object.add_property(
        gc_context,
        "htmlText",
        Executable::Native(get_html_text),
        Some(Executable::Native(set_html_text)),
        ReadOnly.into(),
    );
    object.add_property(
        gc_context,
        "length",
        Executable::Native(get_length),
        None,
        ReadOnly.into(),
    );
    object.add_property(
        gc_context,
        "textWidth",
        Executable::Native(text_width),
        None,
        ReadOnly.into(),
    );
    object.add_property(
        gc_context,
        "textHeight",
        Executable::Native(text_height),
        None,
        ReadOnly.into(),
    );
    object.add_property(
        gc_context,
        "multiline",
        Executable::Native(multiline),
        Some(Executable::Native(set_multiline)),
        ReadOnly.into(),
    );
    object.add_property(
        gc_context,
        "wordWrap",
        Executable::Native(word_wrap),
        Some(Executable::Native(set_word_wrap)),
        ReadOnly.into(),
    );
    object.add_property(
        gc_context,
        "autoSize",
        Executable::Native(auto_size),
        Some(Executable::Native(set_auto_size)),
        ReadOnly.into(),
    );
    object.add_property(
        gc_context,
        "border",
        Executable::Native(get_border),
        Some(Executable::Native(set_border)),
        ReadOnly.into(),
    );
    object.add_property(
        gc_context,
        "embedFonts",
        Executable::Native(get_embed_fonts),
        Some(Executable::Native(set_embed_fonts)),
        ReadOnly.into(),
    );
}

fn get_new_text_format<'gc>(
    text_field: EditText<'gc>,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let tf = text_field.new_text_format();

    Ok(tf.as_avm1_object(avm, context)?.into())
}

fn set_new_text_format<'gc>(
    text_field: EditText<'gc>,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let tf = args.get(0).cloned().unwrap_or(Value::Undefined);

    if let Value::Object(tf) = tf {
        let tf_parsed = TextFormat::from_avm1_object(tf, avm, context)?;
        text_field.set_new_text_format(tf_parsed, context);
    }

    Ok(Value::Undefined.into())
}

fn get_text_format<'gc>(
    text_field: EditText<'gc>,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let (from, to) = match (args.get(0), args.get(1)) {
        (Some(f), Some(t)) => (
            f.coerce_to_f64(avm, context)? as usize,
            t.coerce_to_f64(avm, context)? as usize,
        ),
        (Some(f), None) => {
            let v = f.coerce_to_f64(avm, context)? as usize;
            (v, v.saturating_add(1))
        }
        _ => (0, text_field.text_length()),
    };

    Ok(text_field
        .text_format(from, to)
        .as_avm1_object(avm, context)?
        .into())
}

fn set_text_format<'gc>(
    text_field: EditText<'gc>,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let tf = args.last().cloned().unwrap_or(Value::Undefined);

    if let Value::Object(tf) = tf {
        let tf_parsed = TextFormat::from_avm1_object(tf, avm, context)?;

        let (from, to) = match (args.get(0), args.get(1)) {
            (Some(f), Some(t)) if args.len() > 2 => (
                f.coerce_to_f64(avm, context)? as usize,
                t.coerce_to_f64(avm, context)? as usize,
            ),
            (Some(f), _) if args.len() > 1 => {
                let v = f.coerce_to_f64(avm, context)? as usize;
                (v, v.saturating_add(1))
            }
            _ => (0, text_field.text_length()),
        };

        text_field.set_text_format(from, to, tf_parsed, context);
    }

    Ok(Value::Undefined.into())
}

fn replace_text<'gc>(
    text_field: EditText<'gc>,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let from = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_f64(avm, context)?;
    let to = args
        .get(1)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_f64(avm, context)?;
    let text = args
        .get(2)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_string(avm, context)?
        .into_owned();

    text_field.replace_text(from as usize, to as usize, &text, context);

    Ok(Value::Undefined.into())
}
