//! flash.filter.DisplacementMapFilter object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::displacement_map_filter::DisplacementMapFilterObject;
use crate::avm1::{AvmString, Object, TObject, Value};
use enumset::EnumSet;
use gc_arena::MutationContext;

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    set_map_bitmap(activation, this, args.get(0..1).unwrap_or(&[]))?;
    set_map_point(activation, this, args.get(1..2).unwrap_or(&[]))?;
    set_component_x(activation, this, args.get(2..3).unwrap_or(&[]))?;
    set_component_y(activation, this, args.get(3..4).unwrap_or(&[]))?;
    set_scale_x(activation, this, args.get(4..5).unwrap_or(&[]))?;
    set_scale_y(activation, this, args.get(5..6).unwrap_or(&[]))?;
    set_mode(activation, this, args.get(6..7).unwrap_or(&[]))?;
    set_color(activation, this, args.get(7..8).unwrap_or(&[]))?;
    set_alpha(activation, this, args.get(8..9).unwrap_or(&[]))?;

    Ok(Value::Undefined)
}

pub fn alpha<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(filter) = this.as_displacement_map_filter_object() {
        return Ok(filter.alpha().into());
    }

    Ok(Value::Undefined)
}

pub fn set_alpha<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let alpha = args
        .get(0)
        .unwrap_or(&0.0.into())
        .coerce_to_f64(activation)
        .map(|x| x.max(0.0).min(1.0))?;

    if let Some(filter) = this.as_displacement_map_filter_object() {
        filter.set_alpha(activation.context.gc_context, alpha);

        return Ok(filter.alpha().into());
    }

    Ok(Value::Undefined)
}

pub fn color<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(object) = this.as_displacement_map_filter_object() {
        return Ok(object.color().into());
    }

    Ok(Value::Undefined)
}

pub fn set_color<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let color = args
        .get(0)
        .unwrap_or(&0x000000.into())
        .coerce_to_u32(activation)?;

    if let Some(object) = this.as_displacement_map_filter_object() {
        object.set_color(activation.context.gc_context, color & 0xFFFFFF);
    }

    Ok(Value::Undefined)
}

pub fn component_x<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(object) = this.as_displacement_map_filter_object() {
        return Ok(object.component_x().into());
    }

    Ok(Value::Undefined)
}

pub fn set_component_x<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let component = args
        .get(0)
        .unwrap_or(&0.0.into())
        .coerce_to_i32(activation)?;

    if let Some(object) = this.as_displacement_map_filter_object() {
        object.set_component_x(activation.context.gc_context, component);
    }

    Ok(Value::Undefined)
}

pub fn component_y<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(object) = this.as_displacement_map_filter_object() {
        return Ok(object.component_y().into());
    }

    Ok(Value::Undefined)
}

pub fn set_component_y<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let component = args
        .get(0)
        .unwrap_or(&0.0.into())
        .coerce_to_i32(activation)?;

    if let Some(object) = this.as_displacement_map_filter_object() {
        object.set_component_y(activation.context.gc_context, component);
    }

    Ok(Value::Undefined)
}

pub fn map_bitmap<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(object) = this.as_displacement_map_filter_object() {
        if let Some(map_bitmap) = object.map_bitmap() {
            return Ok(map_bitmap.into());
        }
    }

    Ok(Value::Undefined)
}

pub fn set_map_bitmap<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation);

    if let Some(object) = this.as_displacement_map_filter_object() {
        if bitmap.as_bitmap_data_object().is_some() {
            object.set_map_bitmap(activation.context.gc_context, Some(bitmap));
        }
    }

    Ok(Value::Undefined)
}

pub fn map_point<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(object) = this.as_displacement_map_filter_object() {
        let (x, y) = object.map_point();

        let proto = activation.context.avm1.prototypes.point_constructor;
        let point = proto.construct(activation, &[x.into(), y.into()])?;
        return Ok(point.into());
    }

    Ok(Value::Undefined)
}

pub fn set_map_point<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let obj = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation);

    let x = obj.get("x", activation)?.coerce_to_i32(activation)?;
    let y = obj.get("y", activation)?.coerce_to_i32(activation)?;

    if let Some(object) = this.as_displacement_map_filter_object() {
        object.set_map_point(activation.context.gc_context, (x, y));
    }

    Ok(Value::Undefined)
}

pub fn mode<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(object) = this.as_displacement_map_filter_object() {
        return Ok(Value::String(AvmString::new(
            activation.context.gc_context,
            String::from(object.mode()),
        )));
    }

    Ok(Value::Undefined)
}

pub fn set_mode<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let component = args
        .get(0)
        .unwrap_or(&Value::String(AvmString::new(
            activation.context.gc_context,
            "wrap".to_string(),
        )))
        .coerce_to_string(activation)?;

    if let Some(object) = this.as_displacement_map_filter_object() {
        object.set_mode(activation.context.gc_context, component.as_str().into());
    }

    Ok(Value::Undefined)
}

pub fn scale_x<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(object) = this.as_displacement_map_filter_object() {
        return Ok(object.scale_x().into());
    }

    Ok(Value::Undefined)
}

pub fn set_scale_x<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let scale = args
        .get(0)
        .unwrap_or(&0.0.into())
        .coerce_to_f64(activation)?;

    if let Some(object) = this.as_displacement_map_filter_object() {
        object.set_scale_x(activation.context.gc_context, scale);
    }

    Ok(Value::Undefined)
}

pub fn scale_y<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(object) = this.as_displacement_map_filter_object() {
        return Ok(object.scale_y().into());
    }

    Ok(Value::Undefined)
}

pub fn set_scale_y<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let scale = args
        .get(0)
        .unwrap_or(&0.0.into())
        .coerce_to_f64(activation)?;

    if let Some(object) = this.as_displacement_map_filter_object() {
        object.set_scale_y(activation.context.gc_context, scale);
    }

    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let filter = DisplacementMapFilterObject::empty_object(gc_context, Some(proto));
    let object = filter.as_script_object().unwrap();

    object.add_property(
        gc_context,
        "alpha",
        FunctionObject::function(
            gc_context,
            Executable::Native(alpha),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_alpha),
            Some(fn_proto),
            fn_proto,
        )),
        EnumSet::empty(),
    );

    object.add_property(
        gc_context,
        "color",
        FunctionObject::function(
            gc_context,
            Executable::Native(color),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_color),
            Some(fn_proto),
            fn_proto,
        )),
        EnumSet::empty(),
    );

    object.add_property(
        gc_context,
        "componentX",
        FunctionObject::function(
            gc_context,
            Executable::Native(component_x),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_component_x),
            Some(fn_proto),
            fn_proto,
        )),
        EnumSet::empty(),
    );

    object.add_property(
        gc_context,
        "componentY",
        FunctionObject::function(
            gc_context,
            Executable::Native(component_y),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_component_y),
            Some(fn_proto),
            fn_proto,
        )),
        EnumSet::empty(),
    );

    object.add_property(
        gc_context,
        "mapBitmap",
        FunctionObject::function(
            gc_context,
            Executable::Native(map_bitmap),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_map_bitmap),
            Some(fn_proto),
            fn_proto,
        )),
        EnumSet::empty(),
    );

    object.add_property(
        gc_context,
        "mapPoint",
        FunctionObject::function(
            gc_context,
            Executable::Native(map_point),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_map_point),
            Some(fn_proto),
            fn_proto,
        )),
        EnumSet::empty(),
    );

    object.add_property(
        gc_context,
        "mode",
        FunctionObject::function(
            gc_context,
            Executable::Native(mode),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_mode),
            Some(fn_proto),
            fn_proto,
        )),
        EnumSet::empty(),
    );

    object.add_property(
        gc_context,
        "scaleX",
        FunctionObject::function(
            gc_context,
            Executable::Native(scale_x),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_scale_x),
            Some(fn_proto),
            fn_proto,
        )),
        EnumSet::empty(),
    );

    object.add_property(
        gc_context,
        "scaleY",
        FunctionObject::function(
            gc_context,
            Executable::Native(scale_y),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_scale_y),
            Some(fn_proto),
            fn_proto,
        )),
        EnumSet::empty(),
    );

    filter.into()
}
