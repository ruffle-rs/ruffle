use anyhow::{Context, Error};
use arboard::Clipboard;
use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use ruffle_core::backend::navigator::OpenURLMode;
use ruffle_core::backend::ui::{
    FontDefinition, FullscreenError, LanguageIdentifier, MouseCursor, UiBackend, US_ENGLISH,
};
use std::rc::Rc;
use sys_locale::get_locale;
use tracing::error;
use url::Url;
use winit::window::{Fullscreen, Window};

pub struct DesktopUiBackend {
    window: Rc<Window>,
    cursor_visible: bool,
    clipboard: Clipboard,
    language: LanguageIdentifier,
    preferred_cursor: MouseCursor,
    open_url_mode: OpenURLMode,
}

impl DesktopUiBackend {
    pub fn new(window: Rc<Window>, open_url_mode: OpenURLMode) -> Result<Self, Error> {
        let preferred_language = get_locale();
        let language = preferred_language
            .and_then(|l| l.parse().ok())
            .unwrap_or_else(|| US_ENGLISH.clone());
        Ok(Self {
            window,
            cursor_visible: true,
            clipboard: Clipboard::new().context("Couldn't get platform clipboard")?,
            language,
            preferred_cursor: MouseCursor::Arrow,
            open_url_mode,
        })
    }

    pub fn cursor(&self) -> egui::CursorIcon {
        if self.cursor_visible {
            match self.preferred_cursor {
                MouseCursor::Arrow => egui::CursorIcon::Default,
                MouseCursor::Hand => egui::CursorIcon::PointingHand,
                MouseCursor::IBeam => egui::CursorIcon::Text,
                MouseCursor::Grab => egui::CursorIcon::Grab,
            }
        } else {
            egui::CursorIcon::None
        }
    }
}

const DOWNLOAD_FAILED_MESSAGE: &str = "Ruffle failed to open or download this file.";

impl UiBackend for DesktopUiBackend {
    fn mouse_visible(&self) -> bool {
        self.cursor_visible
    }

    fn set_mouse_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
    }

    fn set_mouse_cursor(&mut self, cursor: MouseCursor) {
        self.preferred_cursor = cursor;
    }

    fn clipboard_content(&mut self) -> String {
        self.clipboard.get_text().unwrap_or_default()
    }

    fn set_clipboard_content(&mut self, content: String) {
        if let Err(e) = self.clipboard.set_text(content) {
            error!("Couldn't set clipboard contents: {:?}", e);
        }
    }

    fn set_fullscreen(&mut self, is_full: bool) -> Result<(), FullscreenError> {
        self.window.set_fullscreen(if is_full {
            Some(Fullscreen::Borderless(None))
        } else {
            None
        });
        Ok(())
    }

    fn display_root_movie_download_failed_message(&self) {
        let dialog = MessageDialog::new()
            .set_level(MessageLevel::Warning)
            .set_title("Ruffle - Load failed")
            .set_description(DOWNLOAD_FAILED_MESSAGE)
            .set_buttons(MessageButtons::Ok);
        dialog.show();
    }

    fn message(&self, message: &str) {
        let dialog = MessageDialog::new()
            .set_level(MessageLevel::Info)
            .set_title("Ruffle")
            .set_description(message)
            .set_buttons(MessageButtons::Ok);
        dialog.show();
    }

    fn display_unsupported_video(&self, url: Url) {
        if url.scheme() == "javascript" {
            tracing::warn!(
                "SWF tried to run a script on desktop, but javascript calls are not allowed"
            );
            return;
        }

        if self.open_url_mode == OpenURLMode::Confirm {
            let message = format!("The SWF file wants to open the website {}", url);
            // TODO: Add a checkbox with a GUI toolkit
            let confirm = MessageDialog::new()
                .set_title("Open website?")
                .set_level(MessageLevel::Info)
                .set_description(message)
                .set_buttons(MessageButtons::OkCancel)
                .show()
                == MessageDialogResult::Ok;
            if !confirm {
                tracing::info!("SWF tried to open a website, but the user declined the request");
                return;
            }
        } else if self.open_url_mode == OpenURLMode::Deny {
            tracing::warn!("SWF tried to open a website, but opening a website is not allowed");
            return;
        }

        // If the user confirmed or if in Allow mode, open the website

        // TODO: This opens local files in the browser while flash opens them
        // in the default program for the respective filetype.
        // This especially includes mailto links. Ruffle opens the browser which opens
        // the preferred program while flash opens the preferred program directly.

        match webbrowser::open(url.as_str()) {
            Ok(_output) => {}
            Err(e) => tracing::error!("Could not open URL {}: {}", url, e),
        };
    }

    fn load_device_font(&self, _name: &str, _register: &dyn FnMut(FontDefinition)) {}

    // Unused on desktop
    fn open_virtual_keyboard(&self) {}

    fn language(&self) -> &LanguageIdentifier {
        &self.language
    }
}
