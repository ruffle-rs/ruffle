use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::TObject;
use crate::avm1::property::Attribute;
use crate::avm1::Object;
use crate::avm1::{ScriptObject, Value};
use crate::context::UpdateContext;
use gc_arena::MutationContext;

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let callback = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation, context);

    this.set("onSelect", callback.into(), activation, context)?;

    let constructor = activation.avm.prototypes.object_constructor;
    let built_in_items = constructor.construct(activation, context, &[])?;

    built_in_items.set("print", true.into(), activation, context)?;
    built_in_items.set("forward_back", true.into(), activation, context)?;
    built_in_items.set("rewind", true.into(), activation, context)?;
    built_in_items.set("loop", true.into(), activation, context)?;
    built_in_items.set("play", true.into(), activation, context)?;
    built_in_items.set("quality", true.into(), activation, context)?;
    built_in_items.set("zoom", true.into(), activation, context)?;
    built_in_items.set("save", true.into(), activation, context)?;

    this.set("builtInItems", built_in_items.into(), activation, context)?;

    let constructor = activation.avm.prototypes.array_constructor;
    let custom_items = constructor.construct(activation, context, &[])?;

    this.set("customItems", custom_items.into(), activation, context)?;

    Ok(Value::Undefined)
}

pub fn copy<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let callback = this
        .get("onSelect", activation, context)?
        .coerce_to_object(activation, context);

    let constructor = activation.avm.prototypes.context_menu_constructor;
    let copy = constructor.construct(activation, context, &[callback.into()])?;

    let built_in = this
        .get("builtInItems", activation, context)?
        .coerce_to_object(activation, context);
    let copy_built_in = copy
        .get("builtInItems", activation, context)?
        .coerce_to_object(activation, context);

    let save = built_in
        .get("save", activation, context)?
        .as_bool(activation.current_swf_version());
    let zoom = built_in
        .get("zoom", activation, context)?
        .as_bool(activation.current_swf_version());
    let quality = built_in
        .get("quality", activation, context)?
        .as_bool(activation.current_swf_version());
    let play = built_in
        .get("play", activation, context)?
        .as_bool(activation.current_swf_version());
    let loop_ = built_in
        .get("loop", activation, context)?
        .as_bool(activation.current_swf_version());
    let rewind = built_in
        .get("rewind", activation, context)?
        .as_bool(activation.current_swf_version());
    let forward_back = built_in
        .get("forward_back", activation, context)?
        .as_bool(activation.current_swf_version());
    let print = built_in
        .get("print", activation, context)?
        .as_bool(activation.current_swf_version());

    copy_built_in.set("save", save.into(), activation, context)?;
    copy_built_in.set("zoom", zoom.into(), activation, context)?;
    copy_built_in.set("quality", quality.into(), activation, context)?;
    copy_built_in.set("play", play.into(), activation, context)?;
    copy_built_in.set("loop", loop_.into(), activation, context)?;
    copy_built_in.set("rewind", rewind.into(), activation, context)?;
    copy_built_in.set("forward_back", forward_back.into(), activation, context)?;
    copy_built_in.set("print", print.into(), activation, context)?;

    let custom_items = this
        .get("customItems", activation, context)?
        .coerce_to_object(activation, context);
    let custom_items_copy = copy
        .get("customItems", activation, context)?
        .coerce_to_object(activation, context);

    for i in 0..custom_items.length() {
        custom_items_copy.set_array_element(i, custom_items.array_element(i), context.gc_context);
    }

    Ok(copy.into())
}

pub fn hide_builtin_items<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let built_in_items = this
        .get("builtInItems", activation, context)?
        .coerce_to_object(activation, context);
    built_in_items.set("zoom", false.into(), activation, context)?;
    built_in_items.set("quality", false.into(), activation, context)?;
    built_in_items.set("play", false.into(), activation, context)?;
    built_in_items.set("loop", false.into(), activation, context)?;
    built_in_items.set("rewind", false.into(), activation, context)?;
    built_in_items.set("forward_back", false.into(), activation, context)?;
    built_in_items.set("print", false.into(), activation, context)?;
    Ok(Value::Undefined)
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
        Attribute::DontEnum | Attribute::DontDelete,
        Some(fn_proto),
    );

    object.force_set_function(
        "hideBuiltInItems",
        hide_builtin_items,
        gc_context,
        Attribute::DontEnum | Attribute::DontDelete,
        Some(fn_proto),
    );

    object.into()
}
