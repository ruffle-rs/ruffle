use crate::cli::OpenUrlMode;
use crate::custom_event::RuffleEvent;
use crate::gui::dialogs::message_dialog::MessageDialogConfiguration;
use crate::gui::{DialogDescriptor, FilePicker, LocalizableText};
use crate::preferences::GlobalPreferences;
use anyhow::{Error, Result, anyhow};
use chrono::{DateTime, Utc};
use egui_winit::clipboard::Clipboard;
use fontdb::{FaceInfo, Family};
use rfd::{
    AsyncFileDialog, FileHandle, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel,
};
use ruffle_core::backend::ui::{
    DialogLoaderError, DialogResultFuture, FileDialogResult, FileFilter, FontDefinition,
    FullscreenError, LanguageIdentifier, MouseCursor, UiBackend,
};
use ruffle_core::font::{FontFileData, FontQuery};
use std::fs::File;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use url::Url;
use winit::event_loop::EventLoopProxy;
use winit::raw_window_handle::HasDisplayHandle;
use winit::window::{Fullscreen, Window};

pub struct DesktopFileDialogResult {
    handle: Option<FileHandle>,
    md: Option<std::fs::Metadata>,
    contents: Vec<u8>,
}

impl DesktopFileDialogResult {
    /// Create a new [`DesktopFileDialogResult`] from a given file handle
    pub async fn new(handle: Option<FileHandle>) -> Self {
        async fn read_file(path: &Path) -> (Option<std::fs::Metadata>, Vec<u8>) {
            let file = match tokio::fs::File::open(path).await {
                Ok(file) => file,
                Err(err) => {
                    let path = path.to_string_lossy();
                    tracing::error!("Error opening file {path}: {err}");
                    return (None, Vec::new());
                }
            };
            let metadata = match file.metadata().await {
                Ok(metadata) => Some(metadata),
                Err(err) => {
                    let path = path.to_string_lossy();
                    tracing::error!("Error reading metadata of file {path}: {err}");
                    None
                }
            };

            let mut contents = Vec::new();
            let mut file = file;
            if let Err(err) = file.read_to_end(&mut contents).await {
                contents.clear();
                let path = path.to_string_lossy();
                tracing::error!("Error reading file {path}: {err}");
            }
            (metadata, contents)
        }

        let (md, contents) = if let Some(ref handle) = handle {
            read_file(handle.path()).await
        } else {
            (None, Vec::new())
        };

        Self {
            handle,
            md,
            contents,
        }
    }
}

impl FileDialogResult for DesktopFileDialogResult {
    fn is_cancelled(&self) -> bool {
        self.handle.is_none()
    }

    fn creation_time(&self) -> Option<DateTime<Utc>> {
        if let Some(md) = &self.md {
            md.created().ok().map(DateTime::<Utc>::from)
        } else {
            None
        }
    }

    fn modification_time(&self) -> Option<DateTime<Utc>> {
        if let Some(md) = &self.md {
            md.modified().ok().map(DateTime::<Utc>::from)
        } else {
            None
        }
    }

    fn file_name(&self) -> Option<String> {
        self.handle.as_ref().map(|handle| handle.file_name())
    }

    fn size(&self) -> Option<u64> {
        self.md.as_ref().map(|md| md.len())
    }

    fn file_type(&self) -> Option<String> {
        if let Some(handle) = &self.handle {
            handle
                .path()
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| ".".to_owned() + x)
        } else {
            None
        }
    }

    fn contents(&self) -> &[u8] {
        &self.contents
    }

    fn write_and_refresh(&mut self, data: &[u8]) {
        // write
        if let Some(handle) = &self.handle {
            let _ = std::fs::write(handle.path(), data);
        }

        // refresh
        let md = self
            .handle
            .as_ref()
            .and_then(|x| std::fs::metadata(x.path()).ok());

        let contents = if let Some(handle) = &self.handle {
            std::fs::read(handle.path()).unwrap_or_default()
        } else {
            Vec::new()
        };

        self.md = md;
        self.contents = contents;
    }
}

pub struct DesktopUiBackend {
    // It's important that `clipboard`` gets dropped before `window`, dropping
    // them the other way around causes a segfault inside `smithay_clipboard`.
    // See:
    // - https://github.com/emilk/egui/issues/7660
    // - https://github.com/emilk/egui/issues/7743
    clipboard: Clipboard,
    window: Arc<Window>,

    event_loop: EventLoopProxy<RuffleEvent>,
    cursor_visible: bool,
    preferences: GlobalPreferences,
    preferred_cursor: MouseCursor,
    font_database: Rc<fontdb::Database>,
    file_picker: FilePicker,
}

impl DesktopUiBackend {
    pub fn new(
        window: Arc<Window>,
        event_loop: EventLoopProxy<RuffleEvent>,
        font_database: Rc<fontdb::Database>,
        preferences: GlobalPreferences,
        file_picker: FilePicker,
    ) -> Result<Self, Error> {
        // The window handle is only relevant to linux/wayland
        // If it fails it'll fallback to x11 or wlr-data-control
        let clipboard = Clipboard::new(window.display_handle().ok().map(|handle| handle.as_raw()));
        Ok(Self {
            window,
            event_loop,
            cursor_visible: true,
            clipboard,
            preferences,
            preferred_cursor: MouseCursor::Arrow,
            font_database,
            file_picker,
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
        self.clipboard.get().unwrap_or_default()
    }

    fn set_clipboard_content(&mut self, content: String) {
        self.clipboard.set_text(content);
    }

    fn set_fullscreen(&mut self, is_full: bool) -> Result<(), FullscreenError> {
        self.window.set_fullscreen(if is_full {
            Some(Fullscreen::Borderless(None))
        } else {
            None
        });
        Ok(())
    }

    fn display_root_movie_download_failed_message(&self, _invalid_swf: bool, _fetch_error: String) {
        let _ = self
            .event_loop
            .send_event(RuffleEvent::OpenDialog(DialogDescriptor::ShowMessage(
                MessageDialogConfiguration::new(
                    LocalizableText::LocalizedText("message-dialog-root-movie-load-error-title"),
                    LocalizableText::LocalizedText(
                        "message-dialog-root-movie-load-error-description",
                    ),
                ),
            )));
    }

    fn message(&self, message: &str) {
        let _ = self
            .event_loop
            .send_event(RuffleEvent::OpenDialog(DialogDescriptor::ShowMessage(
                MessageDialogConfiguration::new(
                    LocalizableText::NonLocalizedText("Ruffle".into()),
                    LocalizableText::NonLocalizedText(message.to_string().into()),
                ),
            )));
    }

    fn display_unsupported_video(&self, url: Url) {
        if url.scheme() == "javascript" {
            tracing::warn!(
                "SWF tried to run a script on desktop, but javascript calls are not allowed"
            );
            return;
        }

        let open_url_mode = self.preferences.open_url_mode();
        if open_url_mode == OpenUrlMode::Confirm {
            let message = format!("The SWF file wants to open the website {url}");
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
        } else if open_url_mode == OpenUrlMode::Deny {
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

    fn load_device_font(&self, query: &FontQuery, register: &mut dyn FnMut(FontDefinition)) {
        let name = &query.name;
        let is_bold = query.is_bold;
        let is_italic = query.is_italic;

        let query = fontdb::Query {
            families: &[Family::Name(name)],
            weight: if is_bold {
                fontdb::Weight::BOLD
            } else {
                fontdb::Weight::NORMAL
            },
            style: if is_italic {
                fontdb::Style::Italic
            } else {
                fontdb::Style::Normal
            },
            ..Default::default()
        };

        // It'd be nice if we can get the full list of candidates... Feature request?
        if let Some(id) = self.font_database.query(&query)
            && let Some(face) = self.font_database.face(id)
        {
            tracing::info!(
                "Loading device font \"{}\" for \"{name}\" (italic: {is_italic}, bold: {is_bold})",
                face.post_script_name
            );

            match load_fontdb_font(name.to_string(), face) {
                Ok(font_definition) => register(font_definition),
                Err(error) => tracing::error!("Error loading font from fontdb: {error}"),
            }
        }
    }

    #[allow(unused_variables)]
    fn sort_device_fonts(
        &self,
        query: &FontQuery,
        register: &mut dyn FnMut(FontDefinition),
    ) -> Vec<FontQuery> {
        #[cfg(feature = "fontconfig")]
        return fontconfig_sort_device_fonts(query, register);

        #[cfg(not(feature = "fontconfig"))]
        return Vec::new();
    }

    // Unused on desktop
    fn open_virtual_keyboard(&self) {}

    fn close_virtual_keyboard(&self) {}

    fn language(&self) -> LanguageIdentifier {
        self.preferences.language()
    }

    fn display_file_open_dialog(&mut self, filters: Vec<FileFilter>) -> Option<DialogResultFuture> {
        let mut dialog = AsyncFileDialog::new();

        for filter in filters {
            if cfg!(target_os = "macos")
                && let Some(mac_type) = filter.mac_type
            {
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

        let result = self.file_picker.show_dialog(dialog, |d| d.pick_file())?;

        Some(Box::pin(async move {
            let result: Result<Box<dyn FileDialogResult>, DialogLoaderError> =
                Ok(Box::new(DesktopFileDialogResult::new(result.await).await));
            result
        }))
    }

    fn display_file_save_dialog(
        &mut self,
        file_name: String,
        title: String,
    ) -> Option<DialogResultFuture> {
        // Select the location to save the file to
        let dialog = AsyncFileDialog::new()
            .set_title(&title)
            .set_file_name(&file_name);

        let result = self.file_picker.show_dialog(dialog, |d| d.save_file())?;

        Some(Box::pin(async move {
            let result: Result<Box<dyn FileDialogResult>, DialogLoaderError> =
                Ok(Box::new(DesktopFileDialogResult::new(result.await).await));
            result
        }))
    }

    fn close_file_dialog(&mut self) {}
}

fn load_font_from_file(
    path: &Path,
    name: String,
    index: u32,
    is_bold: bool,
    is_italic: bool,
) -> Result<FontDefinition<'static>> {
    let file = File::open(path).map_err(|e| anyhow!("Couldn't open font file at {path:?}: {e}"))?;

    // SAFETY: We have to assume that the font file won't change.
    // This assumption is realistic, as we're using system fonts only.
    // However, we never store other references to this data, and we reparse
    // the whole file each time we're accessing any font data.
    // Realistically, when the underlying file or memory region changes,
    // we can expect Ruffle to crash due to SIGBUS or errors when parsing.
    let mmap = unsafe { memmap2::Mmap::map(&file) };

    let mmap = mmap.map_err(|e| anyhow!("Failed to mmap font file at {path:?}: {e}"))?;
    let data = FontFileData::new(mmap);
    Ok(FontDefinition::FontFile {
        name,
        is_bold,
        is_italic,
        data,
        index,
    })
}

fn load_fontdb_font(name: String, face: &FaceInfo) -> Result<FontDefinition<'static>> {
    let is_bold = face.weight > fontdb::Weight::NORMAL;
    let is_italic = face.style != fontdb::Style::Normal;

    match &face.source {
        fontdb::Source::File(path) => {
            load_font_from_file(path, name, face.index, is_bold, is_italic)
        }

        fontdb::Source::Binary(bin) | fontdb::Source::SharedFile(_, bin) => {
            Ok(FontDefinition::FontFile {
                name,
                is_bold,
                is_italic,
                data: FontFileData::new_shared(bin.clone()),
                index: face.index,
            })
        }
    }
}

#[cfg(feature = "fontconfig")]
fn fontconfig_sort_device_fonts(
    query: &FontQuery,
    register: &mut dyn FnMut(FontDefinition),
) -> Vec<FontQuery> {
    use fontconfig::{FontFormat, Pattern};
    use std::sync::LazyLock;

    static FONTCONFIG: LazyLock<Option<fontconfig::Fontconfig>> =
        LazyLock::new(fontconfig::Fontconfig::new);

    let Some(fc) = FONTCONFIG.as_ref() else {
        return Vec::new();
    };

    let Ok(family) = std::ffi::CString::new(query.name.as_str()) else {
        tracing::error!("Cannot sort device fonts, null in font family");
        return Vec::new();
    };

    let mut pattern: Pattern<'static> = Pattern::new(fc);
    pattern.add_string(fontconfig::FC_FAMILY, family.as_c_str());

    if query.is_bold {
        pattern.add_integer(fontconfig::FC_WEIGHT, fontconfig::FC_WEIGHT_BOLD);
    }
    if query.is_italic {
        pattern.add_integer(fontconfig::FC_SLANT, fontconfig::FC_SLANT_ITALIC);
    }

    let font_set = pattern.sort_fonts(true);
    let mut font_queries = Vec::new();
    for font in font_set.iter() {
        let is_ttf = font
            .format()
            .is_ok_and(|f| matches!(f, FontFormat::TrueType));
        if !is_ttf {
            if let Some(name) = font.name() {
                tracing::info!("Skipping font '{name}' because it's not a TTF");
            }
            continue;
        }

        let (
            Some(name), //
            Some(filename),
            Some(index),
            Some(weight),
            Some(slant),
        ) = (
            font.name(),
            font.filename(),
            font.face_index(),
            font.weight(),
            font.slant(),
        )
        else {
            continue;
        };

        let Ok(index) = index.try_into() else {
            continue;
        };

        let is_bold = weight >= fontconfig::FC_WEIGHT_BOLD;
        let is_italic = slant >= fontconfig::FC_SLANT_ITALIC;

        match load_font_from_file(
            Path::new(filename),
            name.to_string(),
            index,
            is_bold,
            is_italic,
        ) {
            Ok(definition) => register(definition),
            Err(err) => {
                tracing::error!("Error loading font from fontconfig: {err}");
                continue;
            }
        }

        let query = FontQuery::new(query.font_type, name.to_string(), is_bold, is_italic);
        font_queries.push(query);
    }
    font_queries
}
