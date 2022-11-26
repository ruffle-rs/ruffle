//! CSS dimension types
use gc_arena::Collect;
use std::cmp::{max, min, Ord};
use std::ops::{Add, AddAssign, Sub};
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

impl<T> Position<T> {
    pub fn set_x(&mut self, x: T) {
        self.x = x;
    }

    pub fn set_y(&mut self, y: T) {
        self.y = y;
    }
}

impl<T> Position<T>
where
    T: Copy,
{
    pub fn x(&self) -> T {
        self.x
    }

    pub fn y(&self) -> T {
        self.y
    }
}

impl<T> Add for Position<T>
where
    T: Add<T, Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> AddAssign for Position<T>
where
    T: AddAssign,
{
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
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

#[allow(dead_code)]
impl<T> Size<T>
where
    T: Copy,
{
    pub fn width(&self) -> T {
        self.width
    }

    pub fn height(&self) -> T {
        self.height
    }
}

impl<T> From<Position<T>> for Size<T> {
    fn from(size: Position<T>) -> Self {
        Self {
            width: size.x,
            height: size.y,
        }
    }
}

/// A type which represents the offset and size of a text box.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Collect)]
#[collect(require_static)]
pub struct BoxBounds<T> {
    offset_x: T,
    extent_x: T,
    offset_y: T,
    extent_y: T,
}

impl<T> Default for BoxBounds<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            offset_x: Default::default(),
            extent_x: Default::default(),
            offset_y: Default::default(),
            extent_y: Default::default(),
        }
    }
}

impl<T> From<BoxBounds<T>> for Rectangle<Twips>
where
    T: Into<Twips> + Add<T, Output = T>,
{
    fn from(bounds: BoxBounds<T>) -> Rectangle<Twips> {
        Rectangle {
            x_min: bounds.offset_x.into(),
            x_max: bounds.extent_x.into(),
            y_min: bounds.offset_y.into(),
            y_max: bounds.extent_y.into(),
        }
    }
}

impl<T> From<Rectangle<Twips>> for BoxBounds<T>
where
    T: From<Twips>,
{
    fn from(bounds: Rectangle<Twips>) -> Self {
        Self {
            offset_x: T::from(bounds.x_min),
            extent_x: T::from(bounds.x_max),
            offset_y: T::from(bounds.y_min),
            extent_y: T::from(bounds.y_max),
        }
    }
}

#[allow(dead_code)]
impl<T> BoxBounds<T>
where
    T: Add<T, Output = T> + Sub<T, Output = T> + Copy,
{
    pub fn from_position_and_size(pos: Position<T>, size: Size<T>) -> Self {
        Self {
            offset_x: pos.x,
            extent_x: pos.x + size.width,
            offset_y: pos.y,
            extent_y: pos.y + size.height,
        }
    }

    pub fn into_position_and_size(self) -> (Position<T>, Size<T>) {
        let width = self.extent_x - self.offset_x;
        let height = self.extent_y - self.offset_y;

        (
            (self.offset_x, self.offset_y).into(),
            (width, height).into(),
        )
    }
}

#[allow(dead_code)]
impl<T> BoxBounds<T>
where
    T: Copy,
{
    pub fn offset_x(&self) -> T {
        self.offset_x
    }

    pub fn offset_y(&self) -> T {
        self.offset_y
    }

    pub fn extent_x(&self) -> T {
        self.extent_x
    }

    pub fn extent_y(&self) -> T {
        self.extent_y
    }

    pub fn origin(&self) -> Position<T> {
        (self.offset_x(), self.offset_y()).into()
    }

    pub fn extent(&self) -> Position<T> {
        (self.extent_x(), self.extent_y()).into()
    }
}

impl<T> BoxBounds<T>
where
    T: Sub<T, Output = T> + Copy,
{
    pub fn width(&self) -> T {
        self.extent_x() - self.offset_x()
    }

    pub fn height(&self) -> T {
        self.extent_y() - self.offset_y()
    }
}

#[allow(dead_code)]
impl<T> BoxBounds<T>
where
    T: Add<T, Output = T> + Copy,
{
    pub fn with_size(self, new_size: Size<T>) -> Self {
        Self {
            offset_x: self.offset_x,
            extent_x: self.offset_x + new_size.width,
            offset_y: self.offset_y,
            extent_y: self.offset_y + new_size.height,
        }
    }
}

impl<T> Add for BoxBounds<T>
where
    T: Add<T> + Ord + Copy,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            offset_x: min(self.offset_x, rhs.offset_x),
            extent_x: max(self.extent_x, rhs.extent_x),
            offset_y: min(self.offset_y, rhs.offset_y),
            extent_y: max(self.extent_y, rhs.extent_y),
        }
    }
}

impl<T> AddAssign for BoxBounds<T>
where
    T: AddAssign<T> + Ord + Copy,
{
    fn add_assign(&mut self, rhs: Self) {
        self.offset_x = min(self.offset_x, rhs.offset_x);
        self.extent_x = max(self.extent_x, rhs.extent_x);
        self.offset_y = min(self.offset_y, rhs.offset_y);
        self.extent_y = max(self.extent_y, rhs.extent_y);
    }
}

impl<T> Add<Position<T>> for BoxBounds<T>
where
    T: Add<T, Output = T> + Copy,
{
    type Output = Self;

    fn add(self, rhs: Position<T>) -> Self::Output {
        Self {
            offset_x: self.offset_x + rhs.x,
            extent_x: self.extent_x + rhs.x,
            offset_y: self.offset_y + rhs.y,
            extent_y: self.extent_y + rhs.y,
        }
    }
}

impl<T> AddAssign<Position<T>> for BoxBounds<T>
where
    T: AddAssign<T> + Copy,
{
    fn add_assign(&mut self, rhs: Position<T>) {
        self.offset_x += rhs.x;
        self.extent_x += rhs.x;
        self.offset_y += rhs.y;
        self.extent_y += rhs.y;
    }
}

impl<T> Add<Size<T>> for BoxBounds<T>
where
    T: Add<T, Output = T> + Copy,
{
    type Output = Self;

    fn add(self, rhs: Size<T>) -> Self::Output {
        Self {
            offset_x: self.offset_x,
            extent_x: self.extent_x + rhs.width,
            offset_y: self.offset_y,
            extent_y: self.extent_y + rhs.height,
        }
    }
}

impl<T> AddAssign<Size<T>> for BoxBounds<T>
where
    T: AddAssign<T> + Copy,
{
    fn add_assign(&mut self, rhs: Size<T>) {
        self.extent_x += rhs.width;
        self.extent_y += rhs.height;
    }
}
