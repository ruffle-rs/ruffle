//! Stage object
//!
//! TODO: This is a very rough stub with not much implementation.
use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::globals::as_broadcaster::BroadcasterFunctions;
use crate::avm1::property::Attribute;
use crate::avm1::{AvmString, Object, ScriptObject, TObject, Value};
use gc_arena::MutationContext;

pub fn create_stage_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Option<Object<'gc>>,
    array_proto: Option<Object<'gc>>,
    fn_proto: Object<'gc>,
    broadcaster_functions: BroadcasterFunctions<'gc>,
) -> Object<'gc> {
    let stage = ScriptObject::object(gc_context, proto);

    broadcaster_functions.initialize(gc_context, stage.into(), array_proto.unwrap());

    stage.add_property(
        gc_context,
        "align",
        FunctionObject::function(
            gc_context,
            Executable::Native(align),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_align),
            Some(fn_proto),
            fn_proto,
        )),
        Attribute::DONT_ENUM | Attribute::DONT_DELETE,
    );

    stage.add_property(
        gc_context,
        "height",
        FunctionObject::function(
            gc_context,
            Executable::Native(height),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );

    stage.add_property(
        gc_context,
        "scaleMode",
        FunctionObject::function(
            gc_context,
            Executable::Native(scale_mode),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_scale_mode),
            Some(fn_proto),
            fn_proto,
        )),
        Attribute::DONT_ENUM | Attribute::DONT_DELETE,
    );

    stage.add_property(
        gc_context,
        "showMenu",
        FunctionObject::function(
            gc_context,
            Executable::Native(show_menu),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_show_menu),
            Some(fn_proto),
            fn_proto,
        )),
        Attribute::DONT_ENUM | Attribute::DONT_DELETE,
    );

    stage.add_property(
        gc_context,
        "width",
        FunctionObject::function(
            gc_context,
            Executable::Native(width),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );

    stage.into()
}

fn align<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let align = activation.context.stage.align();
    let mut s = String::with_capacity(4);
    // Match string values returned by AS.
    // It's possible to have an oxymoronic "LTRB".
    // This acts the same as "TL" (top-left takes priority).
    // This order is different between AVM1 and AVM2!
    use crate::display_object::StageAlign;
    if align.contains(StageAlign::LEFT) {
        s.push('L');
    }
    if align.contains(StageAlign::TOP) {
        s.push('T');
    }
    if align.contains(StageAlign::RIGHT) {
        s.push('R');
    }
    if align.contains(StageAlign::BOTTOM) {
        s.push('B');
    }
    let align = AvmString::new(activation.context.gc_context, s);
    Ok(align.into())
}

fn set_align<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let align = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?
        .parse()
        .unwrap_or_default();
    activation
        .context
        .stage
        .set_align(&mut activation.context, align);
    Ok(Value::Undefined)
}

fn height<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.stage.stage_size().1.into())
}

fn scale_mode<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let scale_mode = AvmString::new(
        activation.context.gc_context,
        activation.context.stage.scale_mode().to_string(),
    );
    Ok(scale_mode.into())
}

fn set_scale_mode<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let scale_mode = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?
        .parse()
        .unwrap_or_default();
    activation
        .context
        .stage
        .set_scale_mode(&mut activation.context, scale_mode);
    Ok(Value::Undefined)
}

fn show_menu<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.stage.show_menu().into())
}

fn set_show_menu<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let show_menu = args
        .get(0)
        .unwrap_or(&true.into())
        .to_owned()
        .as_bool(activation.swf_version());
    activation
        .context
        .stage
        .set_show_menu(&mut activation.context, show_menu);
    Ok(Value::Undefined)
}

fn width<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.stage.stage_size().0.into())
}
