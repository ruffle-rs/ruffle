//! Percentage type

use gc_arena::Collect;

/// Percent units for things that need to be stored as percentages.
///
/// A percentage can be stored in two forms:
///
///  * Unit proportions, which represent 0 through 100% as [0.0, 1.0]
///  * Fractions, which represent 0 through 100% as [0.0, 100.0]
///
/// This unit wrapper provides no coercions; you must explicitly ask for your
/// percentages to be converted in a given form. This is because different VMs
/// represent and store percentages differently. For the same reason, no
/// arithmetic operators are provided on `Percent` to avoid potential implicit
/// coercions.
#[derive(Copy, Clone, Debug, Collect, PartialEq, PartialOrd)]
#[collect(require_static)]
pub enum Percent {
    Unit(f64),
    Fraction(f64),
}

impl Percent {
    /// Construct a percent from a unit proportion.
    pub fn from_unit(unit: f64) -> Self {
        Self::Unit(unit)
    }

    /// Construct a percent from an upper fraction.
    pub fn from_fraction(unit: f64) -> Self {
        Self::Fraction(unit)
    }

    /// Get the unit proportion form of a percentage.
    pub fn into_unit(self) -> f64 {
        match self {
            Self::Unit(unit) => unit,
            Self::Fraction(pct) => pct / 100.0,
        }
    }

    /// Get the fraction form of a percentage.
    pub fn into_fraction(self) -> f64 {
        match self {
            Self::Unit(unit) => unit * 100.0,
            Self::Fraction(pct) => pct,
        }
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

impl From<Degrees> for f64 {
    fn from(degrees: Degrees) -> Self {
        degrees.0
    }
}
