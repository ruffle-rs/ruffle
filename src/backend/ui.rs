pub mod glutin;

pub trait UiBackend {
    fn poll_events(&mut self) -> bool;
}
