use crate::avm1::error::Error;
use crate::avm1::listeners::Listeners;
use crate::avm1::property::Attribute;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Object, ScriptObject, TObject, UpdateContext, Value};

use gc_arena::MutationContext;

pub fn show_mouse<'gc>(
    _avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    let was_visible = context.input.mouse_visible();
    context.input.show_mouse();
    if was_visible {
        Ok(0.into())
    } else {
        Ok(1.into())
    }
}

pub fn hide_mouse<'gc>(
    _avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    let was_visible = context.input.mouse_visible();
    context.input.hide_mouse();
    if was_visible {
        Ok(0.into())
    } else {
        Ok(1.into())
    }
}

pub fn create_mouse_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
    listener: &Listeners<'gc>,
) -> Object<'gc> {
    let mut mouse = ScriptObject::object(gc_context, proto);

    register_listener!(gc_context, mouse, listener, fn_proto, mouse);

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
