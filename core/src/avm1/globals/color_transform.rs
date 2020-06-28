//! ColorTransform object

use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Object, ScriptObject, TObject, UpdateContext, Value};
use crate::color_transform::ColorTransform;
use enumset::EnumSet;
use gc_arena::MutationContext;
use std::borrow::ToOwned;
use std::convert::Into;

pub fn constructor<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    let red_multiplier = args
        .get(0)
        .unwrap_or(&Value::Number(1.into()))
        .to_owned()
        .coerce_to_f64(avm, context)?;
    let green_multiplier = args
        .get(1)
        .unwrap_or(&Value::Number(1.into()))
        .to_owned()
        .coerce_to_f64(avm, context)?;
    let blue_multiplier = args
        .get(2)
        .unwrap_or(&Value::Number(1.into()))
        .to_owned()
        .coerce_to_f64(avm, context)?;
    let alpha_multiplier = args
        .get(3)
        .unwrap_or(&Value::Number(1.into()))
        .to_owned()
        .coerce_to_f64(avm, context)?;
    let red_offset = args
        .get(4)
        .unwrap_or(&Value::Number(0.into()))
        .to_owned()
        .coerce_to_f64(avm, context)?;
    let green_offset = args
        .get(5)
        .unwrap_or(&Value::Number(0.into()))
        .to_owned()
        .coerce_to_f64(avm, context)?;
    let blue_offset = args
        .get(6)
        .unwrap_or(&Value::Number(0.into()))
        .to_owned()
        .coerce_to_f64(avm, context)?;
    let alpha_offset = args
        .get(7)
        .unwrap_or(&Value::Number(0.into()))
        .to_owned()
        .coerce_to_f64(avm, context)?;

    let ct = ColorTransform {
        r_mult: red_multiplier as f32,
        g_mult: green_multiplier as f32,
        b_mult: blue_multiplier as f32,
        a_mult: alpha_multiplier as f32,
        r_add: red_offset as f32,
        g_add: green_offset as f32,
        b_add: blue_offset as f32,
        a_add: alpha_offset as f32,
    };

    apply_color_transform_to_object(ct, this, avm, context)?;

    Ok(Value::Undefined.into())
}

pub fn apply_color_transform_to_object<'gc>(
    color_transform: ColorTransform,
    object: Object<'gc>,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
) -> Result<(), Error<'gc>> {
    object.set("redMultiplier", color_transform.r_mult.into(), avm, context)?;
    object.set(
        "greenMultiplier",
        color_transform.g_mult.into(),
        avm,
        context,
    )?;
    object.set(
        "blueMultiplier",
        color_transform.b_mult.into(),
        avm,
        context,
    )?;
    object.set(
        "alphaMultiplier",
        color_transform.a_mult.into(),
        avm,
        context,
    )?;
    object.set("redOffset", color_transform.r_add.into(), avm, context)?;
    object.set("greenOffset", color_transform.g_add.into(), avm, context)?;
    object.set("blueOffset", color_transform.b_add.into(), avm, context)?;
    object.set("alphaOffset", color_transform.a_add.into(), avm, context)?;
    Ok(())
}

pub fn object_to_color_transform<'gc>(
    object: Object<'gc>,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
) -> Result<ColorTransform, Error<'gc>> {
    let red_multiplier = object
        .get("redMultiplier", avm, context)?
        .coerce_to_f64(avm, context)? as f32;
    let green_multiplier = object
        .get("greenMultiplier", avm, context)?
        .coerce_to_f64(avm, context)? as f32;
    let blue_multiplier = object
        .get("blueMultiplier", avm, context)?
        .coerce_to_f64(avm, context)? as f32;
    let alpha_multiplier = object
        .get("alphaMultiplier", avm, context)?
        .coerce_to_f64(avm, context)? as f32;
    let red_offset = object
        .get("redOffset", avm, context)?
        .coerce_to_f64(avm, context)? as f32;
    let green_offset = object
        .get("greenOffset", avm, context)?
        .coerce_to_f64(avm, context)? as f32;
    let blue_offset = object
        .get("blueOffset", avm, context)?
        .coerce_to_f64(avm, context)? as f32;
    let alpha_offset = object
        .get("alphaOffset", avm, context)?
        .coerce_to_f64(avm, context)? as f32;

    Ok(ColorTransform {
        r_mult: red_multiplier,
        g_mult: green_multiplier,
        b_mult: blue_multiplier,
        a_mult: alpha_multiplier,
        r_add: red_offset,
        g_add: green_offset,
        b_add: blue_offset,
        a_add: alpha_offset,
    })
}

pub fn get_rgb<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    let ct = object_to_color_transform(this, avm, context)?;
    let rgb = ((ct.r_add as u32) << 16) | ((ct.g_add as u32) << 8) | (ct.b_add as u32);
    Ok(Value::Number(rgb.into()).into())
}

pub fn set_rgb<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    let new_rgb = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(avm, context)? as u64;

    let red = ((new_rgb >> 16) & 0xFF) as f64;
    let green = ((new_rgb >> 8) & 0xFF) as f64;
    let blue = (new_rgb & 0xFF) as f64;

    this.set("redOffset", red.into(), avm, context)?;
    this.set("greenOffset", green.into(), avm, context)?;
    this.set("blueOffset", blue.into(), avm, context)?;

    this.set("redMultiplier", 0.into(), avm, context)?;
    this.set("greenMultiplier", 0.into(), avm, context)?;
    this.set("blueMultiplier", 0.into(), avm, context)?;

    Ok(Value::Undefined.into())
}

pub fn create_color_transform_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    point_proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    FunctionObject::function(
        gc_context,
        Executable::Native(constructor),
        fn_proto,
        point_proto,
    )
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let mut object = ScriptObject::object(gc_context, Some(proto));

    object.force_set_function(
        "concat",
        concat,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "toString",
        to_string,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.add_property(
        gc_context,
        "rgb",
        Executable::Native(get_rgb),
        Some(Executable::Native(set_rgb)),
        EnumSet::empty(),
    );

    object.into()
}

fn to_string<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    let ct = object_to_color_transform(this, avm, context)?;

    let formatted = format!("(redMultiplier={}, greenMultiplier={}, blueMultiplier={}, alphaMultiplier={}, redOffset={}, greenOffset={}, blueOffset={}, alphaOffset={})",
            ct.r_mult,
            ct.g_mult,
            ct.b_mult,
            ct.a_mult,
            ct.r_add,
            ct.g_add,
            ct.b_add,
            ct.a_add
    );

    Ok(Value::String(formatted).into())
}

fn concat<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    // Without an arg, does nothing
    if let Some(arg) = _args.get(0) {
        // If given an invalid var, return undefined
        if arg == &Value::Undefined {
            return Ok(Value::Undefined.into());
        }
        let other = arg.coerce_to_object(avm, context);
        let other_ct: ColorTransform = object_to_color_transform(other, avm, context)?;
        let this_ct: ColorTransform = object_to_color_transform(this, avm, context)?;

        let new_ct = ColorTransform {
            r_mult: other_ct.r_mult * this_ct.r_mult,
            g_mult: other_ct.g_mult * this_ct.g_mult,
            b_mult: other_ct.b_mult * this_ct.b_mult,
            a_mult: other_ct.a_mult * this_ct.a_mult,
            r_add: (other_ct.r_add * this_ct.r_mult) + this_ct.r_add,
            g_add: (other_ct.g_add * this_ct.g_mult) + this_ct.g_add,
            b_add: (other_ct.b_add * this_ct.b_mult) + this_ct.b_add,
            a_add: (other_ct.a_add * this_ct.a_mult) + this_ct.a_add,
        };

        // This appears to do nothing if values are out of range
        if 0.0 > new_ct.r_mult
            || new_ct.r_mult > 1.0
            || 0.0 > new_ct.g_mult
            || new_ct.g_mult > 1.0
            || 0.0 > new_ct.b_mult
            || new_ct.b_mult > 1.0
            || 0.0 > new_ct.a_mult
            || new_ct.a_mult > 1.0
            || -255.0 > new_ct.r_add
            || new_ct.r_add > 255.0
            || -255.0 > new_ct.g_add
            || new_ct.g_add > 255.0
            || -255.0 > new_ct.b_add
            || new_ct.b_add > 255.0
            || -255.0 > new_ct.a_add
            || new_ct.a_add > 255.0
        {
            return Ok(Value::Undefined.into());
        }


        apply_color_transform_to_object(new_ct, this, avm, context)?;
    }

    Ok(Value::Undefined.into())
}
