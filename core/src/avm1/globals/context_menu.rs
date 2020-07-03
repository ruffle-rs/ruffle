use gc_arena::MutationContext;
use crate::avm1::{Value, ScriptObject};
use crate::avm1::Object;
use crate::context::UpdateContext;
use crate::avm1::error::Error;
use enumset::EnumSet;
use crate::avm1::activation::Activation;
use crate::avm1::object::TObject;

//TODO: in future this will want to be a custom object, as it has a callback function in constructor arg
//TODO: note: callback should be called when menu is opened but before it is displayed
//TODO: check for hidden props
//TODO: there shuold be a menu prop somewhere (as2:444)

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let obj_proto = activation.avm().prototypes.object;
    let built_in_items = obj_proto.new(activation, context, obj_proto, &[])?;
    let _ = crate::avm1::globals::object::constructor(activation, context, built_in_items, &[]);
    //TODO Check that this lines up with FP
    built_in_items.set("zoom", true.into(), activation, context)?;
    built_in_items.set("quality", true.into(), activation, context)?;
    built_in_items.set("play", true.into(), activation, context)?;
    built_in_items.set("loop", true.into(), activation, context)?;
    built_in_items.set("rewind", true.into(), activation, context)?;
    built_in_items.set("forward_back", true.into(), activation, context)?;
    built_in_items.set("print", true.into(), activation, context)?;

    //TODO: check if these are on the proto or not
    //TODO: are they virt?
    this.set("builtInItems", built_in_items.into(), activation, context)?;

    let array_proto = activation.avm().prototypes.array;
    let custom_items = array_proto.new(activation, context, array_proto, &[])?;
    let _ = crate::avm1::globals::array::constructor(activation, context, custom_items, &[]);

    this.set("customItems", custom_items.into(), activation, context)?;

    Ok(Value::Undefined.into())
}

pub fn copy<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    log::warn!("ContextMenu.copy() not implemented");
    Ok(Value::Undefined.into())
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
    log::warn!("ContextMenu.onSelect() not implemented");
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
