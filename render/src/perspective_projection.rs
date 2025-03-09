#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PerspectiveProjection {
    /// Unit: degree. Must be greater than 0 and less than 180.
    pub field_of_view: f64,

    /// The center of the projection in (x, y).
    pub center: (f64, f64),
}

impl Default for PerspectiveProjection {
    fn default() -> Self {
        Self {
            field_of_view: 55.0,
            center: (250.0, 250.0),
        }
    }
}
