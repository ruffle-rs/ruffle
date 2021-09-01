use std::ops::{Bound, Range, RangeBounds};
use std::ptr::NonNull;

const WIDE_MASK: usize = 0x8000_0000;
pub const MAX_STRING_LEN: usize = WIDE_MASK - 1;

use super::Units;

/// The internal representation of a string slice; doesn't carry any lifetime or ownership information.
/// This has the size of two `usize`s, thanks to some bitpacking:
///  - `data` points to a slice of `u8`s or `u16`s, depending of the value of `Self::is_wide`;
///  - `len` is always less than `WIDE_MASK`;
///  - on 32-bits targets, the high bit of the `len` field is used to store the `is_wide` flag.
#[derive(Copy, Clone)]
pub(super) struct WStrPtr {
    data: NonNull<()>,
    #[cfg(target_pointer_width = "32")]
    packed_len: u32,
    #[cfg(target_pointer_width = "64")]
    len: u32,
    #[cfg(target_pointer_width = "64")]
    is_wide: bool,
}

unsafe impl Send for WStrPtr {}
unsafe impl Sync for WStrPtr {}

impl WStrPtr {
    /// # Safety
    ///
    /// - `data` must point to a valid allocated region (possibly uninitialized):
    ///   - for [`Units::Bytes`], the allocation must be valid for a `[u8; len]`;
    ///   - for [`Units::Wide`], the allocation must be valid for a `[u16; len]`;
    /// - `len <= MAX_STRING_LEN` must hold;
    /// - the allocation pointed by `data` must stay live while this `RawStr`,
    ///   or any reference derived from it, exists.
    #[inline]
    pub unsafe fn new(data: Units<*const u8, *const u16>, len: usize) -> Self {
        let len = len as u32;
        let (is_wide, data) = match data {
            Units::Bytes(us) => (false, NonNull::new_unchecked(us as *mut u8 as *mut ())),
            Units::Wide(us) => (true, NonNull::new_unchecked(us as *mut u16 as *mut ())),
        };

        Self::from_parts(data, len, is_wide)
    }

    /// # Safety
    ///
    /// See `Self::new`.
    unsafe fn from_parts(data: NonNull<()>, len: u32, is_wide: bool) -> Self {
        #[cfg(target_pointer_width = "32")]
        return Self {
            data,
            packed_len: len | if is_wide { WIDE_MASK as u32 } else { 0 },
        };

        #[cfg(target_pointer_width = "64")]
        return Self { data, len, is_wide };
    }

    #[inline]
    pub fn data(self) -> Units<NonNull<u8>, NonNull<u16>> {
        if self.is_wide() {
            Units::Wide(self.data.cast())
        } else {
            Units::Bytes(self.data.cast())
        }
    }

    #[inline]
    pub fn len(self) -> usize {
        #[cfg(target_pointer_width = "64")]
        return self.len as usize;

        #[cfg(target_pointer_width = "32")]
        return self.packed_len as usize & MAX_STRING_LEN;
    }

    #[inline]
    pub fn is_wide(self) -> bool {
        #[cfg(target_pointer_width = "64")]
        return self.is_wide;

        #[cfg(target_pointer_width = "32")]
        return self.packed_len as usize > MAX_STRING_LEN;
    }

    #[inline]
    pub fn units(self) -> Units<NonNull<[u8]>, NonNull<[u16]>> {
        if self.is_wide() {
            let ptr = self.data.cast::<u16>().as_ptr();
            // SAFETY: ptr is non-null.
            Units::Wide(unsafe {
                NonNull::new_unchecked(std::ptr::slice_from_raw_parts_mut(ptr, self.len()))
            })
        } else {
            let ptr = self.data.cast::<u8>().as_ptr();
            // SAFETY: ptr is non-null.
            Units::Bytes(unsafe {
                NonNull::new_unchecked(std::ptr::slice_from_raw_parts_mut(ptr, self.len()))
            })
        }
    }

    /// # Safety
    /// - `index <= self.len()` must hold.
    #[inline]
    pub unsafe fn offset(self, index: usize) -> NonNull<()> {
        // SAFETY: MAX_STRING_LEN = i32::MAX, so index can be casted to isize
        // on 32-bit and 64-bit targets.
        let index = index as isize;
        if self.is_wide() {
            let ptr = self.data.cast::<u16>().as_ptr();
            NonNull::new_unchecked(ptr.offset(index)).cast()
        } else {
            let ptr = self.data.cast::<u8>().as_ptr();
            NonNull::new_unchecked(ptr.offset(index)).cast()
        }
    }

    /// # Safety
    /// - `index < self.len()` must hold.
    /// - the code unit at the given index must be initialized.
    #[inline]
    pub unsafe fn get(self, index: usize) -> u16 {
        let ptr = self.offset(index).as_ptr();
        if self.is_wide() {
            *(ptr as *mut u16)
        } else {
            (*(ptr as *mut u8)) as u16
        }
    }

    /// # Safety
    /// - `range.start <= range.end <= self.len()` must hold.
    #[inline]
    pub unsafe fn slice(self, range: Range<usize>) -> Self {
        let data = self.offset(range.start);
        let len = (range.end - range.start) as u32;
        Self::from_parts(data, len, self.is_wide())
    }

    #[inline]
    pub fn try_slice<R: RangeBounds<usize>>(self, range: R) -> Option<Self> {
        let min = match range.start_bound() {
            Bound::Included(n) => *n,
            Bound::Excluded(n) => n.checked_add(1)?,
            Bound::Unbounded => 0,
        };
        let max = match range.end_bound() {
            Bound::Included(n) => n.checked_add(1)?,
            Bound::Excluded(n) => *n,
            Bound::Unbounded => self.len(),
        };

        if min <= self.len() && max <= self.len() && min <= max {
            // SAFETY: we checked that the indices are in-bounds.
            Some(unsafe { self.slice(min..max) })
        } else {
            None
        }
    }
}
