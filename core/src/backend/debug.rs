//! Backend for handling debugger communication

use crate::backend::navigator::OwnedFuture;
use crate::debugable::{Avm1Msg, DebugMessageOut, PlayerMsg, TargetedMsg};
use crate::loader::Error as LoaderError;

/// A trait that defines the async interactions between a connected debugger and the player
pub trait DebuggerBackend {
    /// Poll a single player debug event
    /// This wil be invoked before each frame and may or may not block until a message is available
    /// The recommendation is to pull events from an internal queue
    fn get_debug_event_player(&mut self) -> Option<PlayerMsg>;

    /// Poll a single targeted debug event
    /// See `get_debug_event_player` for details
    fn get_debug_event_targeted(&mut self) -> Option<(String, TargetedMsg)>;

    /// Poll a single avm1 debug event
    /// See `get_debug_event_player` for details
    fn get_debug_event_avm1(&mut self) -> Option<Avm1Msg>;

    /// Enqueue a debug message to be sent to the attached debugger if it exists
    /// This function should not block
    fn submit_debug_message(&self, _evt: DebugMessageOut);

    /// Attempt to connect to a debugger if one exists
    /// This function is free to block until the connection is established.
    //TODO: docs
    fn connect_debugger(&mut self) -> Option<OwnedFuture<(), LoaderError>>;
}

/// A null debugger backend
/// Never connects or emits events
#[derive(Debug, Default)]
pub struct NullDebuggerBackend {}

impl NullDebuggerBackend {
    pub fn new() -> Self {
        Self::default()
    }
}

impl DebuggerBackend for NullDebuggerBackend {
    fn get_debug_event_player(&mut self) -> Option<PlayerMsg> {
        None
    }

    fn get_debug_event_targeted(&mut self) -> Option<(String, TargetedMsg)> {
        None
    }

    fn get_debug_event_avm1(&mut self) -> Option<Avm1Msg> {
        None
    }

    fn submit_debug_message(&self, _evt: DebugMessageOut) {
        // NOOP
    }

    fn connect_debugger(&mut self) -> Option<OwnedFuture<(), LoaderError>> {
        None
    }
}

//TODO: feature flag all debug changes