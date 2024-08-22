use crate::avm2::{Activation, ArrayObject, ArrayStorage, Error, Object, Value};
use crate::string::AvmString;

pub fn get_arguments_internal<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let values = activation
        .context
        .air_arguments
        .iter()
        .map(|arg| Some(AvmString::new_utf8(activation.context.gc_context, arg).into()))
        .collect::<Vec<Option<Value<'gc>>>>();
    let storage = ArrayStorage::from_storage(values);
    Ok(ArrayObject::from_storage(activation, storage)?.into())
}
