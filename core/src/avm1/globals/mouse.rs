use crate::avm1::property::Attribute;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, ScriptObject, TObject, UpdateContext, Value};

use gc_arena::MutationContext;

pub fn add_listener<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let listeners = this
        .get_local("_listeners", avm, context, this)
        .and_then(|v| v.resolve(avm, context))?
        .as_object()?;
    let listener = args.get(0).unwrap_or(&Value::Undefined).to_owned();
    for i in 0..listeners.length() {
        if listeners.array_element(i) == listener {
            return Ok(true.into());
        }
    }

    listeners.set_array_element(listeners.length(), listener, context.gc_context);
    Ok(true.into())
}

pub fn remove_listener<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let listeners = this
        .get_local("_listeners", avm, context, this)
        .and_then(|v| v.resolve(avm, context))?
        .as_object()?;
    let listener = args.get(0).unwrap_or(&Value::Undefined).to_owned();
    for index in 0..listeners.length() {
        if listeners.array_element(index) == listener {
            let new_length = listeners.length() - 1;

            for i in index..new_length {
                listeners.set_array_element(i, listeners.array_element(i + 1), context.gc_context);
            }

            listeners.delete_array_element(new_length, context.gc_context);
            listeners.delete(context.gc_context, &new_length.to_string());
            listeners.set_length(context.gc_context, new_length);

            return Ok(true.into());
        }
    }

    Ok(false.into())
}

pub fn show_mouse<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("Mouse.show() is intentionally not yet implemented");
    Ok(1.into())
}

pub fn hide_mouse<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("Mouse.hide() is intentionally not yet implemented");
    Ok(1.into())
}

pub fn create_mouse_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Option<Object<'gc>>,
    array_proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    let mut mouse = ScriptObject::object(gc_context, proto);

    mouse.define_value(
        gc_context,
        "_listeners",
        ScriptObject::array(gc_context, array_proto).into(),
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );

    mouse.force_set_function(
        "addListener",
        add_listener,
        gc_context,
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
        fn_proto,
    );

    mouse.force_set_function(
        "removeListener",
        remove_listener,
        gc_context,
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
        fn_proto,
    );

    mouse.force_set_function(
        "show",
        show_mouse,
        gc_context,
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
        fn_proto,
    );

    mouse.force_set_function(
        "hide",
        hide_mouse,
        gc_context,
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
        fn_proto,
    );

    mouse.into()
}

pub fn notify_listeners<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let method = args.get(0).unwrap().as_string()?;
    let globals = avm.globals;
    let mouse = globals
        .get_local("Mouse", avm, context, globals)
        .and_then(|v| v.resolve(avm, context))?
        .as_object()?;
    let listeners = mouse
        .get_local("_listeners", avm, context, mouse)
        .and_then(|v| v.resolve(avm, context))?
        .as_object()?;

    for i in 0..listeners.length() {
        if let Ok(listener) = listeners.array_element(i).as_object() {
            let handler = listener
                .get(method, avm, context)
                .and_then(|v| v.resolve(avm, context))?;
            let _ = handler.call(avm, context, listener, &[]);
        }
    }

    Ok(Value::Undefined.into())
}
