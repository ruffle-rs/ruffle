//! Types and methods related to indices

use crate::string::WStr;

/// An index into an AVM2 Array, Vector, or ByteArray.
///
/// This is exactly the same as an ordinary `u32`, with the exception that it
/// never holds `u32::MAX`, which matches how avmplus deals with indexes into
/// those objects.
#[derive(Clone, Copy, Debug)]
pub struct ArrayIndex(u32);

impl ArrayIndex {
    pub fn from_i32(value: i32) -> Option<Self> {
        // All positive values work (as the largest possible `i32` value is less
        // than `u32::MAX`).

        if value < 0 {
            None
        } else {
            Some(Self(value as u32))
        }
    }

    pub fn from_u32(value: u32) -> Option<Self> {
        // All values work except u32::MAX

        if value == u32::MAX {
            None
        } else {
            Some(Self(value))
        }
    }

    pub fn from_f64(value: f64) -> Option<Self> {
        // All `f64`s that can be losslessly represented by a `u32` work, except
        // `u32::MAX`.

        if value == (value as u32 as f64) {
            Self::from_u32(value as u32)
        } else {
            None
        }
    }

    pub fn from_wstr(name: &WStr) -> Option<Self> {
        // Call `parse_u32_index` with a highest permitted value of
        // `u32::MAX - 1`, which means that `u32::MAX` will be rejected.

        parse_u32_index(name, u32::MAX - 1).map(Self)
    }

    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}

/// Parse a string as a number, using the same logic avmplus uses for parsing
/// array indices and object uint keys.
///
/// This method forbids numbers with leading zeroes and sign. See
/// String::getIntAtom` and `String::parseIndex` in avmplus.
pub fn parse_u32_index(name: &WStr, highest_permitted: u32) -> Option<u32> {
    if name.is_empty() {
        return None;
    }

    // Leading zeroes aren't allowed
    if name.at(0) == (b'0' as u16) && name.len() > 1 {
        return None;
    }

    let mut result: u32 = 0;

    for unit in name.iter() {
        let unit = u8::try_from(unit)
            .ok()
            .filter(|u| *u >= b'0' && *u <= b'9')?;

        result = result.checked_mul(10)?;
        result = result.checked_add((unit - b'0') as u32)?;
    }

    if result > highest_permitted {
        None
    } else {
        Some(result)
    }
}
