use crate::avm2::object::TextureObject;
use crate::avm2::Activation;
use crate::avm2::Error;
use crate::avm2::Object;

pub fn do_compressed_upload<'gc>(
    _: &mut Activation<'_, 'gc>,
    _: TextureObject<'gc>,
    _: Object<'gc>,
    _: usize,
    _: bool,
) -> Result<(), Error<'gc>> {
    Err("Support for compressed textures not compiled in.".into())
}
