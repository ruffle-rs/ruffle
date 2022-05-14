//! Custom event type for desktop ruffle

/// User-defined events.
pub enum RuffleEvent {
    /// Indicates that one or more tasks are ready to poll on our executor.
    TaskPoll,

    /// Indicates that an asynchronous SWF metadata load has been completed.
    OnMetadata(ruffle_core::swf::HeaderExt),
}
