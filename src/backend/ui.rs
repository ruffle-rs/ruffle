#[cfg(not(target_arch = "wasm32"))]
pub mod glutin;

#[cfg(target_arch = "wasm32")]
pub mod web_canvas;

pub trait UiBackend {
    fn poll_events(&mut self) -> bool;
}
