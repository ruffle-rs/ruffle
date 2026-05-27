use std::collections::HashMap;

use crate::test::Font;
use chrono::{DateTime, Utc};
use ruffle_core::backend::ui::{
    DialogResultFuture, FileDialogResult, FileDialogSelection, FileFilter, FontDefinition,
    FullscreenError, LanguageIdentifier, MouseCursor, US_ENGLISH, UiBackend,
};
use ruffle_core::font::{FontFileData, FontQuery};
use url::Url;

/// A simulated file selection, for use in tests.
pub struct TestFileSelection {
    file_name: String,
    contents: Vec<u8>,
}

impl TestFileSelection {
    fn new(file_name: String) -> Self {
        Self {
            file_name,
            contents: b"Hello, World!".to_vec(),
        }
    }
}

impl FileDialogSelection for TestFileSelection {
    fn creation_time(&self) -> Option<DateTime<Utc>> {
        None
    }

    fn modification_time(&self) -> Option<DateTime<Utc>> {
        None
    }

    fn file_name(&self) -> String {
        self.file_name.clone()
    }

    fn size(&self) -> Option<u64> {
        Some(self.contents.len() as u64)
    }

    fn file_type(&self) -> Option<String> {
        Some(".txt".to_string())
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
    fonts: HashMap<FontQuery, Font>,
    font_sorts: HashMap<FontQuery, Vec<FontQuery>>,
    clipboard: String,
}

impl TestUiBackend {
    pub fn new(
        fonts: HashMap<FontQuery, Font>,
        font_sorts: HashMap<FontQuery, Vec<FontQuery>>,
    ) -> Self {
        Self {
            fonts,
            font_sorts,
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

    fn display_root_movie_download_failed_message(&self, _invalid_swf: bool, _fetch_error: String) {
    }

    fn message(&self, _message: &str) {}

    fn open_virtual_keyboard(&self) {}

    fn close_virtual_keyboard(&self) {}

    fn language(&self) -> LanguageIdentifier {
        US_ENGLISH.clone()
    }

    fn display_unsupported_video(&self, _url: Url) {}

    fn load_device_font(&self, query: &FontQuery, register: &mut dyn FnMut(FontDefinition)) {
        let Some(font) = self.fonts.get(query) else {
            return;
        };

        register(FontDefinition::FontFile {
            name: font.family.to_owned(),
            is_bold: font.bold,
            is_italic: font.italic,
            data: FontFileData::new(font.bytes.clone()),
            index: 0,
        });
    }

    fn sort_device_fonts(
        &self,
        query: &FontQuery,
        register: &mut dyn FnMut(FontDefinition),
    ) -> Vec<FontQuery> {
        let Some(sort) = self.font_sorts.get(query) else {
            return Vec::new();
        };
        for query in sort {
            self.load_device_font(query, register);
        }
        sort.clone()
    }

    fn display_file_open_dialog(&mut self, filters: Vec<FileFilter>) -> Option<DialogResultFuture> {
        Some(Box::pin(async move {
            // If filters has the magic debug-select-success filter, then return a fake file for testing
            Ok(
                if filters
                    .iter()
                    .any(|f| f.description == "debug-select-success")
                {
                    FileDialogResult::Selection(Box::new(TestFileSelection::new(
                        "test.txt".to_string(),
                    )))
                } else {
                    FileDialogResult::Canceled
                },
            )
        }))
    }

    fn display_file_save_dialog(
        &mut self,
        file_name: String,
        _title: String,
    ) -> Option<DialogResultFuture> {
        Some(Box::pin(async move {
            // If file_name has the magic debug-success.txt value, then return a fake file for testing
            Ok(if file_name == "debug-success.txt" {
                FileDialogResult::Selection(Box::new(TestFileSelection::new(file_name)))
            } else {
                FileDialogResult::Canceled
            })
        }))
    }

    fn close_file_dialog(&mut self) {}
}
