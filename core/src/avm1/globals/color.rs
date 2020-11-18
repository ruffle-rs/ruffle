//! Color object
//!
//! TODO: This should change when `ColorTransform` changes to match Flash's representation
//! (See GitHub #193)

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property::Attribute::*;
use crate::avm1::{Object, ScriptObject, TObject, Value};
use crate::display_object::{DisplayObject, TDisplayObject};
use enumset::EnumSet;
use gc_arena::MutationContext;

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // The target display object that this color will modify.
    let target = args.get(0).cloned().unwrap_or(Value::Undefined);
    // Set undocumented `target` property
    this.set("target", target, activation)?;
    this.set_attributes(
        activation.context.gc_context,
        Some("target"),
        DontDelete | ReadOnly | DontEnum,
        EnumSet::empty(),
    );

    Ok(Value::Undefined)
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
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
) -> Result<Option<DisplayObject<'gc>>, Error<'gc>> {
    // The target path resolves based on the active tellTarget clip of the stack frame.
    // This means calls on the same `Color` object could set the color of different clips
    // depending on which timeline its called from!
    let target = this.get("target", activation)?;
    // Undefined or empty target is no-op.
    if target != Value::Undefined {
        let start_clip = activation.target_clip_or_root();
        activation.resolve_target_display_object(start_clip, target, false)
    } else {
        Ok(None)
    }
}

fn get_rgb<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(target) = target(activation, this)? {
        let color_transform = target.color_transform();
        let r = ((color_transform.r_add * 255.0) as i32) << 16;
        let g = ((color_transform.g_add * 255.0) as i32) << 8;
        let b = (color_transform.b_add * 255.0) as i32;
        Ok((r | g | b).into())
    } else {
        Ok(Value::Undefined)
    }
}

fn get_transform<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(target) = target(activation, this)? {
        let color_transform = target.color_transform();
        let out = ScriptObject::object(
            activation.context.gc_context,
            Some(activation.context.avm1.prototypes.object),
        );
        out.set("ra", (color_transform.r_mult * 100.0).into(), activation)?;
        out.set("ga", (color_transform.g_mult * 100.0).into(), activation)?;
        out.set("ba", (color_transform.b_mult * 100.0).into(), activation)?;
        out.set("aa", (color_transform.a_mult * 100.0).into(), activation)?;
        out.set("rb", (color_transform.r_add * 255.0).into(), activation)?;
        out.set("gb", (color_transform.g_add * 255.0).into(), activation)?;
        out.set("bb", (color_transform.b_add * 255.0).into(), activation)?;
        out.set("ab", (color_transform.a_add * 255.0).into(), activation)?;
        Ok(out.into())
    } else {
        Ok(Value::Undefined)
    }
}

fn set_rgb<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(target) = target(activation, this)? {
        let mut color_transform = target.color_transform_mut(activation.context.gc_context);
        let rgb = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_f64(activation)? as i32;
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
    Ok(Value::Undefined)
}

fn set_transform<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // TODO: These map from the 0-100% range for mult and the -255-255 range for addition used by ActionScript
    // to the 16-bit range used by the internal representations of the Flash Player.
    // This will get slightly simpler when we change ColorTransform to the proper representation (see #193).
    fn set_color_mult<'gc>(
        activation: &mut Activation<'_, 'gc, '_>,
        transform: Object<'gc>,
        property: &str,
        out: &mut f32,
    ) -> Result<(), Error<'gc>> {
        // The parameters are set only if the property exists on the object itself (prototype excluded).
        if transform.has_own_property(activation, property) {
            let n = transform
                .get(property, activation)?
                .coerce_to_f64(activation)?;
            *out = f32::from(crate::ecma_conversions::f64_to_wrapping_i16(n * 2.56)) / 256.0
        }
        Ok(())
    }

    fn set_color_add<'gc>(
        activation: &mut Activation<'_, 'gc, '_>,
        transform: Object<'gc>,
        property: &str,
        out: &mut f32,
    ) -> Result<(), Error<'gc>> {
        // The parameters are set only if the property exists on the object itself (prototype excluded).
        if transform.has_own_property(activation, property) {
            let n = transform
                .get(property, activation)?
                .coerce_to_f64(activation)?;
            *out = f32::from(crate::ecma_conversions::f64_to_wrapping_i16(n)) / 255.0
        }
        Ok(())
    }

    if let Some(target) = target(activation, this)? {
        let mut color_transform = target.color_transform_mut(activation.context.gc_context);
        let transform = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation);
        set_color_mult(activation, transform, "ra", &mut color_transform.r_mult)?;
        set_color_mult(activation, transform, "ga", &mut color_transform.g_mult)?;
        set_color_mult(activation, transform, "ba", &mut color_transform.b_mult)?;
        set_color_mult(activation, transform, "aa", &mut color_transform.a_mult)?;
        set_color_add(activation, transform, "rb", &mut color_transform.r_add)?;
        set_color_add(activation, transform, "gb", &mut color_transform.g_add)?;
        set_color_add(activation, transform, "bb", &mut color_transform.b_add)?;
        set_color_add(activation, transform, "ab", &mut color_transform.a_add)?;
    }

    Ok(Value::Undefined)
}
