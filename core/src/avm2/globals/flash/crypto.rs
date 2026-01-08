//! `flash.crypto` namespace

use crate::avm2::error::{Error2004Type, make_error_2004};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::{Activation, Error, Value};
use rand::{TryRngCore, rngs::OsRng};

/// Implements `flash.crypto.generateRandomBytes`
pub fn generate_random_bytes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let length = args.get_u32(0);
    if !(1..1025).contains(&length) {
        return Err(make_error_2004(activation, Error2004Type::Error));
    }

    let ba = activation
        .avm2()
        .classes()
        .bytearray
        .construct(activation, &[])?
        .as_object()
        .unwrap();

    let mut ba_write = ba.as_bytearray_mut().unwrap();
    ba_write.set_length(length as usize);

    let mut rng = OsRng {};

    rng.try_fill_bytes(ba_write.bytes_mut()).unwrap();

    Ok(ba.into())
}
