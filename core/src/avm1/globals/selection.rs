use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::globals::as_broadcaster::BroadcasterFunctions;
use crate::avm1::property::Attribute;
use crate::avm1::{Object, ScriptObject, TDisplayObject, TObject, Value};
use gc_arena::MutationContext;

pub fn get_begin_index<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // TODO: Implement
    Ok(Value::Number(-1.0))
}

pub fn get_end_index<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // TODO: Implement
    Ok(Value::Number(-1.0))
}

pub fn get_caret_index<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // TODO: Implement
    Ok(Value::Number(-1.0))
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
        Some(Value::Null) | Some(Value::Undefined) => {
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
    let mut object = ScriptObject::object(gc_context, Some(proto));

    broadcaster_functions.initialize(gc_context, object.into(), array_proto);

    object.force_set_function(
        "getBeginIndex",
        get_begin_index,
        gc_context,
        Attribute::DontDelete | Attribute::DontEnum | Attribute::ReadOnly,
        Some(fn_proto),
    );

    object.force_set_function(
        "getEndIndex",
        get_end_index,
        gc_context,
        Attribute::DontDelete | Attribute::DontEnum | Attribute::ReadOnly,
        Some(fn_proto),
    );

    object.force_set_function(
        "getCaretIndex",
        get_caret_index,
        gc_context,
        Attribute::DontDelete | Attribute::DontEnum | Attribute::ReadOnly,
        Some(fn_proto),
    );

    object.force_set_function(
        "setFocus",
        set_focus,
        gc_context,
        Attribute::DontDelete | Attribute::DontEnum | Attribute::ReadOnly,
        Some(fn_proto),
    );

    object.force_set_function(
        "getFocus",
        get_focus,
        gc_context,
        Attribute::DontDelete | Attribute::DontEnum | Attribute::ReadOnly,
        Some(fn_proto),
    );

    object.into()
}

pub fn create_proto<'gc>(gc_context: MutationContext<'gc, '_>, proto: Object<'gc>) -> Object<'gc> {
    // It's a custom prototype but it's empty.
    ScriptObject::object(gc_context, Some(proto)).into()
}
