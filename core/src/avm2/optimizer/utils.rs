use std::fmt;

/// A bitset only large enough to store 32 bits.
///
/// Trying to create a bitset with more items will result in it being silently
/// truncated.
#[derive(Clone)]
pub struct SmallBitSet {
    bits: u32,
    len: u8,
}

impl SmallBitSet {
    pub fn get(&self, index: usize) -> bool {
        if index >= self.len() {
            panic!("Attempted to access bit out of range");
        }

        ((self.bits >> index) & 1) == 1
    }

    pub fn set(&mut self, index: usize, value: bool) {
        if value {
            self.bits |= 1 << index;
        } else {
            self.bits &= !(1 << index);
        }
    }

    pub fn iter(&self) -> SmallBitSetIter {
        SmallBitSetIter {
            // Clone is free
            state: self.clone(),
            next: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }
}

impl FromIterator<bool> for SmallBitSet {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = bool>,
    {
        let mut len: u8 = 0;
        let mut bits = 0;

        // NOTE: We take only 32 items from the iterator, as that's the most
        // that can fit in this bitset.
        let iter = iter.into_iter().enumerate().take(u32::BITS as usize);

        for (i, bit) in iter {
            if bit {
                bits |= 1 << i;
            }

            // size should never overflow
            len = len.strict_add(1);
        }

        Self { bits, len }
    }
}

pub struct SmallBitSetIter {
    state: SmallBitSet,
    next: usize,
}

impl Iterator for SmallBitSetIter {
    type Item = bool;

    fn next(&mut self) -> Option<bool> {
        if self.next >= self.state.len() {
            None
        } else {
            let value = self.state.get(self.next);
            self.next += 1;

            Some(value)
        }
    }
}

impl fmt::Debug for SmallBitSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut result = String::new();
        for i in 0..self.len() {
            if self.get(i) {
                result.push('1');
            } else {
                result.push('0');
            }
        }

        write!(f, "{}", result)
    }
}
