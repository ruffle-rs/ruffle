use crate::avm2::activation::Activation;
use crate::avm2::error::{Error, make_error_1506};
use crate::avm2::function::FunctionArgs;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2_stub_method;

pub fn mfence<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    // TODO: This should perform an atomic fence, if we ever support Worker

    avm2_stub_method!(activation, "avm2.intrinsics.memory", "mfence");

    Ok(Value::Undefined)
}

pub fn casi32<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    // Negative addresses will be coerced to >i32::MAX, which is guaranteed to
    // be out-of-bounds of the domain memory by a check in `Domain::set_domain_memory`.
    let address = args.get_i32(0) as usize;
    let expected_val = args.get_i32(1);
    let new_val = args.get_i32(2);

    let mut dm = activation
        .caller_domain()
        .expect("Expected caller domain to exist")
        .domain_memory()
        .storage_mut();

    if address > dm.len() - 4 {
        return Err(make_error_1506(activation));
    }

    // Specified address must be a multiple of 4
    if !address.is_multiple_of(4) {
        return Err(make_error_1506(activation));
    }

    // TODO: This should perform an *atomic* compare-and-swap, if we ever
    // support Worker

    avm2_stub_method!(activation, "avm2.intrinsics.memory", "casi32");

    // TODO: Can we unify some of the code here with the domain memory code in
    // `Activation`?

    let original_val = dm.read_at(4, address).map_err(|e| e.to_avm(activation))?;
    let original_val = i32::from_le_bytes(original_val.try_into().unwrap());

    if original_val == expected_val {
        dm.write_at_nongrowing(&new_val.to_le_bytes(), address)
            .map_err(|e| e.to_avm(activation))?;
    }

    Ok(original_val.into())
}
