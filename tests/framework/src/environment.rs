use crate::options::RenderOptions;

pub use ruffle_render::backend::RenderBackend;

pub trait Environment {
    /// Checks if this environment supports rendering the given test.
    ///
    /// This isn't a guarantee that it _will_ construct a renderer,
    /// but rather a check that it theoretically _can_.
    ///
    /// This should be a cheap test to filter out test viability early,
    /// without creating any expensive rendering overhead.
    fn is_render_supported(&self, _requirements: &RenderOptions) -> bool {
        false
    }

    /// Creates a render backend for a new test run.
    ///
    /// This method must return both a [RenderBackend] and [RenderInterface] as a pair.
    ///
    /// All relevant methods in the [RenderInterface] will receive the same [RenderBackend]
    /// that was provided here with that interface.
    ///
    /// If None is returned, a test will be performed without any renderer.
    ///
    /// If [Self::is_render_supported] returned false, this won't be attempted.
    fn create_renderer(
        &self,
        _width: u32,
        _height: u32,
    ) -> Option<(Box<dyn RenderInterface>, Box<dyn RenderBackend>)> {
        None
    }
}

pub trait RenderInterface {
    /// Gets the name of this environment, for use in test reporting.
    ///
    /// This name may be used in file paths, so it should contain appropriate characters for such.
    fn name(&self) -> String;

    /// Capture the stage rendered out by the given render backend.
    ///
    /// The provided backend is guaranteed to be the same one paired with this interface.
    fn capture(&self, renderer: &mut Box<dyn RenderBackend>) -> image::RgbaImage;
}
