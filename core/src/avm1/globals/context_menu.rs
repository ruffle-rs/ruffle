use gc_arena::MutationContext;
use crate::avm1::{Value, ScriptObject};
use crate::avm1::Object;
use crate::context::UpdateContext;
use crate::avm1::error::Error;
use enumset::EnumSet;
use crate::avm1::activation::Activation;
use crate::avm1::object::TObject;

//TODO: note: callback should be called when menu is opened but before it is displayed
//TODO: check for hidden props
//TODO: there shuold be a menu prop somewhere (as2:444)

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {

    if let Some(callback) = args.get(0) {
        let callback_obj = callback.coerce_to_object(activation, context);
        let so = this.as_script_object().unwrap();
        so.set("onSelect", callback_obj.into(), activation, context)?;
    }

    let obj_proto = activation.avm().prototypes.object;
    let built_in_items = obj_proto.new(activation, context, obj_proto, &[])?;
    let _ = crate::avm1::globals::object::constructor(activation, context, built_in_items, &[]);
    built_in_items.set("save", true.into(), activation, context)?;
    built_in_items.set("zoom", true.into(), activation, context)?;
    built_in_items.set("quality", true.into(), activation, context)?;
    built_in_items.set("play", true.into(), activation, context)?;
    built_in_items.set("loop", true.into(), activation, context)?;
    built_in_items.set("rewind", true.into(), activation, context)?;
    built_in_items.set("forward_back", true.into(), activation, context)?;
    built_in_items.set("print", true.into(), activation, context)?;

    this.set("builtInItems", built_in_items.into(), activation, context)?;

    let array_proto = activation.avm().prototypes.array;
    let custom_items = array_proto.new(activation, context, array_proto, &[])?;
    let _ = crate::avm1::globals::array::constructor(activation, context, custom_items, &[]);

    this.set("customItems", custom_items.into(), activation, context)?;

    Ok(Value::Undefined.into())
}

pub fn copy<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {

    let callback = this.get("onSelect", activation, context)?.coerce_to_object(activation, context);

    let context_menu_proto = activation.avm().prototypes.context_menu;
    let copy = context_menu_proto.new(activation, context, context_menu_proto, &[])?;
    let _ = constructor(activation, context, copy, &[
        callback.into()
    ]);

    let copy_built_in = this.get("builtInItems", activation, context)?.coerce_to_object(activation, context);


    let save = this.get("save", activation, context)?.as_bool(activation.swf_version());
    let zoom = this.get("zoom", activation, context)?.as_bool(activation.swf_version());
    let quality = this.get("quality", activation, context)?.as_bool(activation.swf_version());
    let play = this.get("play", activation, context)?.as_bool(activation.swf_version());
    let loop_ = this.get("loop", activation, context)?.as_bool(activation.swf_version());
    let rewind = this.get("rewind", activation, context)?.as_bool(activation.swf_version());
    let forward_back = this.get("forward_back", activation, context)?.as_bool(activation.swf_version());
    let print = this.get("print", activation, context)?.as_bool(activation.swf_version());

    copy_built_in.set("save", save.into(), activation, context)?;
    copy_built_in.set("zoom", zoom.into(), activation, context)?;
    copy_built_in.set("quality", quality.into(), activation, context)?;
    copy_built_in.set("play", play.into(), activation, context)?;
    copy_built_in.set("loop", loop_.into(), activation, context)?;
    copy_built_in.set("rewind", rewind.into(), activation, context)?;
    copy_built_in.set("forward_back", forward_back.into(), activation, context)?;
    copy_built_in.set("print", print.into(), activation, context)?;

    let custom_items = this.get("customItems", activation, context)?.coerce_to_object(activation, context);
    let custom_items_copy = this.get("customItems", activation, context)?.coerce_to_object(activation, context);

    for i in 0..custom_items.length() {
        custom_items_copy.set_array_element(i, custom_items.array_element(i).into(), context.gc_context);
    }

    Ok(copy.into())
}

pub fn hide_builtin_items<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let built_in_items = this.get("builtInItems", activation, context)?.coerce_to_object(activation, context);
    built_in_items.set("zoom", false.into(), activation, context)?;
    built_in_items.set("quality", false.into(), activation, context)?;
    built_in_items.set("play", false.into(), activation, context)?;
    built_in_items.set("loop", false.into(), activation, context)?;
    built_in_items.set("rewind", false.into(), activation, context)?;
    built_in_items.set("forward_back", false.into(), activation, context)?;
    built_in_items.set("print", false.into(), activation, context)?;
    Ok(Value::Undefined.into())
}

pub fn on_select<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // By default does nothing
    Ok(Value::Undefined.into())
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let mut object = ScriptObject::object(gc_context, Some(proto));

    //TODO: check the proto, these should only be on instances apparently
    object.force_set_function(
        "copy",
        copy,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto)
    );

    object.force_set_function(
        "hideBuiltInItems",
        hide_builtin_items,
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
