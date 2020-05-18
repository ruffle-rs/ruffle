use crate::avm1::error::Error;
use crate::avm1::function::Executable;
use crate::avm1::globals::display_object;
use crate::avm1::property::Attribute::*;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Object, ScriptObject, TObject, UpdateContext, Value};
use crate::display_object::{EditText, TDisplayObject};
use crate::html::TextFormat;
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
                text_field.set_text(value.coerce_to_string(avm, context)?.to_string(), context);
            }
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
        "getNewTextFormat" => |text_field: EditText<'gc>, avm: &mut Avm1<'gc>, context: &mut UpdateContext<'_, 'gc, '_>, _args| {
            let tf = text_field.new_text_format();

            Ok(tf.as_avm1_object(avm, context)?.into())
        },
        "setNewTextFormat" => |text_field: EditText<'gc>, avm: &mut Avm1<'gc>, context: &mut UpdateContext<'_, 'gc, '_>, args: &[Value<'gc>]| {
            let tf = args.get(0).cloned().unwrap_or(Value::Undefined);

            if let Value::Object(tf) = tf {
                let tf_parsed = TextFormat::from_avm1_object(tf, avm, context)?;
                text_field.set_new_text_format(tf_parsed, context);
            }

            Ok(Value::Undefined.into())
        },
        "getTextFormat" => |text_field: EditText<'gc>, avm: &mut Avm1<'gc>, context: &mut UpdateContext<'_, 'gc, '_>, args: &[Value<'gc>]| {
            let (from, to) = match (args.get(0), args.get(1)) {
                (Some(f), Some(t)) => (f.as_number(avm, context)? as usize, t.as_number(avm, context)? as usize),
                (Some(f), None) => {
                    let v = f.as_number(avm, context)? as usize;
                    (v, v.saturating_add(1))
                },
                _ => (0, text_field.text_length())
            };

            Ok(text_field.text_format(from, to).as_avm1_object(avm, context)?.into())
        },
        "setTextFormat" => |text_field: EditText<'gc>, avm: &mut Avm1<'gc>, context: &mut UpdateContext<'_, 'gc, '_>, args: &[Value<'gc>]| {
            let tf = args.last().cloned().unwrap_or(Value::Undefined);

            if let Value::Object(tf) = tf {
                let tf_parsed = TextFormat::from_avm1_object(tf, avm, context)?;

                let (from, to) = match (args.get(0), args.get(1)) {
                    (Some(f), Some(t)) if args.len() > 2 => (f.as_number(avm, context)? as usize, t.as_number(avm, context)? as usize),
                    (Some(f), _) if args.len() > 1 => {
                        let v = f.as_number(avm, context)? as usize;
                        (v, v.saturating_add(1))
                    },
                    _ => (0, text_field.text_length())
                };

                text_field.set_text_format(from, to, tf_parsed, context);
            }

            Ok(Value::Undefined.into())
        },
        "replaceText" => |text_field: EditText<'gc>, avm: &mut Avm1<'gc>, context: &mut UpdateContext<'_, 'gc, '_>, args: &[Value<'gc>]| {
            let from = args.get(0).cloned().unwrap_or(Value::Undefined).as_number(avm, context)?;
            let to = args.get(1).cloned().unwrap_or(Value::Undefined).as_number(avm, context)?;
            let text = args.get(2).cloned().unwrap_or(Value::Undefined).coerce_to_string(avm, context)?;

            text_field.replace_text(from as usize, to as usize, &text, context);

            Ok(Value::Undefined.into())
        }
    );

    object.into()
}

pub fn attach_virtual_properties<'gc>(gc_context: MutationContext<'gc, '_>, object: Object<'gc>) {
    object.add_property(
        gc_context,
        "text",
        Executable::Native(get_text),
        Some(Executable::Native(set_text)),
        DontDelete | ReadOnly | DontEnum,
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
}
