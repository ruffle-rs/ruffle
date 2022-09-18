use crate::avm2::object::TObject;
use crate::avm2::Activation;
use crate::avm2::AvmString;
use crate::avm2::Value;

use super::ClassObject;

/// An error generated while handling AVM2 logic
#[derive(Debug)]
pub enum Error<'gc> {
    /// A thrown error. This can be produced by an explicit 'throw'
    /// opcode, or by a native implementation that throws an exception.
    /// This can be caught by any catch blocks created by ActionScript code
    AvmError(Value<'gc>),
    /// An internal VM error. This cannot be caught by ActionScript code -
    /// it will either be logged by Ruffle, or cause the player to
    /// stop executing.
    RustError(Box<dyn std::error::Error>),
}

// This type is used very frequently, so make sure it doesn't unexpectedly grow.
// For now, we only test on Nightly, since a new niche optimization was recently
// added (https://github.com/rust-lang/rust/pull/94075) that shrinks the size
// relative to stable.

#[rustversion::nightly]
#[cfg(target_arch = "wasm32")]
static_assertions::assert_eq_size!(Result<Value<'_>, Error<'_>>, [u8; 24]);

#[rustversion::nightly]
#[cfg(target_pointer_width = "64")]
static_assertions::assert_eq_size!(Result<Value<'_>, Error<'_>>, [u8; 32]);

#[inline(never)]
#[cold]
pub fn range_error<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    message: &str,
    code: u32,
) -> Result<Value<'gc>, Error<'gc>> {
    let class = activation.avm2().classes().rangeerror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
pub fn argument_error<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    message: &str,
    code: u32,
) -> Result<Value<'gc>, Error<'gc>> {
    let class = activation.avm2().classes().argumenterror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
pub fn type_error<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    message: &str,
    code: u32,
) -> Result<Value<'gc>, Error<'gc>> {
    let class = activation.avm2().classes().typeerror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
pub fn reference_error<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    message: &str,
    code: u32,
) -> Result<Value<'gc>, Error<'gc>> {
    let class = activation.avm2().classes().referenceerror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
pub fn verify_error<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    message: &str,
    code: u32,
) -> Result<Value<'gc>, Error<'gc>> {
    let class = activation.avm2().classes().verifyerror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
pub fn io_error<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    message: &str,
    code: u32,
) -> Result<Value<'gc>, Error<'gc>> {
    let class = activation.avm2().classes().ioerror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
pub fn eof_error<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    message: &str,
    code: u32,
) -> Result<Value<'gc>, Error<'gc>> {
    let class = activation.avm2().classes().eoferror;
    error_constructor(activation, class, message, code)
}

fn error_constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    class: ClassObject<'gc>,
    message: &str,
    code: u32,
) -> Result<Value<'gc>, Error<'gc>> {
    let message = AvmString::new_utf8(activation.context.gc_context, message);
    Ok(class
        .construct(activation, &[message.into(), code.into()])?
        .into())
}

impl<'gc> std::fmt::Display for Error<'gc> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

// Ideally, all of these impls would be unified under a single
// `impl<E: std::error::Error> From<E> for Error<'gc>`
// However, this would conflict with the 'str' and 'String'
// impls, which are still widely used.

impl<'gc, 'a> From<&'a str> for Error<'gc> {
    fn from(val: &'a str) -> Error<'gc> {
        Error::RustError(val.into())
    }
}

impl<'gc> From<String> for Error<'gc> {
    fn from(val: String) -> Error<'gc> {
        Error::RustError(val.into())
    }
}

impl<'gc> From<ruffle_render::error::Error> for Error<'gc> {
    fn from(val: ruffle_render::error::Error) -> Error<'gc> {
        Error::RustError(val.into())
    }
}
