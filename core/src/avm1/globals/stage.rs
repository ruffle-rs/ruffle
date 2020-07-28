//! Stage object
//!
//! TODO: This is a very rough stub with not much implementation.
use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::globals::as_broadcaster::BroadcasterFunctions;
use crate::avm1::property::Attribute;
use crate::avm1::{Object, ScriptObject, TObject, UpdateContext, Value};
use crate::avm_warn;
use gc_arena::MutationContext;

pub fn create_stage_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Option<Object<'gc>>,
    array_proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
    broadcaster_functions: BroadcasterFunctions<'gc>,
) -> Object<'gc> {
    let stage = ScriptObject::object(gc_context, proto);

    broadcaster_functions.initialize(gc_context, stage.into(), array_proto.unwrap());

    stage.add_property(
        gc_context,
        "align",
        FunctionObject::function(gc_context, Executable::Native(align), fn_proto, fn_proto),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_align),
            fn_proto,
            fn_proto,
        )),
        Attribute::DontEnum | Attribute::DontDelete,
    );

    stage.add_property(
        gc_context,
        "height",
        FunctionObject::function(gc_context, Executable::Native(height), fn_proto, fn_proto),
        None,
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );

    stage.add_property(
        gc_context,
        "scaleMode",
        FunctionObject::function(
            gc_context,
            Executable::Native(scale_mode),
            fn_proto,
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_scale_mode),
            fn_proto,
            fn_proto,
        )),
        Attribute::DontEnum | Attribute::DontDelete,
    );

    stage.add_property(
        gc_context,
        "showMenu",
        FunctionObject::function(
            gc_context,
            Executable::Native(show_menu),
            fn_proto,
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_show_menu),
            fn_proto,
            fn_proto,
        )),
        Attribute::DontEnum | Attribute::DontDelete,
    );

    stage.add_property(
        gc_context,
        "width",
        FunctionObject::function(gc_context, Executable::Native(width), fn_proto, fn_proto),
        None,
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );

    stage.into()
}

fn align<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "Stage.align: unimplemented");
    Ok("".into())
}

fn set_align<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "Stage.align: unimplemented");
    Ok(Value::Undefined)
}

fn height<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(context.stage_size.1.to_pixels().into())
}

fn scale_mode<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "Stage.scaleMode: unimplemented");
    Ok("noScale".into())
}

fn set_scale_mode<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "Stage.scaleMode: unimplemented");
    Ok(Value::Undefined)
}

fn show_menu<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "Stage.showMenu: unimplemented");
    Ok(true.into())
}

fn set_show_menu<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "Stage.showMenu: unimplemented");
    Ok(Value::Undefined)
}

fn width<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(context.stage_size.0.to_pixels().into())
}
