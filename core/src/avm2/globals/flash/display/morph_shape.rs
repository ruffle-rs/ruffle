use crate::avm2::{error::argument_error, Activation, ClassObject, Error, Object};

pub fn morph_shape_allocator<'gc>(
    _class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    // The actual instantiation happens in `MorphShape::construct_frame`
    return Err(Error::AvmError(argument_error(
        activation,
        "Error #2012: MorphShape$ class cannot be instantiated.",
        2012,
    )?));
}
