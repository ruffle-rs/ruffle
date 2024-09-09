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
}

impl FilePicker {
    pub fn new(parent: Weak<Window>) -> Self {
        Self {
            data: Arc::new(FilePickerData {
                parent,
                picking: AtomicBool::new(false),
            }),
        }
    }

    pub async fn pick_file(&self, dir: Option<PathBuf>) -> Option<PathBuf> {
        if self.data.picking.swap(true, Ordering::SeqCst) {
            // Already picking
            return None;
        }

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

        let result = dialog.pick_file().await.map(|h| h.into());
        self.data.picking.store(false, Ordering::SeqCst);
        result
    }
}
