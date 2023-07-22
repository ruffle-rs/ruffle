use super::JavascriptPlayer;
use ruffle_core::backend::ui::{
    FullscreenError, LanguageIdentifier, MouseCursor, UiBackend, US_ENGLISH,
};
use ruffle_web_common::JsResult;
use std::borrow::Cow;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, HtmlDocument, HtmlTextAreaElement};

/// An implementation of `UiBackend` utilizing `web_sys` bindings to input APIs.
pub struct WebUiBackend {
    js_player: JavascriptPlayer,
    canvas: HtmlCanvasElement,
    cursor_visible: bool,
    cursor: MouseCursor,
    language: LanguageIdentifier,
    clipboard_content: String,
}

impl WebUiBackend {
    pub fn new(js_player: JavascriptPlayer, canvas: &HtmlCanvasElement) -> Self {
        let window = web_sys::window().expect("window()");
        let preferred_language = window.navigator().language();
        let language = preferred_language
            .and_then(|l| l.parse().ok())
            .unwrap_or_else(|| US_ENGLISH.clone());
        Self {
            js_player,
            canvas: canvas.clone(),
            cursor_visible: true,
            cursor: MouseCursor::Arrow,
            language,
            clipboard_content: "".into(),
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

    fn clipboard_content(&mut self) -> String {
        // On web, clipboard content is not directly accessible due to security restrictions,
        // but pasting from the clipboard is supported via the JS `paste` event
        self.clipboard_content.to_owned()
    }

    fn set_clipboard_content(&mut self, content: String) {
        self.clipboard_content = content.to_owned();
        // We use `document.execCommand("copy")` as `navigator.clipboard.writeText("string")`
        // is available only in secure contexts (HTTPS).
        if let Some(element) = self.canvas.parent_element() {
            let window = web_sys::window().expect("window()");
            let document: HtmlDocument = window
                .document()
                .expect("document()")
                .dyn_into()
                .expect("document() didn't give us a document");
            let textarea: HtmlTextAreaElement = document
                .create_element("textarea")
                .expect("create_element() must succeed")
                .dyn_into()
                .expect("create_element(\"textarea\") didn't give us a textarea");

            let editing_text = self.js_player.is_virtual_keyboard_focused();
            textarea.set_value(&content);
            let _ = element.append_child(&textarea);
            textarea.select();

            match document.exec_command("copy") {
                Ok(success) => {
                    if !success {
                        tracing::warn!(
                            "Couldn't set clipboard contents: The browser rejected the call"
                        );
                    }
                }
                Err(e) => tracing::error!("Couldn't set clipboard contents: {:?}", e),
            }

            let _ = element.remove_child(&textarea);
            if editing_text {
                // Return focus to the text area
                self.js_player.open_virtual_keyboard();
            }
        }
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

    fn display_root_movie_download_failed_message(&self) {
        self.js_player.display_root_movie_download_failed_message()
    }

    fn message(&self, message: &str) {
        self.js_player.display_message(message);
    }

    fn open_virtual_keyboard(&self) {
        self.js_player.open_virtual_keyboard()
    }

    fn language(&self) -> &LanguageIdentifier {
        &self.language
    }
}
