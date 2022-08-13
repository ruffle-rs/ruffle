use gc_arena::Collect;
use ruffle_render::matrix::Matrix;
use swf::Twips;

#[derive(Clone, Collect, Debug, Default, Eq, PartialEq)]
#[collect(require_static)]
pub struct BoundingBox {
    pub x_min: Twips,
    pub y_min: Twips,
    pub x_max: Twips,
    pub y_max: Twips,
    pub valid: bool,
}

impl BoundingBox {
    /// Clamps the given point inside this bounding box.
    pub fn clamp(&self, (x, y): (Twips, Twips)) -> (Twips, Twips) {
        if self.valid {
            (
                x.clamp(self.x_min, self.x_max),
                y.clamp(self.y_min, self.y_max),
            )
        } else {
            (x, y)
        }
    }

    pub fn transform(&self, matrix: &Matrix) -> Self {
        if !self.valid {
            return Self::default();
        }

        use std::cmp::{max, min};
        let pt0 = *matrix * (self.x_min, self.y_min);
        let pt1 = *matrix * (self.x_min, self.y_max);
        let pt2 = *matrix * (self.x_max, self.y_min);
        let pt3 = *matrix * (self.x_max, self.y_max);
        BoundingBox {
            x_min: min(pt0.0, min(pt1.0, min(pt2.0, pt3.0))),
            y_min: min(pt0.1, min(pt1.1, min(pt2.1, pt3.1))),
            x_max: max(pt0.0, max(pt1.0, max(pt2.0, pt3.0))),
            y_max: max(pt0.1, max(pt1.1, max(pt2.1, pt3.1))),
            valid: true,
        }
    }

    pub fn encompass(&mut self, x: Twips, y: Twips) {
        if self.valid {
            if x < self.x_min {
                self.x_min = x;
            }
            if x > self.x_max {
                self.x_max = x;
            }
            if y < self.y_min {
                self.y_min = y;
            }
            if y > self.y_max {
                self.y_max = y;
            }
        } else {
            self.x_min = x;
            self.x_max = x;
            self.y_min = y;
            self.y_max = y;
            self.valid = true;
        }
    }

    pub fn union(&mut self, other: &BoundingBox) {
        use std::cmp::{max, min};
        if other.valid {
            if self.valid {
                self.x_min = min(self.x_min, other.x_min);
                self.x_max = max(self.x_max, other.x_max);
                self.y_min = min(self.y_min, other.y_min);
                self.y_max = max(self.y_max, other.y_max);
            } else {
                *self = other.clone();
            }
        }
    }

    pub fn intersects(&self, other: &BoundingBox) -> bool {
        if !self.valid || !other.valid {
            return false;
        }

        use std::cmp::{max, min};
        let x_min = max(self.x_min, other.x_min);
        let y_min = max(self.y_min, other.y_min);
        let x_max = min(self.x_max, other.x_max);
        let y_max = min(self.y_max, other.y_max);

        x_min <= x_max && y_min <= y_max
    }

    pub fn contains(&self, (x, y): (Twips, Twips)) -> bool {
        self.valid && x >= self.x_min && x <= self.x_max && y >= self.y_min && y <= self.y_max
    }

    /// Set the X coordinate to a particular value, maintaining the width of
    /// this box (if possible).
    pub fn set_x(&mut self, x: Twips) {
        let width = self.width();
        self.x_min = x;
        self.x_max = x + width;

        if self.y_max >= self.y_min {
            self.valid = true;
        }
    }

    /// Set the Y coordinate to a particular value, maintaining the width of
    /// this box (if possible).
    pub fn set_y(&mut self, y: Twips) {
        let height = self.height();
        self.y_min = y;
        self.y_max = y + height;

        if self.x_max >= self.x_min {
            self.valid = true;
        }
    }

    /// Determine the width of the bounding box.
    pub fn width(&self) -> Twips {
        if self.valid {
            self.x_max - self.x_min
        } else {
            Default::default()
        }
    }

    /// Adjust the width of the bounding box.
    pub fn set_width(&mut self, width: Twips) {
        self.x_max = self.x_min + width;

        self.valid = self.x_max >= self.x_min && self.y_max >= self.y_min;
    }

    /// Determine the height of the bounding box.
    pub fn height(&self) -> Twips {
        if self.valid {
            self.y_max - self.y_min
        } else {
            Default::default()
        }
    }

    /// Adjust the height of the bounding box.
    pub fn set_height(&mut self, height: Twips) {
        self.y_max = self.y_min + height;

        self.valid = self.x_max >= self.x_min && self.y_max >= self.y_min;
    }
}

impl From<swf::Rectangle> for BoundingBox {
    fn from(rect: swf::Rectangle) -> Self {
        Self {
            x_min: rect.x_min,
            y_min: rect.y_min,
            x_max: rect.x_max,
            y_max: rect.y_max,
            valid: true,
        }
    }
}

impl From<&swf::Rectangle> for BoundingBox {
    fn from(rect: &swf::Rectangle) -> Self {
        Self {
            x_min: rect.x_min,
            y_min: rect.y_min,
            x_max: rect.x_max,
            y_max: rect.y_max,
            valid: true,
        }
    }
}
