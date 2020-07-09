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
    let mut so = this.as_script_object().unwrap();

    let callback = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation, context);

    so.set("onSelect", callback.into(), activation, context)?;

    so.force_set_function(
        "copy",
        copy,
        context.gc_context,
        Attribute::DontEnum,
        Some(context.system_prototypes.function),
    );

    so.force_set_function(
        "hideBuiltInItems",
        hide_builtin_items,
        context.gc_context,
        Attribute::DontEnum,
        Some(context.system_prototypes.function),
    );

    let obj_proto = activation.avm.prototypes.object;
    let built_in_items = obj_proto.new(activation, context, obj_proto, &[])?;
    let _ = crate::avm1::globals::object::constructor(activation, context, built_in_items, &[]);
    built_in_items.set("print", true.into(), activation, context)?;
    built_in_items.set("forward_back", true.into(), activation, context)?;
    built_in_items.set("rewind", true.into(), activation, context)?;
    built_in_items.set("loop", true.into(), activation, context)?;
    built_in_items.set("play", true.into(), activation, context)?;
    built_in_items.set("quality", true.into(), activation, context)?;
    built_in_items.set("zoom", true.into(), activation, context)?;
    built_in_items.set("save", true.into(), activation, context)?;

    this.set("builtInItems", built_in_items.into(), activation, context)?;

    let array_proto = activation.avm.prototypes.array;
    let custom_items = array_proto.new(activation, context, array_proto, &[])?;
    let _ = crate::avm1::globals::array::constructor(activation, context, custom_items, &[]);

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

    let context_menu_proto = activation.avm.prototypes.context_menu;
    let copy = context_menu_proto.new(activation, context, context_menu_proto, &[])?;
    let _ = constructor(activation, context, copy, &[callback.into()]);

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

pub fn create_proto<'gc>(gc_context: MutationContext<'gc, '_>, proto: Object<'gc>) -> Object<'gc> {
    ScriptObject::object(gc_context, Some(proto)).into()
}
