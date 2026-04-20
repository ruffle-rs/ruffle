use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::globals::as_broadcaster::BroadcasterFunctions;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations};
use crate::avm1::{Object, Value};

const OBJECT_DECLS: StaticDeclarations = declare_static_properties! {
    "show" => method(show_mouse; DONT_DELETE | DONT_ENUM | READ_ONLY);
    "hide" => method(hide_mouse; DONT_DELETE | DONT_ENUM | READ_ONLY);
};

pub fn create<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    broadcaster_functions: BroadcasterFunctions<'gc>,
    array_proto: Object<'gc>,
) -> Object<'gc> {
    let mouse = Object::new(context.strings, Some(context.object_proto));
    broadcaster_functions.initialize(context.strings, mouse, array_proto);
    context.define_properties_on(mouse, OBJECT_DECLS(context));
    mouse
}

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
