use super::JavascriptPlayer;
use ruffle_core::backend::ui::{FullscreenError, MouseCursor, UiBackend};
use ruffle_web_common::JsResult;
use std::borrow::Cow;
use web_sys::HtmlCanvasElement;

/// An implementation of `UiBackend` utilizing `web_sys` bindings to input APIs.
pub struct WebUiBackend {
    js_player: JavascriptPlayer,
    canvas: HtmlCanvasElement,
    cursor_visible: bool,
    cursor: MouseCursor,
}

impl WebUiBackend {
    pub fn new(js_player: JavascriptPlayer, canvas: &HtmlCanvasElement) -> Self {
        Self {
            js_player,
            canvas: canvas.clone(),
            cursor_visible: true,
            cursor: MouseCursor::Arrow,
        }
    }

    fn update_mouse_cursor(&self) {
        let cursor = if self.cursor_visible {
            match self.cursor {
                MouseCursor::Arrow => "auto",
                MouseCursor::Hand => "pointer",
                MouseCursor::IBeam => "text",
                MouseCursor::Grab => "grab",
            }
        } else {
            "none"
        };
        self.canvas
            .style()
            .set_property("cursor", cursor)
            .warn_on_error();
    }
}

impl UiBackend for WebUiBackend {
    fn mouse_visible(&self) -> bool {
        self.cursor_visible
    }

    fn set_mouse_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
        self.update_mouse_cursor();
    }

    fn set_mouse_cursor(&mut self, cursor: MouseCursor) {
        self.cursor = cursor;
        self.update_mouse_cursor();
    }

    fn set_clipboard_content(&mut self, _content: String) {
        log::warn!("set clipboard not implemented");
    }

    fn set_fullscreen(&mut self, is_full: bool) -> Result<(), FullscreenError> {
        match self.js_player.set_fullscreen(is_full) {
            Ok(_) => Ok(()),
            Err(jsval) => Err(jsval
                .as_string()
                .map(Cow::Owned)
                .unwrap_or_else(|| Cow::Borrowed("Failed to change full screen state"))),
        }
    }

    fn display_unsupported_message(&self) {
        self.js_player.display_unsupported_message()
    }

    fn display_root_movie_download_failed_message(&self) {
        self.js_player.display_root_movie_download_failed_message()
    }

    fn message(&self, message: &str) {
        self.js_player.display_message(message);
    }
}
