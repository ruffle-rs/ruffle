//! Percentage type

use gc_arena::Collect;

/// Percent units for things that need to be stored as percentages.
///
/// Actual percentages (0-100) can be stored in here by `From` and `Into`
/// coercions. Thus, this wrapper serves as a unit marker. To convert into unit
/// ranges (0-1), use the `from_unit` and `into_unit` methods.
///
/// No arithmetic operators are provided on percentages as most of the math
/// they are involved in should be done in unit proportions rather than
/// percentages.
#[derive(Copy, Clone, Debug, Collect, PartialEq, PartialOrd)]
#[collect(require_static)]
pub struct Percent(f64);

impl Percent {
    /// Convert a unit proportion into a percentage.
    pub fn from_unit(unit: f64) -> Self {
        Self(unit * 100.0)
    }

    /// Convert a percentage into a unit proportion.
    pub fn into_unit(self) -> f64 {
        self.0 / 100.0
    }
}

impl From<f64> for Percent {
    fn from(percent: f64) -> Self {
        Self(percent)
    }
}

impl Into<f64> for Percent {
    fn into(self) -> f64 {
        self.0
    }
}

/// Degree units for things that need to be stored as degrees.
///
/// Actual degrees (0-360, or -179-180) can be stored in here by `From` and
/// `Into` coercions. No wrapping is done on the type to keep the conversion
/// lossless. To convert into radians (0-2π, or -π-π), use the `from_radians`
/// and `into_radians` methods.
///
/// No arithmetic operators are provided on degrees as most of the math they
/// are involved in should be done in unit proportions rather than percentages.
#[derive(Copy, Clone, Debug, Collect, PartialEq, PartialOrd)]
#[collect(require_static)]
pub struct Degrees(f64);

impl Degrees {
    /// Convert a radian value into degrees.
    pub fn from_radians(rads: f64) -> Self {
        Self(rads.to_degrees())
    }

    /// Convert a degree value into radians.
    pub fn into_radians(self) -> f64 {
        self.0.to_radians()
    }
}

impl From<f64> for Degrees {
    fn from(degrees: f64) -> Self {
        Self(degrees)
    }
}

impl Into<f64> for Degrees {
    fn into(self) -> f64 {
        self.0
    }
}
