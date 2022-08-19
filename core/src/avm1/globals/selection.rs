use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::globals::as_broadcaster::BroadcasterFunctions;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, ScriptObject, TDisplayObject, TObject, Value};
use crate::display_object::{EditText, TextSelection};
use gc_arena::MutationContext;

const OBJECT_DECLS: &[Declaration] = declare_properties! {
    "getBeginIndex" => method(get_begin_index; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getEndIndex" => method(get_end_index; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getCaretIndex" => method(get_caret_index; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "setSelection" => method(set_selection; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "setFocus" => method(set_focus; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getFocus" => method(get_focus; DONT_ENUM | DONT_DELETE | READ_ONLY);
};

pub fn get_begin_index<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(selection) = activation
        .context
        .focus_tracker
        .get()
        .and_then(|o| o.as_edit_text())
        .and_then(EditText::selection)
    {
        Ok(selection.start().into())
    } else {
        Ok((-1).into())
    }
}

pub fn get_end_index<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(selection) = activation
        .context
        .focus_tracker
        .get()
        .and_then(|o| o.as_edit_text())
        .and_then(EditText::selection)
    {
        Ok(selection.end().into())
    } else {
        Ok((-1).into())
    }
}

pub fn get_caret_index<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(selection) = activation
        .context
        .focus_tracker
        .get()
        .and_then(|o| o.as_edit_text())
        .and_then(EditText::selection)
    {
        Ok(selection.to().into())
    } else {
        Ok((-1).into())
    }
}

pub fn set_selection<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.is_empty() {
        return Ok(Value::Undefined);
    }

    if let Some(edit_box) = activation
        .context
        .focus_tracker
        .get()
        .and_then(|o| o.as_edit_text())
    {
        let start = args
            .get(0)
            .map(|v| v.coerce_to_i32(activation))
            .transpose()?
            .unwrap_or(0)
            .max(0);
        let end = args
            .get(1)
            .map(|v| v.coerce_to_i32(activation))
            .transpose()?
            .unwrap_or(i32::MAX)
            .max(0);
        let selection = TextSelection::for_range(start as usize, end as usize);
        edit_box.set_selection(Some(selection), activation.context.gc_context);
    }
    Ok(Value::Undefined)
}

pub fn get_focus<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let focus = activation.context.focus_tracker.get();
    match focus {
        Some(focus) => Ok(focus.object()),
        None => Ok(Value::Null),
    }
}

pub fn set_focus<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let tracker = activation.context.focus_tracker;
    match args.get(0) {
        Some(Value::Undefined | Value::Null) => {
            tracker.set(None, &mut activation.context);
            Ok(true.into())
        }
        Some(Value::Object(obj)) => {
            if let Some(display_object) = obj.as_display_object() {
                if display_object.is_focusable() {
                    tracker.set(Some(display_object), &mut activation.context);
                }
                // [NA] Note: The documentation says true is success and false is failure,
                // but from testing this seems to be opposite.
                Ok(false.into())
            } else {
                Ok(true.into())
            }
        }
        _ => Ok(false.into()),
    }
}

pub fn create_selection_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
    broadcaster_functions: BroadcasterFunctions<'gc>,
    array_proto: Object<'gc>,
) -> Object<'gc> {
    let object = ScriptObject::new(gc_context, Some(proto));
    broadcaster_functions.initialize(gc_context, object.into(), array_proto);
    define_properties_on(OBJECT_DECLS, gc_context, object, fn_proto);
    object.into()
}

pub fn create_proto<'gc>(gc_context: MutationContext<'gc, '_>, proto: Object<'gc>) -> Object<'gc> {
    // It's a custom prototype but it's empty.
    ScriptObject::new(gc_context, Some(proto)).into()
}
