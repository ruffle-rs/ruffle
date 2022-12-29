use crate::player::DebugMessageIn;
use crate::player::DebugMessageOut;

pub trait DebugProvider {
    /// Dispatch a debugging event to this type
    fn dispatch(&mut self, evt: DebugMessageIn) -> Option<DebugMessageOut>;
}

pub enum Debuggable<'gc> {
    MovieClip(MovieClipDebugger<'gc>)
}

impl<'gc> DebugProvider for Debuggable<'gc> {
    fn dispatch(&mut self, evt: DebugMessageIn) -> Option<DebugMessageOut> {
        match self {
            Self::MovieClip(x) => x.dispatch(evt)
        }
    }
}

use crate::display_object::MovieClip;
pub struct MovieClipDebugger<'gc> {
    tgt: MovieClip<'gc>,
}

impl<'gc> MovieClipDebugger<'gc> {
    pub fn with(tgt: MovieClip<'gc>) -> Self {
        Self {
            tgt,
        }
    }
}

impl<'gc> DebugProvider for MovieClipDebugger<'gc> {
    fn dispatch(&mut self, evt: DebugMessageIn) -> Option<DebugMessageOut> {
        match evt {
            DebugMessageIn::GetCurrentFrame => Some(DebugMessageOut::CurrentFrame { num: self.tgt.current_frame()}),
            _ => None,
        }
    }
}