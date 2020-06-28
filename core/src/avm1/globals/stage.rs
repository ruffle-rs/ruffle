//! Stage object
//!
//! TODO: This is a very rough stub with not much implementation.
use crate::avm1::error::Error;
use crate::avm1::function::Executable;
use crate::avm1::property::Attribute;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::stack_frame::StackFrame;
use crate::avm1::{Object, ScriptObject, TObject, UpdateContext, Value};
use gc_arena::MutationContext;

pub fn create_stage_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Option<Object<'gc>>,
    _array_proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    let mut stage = ScriptObject::object(gc_context, proto);

    stage.force_set_function(
        "addListener",
        add_listener,
        gc_context,
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
        fn_proto,
    );

    stage.add_property(
        gc_context,
        "align",
        Executable::Native(align),
        Some(Executable::Native(set_align)),
        Attribute::DontEnum | Attribute::DontDelete,
    );

    stage.add_property(
        gc_context,
        "height",
        Executable::Native(height),
        None,
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );

    stage.force_set_function(
        "removeListener",
        remove_listener,
        gc_context,
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
        fn_proto,
    );

    stage.add_property(
        gc_context,
        "scaleMode",
        Executable::Native(scale_mode),
        Some(Executable::Native(set_scale_mode)),
        Attribute::DontEnum | Attribute::DontDelete,
    );

    stage.add_property(
        gc_context,
        "showMenu",
        Executable::Native(show_menu),
        Some(Executable::Native(set_show_menu)),
        Attribute::DontEnum | Attribute::DontDelete,
    );

    stage.add_property(
        gc_context,
        "width",
        Executable::Native(width),
        None,
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );

    stage.into()
}

fn add_listener<'gc>(
    _activation: &mut StackFrame<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    log::warn!("Stage.addListener: unimplemented");
    Ok(Value::Undefined.into())
}

fn align<'gc>(
    _activation: &mut StackFrame<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    log::warn!("Stage.align: unimplemented");
    Ok("".into())
}

fn set_align<'gc>(
    _activation: &mut StackFrame<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    log::warn!("Stage.align: unimplemented");
    Ok(Value::Undefined.into())
}

fn height<'gc>(
    _activation: &mut StackFrame<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    Ok(context.stage_size.1.to_pixels().into())
}

fn remove_listener<'gc>(
    _activation: &mut StackFrame<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    log::warn!("Stage.removeListener: unimplemented");
    Ok("".into())
}

fn scale_mode<'gc>(
    _activation: &mut StackFrame<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    log::warn!("Stage.scaleMode: unimplemented");
    Ok("noScale".into())
}

fn set_scale_mode<'gc>(
    _activation: &mut StackFrame<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    log::warn!("Stage.scaleMode: unimplemented");
    Ok(Value::Undefined.into())
}

fn show_menu<'gc>(
    _activation: &mut StackFrame<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    log::warn!("Stage.showMenu: unimplemented");
    Ok(true.into())
}

fn set_show_menu<'gc>(
    _activation: &mut StackFrame<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    log::warn!("Stage.showMenu: unimplemented");
    Ok(Value::Undefined.into())
}

fn width<'gc>(
    _activation: &mut StackFrame<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    Ok(context.stage_size.0.to_pixels().into())
}
