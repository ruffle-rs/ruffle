//! flash.filters.ColorMatrixFilter object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::color_matrix_filter::ColorMatrixFilterObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{ArrayObject, Object, TObject, Value};
use gc_arena::MutationContext;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "matrix" => property(matrix, set_matrix);
};

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    set_matrix(activation, this, args.get(0..1).unwrap_or_default())?;

    Ok(this.into())
}

pub fn matrix<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(filter) = this.as_color_matrix_filter_object() {
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

    if let Value::Object(obj) = matrix {
        let length = obj.length(activation)?.min(20);
        let mut arr = [0.0; 4 * 5];

        for (i, item) in arr.iter_mut().enumerate().take(length as usize) {
            *item = obj
                .get_element(activation, i as i32)
                .coerce_to_f64(activation)?;
        }

        if let Some(filter) = this.as_color_matrix_filter_object() {
            filter.set_matrix(activation.context.gc_context, arr);
        }
    }

    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let color_matrix_filter = ColorMatrixFilterObject::empty_object(gc_context, proto);
    let object = color_matrix_filter.as_script_object().unwrap();
    define_properties_on(PROTO_DECLS, gc_context, object, fn_proto);
    color_matrix_filter.into()
}
