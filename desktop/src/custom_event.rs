//! Custom event type for desktop ruffle

/// User-defined events.
pub enum RuffleEvent {
    /// Indicates that one or more tasks are ready to poll on our executor.
    TaskPoll,

    /// Indicates that an asynchronous SWF metadata load has been completed.
    OnMetadata(ruffle_core::swf::HeaderExt),

    /// The user requested to open a new local SWF.
    OpenFile,

    /// The user requested to open a URL.
    OpenURL(url::Url),

    /// The user requested to exit Ruffle.
    ExitRequested,

    /// The user selected an item in the right-click context menu.
    ContextMenuItemClicked(usize),
}
