//! flash.filters.ConvolutionFilter object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::convolution_filter::ConvolutionFilterObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{ArrayObject, Object, TObject, Value};
use crate::types::F64Extension;
use gc_arena::MutationContext;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "alpha" => property(alpha, set_alpha);
    "bias" => property(bias, set_bias);
    "clamp" => property(clamp, set_clamp);
    "color" => property(color, set_color);
    "divisor" => property(divisor, set_divisor);
    "matrix" => property(matrix, set_matrix);
    "matrixX" => property(matrix_x, set_matrix_x);
    "matrixY" => property(matrix_y, set_matrix_y);
    "preserveAlpha" => property(preserve_alpha, set_preserve_alpha);
};

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    set_matrix_x(activation, this, args.get(0..1).unwrap_or_default())?;
    set_matrix_y(activation, this, args.get(1..2).unwrap_or_default())?;
    set_matrix(activation, this, args.get(2..3).unwrap_or_default())?;
    set_divisor(activation, this, args.get(3..4).unwrap_or_default())?;
    set_bias(activation, this, args.get(4..5).unwrap_or_default())?;
    set_preserve_alpha(activation, this, args.get(5..6).unwrap_or_default())?;
    set_clamp(activation, this, args.get(6..7).unwrap_or_default())?;
    set_color(activation, this, args.get(7..8).unwrap_or_default())?;
    set_alpha(activation, this, args.get(8..9).unwrap_or_default())?;

    Ok(this.into())
}

pub fn alpha<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(filter) = this.as_convolution_filter_object() {
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
        .unwrap_or(&0.into())
        .coerce_to_f64(activation)
        .map(|x| x.clamp_also_nan(0.0, 1.0))?;

    if let Some(filter) = this.as_convolution_filter_object() {
        filter.set_alpha(activation.context.gc_context, alpha);
    }

    Ok(Value::Undefined)
}

pub fn bias<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(filter) = this.as_convolution_filter_object() {
        return Ok(filter.bias().into());
    }

    Ok(Value::Undefined)
}

pub fn set_bias<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bias = args.get(0).unwrap_or(&0.into()).coerce_to_f64(activation)?;

    if let Some(filter) = this.as_convolution_filter_object() {
        filter.set_bias(activation.context.gc_context, bias);
    }

    Ok(Value::Undefined)
}

pub fn clamp<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(filter) = this.as_convolution_filter_object() {
        return Ok(filter.clamp().into());
    }

    Ok(Value::Undefined)
}

pub fn set_clamp<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let clamp = args
        .get(0)
        .unwrap_or(&true.into())
        .as_bool(activation.swf_version());

    if let Some(filter) = this.as_convolution_filter_object() {
        filter.set_clamp(activation.context.gc_context, clamp);
    }

    Ok(Value::Undefined)
}

pub fn color<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(object) = this.as_convolution_filter_object() {
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

    if let Some(object) = this.as_convolution_filter_object() {
        object.set_color(activation.context.gc_context, color & 0xFFFFFF);
    }

    Ok(Value::Undefined)
}

pub fn divisor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(filter) = this.as_convolution_filter_object() {
        return Ok(filter.divisor().into());
    }

    Ok(Value::Undefined)
}

pub fn set_divisor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let divisor = args.get(0).unwrap_or(&1.into()).coerce_to_f64(activation)?;

    if let Some(filter) = this.as_convolution_filter_object() {
        filter.set_divisor(activation.context.gc_context, divisor);
    }

    Ok(Value::Undefined)
}

pub fn matrix<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(filter) = this.as_convolution_filter_object() {
        return Ok(ArrayObject::new(
            activation.context.gc_context,
            activation.context.avm1.prototypes().array,
            filter.matrix().iter().map(|&x| x.into()),
        )
        .into());
    }

    Ok(Value::Undefined)
}

pub fn set_matrix<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let matrix = args.get(0).unwrap_or(&Value::Undefined);

    if let Some(filter) = this.as_convolution_filter_object() {
        if let Value::Object(obj) = matrix {
            let length = obj.length(activation)? as usize;

            let arr_len = length.max(filter.matrix_x() as usize * filter.matrix_y() as usize);
            let mut new_matrix = vec![0.0; arr_len];

            for (i, item) in new_matrix.iter_mut().enumerate().take(length) {
                *item = obj
                    .get_element(activation, i as i32)
                    .coerce_to_f64(activation)?;
            }

            filter.set_matrix(activation.context.gc_context, new_matrix);
        } else {
            let arr_len = filter.matrix_x() * filter.matrix_y();
            let new_matrix = (0..arr_len).map(|_| 0.0).collect::<Vec<_>>();
            filter.set_matrix(activation.context.gc_context, new_matrix);
        }
    }

    Ok(Value::Undefined)
}

pub fn matrix_x<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(filter) = this.as_convolution_filter_object() {
        return Ok(filter.matrix_x().into());
    }

    Ok(Value::Undefined)
}

pub fn set_matrix_x<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let matrix_x = args
        .get(0)
        .unwrap_or(&0.into())
        .coerce_to_i32(activation)
        .map(|x| x.clamp(0, 15))? as u8;

    if let Some(filter) = this.as_convolution_filter_object() {
        filter.set_matrix_x(activation.context.gc_context, matrix_x);
    }

    Ok(Value::Undefined)
}

pub fn matrix_y<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(filter) = this.as_convolution_filter_object() {
        return Ok(filter.matrix_y().into());
    }

    Ok(Value::Undefined)
}

pub fn set_matrix_y<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let matrix_y = args
        .get(0)
        .unwrap_or(&0.into())
        .coerce_to_i32(activation)
        .map(|x| x.clamp(0, 15))? as u8;

    if let Some(filter) = this.as_convolution_filter_object() {
        filter.set_matrix_y(activation.context.gc_context, matrix_y);
    }

    Ok(Value::Undefined)
}

pub fn preserve_alpha<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(filter) = this.as_convolution_filter_object() {
        return Ok(filter.preserve_alpha().into());
    }

    Ok(Value::Undefined)
}

pub fn set_preserve_alpha<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let preserve_alpha = args
        .get(0)
        .unwrap_or(&true.into())
        .as_bool(activation.swf_version());

    if let Some(filter) = this.as_convolution_filter_object() {
        filter.set_preserve_alpha(activation.context.gc_context, preserve_alpha);
    }

    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let filter = ConvolutionFilterObject::empty_object(gc_context, proto);
    let object = filter.raw_script_object();
    define_properties_on(PROTO_DECLS, gc_context, object, fn_proto);
    filter.into()
}
