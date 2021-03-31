//! flash.filter.ColorMatrixFilter object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::color_matrix_filter::ColorMatrixFilterObject;
use crate::avm1::property::Attribute;
use crate::avm1::{Object, ScriptObject, TObject, Value};
use gc_arena::MutationContext;

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
        let array = ScriptObject::array(
            activation.context.gc_context,
            Some(activation.context.avm1.prototypes.array),
        );

        let arr = filter.matrix();

        for (index, item) in arr.iter().copied().enumerate() {
            array.set_array_element(index, item.into(), activation.context.gc_context);
        }

        return Ok(array.into());
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
        let arr_len = obj.length().min(20);
        let mut arr = [0.0; 4 * 5];

        for (index, item) in arr.iter_mut().enumerate().take(arr_len) {
            let elem = obj.array_element(index).coerce_to_f64(activation)?;
            *item = elem;
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
    let color_matrix_filter = ColorMatrixFilterObject::empty_object(gc_context, Some(proto));
    let object = color_matrix_filter.as_script_object().unwrap();

    object.add_property(
        gc_context,
        "matrix",
        FunctionObject::function(
            gc_context,
            Executable::Native(matrix),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_matrix),
            Some(fn_proto),
            fn_proto,
        )),
        Attribute::empty(),
    );

    color_matrix_filter.into()
}
