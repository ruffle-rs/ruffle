use crate::context::UpdateContext;
use crate::debug::debug_message_out::DebugMessageOut;
use crate::debug::targeted_message::TargetedMsg;

pub trait DebugProvider<'gc> {
    /// Dispatch a debugging event to this type
    fn dispatch(
        &mut self,
        evt: TargetedMsg,
        context: &mut UpdateContext<'_, 'gc>,
    ) -> Option<DebugMessageOut>;
}
