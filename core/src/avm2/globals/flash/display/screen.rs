use crate::avm2::{Activation, ArrayObject, ArrayStorage, Error, Object, Value};

pub fn get_screens_internal<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let screens = activation
        .context
        .ui
        .get_screens_sizes()
        .into_iter()
        .map(|screen| {
            let storage =
                ArrayStorage::from_storage(vec![Some(screen.0.into()), Some(screen.1.into())]);
            Ok(Some(ArrayObject::from_storage(activation, storage)?.into()))
        })
        .collect::<Result<Vec<Option<Value<'gc>>>, Error<'gc>>>()?;
    let storage = ArrayStorage::from_storage(screens);
    Ok(ArrayObject::from_storage(activation, storage)?.into())
}
