use crate::avm2::activation::Activation;
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;

pub fn hide_built_in_items<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    // TODO: replace this by a proper implementation.
    log::warn!("flash.ui.ContextMenu is a stub");
    activation
        .context
        .stage
        .set_show_menu(&mut activation.context, false);

    Ok(Value::Undefined)
}
