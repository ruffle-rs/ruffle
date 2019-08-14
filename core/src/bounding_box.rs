use crate::matrix::Matrix;
use swf::Twips;

#[derive(Clone, Debug)]
pub struct BoundingBox {
    pub x_min: Twips,
    pub y_min: Twips,
    pub x_max: Twips,
    pub y_max: Twips,
    pub valid: bool,
}

impl BoundingBox {
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
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self {
            x_min: Default::default(),
            y_min: Default::default(),
            x_max: Default::default(),
            y_max: Default::default(),
            valid: false,
        }
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
