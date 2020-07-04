//! ColorTransform object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::Executable;
use crate::avm1::{Object, TObject, Value};
use crate::context::UpdateContext;
use enumset::EnumSet;
use gc_arena::{Gc, MutationContext};

use crate::avm1::object::color_transform_object::ColorTransformObject;
use crate::color_transform::ColorTransform;
use std::convert::Into;

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

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let red_multiplier = args
        .get(0)
        .unwrap_or(&Value::Number(1.into()))
        .coerce_to_f64(activation, context)?;
    let green_multiplier = args
        .get(1)
        .unwrap_or(&Value::Number(1.into()))
        .coerce_to_f64(activation, context)?;
    let blue_multiplier = args
        .get(2)
        .unwrap_or(&Value::Number(1.into()))
        .coerce_to_f64(activation, context)?;
    let alpha_multiplier = args
        .get(3)
        .unwrap_or(&Value::Number(1.into()))
        .coerce_to_f64(activation, context)?;
    let red_offset = args
        .get(4)
        .unwrap_or(&Value::Number(0.into()))
        .coerce_to_f64(activation, context)?;
    let green_offset = args
        .get(5)
        .unwrap_or(&Value::Number(0.into()))
        .coerce_to_f64(activation, context)?;
    let blue_offset = args
        .get(6)
        .unwrap_or(&Value::Number(0.into()))
        .coerce_to_f64(activation, context)?;
    let alpha_offset = args
        .get(7)
        .unwrap_or(&Value::Number(0.into()))
        .coerce_to_f64(activation, context)?;

    let ct = this.as_color_transform_object().unwrap();
    ct.set_red_multiplier(context.gc_context, red_multiplier);
    ct.set_green_multiplier(context.gc_context, green_multiplier);
    ct.set_blue_multiplier(context.gc_context, blue_multiplier);
    ct.set_alpha_multiplier(context.gc_context, alpha_multiplier);
    ct.set_red_offset(context.gc_context, red_offset);
    ct.set_green_offset(context.gc_context, green_offset);
    ct.set_blue_offset(context.gc_context, blue_offset);
    ct.set_alpha_offset(context.gc_context, alpha_offset);

    Ok(Value::Undefined)
}

// We'll need this soon!
#[allow(dead_code)]
pub fn object_to_color_transform<'gc>(
    object: Object<'gc>,
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
) -> Result<ColorTransform, Error<'gc>> {
    let red_multiplier = object
        .get("redMultiplier", activation, context)?
        .coerce_to_f64(activation, context)? as f32;
    let green_multiplier = object
        .get("greenMultiplier", activation, context)?
        .coerce_to_f64(activation, context)? as f32;
    let blue_multiplier = object
        .get("blueMultiplier", activation, context)?
        .coerce_to_f64(activation, context)? as f32;
    let alpha_multiplier = object
        .get("alphaMultiplier", activation, context)?
        .coerce_to_f64(activation, context)? as f32;
    let red_offset = object
        .get("redOffset", activation, context)?
        .coerce_to_f64(activation, context)? as f32;
    let green_offset = object
        .get("greenOffset", activation, context)?
        .coerce_to_f64(activation, context)? as f32;
    let blue_offset = object
        .get("blueOffset", activation, context)?
        .coerce_to_f64(activation, context)? as f32;
    let alpha_offset = object
        .get("alphaOffset", activation, context)?
        .coerce_to_f64(activation, context)? as f32;

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
    _activation: &mut Activation<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let ct = this.as_color_transform_object().unwrap();

    let rgb = ((ct.get_red_offset() as u32) << 16)
        | ((ct.get_green_offset() as u32) << 8)
        | (ct.get_blue_offset() as u32);
    Ok(Value::Number(rgb.into()))
}

pub fn set_rgb<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let new_rgb = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_u32(activation, context)?;

    let red = ((new_rgb >> 16) & 0xFF) as f64;
    let green = ((new_rgb >> 8) & 0xFF) as f64;
    let blue = (new_rgb & 0xFF) as f64;

    let ct = this.as_color_transform_object().unwrap();

    ct.set_red_offset(context.gc_context, red);
    ct.set_green_offset(context.gc_context, green);
    ct.set_blue_offset(context.gc_context, blue);

    ct.set_red_multiplier(context.gc_context, 0.0);
    ct.set_green_multiplier(context.gc_context, 0.0);
    ct.set_blue_multiplier(context.gc_context, 0.0);

    Ok(Value::Undefined)
}

macro_rules! color_transform_value_accessor {
    ($([$get_ident: ident, $set_ident: ident],)*) => {
        $(
            pub fn $set_ident<'gc>(
                activation: &mut Activation<'_, 'gc>,
                context: &mut UpdateContext<'_, 'gc, '_>,
                this: Object<'gc>,
                args: &[Value<'gc>],
            ) -> Result<Value<'gc>, Error<'gc>> {
                let new_val = args
                    .get(0)
                    .unwrap_or(&Value::Undefined)
                    .coerce_to_f64(activation, context)?;

                let ct = this.as_color_transform_object().unwrap();
                ct.$set_ident(context.gc_context, new_val);
                Ok(Value::Undefined.into())
            }

            pub fn $get_ident<'gc>(
                _activation: &mut Activation<'_, 'gc>,
                _context: &mut UpdateContext<'_, 'gc, '_>,
                this: Object<'gc>,
                _args: &[Value<'gc>],
            ) -> Result<Value<'gc>, Error<'gc>> {
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

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let color_transform_object =
        ColorTransformObject::empty_color_transform_object(gc_context, Some(proto));
    let mut object = color_transform_object.as_script_object().unwrap();

    with_color_transform!(object, gc_context,
        "alphaMultiplier" => [get_alpha_multiplier, set_alpha_multiplier],
        "redMultiplier" => [get_red_multiplier, set_red_multiplier],
        "greenMultiplier" => [get_green_multiplier, set_green_multiplier],
        "blueMultiplier" => [get_blue_multiplier, set_blue_multiplier],
        "alphaOffset" => [get_alpha_offset, set_alpha_offset],
        "redOffset" => [get_red_offset, set_red_offset],
        "greenOffset" => [get_green_offset, set_green_offset],
        "blueOffset" => [get_blue_offset, set_blue_offset],
        "rgb" => [get_rgb, set_rgb],
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
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let formatted = format!("(redMultiplier={}, greenMultiplier={}, blueMultiplier={}, alphaMultiplier={}, redOffset={}, greenOffset={}, blueOffset={}, alphaOffset={})",
            this.get("redMultiplier", activation, context)?.coerce_to_string(activation, context)?,
            this.get("greenMultiplier", activation, context)?.coerce_to_string(activation, context)?,
            this.get("blueMultiplier", activation, context)?.coerce_to_string(activation, context)?,
            this.get("alphaMultiplier", activation, context)?.coerce_to_string(activation, context)?,
            this.get("redOffset", activation, context)?.coerce_to_string(activation, context)?,
            this.get("greenOffset", activation, context)?.coerce_to_string(activation, context)?,
            this.get("blueOffset", activation, context)?.coerce_to_string(activation, context)?,
            this.get("alphaOffset", activation, context)?.coerce_to_string(activation, context)?
    );

    Ok(Value::String(Gc::allocate(context.gc_context, formatted)))
}

fn concat<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Without an arg, does nothing
    if let Some(arg) = _args.get(0) {
        // If given an invalid var, return undefined
        if arg == &Value::Undefined {
            return Ok(Value::Undefined);
        }
        let other = arg.coerce_to_object(activation, context);

        let other_ct = other.as_color_transform_object().unwrap();
        let this_ct = this.as_color_transform_object().unwrap();

        let red_multiplier = other_ct.get_red_multiplier() * this_ct.get_red_multiplier();
        let green_multiplier = other_ct.get_green_multiplier() * this_ct.get_green_multiplier();
        let blue_multiplier = other_ct.get_blue_multiplier() * this_ct.get_blue_multiplier();
        let alpha_multiplier = other_ct.get_alpha_multiplier() * this_ct.get_alpha_multiplier();
        let red_offset =
            (other_ct.get_red_offset() * this_ct.get_red_multiplier()) + this_ct.get_red_offset();
        let green_offset = (other_ct.get_green_offset() * this_ct.get_green_multiplier())
            + this_ct.get_green_offset();
        let blue_offset = (other_ct.get_blue_offset() * this_ct.get_blue_multiplier())
            + this_ct.get_blue_offset();
        let alpha_offset = (other_ct.get_alpha_offset() * this_ct.get_alpha_multiplier())
            + this_ct.get_alpha_offset();

        this_ct.set_red_multiplier(context.gc_context, red_multiplier);
        this_ct.set_green_multiplier(context.gc_context, green_multiplier);
        this_ct.set_blue_multiplier(context.gc_context, blue_multiplier);
        this_ct.set_alpha_multiplier(context.gc_context, alpha_multiplier);
        this_ct.set_red_offset(context.gc_context, red_offset);
        this_ct.set_green_offset(context.gc_context, green_offset);
        this_ct.set_blue_offset(context.gc_context, blue_offset);
        this_ct.set_alpha_offset(context.gc_context, alpha_offset);
    }

    Ok(Value::Undefined)
}
