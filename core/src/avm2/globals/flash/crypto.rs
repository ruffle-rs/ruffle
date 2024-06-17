//! `flash.crypto` namespace

use crate::avm2::error::{make_error_2004, Error2004Type};
use crate::avm2::object::TObject;
use crate::avm2::{Activation, Error, Object, Value};
use rand::{rngs::OsRng, RngCore};

/// Implements `flash.crypto.generateRandomBytes`
pub fn generate_random_bytes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let length = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_u32(activation)?;
    if !(1..1025).contains(&length) {
        return Err(make_error_2004(activation, Error2004Type::Error));
    }

    let ba_class = activation.context.avm2.classes().bytearray;
    let ba = ba_class.construct(activation, &[])?;
    let mut ba_write = ba.as_bytearray_mut(activation.context.gc_context).unwrap();
    ba_write.set_length(length as usize);

    let mut rng = OsRng {};

    rng.fill_bytes(ba_write.bytes_mut());

    Ok(ba.into())
}
