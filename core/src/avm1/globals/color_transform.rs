//! ColorTransform object

use crate::avm1::color_transform_object::ColorTransformObject;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Object, ScriptObject, TObject, UpdateContext, Value};
use crate::color_transform::ColorTransform;
use enumset::EnumSet;
use gc_arena::MutationContext;
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
        .coerce_to_f64(avm, context)?;
    let green_multiplier = args
        .get(1)
        .unwrap_or(&Value::Number(1.into()))
        .coerce_to_f64(avm, context)?;
    let blue_multiplier = args
        .get(2)
        .unwrap_or(&Value::Number(1.into()))
        .coerce_to_f64(avm, context)?;
    let alpha_multiplier = args
        .get(3)
        .unwrap_or(&Value::Number(1.into()))
        .coerce_to_f64(avm, context)?;
    let red_offset = args
        .get(4)
        .unwrap_or(&Value::Number(0.into()))
        .coerce_to_f64(avm, context)?;
    let green_offset = args
        .get(5)
        .unwrap_or(&Value::Number(0.into()))
        .coerce_to_f64(avm, context)?;
    let blue_offset = args
        .get(6)
        .unwrap_or(&Value::Number(0.into()))
        .coerce_to_f64(avm, context)?;
    let alpha_offset = args
        .get(7)
        .unwrap_or(&Value::Number(0.into()))
        .coerce_to_f64(avm, context)?;

    let ct = this.as_color_transform_object().unwrap();
    ct.set_red_multiplier(context.gc_context, red_multiplier);
    ct.set_green_multiplier(context.gc_context, green_multiplier);
    ct.set_blue_multiplier(context.gc_context, blue_multiplier);
    ct.set_alpha_multiplier(context.gc_context, alpha_multiplier);
    ct.set_red_offset(context.gc_context, red_offset);
    ct.set_green_offset(context.gc_context, green_offset);
    ct.set_blue_offset(context.gc_context, blue_offset);
    ct.set_alpha_offset(context.gc_context, alpha_offset);

    Ok(Value::Undefined.into())
}

pub fn object_to_color_transform<'gc>(
    object: Object<'gc>,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
) -> Result<ColorTransform, Error<'gc>> {
    let red_multiplier = 0.0; /*object
                              .get("redMultiplier", avm, context)?
                              .coerce_to_f64(avm, context)? as f32;*/
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

    println!("Self: {:?}", this);

    let ct = this.as_color_transform_object().unwrap();

    /*ct.set_red_offset(context.gc_context, red);
    ct.set_green_offset(context.gc_context, green);
    ct.set_blue_offset(context.gc_context, blue);

    ct.set_red_multiplier(context.gc_context, 0.0);
    ct.set_green_multiplier(context.gc_context, 0.0);
    ct.set_blue_multiplier(context.gc_context, 0.0);*/

    Ok(Value::Undefined.into())
}

pub fn create_color_transform_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    color_transform_proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    FunctionObject::function(
        gc_context,
        Executable::Native(constructor),
        fn_proto,
        color_transform_proto,
    )
}

macro_rules! color_transform_value_accessor {
    ($([$get_ident: ident, $set_ident: ident],)*) => {
        $(
            pub fn $set_ident<'gc>(
                avm: &mut Avm1<'gc>,
                context: &mut UpdateContext<'_, 'gc, '_>,
                this: Object<'gc>,
                args: &[Value<'gc>],
            ) -> Result<ReturnValue<'gc>, Error<'gc>> {
                //TODO: update as comment
                let new_val = args
                    .get(0)
                    .unwrap_or(&Value::Undefined)
                    .coerce_to_f64(avm, context)?;

                let ct = this.as_color_transform_object().unwrap();
                ct.$set_ident(context.gc_context, new_val);
                Ok(Value::Undefined.into())
            }

            pub fn $get_ident<'gc>(
                _avm: &mut Avm1<'gc>,
                _context: &mut UpdateContext<'_, 'gc, '_>,
                this: Object<'gc>,
                _args: &[Value<'gc>],
            ) -> Result<ReturnValue<'gc>, Error<'gc>> {
                let ct = this.as_color_transform_object().unwrap();
                Ok(Value::Number(ct.$get_ident()).into())
            }
        )*
    }
}

color_transform_value_accessor!(
    [get_red_multiplier, set_red_multiplier],
    [get_green_multiplier, set_green_multiplier],
    [get_blue_multiplier, set_blue_multiplier],
    [get_alpha_multiplier, set_alpha_multiplier],
    [get_red_offset, set_red_offset],
    [get_green_offset, set_green_offset],
    [get_blue_offset, set_blue_offset],
    [get_alpha_offset, set_alpha_offset],
);

macro_rules! with_color_transform {
    ($obj: ident, $gc: ident, $($name: expr => [$get: ident, $set: ident],)*) => {
        $(
            $obj.add_property(
                $gc,
                $name,
                Executable::Native($get),
                Some(Executable::Native($set)),
                EnumSet::empty(),
            );
        )*
    }
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let color_transform_object =
        ColorTransformObject::empty_color_transform_object(gc_context, Some(proto));
    let mut object = color_transform_object.as_script_object().unwrap();

    with_color_transform!(object, gc_context,
        "rgb" => [get_rgb, set_rgb],
        "redMultiplier" => [get_red_multiplier, set_red_multiplier],
        "greenMultiplier" => [get_green_multiplier, set_green_multiplier],
        "blueMultiplier" => [get_blue_multiplier, set_blue_multiplier],
        "alphaMultiplier" => [get_alpha_multiplier, set_alpha_multiplier],
        "redOffset" => [get_red_offset, set_red_offset],
        "greenOffset" => [get_green_offset, set_green_offset],
        "blueOffset" => [get_blue_offset, set_blue_offset],
        "alphaOffset" => [get_alpha_offset, set_alpha_offset],
    );

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

    color_transform_object.into()
}

fn to_string<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    let ct = object_to_color_transform(this, avm, context)?;

    let formatted = format!("(redMultiplier={}, greenMultiplier={}, blueMultiplier={}, alphaMultiplier={}, redOffset={}, greenOffset={}, blueOffset={}, alphaOffset={})",
            this.get("redMultiplier", avm, context)?.coerce_to_string(avm, context)?,
            this.get("greenMultiplier", avm, context)?.coerce_to_string(avm, context)?,
            this.get("blueMultiplier", avm, context)?.coerce_to_string(avm, context)?,
            this.get("alphaMultiplier", avm, context)?.coerce_to_string(avm, context)?,
            this.get("redOffset", avm, context)?.coerce_to_string(avm, context)?,
            this.get("greenOffset", avm, context)?.coerce_to_string(avm, context)?,
            this.get("blueOffset", avm, context)?.coerce_to_string(avm, context)?,
            this.get("alphaOffset", avm, context)?.coerce_to_string(avm, context)?
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

        let other_ct = other.as_color_transform_object().unwrap();
        let this_ct = this.as_color_transform_object().unwrap();

        let red_multiplier = other_ct.get_red_multiplier() * this_ct.get_red_multiplier();
        let green_multiplier = other_ct.get_green_multiplier() * this_ct.get_green_multiplier();
        let blue_multiplier = other_ct.get_blue_multiplier() * this_ct.get_blue_multiplier();
        let alpha_multiplier = other_ct.get_alpha_multiplier() * this_ct.get_alpha_multiplier();
        let red_offset = (other_ct.get_red_multiplier() * this_ct.get_red_multiplier())
            + this_ct.get_red_multiplier();
        let green_offset = (other_ct.get_green_offset() * this_ct.get_green_multiplier())
            + this_ct.get_green_offset();
        let blue_offset = (other_ct.get_blue_offset() * this_ct.get_blue_multiplier())
            + this_ct.get_blue_offset();
        let alpha_offset = (other_ct.get_alpha_offset() * this_ct.get_alpha_multiplier())
            + this_ct.get_alpha_offset();

        // This appears to do nothing if values are out of range
        if 0.0 > red_multiplier
            || red_multiplier > 1.0
            || 0.0 > green_multiplier
            || green_multiplier > 1.0
            || 0.0 > blue_multiplier
            || blue_multiplier > 1.0
            || 0.0 > alpha_multiplier
            || alpha_multiplier > 1.0
            || -255.0 > red_offset
            || red_offset > 255.0
            || -255.0 > green_offset
            || green_offset > 255.0
            || -255.0 > blue_offset
            || blue_offset > 255.0
            || -255.0 > alpha_offset
            || alpha_offset > 255.0
        {
            return Ok(Value::Undefined.into());
        }

        this_ct.set_red_multiplier(context.gc_context, red_multiplier);
        this_ct.set_green_multiplier(context.gc_context, green_multiplier);
        this_ct.set_blue_multiplier(context.gc_context, blue_multiplier);
        this_ct.set_alpha_multiplier(context.gc_context, alpha_multiplier);
        this_ct.set_red_offset(context.gc_context, red_offset);
        this_ct.set_green_offset(context.gc_context, green_offset);
        this_ct.set_blue_offset(context.gc_context, blue_offset);
        this_ct.set_alpha_offset(context.gc_context, alpha_offset);
    }

    Ok(Value::Undefined.into())
}
