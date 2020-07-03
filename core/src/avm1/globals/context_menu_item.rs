use gc_arena::MutationContext;
use crate::avm1::{Value, ScriptObject};
use crate::avm1::Object;
use crate::context::UpdateContext;
use crate::avm1::error::Error;
use enumset::EnumSet;
use crate::avm1::activation::Activation;
use crate::avm1::object::TObject;

// TODO: should appear at the top of the context menu with a seperator between it and built ins
//TODO: Max 15 custom items/context menu
//TODO: items must have a visible name (at leat one char, not whitespace/newline/control)
//TODO: length <= 100
//TODO: can't be the same as an existing item (custom or otherwise)
//TODO: two items are the same if they have the same caption, ignoing case, punctuation and whitespace


pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let caption = args.get(0)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .coerce_to_string(activation, context)?
        .to_string();
    let _callback = args.get(1)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .coerce_to_object(activation, context);
    let separator_before = args.get(2)
        .unwrap_or(&Value::Bool(false))
        .to_owned()
        .as_bool(activation.swf_version());
    let enabled = args.get(3)
        .unwrap_or(&Value::Bool(true))
        .to_owned()
        .as_bool(activation.swf_version());
    let visible = args.get(4)
        .unwrap_or(&Value::Bool(true))
        .to_owned()
        .as_bool(activation.swf_version());

    //TODO: check for virt
    this.set("caption", caption.into(), activation, context)?;
    this.set("enabled", enabled.into(), activation, context)?;
    this.set("separatorBefore", separator_before.into(), activation, context)?;
    this.set("visible", visible.into(), activation, context)?;

    Ok(Value::Undefined.into())
}

pub fn copy<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    log::warn!("ContextMenuItem.copy() not implemented");
    Ok(Value::Undefined.into())
}

pub fn on_select<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    log::warn!("ContextMenuItem.onSelect() not implemented");
    Ok(Value::Undefined.into())
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let mut object = ScriptObject::object(gc_context, Some(proto));

    object.force_set_function(
        "copy",
        copy,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto)
    );

    object.force_set_function(
        "onSelect",
        on_select,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto)
    );

    object.into()
}
