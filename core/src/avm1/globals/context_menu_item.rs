use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::TObject;
use crate::avm1::Object;
use crate::avm1::{ScriptObject, Value};
use crate::context::UpdateContext;
use enumset::EnumSet;
use gc_arena::MutationContext;

// TODO: should appear at the top of the context menu with a seperator between it and built ins
//TODO: items must have a visible name (at leat one char, not punc,whitespace/newline/control)

//TODO: can't be the same as an existing item (custom or otherwise) (filtered out) (TODO: does this consider punc,etc)
//TODO: two items are the same if they have the same caption[..100], ignoing case, punctuation and whitespace

//TODO: filter out any bad/duplicates then take first 15

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let caption = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .coerce_to_string(activation, context)?
        .to_string();
    let callback = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .coerce_to_object(activation, context);
    let separator_before = args
        .get(2)
        .unwrap_or(&Value::Bool(false))
        .to_owned()
        .as_bool(activation.swf_version());
    let enabled = args
        .get(3)
        .unwrap_or(&Value::Bool(true))
        .to_owned()
        .as_bool(activation.swf_version());
    let visible = args
        .get(4)
        .unwrap_or(&Value::Bool(true))
        .to_owned()
        .as_bool(activation.swf_version());

    this.set("caption", caption.into(), activation, context)?;
    this.set("enabled", enabled.into(), activation, context)?;
    this.set(
        "separatorBefore",
        separator_before.into(),
        activation,
        context,
    )?;
    this.set("visible", visible.into(), activation, context)?;

    let mut so = this.as_script_object().unwrap();

    so.force_set_function(
        "copy",
        copy,
        context.gc_context,
        EnumSet::empty(),
        Some(context.system_prototypes.function),
    );

    this.set("onSelect", callback.into(), activation, context)?;

    Ok(Value::Undefined.into())
}

pub fn copy<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let caption = this
        .get("caption", activation, context)?
        .coerce_to_string(activation, context)?
        .to_string();
    let callback = this
        .get("onSelect", activation, context)?
        .coerce_to_object(activation, context);

    let enabled = this
        .get("enabled", activation, context)?
        .as_bool(activation.swf_version());
    let separator_before = this
        .get("separator_before", activation, context)?
        .as_bool(activation.swf_version());
    let visible = this
        .get("visible", activation, context)?
        .as_bool(activation.swf_version());

    let context_menu_item_proto = activation.avm().prototypes.context_menu_item;
    let copy = context_menu_item_proto.new(activation, context, context_menu_item_proto, &[])?;
    let _ = constructor(
        activation,
        context,
        copy,
        &[
            caption.into(),
            callback.into(),
            separator_before.into(),
            enabled.into(),
            visible.into(),
        ],
    );

    Ok(copy.into())
}

pub fn create_proto<'gc>(gc_context: MutationContext<'gc, '_>, proto: Object<'gc>) -> Object<'gc> {
    let object = ScriptObject::object(gc_context, Some(proto));
    object.into()
}
