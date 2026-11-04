/// Percent units for things that need to be stored as percentages.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Percent(f64);

impl Percent {
    /// Return as percentage in [0.0, 100.0].
    pub fn percent(self) -> f64 {
        self.0
    }

    /// Convert a unit proportion in [0.0, 1.0] to percentage.
    pub fn from_unit(unit: f64) -> Self {
        Self(unit * 100.0)
    }

    /// Return as unit proportion in [0.0, 1.0].
    pub fn unit(self) -> f64 {
        self.0 / 100.0
    }
}

impl From<f64> for Percent {
    fn from(percent: f64) -> Self {
        Self(percent)
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
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
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

    /// Wrap into `[-180, 180]`, matching Flash's 2D `DisplayObject.rotation`.
    pub fn wrap_180(degrees: f64) -> f64 {
        let mut wrapped = degrees % 360.0;
        if wrapped < -180.0 {
            wrapped += 360.0;
        } else if wrapped > 180.0 {
            wrapped -= 360.0;
        }
        wrapped
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_180_matches_flash_rotation_range() {
        assert_eq!(Degrees::wrap_180(0.0), 0.0);
        assert_eq!(Degrees::wrap_180(180.0), 180.0);
        assert_eq!(Degrees::wrap_180(-180.0), -180.0);
        assert_eq!(Degrees::wrap_180(181.0), -179.0);
        assert_eq!(Degrees::wrap_180(-181.0), 179.0);
        assert_eq!(Degrees::wrap_180(360.0), 0.0);
        assert_eq!(Degrees::wrap_180(540.0), 180.0);
        assert_eq!(Degrees::wrap_180(-540.0), -180.0);
    }
}
