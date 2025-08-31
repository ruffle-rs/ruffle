use crate::avm2::activation::Activation;
use crate::avm2::value::Value;
use crate::avm2::Error;

pub fn call_handler<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    activation
        .avm2()
        .classes()
        .urierror
        .construct(activation, args)
}
