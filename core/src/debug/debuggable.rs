use crate::context::UpdateContext;
use crate::debug::debug_message_out::DebugMessageOut;
use crate::debug::debug_provider::DebugProvider;
use crate::debug::movie_clip_debugger::MovieClipDebugger;
use crate::debug::targeted_message::TargetedMsg;

pub enum Debuggable<'gc> {
    MovieClip(MovieClipDebugger<'gc>),
}

impl<'gc> DebugProvider<'gc> for Debuggable<'gc> {
    fn dispatch(
        &mut self,
        evt: TargetedMsg,
        context: &mut UpdateContext<'_, 'gc>,
    ) -> Option<DebugMessageOut> {
        match self {
            Self::MovieClip(x) => x.dispatch(evt, context),
        }
    }
}
