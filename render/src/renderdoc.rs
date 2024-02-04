use std::cell::RefCell;

use renderdoc::RenderDoc;

thread_local! {
    pub static RENDERDOC: RefCell<Option<Result<RenderDoc<renderdoc::V141>, renderdoc::Error>>> = const { RefCell::new(None) };
}

pub fn start_frame_capture() {
    RENDERDOC.with(|renderdoc_cell| {
        let mut write = renderdoc_cell.borrow_mut();
        let renderdoc = write.get_or_insert_with(RenderDoc::new);
        match renderdoc {
            Ok(renderdoc) => renderdoc.start_frame_capture(std::ptr::null(), std::ptr::null()),
            Err(e) => tracing::error!("Renderdoc was not initialized: {:?}", e),
        }
    })
}

pub fn end_frame_capture() {
    RENDERDOC.with(|renderdoc_cell| {
        let mut write = renderdoc_cell.borrow_mut();
        let renderdoc = write.as_mut().expect("start_frame_capture was not called");
        // We already logged an error in `start_frame_capture` if Renderdoc wasn't initialized,
        // so there's no need to log one again here
        if let Ok(renderdoc) = renderdoc {
            renderdoc.end_frame_capture(std::ptr::null(), std::ptr::null());
        }
    })
}
