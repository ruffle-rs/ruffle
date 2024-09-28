use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::globals::as_broadcaster::BroadcasterFunctions;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, ScriptObject, Value};
use crate::string::StringContext;

const OBJECT_DECLS: &[Declaration] = declare_properties! {
    "show" => method(show_mouse; DONT_DELETE | DONT_ENUM | READ_ONLY);
    "hide" => method(hide_mouse; DONT_DELETE | DONT_ENUM | READ_ONLY);
};

pub fn show_mouse<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let was_visible = activation.context.ui.mouse_visible();
    activation.context.ui.set_mouse_visible(true);
    Ok(if was_visible { 0 } else { 1 }.into())
}

pub fn hide_mouse<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let was_visible = activation.context.ui.mouse_visible();
    activation.context.ui.set_mouse_visible(false);
    Ok(if was_visible { 0 } else { 1 }.into())
}

pub fn create_mouse_object<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
    broadcaster_functions: BroadcasterFunctions<'gc>,
    array_proto: Object<'gc>,
) -> Object<'gc> {
    let mouse = ScriptObject::new(context.gc_context, Some(proto));
    broadcaster_functions.initialize(context.gc_context, mouse.into(), array_proto);
    define_properties_on(OBJECT_DECLS, context, mouse, fn_proto);
    mouse.into()
}
