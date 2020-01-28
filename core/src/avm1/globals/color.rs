//! Color object
//!
//! TODO: This should change when `ColorTransform` changes to match Flash's representation
//! (See GitHub #193)

use crate::avm1::property::Attribute::*;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, ScriptObject, TObject, UpdateContext, Value};
use crate::display_object::{DisplayObject, TDisplayObject};
use enumset::EnumSet;
use gc_arena::MutationContext;

pub fn constructor<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    mut this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    // The target display object that this color will modify.
    let target = args.get(0).cloned().unwrap_or(Value::Undefined);
    // Set undocumented `target` property
    this.set("target", target, avm, context)?;
    this.set_attributes(
        context.gc_context,
        Some("target"),
        DontDelete | ReadOnly | DontEnum,
        EnumSet::empty(),
    );

    Ok(Value::Undefined.into())
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let mut object = ScriptObject::object(gc_context, Some(proto));

    object.force_set_function(
        "getRGB",
        get_rgb,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );

    object.force_set_function(
        "getTransform",
        get_transform,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );

    object.force_set_function(
        "setRGB",
        set_rgb,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );

    object.force_set_function(
        "setTransform",
        set_transform,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );

    object.into()
}

/// Gets the target display object of this color transform.
fn target<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
) -> Result<Option<DisplayObject<'gc>>, Error> {
    let target = this.get("target", avm, context)?.resolve(avm, context)?;
    avm.resolve_target_display_object(context, target)
}

fn get_rgb<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(target) = target(avm, context, this)? {
        let color_transform = target.color_transform();
        let r = ((color_transform.r_add * 255.0) as i32) << 16;
        let g = ((color_transform.g_add * 255.0) as i32) << 8;
        let b = (color_transform.b_add * 255.0) as i32;
        Ok((r | g | b).into())
    } else {
        Ok(Value::Undefined.into())
    }
}

fn get_transform<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(target) = target(avm, context, this)? {
        let color_transform = target.color_transform();
        let out = ScriptObject::object(context.gc_context, Some(avm.prototypes.object));
        out.set("ra", (color_transform.r_mult * 100.0).into(), avm, context)?;
        out.set("ga", (color_transform.g_mult * 100.0).into(), avm, context)?;
        out.set("ba", (color_transform.b_mult * 100.0).into(), avm, context)?;
        out.set("aa", (color_transform.a_mult * 100.0).into(), avm, context)?;
        out.set("rb", (color_transform.r_add * 255.0).into(), avm, context)?;
        out.set("gb", (color_transform.g_add * 255.0).into(), avm, context)?;
        out.set("bb", (color_transform.b_add * 255.0).into(), avm, context)?;
        out.set("ab", (color_transform.a_add * 255.0).into(), avm, context)?;
        Ok(out.into())
    } else {
        Ok(Value::Undefined.into())
    }
}

fn set_rgb<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(target) = target(avm, context, this)? {
        let mut color_transform = target.color_transform_mut(context.gc_context);
        let rgb = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .as_number(avm, context)? as i32;
        let r = (((rgb >> 16) & 0xff) as f32) / 255.0;
        let g = (((rgb >> 8) & 0xff) as f32) / 255.0;
        let b = ((rgb & 0xff) as f32) / 255.0;

        color_transform.r_mult = 0.0;
        color_transform.g_mult = 0.0;
        color_transform.b_mult = 0.0;
        color_transform.r_add = r;
        color_transform.g_add = g;
        color_transform.b_add = b;
    }
    Ok(Value::Undefined.into())
}

fn set_transform<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(target) = target(avm, context, this)? {
        let mut color_transform = target.color_transform_mut(context.gc_context);
        if let Ok(transform) = args.get(0).unwrap_or(&Value::Undefined).as_object() {
            color_transform.r_mult = transform
                .get("ra", avm, context)?
                .resolve(avm, context)?
                .as_number(avm, context)? as f32
                / 100.0;
            color_transform.g_mult = transform
                .get("ga", avm, context)?
                .resolve(avm, context)?
                .as_number(avm, context)? as f32
                / 100.0;
            color_transform.b_mult = transform
                .get("ba", avm, context)?
                .resolve(avm, context)?
                .as_number(avm, context)? as f32
                / 100.0;
            color_transform.a_mult = transform
                .get("aa", avm, context)?
                .resolve(avm, context)?
                .as_number(avm, context)? as f32
                / 100.0;
            color_transform.r_add = transform
                .get("rb", avm, context)?
                .resolve(avm, context)?
                .as_number(avm, context)? as f32
                / 255.0;
            color_transform.g_add = transform
                .get("gb", avm, context)?
                .resolve(avm, context)?
                .as_number(avm, context)? as f32
                / 255.0;
            color_transform.b_add = transform
                .get("bb", avm, context)?
                .resolve(avm, context)?
                .as_number(avm, context)? as f32
                / 255.0;
            color_transform.a_add = transform
                .get("ab", avm, context)?
                .resolve(avm, context)?
                .as_number(avm, context)? as f32
                / 255.0;
        }
    }

    Ok(Value::Undefined.into())
}
