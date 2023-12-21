use chrono::{DateTime, Utc};
use image::EncodableLayout;
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
}

impl TestFileDialogResult {
    fn new_canceled() -> Self {
        Self {
            canceled: true,
            file_name: None,
        }
    }

    fn new_success(file_name: String) -> Self {
        Self {
            canceled: false,
            file_name: Some(file_name),
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
        (!self.is_cancelled()).then(|| self.file_name.clone().unwrap())
    }

    fn size(&self) -> Option<u64> {
        None
    }

    fn file_type(&self) -> Option<String> {
        (!self.is_cancelled()).then(|| ".txt".to_string())
    }

    fn creator(&self) -> Option<String> {
        None
    }

    fn contents(&self) -> &[u8] {
        b"Hello, World!".as_bytes()
    }

    fn write(&self, _data: &[u8]) {}

    fn refresh(&mut self) {}
}

/// This is an implementation of [`UiBackend`], designed for use in tests
///
/// Fundamentally, this is mostly the same as [`NullUiBackend`] with the following differences:
/// * Attempting to display an open dialog with a filter with description "debug-select-success" will simulate successfully selecting a file,
///   otherwise a user cancellation will be simulated
/// * Attempting to display a file save dialog with a file name hint of "debug-success.txt" will simulate successfully selecting a destination
///   otherwise a user cancellation will be simulated
#[derive(Default)]
pub struct TestUiBackend;

impl UiBackend for TestUiBackend {
    fn mouse_visible(&self) -> bool {
        true
    }

    fn set_mouse_visible(&mut self, _visible: bool) {}

    fn set_mouse_cursor(&mut self, _cursor: MouseCursor) {}

    fn clipboard_content(&mut self) -> String {
        "".to_string()
    }

    fn set_clipboard_content(&mut self, _content: String) {}

    fn set_fullscreen(&mut self, _is_full: bool) -> Result<(), FullscreenError> {
        Ok(())
    }

    fn display_root_movie_download_failed_message(&self) {}

    fn message(&self, _message: &str) {}

    fn open_virtual_keyboard(&self) {}

    fn language(&self) -> &LanguageIdentifier {
        &US_ENGLISH
    }

    fn display_unsupported_video(&self, _url: Url) {}

    fn load_device_font(
        &self,
        _name: &str,
        _is_bold: bool,
        _is_italic: bool,
        _register: &mut dyn FnMut(FontDefinition),
    ) {
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
