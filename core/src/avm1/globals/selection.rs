use ruffle_macros::istr;

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::globals::as_broadcaster::BroadcasterFunctions;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations};
use crate::avm1::{Object, Value};
use crate::display_object::{EditText, TDisplayObject, TInteractiveObject, TextSelection};

const OBJECT_DECLS: StaticDeclarations = declare_static_properties! {
    "getBeginIndex" => method(get_begin_index; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getEndIndex" => method(get_end_index; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getCaretIndex" => method(get_caret_index; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getFocus" => method(get_focus; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "setFocus" => method(set_focus; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "setSelection" => method(set_selection; DONT_ENUM | DONT_DELETE | READ_ONLY);
};

pub fn create<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    broadcaster_fns: BroadcasterFunctions<'gc>,
    array_proto: Object<'gc>,
) -> Object<'gc> {
    let selection = Object::new(context.strings, Some(context.object_proto));
    broadcaster_fns.initialize(context.strings, selection, array_proto);
    context.define_properties_on(selection, OBJECT_DECLS(context));
    selection
}

pub fn get_begin_index<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(selection) = activation
        .context
        .focus_tracker
        .get_as_edit_text()
        .and_then(EditText::selection)
    {
        Ok(selection.start().into())
    } else {
        Ok((-1).into())
    }
}

pub fn get_end_index<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(selection) = activation
        .context
        .focus_tracker
        .get_as_edit_text()
        .and_then(EditText::selection)
    {
        Ok(selection.end().into())
    } else {
        Ok((-1).into())
    }
}

pub fn get_caret_index<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(selection) = activation
        .context
        .focus_tracker
        .get_as_edit_text()
        .and_then(EditText::selection)
    {
        Ok(selection.to().into())
    } else {
        Ok((-1).into())
    }
}

pub fn set_selection<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.is_empty() {
        return Ok(Value::Undefined);
    }

    if let Some(edit_box) = activation.context.focus_tracker.get_as_edit_text() {
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
        edit_box.set_selection(Some(selection));
    }
    Ok(Value::Undefined)
}

pub fn get_focus<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let focus = activation.context.focus_tracker.get();
    Ok(match focus {
        Some(focus) => focus
            .as_displayobject()
            .object1_or_undef()
            .coerce_to_string(activation)
            .unwrap_or_else(|_| istr!(""))
            .into(),
        None => Value::Null,
    })
}

pub fn set_focus<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let tracker = activation.context.focus_tracker;
    match args.get(0) {
        None => Ok(false.into()),
        Some(Value::Undefined | Value::Null) => {
            tracker.set(None, activation.context);
            Ok(true.into())
        }
        Some(focus) => {
            let start_clip = activation.target_clip_or_root();
            let object = activation.resolve_target_display_object(start_clip, *focus, false)?;
            if let Some(object) = object.and_then(|o| o.as_interactive()) {
                if object.is_focusable(activation.context) {
                    tracker.set(Some(object), activation.context);
                    return Ok(true.into());
                }
            }
            Ok(false.into())
        }
    }
}
