use crate::avm2::{error::argument_error, Activation, ClassObject, Error, Object};

pub fn avm1movie_allocator<'gc>(
    _class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    return Err(Error::AvmError(argument_error(
        activation,
        "Error #2012: AVM1Movie$ class cannot be instantiated.",
        2012,
    )?));
}
