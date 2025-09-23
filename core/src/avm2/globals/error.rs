use crate::avm2::activation::Activation;
pub use crate::avm2::object::error_allocator;
use crate::avm2::string::AvmString;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::PlayerMode;

pub fn get_stack_trace<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    // See <https://docs.ruffle.rs/en_US/FlashPlatform/reference/actionscript/3/Error.html#getStackTrace()>
    // But note that the behavior also depends on SWF version.
    let stack_trace_enabled = if activation.context.player_version >= 18
        && activation.caller_movie_or_root().version() >= 18
    {
        // For Flash Player 11.5+ and SWF>=18, stack traces are always enabled.
        true
    } else {
        // For Flash Player Player 11.4 and earlier, or for SWF<18, stack traces are enabled for debug only.
        activation.context.player_mode == PlayerMode::Debug
    };

    if !stack_trace_enabled {
        return Ok(Value::Null);
    }

    if let Some(error) = this.as_error_object() {
        let call_stack = error.call_stack();
        if !call_stack.is_empty() {
            return Ok(AvmString::new(activation.gc(), error.display_full()).into());
        }
    }
    Ok(Value::Null)
}
