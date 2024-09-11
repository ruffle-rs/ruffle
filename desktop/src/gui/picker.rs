use super::text;
use crate::preferences::GlobalPreferences;
use rfd::AsyncFileDialog;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Weak,
    },
};
use winit::window::Window;

#[derive(Clone)]
pub struct FilePicker {
    data: Arc<FilePickerData>,
}

struct FilePickerData {
    parent: Weak<Window>,
    picking: AtomicBool,
    preferences: GlobalPreferences,
}

impl FilePicker {
    pub fn new(parent: Weak<Window>, preferences: GlobalPreferences) -> Self {
        Self {
            data: Arc::new(FilePickerData {
                parent,
                picking: AtomicBool::new(false),
                preferences,
            }),
        }
    }

    pub async fn pick_ruffle_file(&self, dir: Option<PathBuf>) -> Option<PathBuf> {
        if self.data.picking.swap(true, Ordering::SeqCst) {
            // Already picking
            return None;
        }

        let locale = &self.data.preferences.language();
        let mut dialog = AsyncFileDialog::new()
            .add_filter(
                text(locale, "file-picker-filter-supported"),
                &["swf", "spl", "ruf"],
            )
            .add_filter(text(locale, "file-picker-filter-swf"), &["swf"])
            .add_filter(text(locale, "file-picker-filter-spl"), &["spl"])
            .add_filter(text(locale, "file-picker-filter-ruf"), &["ruf"])
            .add_filter(text(locale, "file-picker-filter-all"), &["*"])
            .set_title(text(locale, "file-picker-title-open-file"));

        if let Some(dir) = dir {
            dialog = dialog.set_directory(dir);
        }

        if let Some(parent) = self.data.parent.upgrade() {
            dialog = dialog.set_parent(&parent);
        }

        let result = dialog.pick_file().await.map(|h| h.into());
        self.data.picking.store(false, Ordering::SeqCst);
        result
    }
}
