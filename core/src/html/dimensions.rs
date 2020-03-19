//! CSS dimension types
use gc_arena::Collect;
use std::cmp::{max, min, Ord};
use std::ops::{Add, AddAssign};
use swf::{Rectangle, Twips};

/// A type which represents the top-left position of a layout box.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Collect)]
#[collect(require_static)]
pub struct Position<T> {
    x: T,
    y: T,
}

impl<T> Default for Position<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            x: T::default(),
            y: T::default(),
        }
    }
}

impl<T> From<(T, T)> for Position<T> {
    fn from(pair: (T, T)) -> Self {
        Self {
            x: pair.0,
            y: pair.1,
        }
    }
}

/// A type which represents the size of a layout box.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Collect)]
#[collect(require_static)]
pub struct Size<T> {
    width: T,
    height: T,
}

impl<T> Default for Size<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            width: T::default(),
            height: T::default(),
        }
    }
}

impl<T> From<(T, T)> for Size<T> {
    fn from(pair: (T, T)) -> Self {
        Self {
            width: pair.0,
            height: pair.1,
        }
    }
}

/// A type which represents the offset and size of text-orientation-relative
/// boxes.
///
/// For the purposes of internationalization, the two axes are called "primary"
/// and "secondary", and the directions are "leading" for the direction of text
/// flow and "trailing" for the opposite direction. The meaning of these axis
/// and directions are determined by the `WritingMode` of a given `LayoutBox`.
///
/// To obtain concretely-oriented dimensions, use `into_screen_space` to
/// transform the coordinates into a pair of top, left, right, and bottom
/// coordinates.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Collect)]
#[collect(require_static)]
pub struct BoxBounds<T> {
    offset_x: T,
    width: T,
    offset_y: T,
    height: T,
}

impl<T> Default for BoxBounds<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            offset_x: Default::default(),
            width: Default::default(),
            offset_y: Default::default(),
            height: Default::default(),
        }
    }
}

impl<T> Into<Rectangle> for BoxBounds<T>
where
    T: Into<Twips> + Add<T, Output = T> + Clone,
{
    fn into(self) -> Rectangle {
        let x_max = self.extent_x().into();
        let y_max = self.extent_y().into();

        Rectangle {
            x_min: self.offset_x.into(),
            x_max,
            y_min: self.offset_y.into(),
            y_max,
        }
    }
}

impl<T> From<Rectangle> for BoxBounds<T>
where
    T: From<Twips>,
{
    fn from(bounds: Rectangle) -> Self {
        Self {
            offset_x: T::from(bounds.x_min),
            width: T::from(bounds.x_max - bounds.x_min),
            offset_y: T::from(bounds.y_min),
            height: T::from(bounds.y_max - bounds.y_min),
        }
    }
}

impl<T> BoxBounds<T>
where
    T: Clone,
{
    pub fn offset_x(&self) -> T {
        self.offset_x.clone()
    }

    pub fn offset_y(&self) -> T {
        self.offset_y.clone()
    }

    pub fn width(&self) -> T {
        self.width.clone()
    }

    pub fn height(&self) -> T {
        self.height.clone()
    }

    pub fn origin(&self) -> Position<T> {
        Position::from((self.offset_x(), self.offset_y()))
    }

    pub fn size(&self) -> Size<T> {
        Size::from((self.width(), self.height()))
    }
}

impl<T> BoxBounds<T>
where
    T: Add<T, Output = T> + Clone,
{
    pub fn extent_x(&self) -> T {
        self.offset_x.clone() + self.width.clone()
    }

    pub fn extent_y(&self) -> T {
        self.offset_y.clone() + self.height.clone()
    }
}

impl<T> Add for BoxBounds<T>
where
    T: Add<T> + Ord,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            offset_x: min(self.offset_x, rhs.offset_x),
            width: max(self.width, rhs.width),
            offset_y: min(self.offset_y, rhs.offset_y),
            height: max(self.height, rhs.height),
        }
    }
}

impl<T> AddAssign for BoxBounds<T>
where
    T: AddAssign<T> + Ord + Clone,
{
    fn add_assign(&mut self, rhs: Self) {
        self.offset_x = min(self.offset_x.clone(), rhs.offset_x);
        self.width = max(self.width.clone(), rhs.width);
        self.offset_y = min(self.offset_y.clone(), rhs.offset_y);
        self.height = max(self.height.clone(), rhs.height);
    }
}

impl<T> Add<Position<T>> for BoxBounds<T>
where
    T: Add<T, Output = T> + Clone,
{
    type Output = Self;

    fn add(self, rhs: Position<T>) -> Self::Output {
        Self {
            offset_x: self.offset_x + rhs.x,
            width: self.width,
            offset_y: self.offset_y + rhs.y,
            height: self.height,
        }
    }
}

impl<T> AddAssign<Position<T>> for BoxBounds<T>
where
    T: AddAssign<T> + Clone,
{
    fn add_assign(&mut self, rhs: Position<T>) {
        self.offset_x += rhs.x;
        self.offset_y += rhs.y;
    }
}

impl<T> Add<Size<T>> for BoxBounds<T>
where
    T: Add<T, Output = T> + Clone,
{
    type Output = Self;

    fn add(self, rhs: Size<T>) -> Self::Output {
        Self {
            offset_x: self.offset_x,
            width: self.width + rhs.width,
            offset_y: self.offset_y,
            height: self.height + rhs.height,
        }
    }
}

impl<T> AddAssign<Size<T>> for BoxBounds<T>
where
    T: AddAssign<T> + Clone,
{
    fn add_assign(&mut self, rhs: Size<T>) {
        self.width += rhs.width;
        self.height += rhs.height;
    }
}
