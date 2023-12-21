use super::JavascriptPlayer;
use rfd::{AsyncFileDialog, FileHandle};
use ruffle_core::backend::ui::{
    DialogLoaderError, DialogResultFuture, FileDialogResult, FileFilter,
};
use ruffle_core::backend::ui::{
    FontDefinition, FullscreenError, LanguageIdentifier, MouseCursor, UiBackend, US_ENGLISH,
};
use ruffle_web_common::JsResult;
use std::borrow::Cow;
use url::Url;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, HtmlDocument, HtmlTextAreaElement};

use chrono::{DateTime, Utc};

#[cfg(target_arch = "wasm32")]
use chrono::{NaiveDateTime, TimeZone};

#[derive(Debug)]
struct FullScreenError {
    jsval: String,
}

impl std::fmt::Display for FullScreenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.jsval)
    }
}

impl std::error::Error for FullScreenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub struct WebFileDialogResult {
    handle: Option<FileHandle>,
    contents: Vec<u8>,
}

impl WebFileDialogResult {
    pub async fn new(handle: Option<FileHandle>) -> Self {
        let contents = if let Some(handle) = handle.as_ref() {
            handle.read().await
        } else {
            Vec::new()
        };

        Self { handle, contents }
    }
}

fn get_extension_from_filename(filename: &str) -> Option<String> {
    std::path::Path::new(filename)
        .extension()
        .and_then(|x| x.to_str())
        .map(|x| ".".to_owned() + x)
}

impl FileDialogResult for WebFileDialogResult {
    fn is_cancelled(&self) -> bool {
        self.handle.is_none()
    }

    fn creation_time(&self) -> Option<DateTime<Utc>> {
        // Creation time is not available in JS
        None
    }

    fn modification_time(&self) -> Option<DateTime<Utc>> {
        #[cfg(target_arch = "wasm32")]
        if let Some(handle) = &self.handle {
            NaiveDateTime::from_timestamp_opt(handle.inner().last_modified() as i64, 0)
                .map(|ts| Utc.from_utc_datetime(&ts))
        } else {
            None
        }

        #[cfg(not(target_arch = "wasm32"))]
        None
    }

    fn file_name(&self) -> Option<String> {
        #[cfg(target_arch = "wasm32")]
        return self.handle.as_ref().map(|handle| handle.file_name());

        #[cfg(not(target_arch = "wasm32"))]
        None
    }

    fn size(&self) -> Option<u64> {
        #[cfg(target_arch = "wasm32")]
        return self.handle.as_ref().map(|x| x.inner().size() as u64);
        #[cfg(not(target_arch = "wasm32"))]
        None
    }

    fn file_type(&self) -> Option<String> {
        if let Some(handle) = &self.handle {
            get_extension_from_filename(&handle.file_name())
        } else {
            None
        }
    }

    fn creator(&self) -> Option<String> {
        None
    }

    fn contents(&self) -> &[u8] {
        &self.contents
    }

    fn write(&self, _data: &[u8]) {
        //NOOP
    }

    fn refresh(&mut self) {}
}

/// An implementation of `UiBackend` utilizing `web_sys` bindings to input APIs.
pub struct WebUiBackend {
    js_player: JavascriptPlayer,
    canvas: HtmlCanvasElement,
    cursor_visible: bool,
    cursor: MouseCursor,
    language: LanguageIdentifier,
    clipboard_content: String,

    /// Is a dialog currently open
    dialog_open: bool,
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
            dialog_open: false,
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

    fn display_unsupported_video(&self, url: Url) {
        self.js_player.display_unsupported_video(url.as_str());
    }

    fn load_device_font(
        &self,
        _name: &str,
        _is_bold: bool,
        _is_italic: bool,
        _register: &mut dyn FnMut(FontDefinition),
    ) {
        // Because fonts must be loaded instantly (no async),
        // we actually just provide them all upfront at time of Player creation.
    }

    fn display_file_open_dialog(&mut self, filters: Vec<FileFilter>) -> Option<DialogResultFuture> {
        // Prevent opening multiple dialogs at the same time
        if self.dialog_open {
            return None;
        }
        self.dialog_open = true;

        // Create the dialog future
        Some(Box::pin(async move {
            let mut dialog = AsyncFileDialog::new();

            for filter in filters {
                let window = web_sys::window().expect("window()");
                let navigator = window.navigator();
                let platform = navigator.platform().expect("navigator.platform");

                if platform.contains("Mac") && filter.mac_type.is_some() {
                    let mac_type = filter.mac_type.expect("Cant fail");
                    let extensions: Vec<&str> = mac_type.split(';').collect();
                    dialog = dialog.add_filter(&filter.description, &extensions);
                } else {
                    let extensions: Vec<&str> = filter
                        .extensions
                        .split(';')
                        .map(|x| x.trim_start_matches("*."))
                        .collect();
                    dialog = dialog.add_filter(&filter.description, &extensions);
                }
            }

            let result: Result<Box<dyn FileDialogResult>, DialogLoaderError> = Ok(Box::new(
                WebFileDialogResult::new(dialog.pick_file().await).await,
            ));
            result
        }))
    }

    fn close_file_dialog(&mut self) {
        self.dialog_open = false;
    }

    fn display_file_save_dialog(
        &mut self,
        file_name: String,
        title: String,
    ) -> Option<DialogResultFuture> {
        // Prevent opening multiple dialogs at the same time
        if self.dialog_open {
            return None;
        }
        self.dialog_open = true;

        // Create the dialog future
        Some(Box::pin(async move {
            // Select the location to save the file to
            let dialog = AsyncFileDialog::new()
                .set_title(&title)
                .set_file_name(&file_name);

            let result: Result<Box<dyn FileDialogResult>, DialogLoaderError> = Ok(Box::new(
                WebFileDialogResult::new(dialog.save_file().await).await,
            ));
            result
        }))
    }
}
