//! Raw pointers to `WStr` slices.
//!
//! # Internal representation
//!
//! What we actually want here is a custom DST, but they don't exist so we must cheat
//! and abuse the slice metadata field.
//!
//! The data pointer points to the start of the units buffer, which is either a
//! `[u8]` (`Units::Bytes`) or a `[u16]` (`Units::Wide`).
//!
//! String lengths are limited to less than `2³¹` units long, and the kind of
//! buffer is indicated by the 32nd bit of the raw slice length:
//!  - for `Units::Bytes`, it is a zero;
//!  - for `Units::Wide`, it is a one.
//!
//! Note that on 64-bits targets, this leaves the high 32 bits of the length unused.
//!
//! # (Un)soundness
//!
//! Unfortunately, this scheme is technically unsound under Stacked Borrows because of provenance:
//! when we cast a `*mut [u8]` or `*mut [u16]` to a `*mut WStr`, we lose the provenance over the
//! original buffer as `[()]` always occupies zero bytes (which is what allows use to mess up the slice length).
//!
//! As such, when we access the buffer data (e.g. in `read_at` or when converting back to raw buffer references),
//! the read is considered out-of-bounds by Stacked Borrows and causes undefined behavior.
//!
//! This unsoundness doesn't seem to manifest in practice though, as Rust doesn't pass through slice
//! length information to LLVM (yet?).
//!
//! One observable consequence of this is that `std::mem::size_of_val::<WStr>` won't return the actual
//! byte length of the string contents, but will instead always return 0.

use core::mem::transmute;
use core::ops::Range;
use core::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};

use super::{Units, WStr};

#[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
compile_error!("WStr only supports 32-bits and 64-bits targets");

const WIDE_MASK: u32 = 0x8000_0000;
const _: () = assert!(WIDE_MASK as usize == WStr::MAX_LEN + 1);

/// The metadata of a `WStr` pointer. This is always 4 bytes wide, even on 64-bits targets.
///
/// The layout of `WStr` depends on the value of `self.is_wide()`:
///  - if `false`, it has the layout of `[u8; self.len()]`;
///  - if `true`, it has the layout of `[u16; self.len()]`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct WStrMetadata(u32);

impl WStrMetadata {
    /// SAFETY: raw must fit in a u32
    #[inline(always)]
    const unsafe fn from_usize(raw: usize) -> Self {
        if raw > u32::MAX as usize {
            if cfg!(debug_assertions) {
                panic!("invalid WStr metadata");
            } else {
                core::hint::unreachable_unchecked()
            }
        }

        Self(raw as u32)
    }

    /// Assemble `WStr` metadata from its components.
    ///
    /// # Safety
    /// `len` must be less than or equal to `WStr::MAX_LEN`.
    #[inline(always)]
    pub const unsafe fn new(len: usize, is_wide: bool) -> Self {
        Self::from_usize(len | if is_wide { WIDE_MASK as usize } else { 0 })
    }

    /// Assemble `WStr` metadata from its components.
    ///
    /// Unlike `Self::new`, this is safe, but passing a `len` bigger
    /// than `WStr::MAX_LEN` will give a bogus result.
    #[inline(always)]
    pub const fn new32(len: u32, is_wide: bool) -> Self {
        Self(len | if is_wide { WIDE_MASK } else { 0 })
    }

    /// Gets the metadata of the given pointer.
    ///
    /// # Safety
    /// `ptr` must be non-null and its metadata must be valid.
    #[inline(always)]
    pub const unsafe fn of(ptr: *const WStr) -> Self {
        Self::from_usize(raw_len(ptr as *const _ as *mut [()]))
    }

    /// Gets the metadata of the given mutable pointer.
    ///
    /// # Safety
    /// `ptr` must be non-null and its metadata must be valid.
    #[inline(always)]
    pub const unsafe fn of_mut(ptr: *mut WStr) -> Self {
        Self::from_usize(raw_len(ptr as *mut [()]))
    }

    /// Returns whether this metadata describes a wide `WStr`.
    #[inline(always)]
    pub const fn is_wide(self) -> bool {
        (self.0 & WIDE_MASK) != 0
    }

    /// Returns the length of the described `WStr`. This is never greater than `WStr::MAX_LEN`.
    #[allow(clippy::len_without_is_empty)]
    #[inline(always)]
    pub const fn len(self) -> usize {
        (self.0 & (WIDE_MASK - 1)) as usize
    }

    /// Same as `Self::len`, but returns an `u32`.
    #[inline(always)]
    pub const fn len32(self) -> u32 {
        self.0 & (WIDE_MASK - 1)
    }
}

/// Replacement for unstable `<*mut [T]>::len` method.
///
/// # Safety
/// `ptr` must be non-null.
#[inline(always)]
const unsafe fn raw_len<T>(ptr: *mut [T]) -> usize {
    core::ptr::NonNull::new_unchecked(ptr).len()
}

/// Creates a `WStr` pointer from its raw parts.
#[inline(always)]
pub const fn from_raw_parts(data: *const (), metadata: WStrMetadata) -> *const WStr {
    slice_from_raw_parts(data, metadata.0 as usize) as *const WStr
}

/// Creates a mutable `WStr` pointer from its raw parts.
#[inline(always)]
pub fn from_raw_parts_mut(data: *mut (), metadata: WStrMetadata) -> *mut WStr {
    slice_from_raw_parts_mut(data, metadata.0 as usize) as *mut WStr
}

/// Creates a `WStr` pointer from a raw units buffer.
///
/// # Safety
///  - the buffer must be non-null.
///  - the buffer length must be less than or equals to `WStr::MAX_LEN`.
#[inline]
pub const unsafe fn from_units(units: Units<*const [u8], *const [u16]>) -> *const WStr {
    let (data, len, is_wide) = match units {
        Units::Bytes(us) => (us as *const (), raw_len::<u8>(us as *mut _), false),
        Units::Wide(us) => (us as *const (), raw_len::<u16>(us as *mut _), true),
    };

    from_raw_parts(data, WStrMetadata::new(len, is_wide))
}

/// Creates a `WStr` pointer from a mutable, raw units buffer.
///
/// # Safety
///  - the buffer must be non-null.
///  - the buffer length must be less than or equals to `WStr::MAX_LEN`.
#[inline(always)]
pub const unsafe fn from_units_mut(units: Units<*mut [u8], *mut [u16]>) -> *mut WStr {
    // SAFETY: `Units` is `repr(C)` so the transmute is sound.
    from_units(transmute::<
        Units<*mut [u8], *mut [u16]>,
        Units<*const [u8], *const [u16]>,
    >(units)) as *mut WStr
}

/// Gets a pointer to the buffer designated by `ptr`.
///
/// # Safety
///  - `ptr` must be non-null and its metadata must be valid.
#[inline]
pub const unsafe fn units(ptr: *const WStr) -> Units<*const [u8], *const [u16]> {
    let (data, meta) = (ptr as *const (), WStrMetadata::of(ptr));
    if meta.is_wide() {
        Units::Wide(slice_from_raw_parts(data as *const u16, meta.len()))
    } else {
        Units::Bytes(slice_from_raw_parts(data as *const u8, meta.len()))
    }
}

/// Gets a mutable pointer to the buffer designated by `ptr`.
///
/// # Safety
///  - `ptr` must be non-null and its metadata must be valid.
#[inline(always)]
pub const unsafe fn units_mut(ptr: *mut WStr) -> Units<*mut [u8], *mut [u16]> {
    // SAFETY: `Units` is `repr(C)` so the transmute is sound.
    transmute(units(ptr))
}

/// Gets a pointer to the `n`th unit of this `WStr`.
///
/// # Safety
///  - `ptr` must point to a valid `WStr`;
///  - `i` must be less than or equals to `metadata(ptr).len()`.
#[inline]
pub const unsafe fn offset(ptr: *const WStr, i: usize) -> Units<*const u8, *const u16> {
    if WStrMetadata::of(ptr).is_wide() {
        Units::Wide((ptr as *mut u16).add(i))
    } else {
        Units::Bytes((ptr as *mut u8).add(i))
    }
}

/// Gets a mutable pointer to the `n`th unit of this `WStr`.
///
/// # Safety
///  - `ptr` must point to a valid `WStr`;
///  - `i` must be less than or equals to `metadata(ptr).len()`.
#[inline(always)]
pub unsafe fn offset_mut(ptr: *mut WStr, i: usize) -> Units<*mut u8, *mut u16> {
    // SAFETY: `Units` is `repr(C)` so the transmute is sound.
    transmute(offset(ptr, i))
}

/// Dereferences the `n`th unit of this `WStr`.
///
/// # Safety
///  - `ptr` must point to a valid `WStr` for reading;
///  - `i` must be less than `metadata(ptr).len()`.
#[inline]
pub const unsafe fn read_at(ptr: *const WStr, i: usize) -> u16 {
    match offset(ptr, i) {
        Units::Bytes(p) => *p as u16,
        Units::Wide(p) => *p,
    }
}

/// Returns a pointer to a subslice of this `WStr`.
///
/// # Safety
///  - `ptr` must point to a valid `WStr`;
///  - `range.start` must be less than or equals to `range.end`;
///  - `range.end` must be less than or equals to `metadata(ptr).len()`.
#[inline]
pub const unsafe fn slice(ptr: *const WStr, range: Range<usize>) -> *const WStr {
    let len = range.end - range.start;
    let (data, is_wide) = match offset(ptr, range.start) {
        Units::Bytes(p) => (p as *const (), false),
        Units::Wide(p) => (p as *const (), true),
    };
    from_raw_parts(data, WStrMetadata::new(len, is_wide))
}

/// Returns a mutable pointer to a subslice of this `WStr`.
///
/// # Safety
///  - `ptr` must point to a valid `WStr`;
///  - `range.start` must be less than or equals to `range.end`;
///  - `range.end` must be less than or equals to `metadata(ptr).len()`.
#[inline(always)]
pub unsafe fn slice_mut(ptr: *mut WStr, range: Range<usize>) -> *mut WStr {
    slice(ptr, range) as *mut WStr
}
