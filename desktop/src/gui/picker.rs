use rfd::AsyncFileDialog;
use std::{
    path::PathBuf,
    sync::{Arc, Weak},
};
use winit::window::Window;

#[derive(Clone)]
pub struct FilePicker {
    data: Arc<FilePickerData>,
}

struct FilePickerData {
    parent: Weak<Window>,
}

impl FilePicker {
    pub fn new(parent: Weak<Window>) -> Self {
        Self {
            data: Arc::new(FilePickerData { parent }),
        }
    }

    pub async fn pick_file(&self, dir: Option<PathBuf>) -> Option<PathBuf> {
        let mut dialog = AsyncFileDialog::new()
            .add_filter("Flash Files", &["swf", "spl", "ruf"])
            .add_filter("All Files", &["*"])
            .set_title("Load a Flash File");

        if let Some(dir) = dir {
            dialog = dialog.set_directory(dir);
        }

        if let Some(parent) = self.data.parent.upgrade() {
            dialog = dialog.set_parent(&parent);
        }

        dialog.pick_file().await.map(|h| h.into())
    }
}
