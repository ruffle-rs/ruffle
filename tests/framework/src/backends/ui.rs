use crate::test::Font;
use chrono::{DateTime, Utc};
use ruffle_core::backend::ui::{
    DialogLoaderError, DialogResultFuture, FileDialogResult, FileFilter, FontDefinition,
    FullscreenError, LanguageIdentifier, MouseCursor, UiBackend, US_ENGLISH,
};
use url::Url;

/// A simulated file dialog response, for use in tests
///
/// Currently this can only simulate either a user cancellation result, or a successful file selection
#[derive(Default)]
pub struct TestFileDialogResult {
    canceled: bool,
    file_name: Option<String>,
    contents: Vec<u8>,
}

impl TestFileDialogResult {
    fn new_canceled() -> Self {
        Self {
            canceled: true,
            file_name: None,
            contents: Vec::new(),
        }
    }

    fn new_success(file_name: String) -> Self {
        Self {
            canceled: false,
            file_name: Some(file_name),
            contents: b"Hello, World!".to_vec(),
        }
    }
}

impl FileDialogResult for TestFileDialogResult {
    fn is_cancelled(&self) -> bool {
        self.canceled
    }

    fn creation_time(&self) -> Option<DateTime<Utc>> {
        None
    }

    fn modification_time(&self) -> Option<DateTime<Utc>> {
        None
    }

    fn file_name(&self) -> Option<String> {
        self.file_name.clone()
    }

    fn size(&self) -> Option<u64> {
        Some(self.contents.len() as u64)
    }

    fn file_type(&self) -> Option<String> {
        (!self.is_cancelled()).then(|| ".txt".to_string())
    }

    fn contents(&self) -> &[u8] {
        &self.contents
    }

    fn write_and_refresh(&mut self, data: &[u8]) {
        self.contents = data.to_vec();
    }
}

/// This is an implementation of [`UiBackend`], designed for use in tests
///
/// Fundamentally, this is mostly the same as [`NullUiBackend`] with the following differences:
/// * Attempting to display an open dialog with a filter with description "debug-select-success" will simulate successfully selecting a file,
///   otherwise a user cancellation will be simulated
/// * Attempting to display a file save dialog with a file name hint of "debug-success.txt" will simulate successfully selecting a destination
///   otherwise a user cancellation will be simulated
/// * Simulated in-memory clipboard
pub struct TestUiBackend {
    fonts: Vec<Font>,
    clipboard: String,
}

impl TestUiBackend {
    pub fn new(fonts: Vec<Font>) -> Self {
        Self {
            fonts,
            clipboard: "".to_string(),
        }
    }
}

impl UiBackend for TestUiBackend {
    fn mouse_visible(&self) -> bool {
        true
    }

    fn set_mouse_visible(&mut self, _visible: bool) {}

    fn set_mouse_cursor(&mut self, _cursor: MouseCursor) {}

    fn clipboard_content(&mut self) -> String {
        self.clipboard.clone()
    }

    fn set_clipboard_content(&mut self, content: String) {
        self.clipboard = content;
    }

    fn set_fullscreen(&mut self, _is_full: bool) -> Result<(), FullscreenError> {
        Ok(())
    }

    fn display_root_movie_download_failed_message(&self, _invalid_swf: bool) {}

    fn message(&self, _message: &str) {}

    fn open_virtual_keyboard(&self) {}

    fn close_virtual_keyboard(&self) {}

    fn language(&self) -> LanguageIdentifier {
        US_ENGLISH.clone()
    }

    fn display_unsupported_video(&self, _url: Url) {}

    fn load_device_font(
        &self,
        name: &str,
        is_bold: bool,
        is_italic: bool,
        register: &mut dyn FnMut(FontDefinition),
    ) {
        for font in &self.fonts {
            if font.family != name || font.bold != is_bold || font.italic != is_italic {
                continue;
            }

            register(FontDefinition::FontFile {
                name: name.to_owned(),
                is_bold,
                is_italic,
                data: font.bytes.clone(),
                index: 0,
            });
            break;
        }
    }

    fn display_file_open_dialog(&mut self, filters: Vec<FileFilter>) -> Option<DialogResultFuture> {
        Some(Box::pin(async move {
            // If filters has the magic debug-select-success filter, then return a fake file for testing

            let result: Result<Box<dyn FileDialogResult>, DialogLoaderError> = if filters
                .iter()
                .any(|f| f.description == "debug-select-success")
            {
                Ok(Box::new(TestFileDialogResult::new_success(
                    "test.txt".to_string(),
                )))
            } else {
                Ok(Box::new(TestFileDialogResult::new_canceled()))
            };

            result
        }))
    }

    fn display_file_save_dialog(
        &mut self,
        file_name: String,
        _title: String,
    ) -> Option<DialogResultFuture> {
        Some(Box::pin(async move {
            // If file_name has the magic debug-success.txt value, then return a fake file for testing

            let result: Result<Box<dyn FileDialogResult>, DialogLoaderError> =
                if file_name == "debug-success.txt" {
                    Ok(Box::new(TestFileDialogResult::new_success(file_name)))
                } else {
                    Ok(Box::new(TestFileDialogResult::new_canceled()))
                };

            result
        }))
    }

    fn close_file_dialog(&mut self) {}
}
