//! Custom event type for desktop ruffle

use ruffle_core::events::PlayerNotification;

use crate::gui::DialogDescriptor;
use crate::player::{LaunchOptions, PlayerRunnable};

/// User-defined events.
pub enum RuffleEvent {
    /// Indicates that a task is ready to be polled.
    TaskPoll(PlayerRunnable),

    /// Indicates that an asynchronous SWF metadata load has been completed.
    OnMetadata(ruffle_core::swf::HeaderExt),

    /// The user requested to pick and then open a file.
    BrowseAndOpen(Box<LaunchOptions>),

    /// The user requested to open a movie.
    Open(url::Url, Box<LaunchOptions>),

    /// The user requested to close the current SWF.
    CloseFile,

    /// The user requested to enter full screen.
    EnterFullScreen,

    /// The user requested to exit full screen.
    ExitFullScreen,

    /// The user requested to exit Ruffle.
    ExitRequested,

    /// The user selected an item in the right-click context menu.
    ContextMenuItemClicked(usize),

    /// The movie wants to open a dialog.
    OpenDialog(DialogDescriptor),

    /// Ruffle core has a notification to handle.
    PlayerNotification(PlayerNotification),

    /// Export Ruffle Bundle from currently playing content and open save dialog.
    ExportBundle,
}
