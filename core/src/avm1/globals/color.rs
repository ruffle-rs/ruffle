//! Color object
//!
//! TODO: This should change when `ColorTransform` changes to match Flash's representation
//! (See GitHub #193)

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property::Attribute;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, ScriptObject, TObject, Value};
use crate::display_object::{DisplayObject, TDisplayObject};
use crate::string::StringContext;

use swf::Fixed8;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "getRGB" => method(get_rgb; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getTransform" => method(get_transform; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "setRGB" => method(set_rgb; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "setTransform" => method(set_transform; DONT_ENUM | DONT_DELETE | READ_ONLY);
};

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // The target display object that this color will modify.
    let target = args.get(0).cloned().unwrap_or(Value::Undefined);
    // Set undocumented `target` property
    this.define_value(
        activation.context.gc_context,
        "target",
        target,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );
    Ok(this.into())
}

pub fn create_proto<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, context, object, fn_proto);
    object.into()
}

/// Gets the target display object of this color transform.
fn target<'gc>(
    activation: &mut Activation<'_, 'gc>,
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
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(target) = target(activation, this)? {
        let base = target.base();
        let color_transform = base.color_transform();
        let r = ((color_transform.r_add) as i32) << 16;
        let g = ((color_transform.g_add) as i32) << 8;
        let b = (color_transform.b_add) as i32;
        Ok((r | g | b).into())
    } else {
        Ok(Value::Undefined)
    }
}

fn get_transform<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(target) = target(activation, this)? {
        let base = target.base();
        let color_transform = base.color_transform();
        let out = ScriptObject::new(
            activation.context.gc_context,
            Some(activation.context.avm1.prototypes().object),
        );
        out.set(
            "ra",
            (color_transform.r_multiply.to_f64() * 100.0).into(),
            activation,
        )?;
        out.set(
            "ga",
            (color_transform.g_multiply.to_f64() * 100.0).into(),
            activation,
        )?;
        out.set(
            "ba",
            (color_transform.b_multiply.to_f64() * 100.0).into(),
            activation,
        )?;
        out.set(
            "aa",
            (color_transform.a_multiply.to_f64() * 100.0).into(),
            activation,
        )?;
        out.set("rb", color_transform.r_add.into(), activation)?;
        out.set("gb", color_transform.g_add.into(), activation)?;
        out.set("bb", color_transform.b_add.into(), activation)?;
        out.set("ab", color_transform.a_add.into(), activation)?;
        Ok(out.into())
    } else {
        Ok(Value::Undefined)
    }
}

fn set_rgb<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(target) = target(activation, this)? {
        target.set_transformed_by_script(activation.context.gc_context, true);
        if let Some(parent) = target.parent() {
            parent.invalidate_cached_bitmap(activation.context.gc_context);
        }

        let rgb = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(activation)?;
        let [b, g, r, _] = rgb.to_le_bytes();

        let mut base = target.base_mut(activation.context.gc_context);
        let color_transform = base.color_transform_mut();
        color_transform.r_multiply = Fixed8::ZERO;
        color_transform.g_multiply = Fixed8::ZERO;
        color_transform.b_multiply = Fixed8::ZERO;
        color_transform.r_add = r.into();
        color_transform.g_add = g.into();
        color_transform.b_add = b.into();
    }
    Ok(Value::Undefined)
}

fn set_transform<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    fn set_color_mult<'gc>(
        activation: &mut Activation<'_, 'gc>,
        transform: Object<'gc>,
        property: &'static str,
        out: &mut Fixed8,
    ) -> Result<(), Error<'gc>> {
        // The parameters are set only if the property exists on the object itself (prototype excluded).
        if transform.has_own_property(activation, property.into()) {
            let n = transform
                .get(property, activation)?
                .coerce_to_f64(activation)?;
            *out = Fixed8::from_bits(crate::ecma_conversions::f64_to_wrapping_i16(
                n * 256.0 / 100.0,
            ));
        }
        Ok(())
    }

    fn set_color_add<'gc>(
        activation: &mut Activation<'_, 'gc>,
        transform: Object<'gc>,
        property: &'static str,
        out: &mut i16,
    ) -> Result<(), Error<'gc>> {
        // The parameters are set only if the property exists on the object itself (prototype excluded).
        if transform.has_own_property(activation, property.into()) {
            *out = transform
                .get(property, activation)?
                .coerce_to_i16(activation)?;
        }
        Ok(())
    }

    if let Some(target) = target(activation, this)? {
        target.set_transformed_by_script(activation.context.gc_context, true);
        if let Some(parent) = target.parent() {
            parent.invalidate_cached_bitmap(activation.context.gc_context);
        }

        let mut base = target.base_mut(activation.context.gc_context);
        let color_transform = base.color_transform_mut();
        let transform = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation);
        set_color_mult(activation, transform, "ra", &mut color_transform.r_multiply)?;
        set_color_mult(activation, transform, "ga", &mut color_transform.g_multiply)?;
        set_color_mult(activation, transform, "ba", &mut color_transform.b_multiply)?;
        set_color_mult(activation, transform, "aa", &mut color_transform.a_multiply)?;
        set_color_add(activation, transform, "rb", &mut color_transform.r_add)?;
        set_color_add(activation, transform, "gb", &mut color_transform.g_add)?;
        set_color_add(activation, transform, "bb", &mut color_transform.b_add)?;
        set_color_add(activation, transform, "ab", &mut color_transform.a_add)?;
    }

    Ok(Value::Undefined)
}
