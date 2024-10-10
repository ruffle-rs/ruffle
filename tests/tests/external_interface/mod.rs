// This lint is helpful, but right now we have too many instances of it.
// TODO: Remove this once all instances are fixed.
#![allow(clippy::needless_pass_by_ref_mut)]

use ruffle_core::context::UpdateContext;
use ruffle_core::external::ExternalInterfaceProvider;
use ruffle_core::external::{Value as ExternalValue, Value};

pub mod tests;

#[derive(Default)]
pub struct ExternalInterfaceTestProvider {}

impl ExternalInterfaceTestProvider {
    pub fn new() -> Self {
        Default::default()
    }
}

fn do_trace(context: &mut UpdateContext<'_>, args: &[ExternalValue]) -> ExternalValue {
    context.avm_trace(&format!("[ExternalInterface] trace: {args:?}"));
    "Traced!".into()
}

fn do_ping(context: &mut UpdateContext<'_>, _args: &[ExternalValue]) -> ExternalValue {
    context.avm_trace("[ExternalInterface] ping");
    "Pong!".into()
}

fn do_reentry(context: &mut UpdateContext<'_>, _args: &[ExternalValue]) -> ExternalValue {
    context.avm_trace("[ExternalInterface] starting reentry");
    if let Some(callback) = context.external_interface.get_callback("callWith") {
        callback.call(
            context,
            "callWith",
            vec!["trace".into(), "successful reentry!".into()],
        )
    } else {
        ExternalValue::Null
    }
}

impl ExternalInterfaceProvider for ExternalInterfaceTestProvider {
    fn call_method(
        &self,
        context: &mut UpdateContext<'_>,
        name: &str,
        args: &[ExternalValue],
    ) -> ExternalValue {
        match name {
            "trace" => do_trace(context, args),
            "ping" => do_ping(context, args),
            "reentry" => do_reentry(context, args),
            _ => Value::Null,
        }
    }

    fn on_callback_available(&self, _name: &str) {}

    fn get_id(&self) -> Option<String> {
        None
    }
}
