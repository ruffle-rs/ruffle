use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::globals::as_broadcaster::BroadcasterFunctions;
use crate::avm1::property::Attribute;
use crate::avm1::{Object, ScriptObject, Value};
use gc_arena::MutationContext;

pub fn show_mouse<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let was_visible = activation.context.ui.mouse_visible();
    activation.context.ui.set_mouse_visible(true);
    Ok(if was_visible { 0 } else { 1 }.into())
}

pub fn hide_mouse<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let was_visible = activation.context.ui.mouse_visible();
    activation.context.ui.set_mouse_visible(false);
    Ok(if was_visible { 0 } else { 1 }.into())
}

pub fn create_mouse_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
    broadcaster_functions: BroadcasterFunctions<'gc>,
    array_proto: Object<'gc>,
) -> Object<'gc> {
    let mut mouse = ScriptObject::object(gc_context, proto);

    broadcaster_functions.initialize(gc_context, mouse.into(), array_proto);

    mouse.force_set_function(
        "show",
        show_mouse,
        gc_context,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
        fn_proto,
    );

    mouse.force_set_function(
        "hide",
        hide_mouse,
        gc_context,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
        fn_proto,
    );

    mouse.into()
}
