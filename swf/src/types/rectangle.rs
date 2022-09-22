/// A rectangular region defined by minimum and maximum x- and y-coordinate positions.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Rectangle<T> {
    /// The minimum x-position of the rectangle.
    pub x_min: T,

    /// The maximum x-position of the rectangle.
    pub x_max: T,

    /// The minimum y-position of the rectangle.
    pub y_min: T,

    /// The maximum y-position of the rectangle.
    pub y_max: T,
}
