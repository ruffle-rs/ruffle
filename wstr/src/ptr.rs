use std::ops::Range;
use std::ptr::slice_from_raw_parts_mut;

use gc_arena::Collect;

use super::Units;

#[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
compile_error!("WStr only supports 32-bits and 64-bits targets");

/// The maximum string length, equals to 2³¹-1.
pub const MAX_STRING_LEN: usize = 0x7FFF_FFFF;
const WIDE_MASK: usize = MAX_STRING_LEN + 1;

/// A UCS2 string slice, analoguous to `&'a str`.
#[derive(Collect)]
#[collect(require_static)]
#[repr(transparent)]
pub struct WStr {
    /// The internal `WStr` representation.
    ///
    /// What we actually want here is a custom DST, but they don't exist be we must cheat
    /// and abuse the slice metadata field.
    ///
    /// The data pointer points to the start of the units buffer, which is either a
    /// `[u8]` (`Units::Bytes`) or a `[u16]` (`Units::Wide`).
    ///
    /// String lengths are limited to less than `2³¹` units long, and the kind of
    /// buffer is indicated by the 32nd bit of the raw slice length:
    ///  - for `Units::Bytes`, it is a zero;
    ///  - for `Units::Wide`, it is a one.
    ///
    /// Note that on 64-bits targets, this leaves the high 32 bits of the length unused.
    /// (TODO: find a nice way to expose them for usage by other types?)
    ///
    /// # (Un)soundness
    ///
    /// Unfortunately, this scheme is technically unsound under Stacked Borrows because of provenance:
    /// when we cast a `*mut [u8]` or `*mut [u16]` to a `*mut WStr`, we lose the provenance over the
    /// original buffer as `[()]` always occupies zero bytes (which is what allows use to mess up the slice length).
    ///
    /// As such, when we access the buffer data (e.g. in `read_at` or when converting back to raw buffer references),
    /// the read is considered out-of-bounds by Stacked Borrows and causes undefined behavior.
    ///
    /// This unsoundness doesn't seem to manifest in practice though, as Rust doesn't pass through slice
    /// length information to LLVM (yet?).
    ///
    /// One observable consequence of this is that `std::mem::size_of_val::<WStr>` won't return the actual
    /// byte length of the string contents, but will instead always return 0.
    _repr: [()],
}

/// Convenience method to turn a `&T` into a `*mut T`.
#[inline]
pub fn ptr_mut<T: ?Sized>(t: &T) -> *mut T {
    t as *const T as *mut T
}

/// Replacement for unstable `<*mut [T]>::len` method.
///
/// # Safety
///  - `ptr` must point to an allocated slice of the correct type.
#[inline]
unsafe fn raw_len<T>(ptr: *mut [T]) -> usize {
    let fake = ptr as *mut [()];
    // SAFETY: `ptr` points to *some* allocated storage, so we
    // can read a `[()]` (which takes up 0 bytes) out of it.
    (*fake).len()
}

/// Returns an untyped pointer to the raw `WStr` buffer.
///
/// Depending on the value of `is_wide(ptr)`, this points to a buffer of `u8`s or `u16`s.
#[inline]
pub fn data(ptr: *mut WStr) -> *mut () {
    ptr.cast::<()>()
}

/// Returns the length of the raw `WStr` slice.
///
/// This is always less than or equals to `MAX_STRING_LEN`.
///
/// # Safety
///  - `ptr` must point to some allocated storage of arbitrary size.
#[inline]
pub unsafe fn len(ptr: *mut WStr) -> usize {
    raw_len(ptr as *mut [()]) & MAX_STRING_LEN
}

/// Returns `true` if the raw `WStr` slice is wide.
///
/// # Safety
///  - `ptr` must point to some allocated storage of arbitrary size.
#[inline]
pub unsafe fn is_wide(ptr: *mut WStr) -> bool {
    raw_len(ptr as *mut [()]) & WIDE_MASK != 0
}

/// Creates a `WStr` pointer from its raw parts.
///
/// # Safety
///  - `len` must be less than or equals to `MAX_STRING_LEN`
///  - `data` must point to allocated storage fitting the layout of:
///     - `[u8; len]` if `is_wide` is `false`;
///     - `[u16; len]` if `is_wide` is `true`.
#[inline]
pub unsafe fn from_raw_parts(data: *mut (), len: usize, is_wide: bool) -> *mut WStr {
    let raw_len = len | if is_wide { WIDE_MASK } else { 0 };
    let slice = std::ptr::slice_from_raw_parts(data, raw_len);
    slice as *mut WStr
}

/// Creates a `WStr` pointer from a raw units buffer.
///
/// # Safety
///  - the buffer length must be less than or equals to `MAX_STRING_LEN`
///  - the buffer must point to allocated storage fitting the layout of:
///     - `[u8; len]` if it is a `Units::Bytes`;
///     - `[u16; len]` if it is a `Units::Wide`.
#[inline]
pub unsafe fn from_units(units: Units<*mut [u8], *mut [u16]>) -> *mut WStr {
    let (data, len, is_wide) = match units {
        Units::Bytes(us) => (us as *mut (), raw_len(us), false),
        Units::Wide(us) => (us as *mut (), raw_len(us), true),
    };
    from_raw_parts(data, len, is_wide)
}

/// Gets a reference to the buffer pointed by `ptr`.
///
/// # Safety
///  - `ptr` must point to some allocated storage of arbitrary size.
#[inline]
pub unsafe fn units(ptr: *mut WStr) -> Units<*mut [u8], *mut [u16]> {
    let (data, len) = (data(ptr), len(ptr));
    if is_wide(ptr) {
        Units::Wide(slice_from_raw_parts_mut(data as *mut u16, len))
    } else {
        Units::Bytes(slice_from_raw_parts_mut(data as *mut u8, len))
    }
}

/// Gets a pointer to the `n`th unit of this `WStr.
///
/// # Safety
///  - `ptr` must point to a valid `WStr`;
///  - `i` must be less than or equals to `len(ptr)`.
#[inline]
pub unsafe fn offset(ptr: *mut WStr, i: usize) -> Units<*mut u8, *mut u16> {
    // SAFETY: we have `index <= len(ptr) <= MAX_STRING_LEN < i32::MAX`, so:
    //  - `i` can be casted to `isize` on 32-bit and 64-bit targets;
    //  - the offset call is in bounds.
    let n = i as isize;
    if is_wide(ptr) {
        Units::Wide((ptr as *mut u16).offset(n))
    } else {
        Units::Bytes((ptr as *mut u8).offset(n))
    }
}
/// Dereferences the `n`th unit of this `WStr`.
///
/// # Safety
///  - `ptr` must point to a valid `WStr` for reading;
///  - `i` must be less than `len(ptr)`.
pub unsafe fn read_at(ptr: *mut WStr, i: usize) -> u16 {
    match offset(ptr, i) {
        Units::Bytes(p) => (*p).into(),
        Units::Wide(p) => *p,
    }
}

/// Returns a pointer to a subslice of this `WStr`.
///
/// # Safety
///  - `ptr` must point to a valid `WStr`;
///  - `range.start` must be less than or equals to `range.end`;
///  - `range.end` must be less than or equals to `len(ptr)`.
#[inline]
pub unsafe fn slice(ptr: *mut WStr, range: Range<usize>) -> *mut WStr {
    let len = range.end - range.start;
    let (data, is_wide) = match offset(ptr, range.start) {
        Units::Bytes(p) => (p as *mut (), false),
        Units::Wide(p) => (p as *mut (), true),
    };
    from_raw_parts(data, len, is_wide)
}
