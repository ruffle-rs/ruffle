use gc_arena::Collect;

/// Represents the sound transform of sounds played inside a Flash MovieClip.
/// Every value is a percentage (0-100), but out of range values are allowed.
/// In AVM1, this is returned by `Sound.getTransform`.
/// In AVM2, this is returned by `Sprite.soundTransform`.
#[derive(Debug, PartialEq, Eq, Clone, Collect)]
#[collect(require_static)]
pub struct SoundTransform {
    pub volume: i32,
    pub left_to_left: i32,
    pub left_to_right: i32,
    pub right_to_left: i32,
    pub right_to_right: i32,
}

impl SoundTransform {
    pub const MAX_VOLUME: i32 = 100;

    /// Applies another SoundTransform on top of this SoundTransform.
    pub fn concat(&mut self, other: &SoundTransform) {
        const MAX_VOLUME: i64 = SoundTransform::MAX_VOLUME as i64;

        // It seems like Flash masks the results below to 30-bit integers:
        // * Negative values are equivalent to their absolute value (their sign bit is unset).
        // * Specifically, 0x40000000, -0x40000000 and -0x80000000 are equivalent to zero.
        const MASK: i32 = (1 << 30) - 1;

        self.volume = (i64::from(self.volume) * i64::from(other.volume) / MAX_VOLUME) as i32 & MASK;

        // This is a 2x2 matrix multiply between the transforms.
        // Done with integer math to match Flash behavior.
        let ll0: i64 = self.left_to_left.into();
        let lr0: i64 = self.left_to_right.into();
        let rl0: i64 = self.right_to_left.into();
        let rr0: i64 = self.right_to_right.into();
        let ll1: i64 = other.left_to_left.into();
        let lr1: i64 = other.left_to_right.into();
        let rl1: i64 = other.right_to_left.into();
        let rr1: i64 = other.right_to_right.into();
        self.left_to_left = ((ll0 * ll1 + rl0 * lr1) / MAX_VOLUME) as i32 & MASK;
        self.left_to_right = ((lr0 * ll1 + rr0 * lr1) / MAX_VOLUME) as i32 & MASK;
        self.right_to_left = ((ll0 * rl1 + rl0 * rr1) / MAX_VOLUME) as i32 & MASK;
        self.right_to_right = ((lr0 * rl1 + rr0 * rr1) / MAX_VOLUME) as i32 & MASK;
    }

    /// Returns the pan of this transform.
    /// -100 is full left and 100 is full right.
    /// This matches the behavior of AVM1 `Sound.getPan()`
    pub fn pan(&self) -> i32 {
        // It's not clear why Flash has the weird `abs` behavior, but this
        // mathes the values that Flash returns (see `sound` regression test).
        if self.left_to_left != Self::MAX_VOLUME {
            Self::MAX_VOLUME - self.left_to_left.abs()
        } else {
            self.right_to_right.abs() - Self::MAX_VOLUME
        }
    }

    /// Sets this transform of this pan.
    /// -100 is full left and 100 is full right.
    /// This matches the behavior of AVM1 `Sound.setPan()`.
    pub fn set_pan(&mut self, pan: i32) {
        if pan >= 0 {
            self.left_to_left = Self::MAX_VOLUME - pan;
            self.right_to_right = Self::MAX_VOLUME;
        } else {
            self.left_to_left = Self::MAX_VOLUME;
            self.right_to_right = Self::MAX_VOLUME + pan;
        }
        self.left_to_right = 0;
        self.right_to_left = 0;
    }
}

impl Default for SoundTransform {
    fn default() -> Self {
        Self {
            volume: 100,
            left_to_left: 100,
            left_to_right: 0,
            right_to_left: 0,
            right_to_right: 100,
        }
    }
}
