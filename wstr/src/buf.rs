use alloc::borrow::{Cow, ToOwned};
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::mem::{self, size_of, ManuallyDrop};
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

use super::utils::{encode_raw_utf16, split_ascii_prefix, split_ascii_prefix_bytes, DecodeAvmUtf8};
use super::{ptr, Units, WStr, WStrMetadata};

/// An owned, extensible UCS2 string, analogous to `String`.
pub struct WString {
    data: NonNull<()>,
    meta: WStrMetadata,
    // Always less than `WStr::MAX_LEN`
    capacity: u32,
}

#[cfg(target_pointer_width = "32")]
const _: () = assert!(size_of::<WString>() == 12);

#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<WString>() == 16);

impl WString {
    /// Creates a new empty `WString`.
    #[inline]
    pub fn new() -> Self {
        Self::from_buf(Units::Bytes(Vec::new()))
    }

    /// Creates a new empty `WString` with the given capacity and wideness.
    #[inline]
    pub fn with_capacity(capacity: usize, wide: bool) -> Self {
        if capacity > WStr::MAX_LEN {
            super::panic_on_invalid_length(capacity);
        }

        // SAFETY: the buffer is created empty, and we checked the capacity above.
        unsafe {
            Self::from_buf_unchecked(if wide {
                Units::Wide(Vec::with_capacity(capacity))
            } else {
                Units::Bytes(Vec::with_capacity(capacity))
            })
        }
    }

    /// Creates a `WString` from an owned buffer containing 1 or 2-bytes code units,
    /// without checking its length or capacity.
    ///
    /// # Safety
    ///
    /// The length and the capacity cannot be greater than `WStr::MAX_LEN`.
    #[inline]
    pub unsafe fn from_buf_unchecked(buf: Units<Vec<u8>, Vec<u16>>) -> Self {
        // SAFETY: we take ownership of the buffer; avoid double frees
        let mut buf = ManuallyDrop::new(buf);
        let (cap, len, ptr, is_wide) = match buf.deref_mut() {
            Units::Bytes(buf) => (buf.capacity(), buf.len(), buf.as_mut_ptr() as *mut _, false),
            Units::Wide(buf) => (buf.capacity(), buf.len(), buf.as_mut_ptr() as *mut _, true),
        };

        Self {
            data: NonNull::new_unchecked(ptr),
            meta: ptr::WStrMetadata::new(len, is_wide),
            capacity: cap as u32,
        }
    }

    /// Creates a `WString` from an owned buffer containing 1 or 2-bytes code units.
    #[inline]
    pub fn from_buf(buf: impl Into<Units<Vec<u8>, Vec<u16>>>) -> Self {
        // Tries to shrink the capacity below the maximum allowed WStr length.
        #[cold]
        fn shrink<T>(buf: &mut Vec<T>) {
            assert!(buf.capacity() > WStr::MAX_LEN);

            let len = buf.len();
            if len > WStr::MAX_LEN {
                super::panic_on_invalid_length(len);
            }

            buf.shrink_to(WStr::MAX_LEN);
            let ptr = ManuallyDrop::new(mem::take(buf)).as_mut_ptr();
            // SAFETY:
            // Per its contract, `Vec::shrink_to` reallocated the buffer to have
            // a capacity between `WStr::MAX_LEN` and `buf.capacity()`.
            unsafe {
                *buf = Vec::from_raw_parts(ptr, len, WStr::MAX_LEN);
            }
        }

        #[inline(always)]
        fn ensure_valid_cap<T>(buf: &mut Vec<T>) {
            if buf.capacity() > WStr::MAX_LEN {
                shrink(buf)
            }
        }

        let mut buf = buf.into();
        match &mut buf {
            Units::Bytes(buf) => ensure_valid_cap(buf),
            Units::Wide(buf) => ensure_valid_cap(buf),
        }

        // SAFETY: the length and the capacity was checked above.
        unsafe { Self::from_buf_unchecked(buf) }
    }

    /// Creates a `WString` from an pre-existing `WStr`.
    #[inline]
    pub fn from_wstr(s: &WStr) -> Self {
        Self::from_buf(match s.units() {
            Units::Bytes(us) => Units::Bytes(us.to_owned()),
            Units::Wide(us) => Units::Wide(us.to_owned()),
        })
    }

    /// Creates a `WString` from an UTF-8 `str`.
    #[inline]
    pub fn from_utf8(s: &str) -> Self {
        let (ascii, tail) = split_ascii_prefix(s);
        Self::from_utf8_inner(ascii, tail)
    }

    /// Creates a `WString` from an UTF-8 `String`, reusing the allocation if possible.
    #[inline]
    pub fn from_utf8_owned(s: String) -> Self {
        let (ascii, tail) = split_ascii_prefix(&s);
        if tail.is_empty() {
            // We can directly reinterpret ASCII bytes as LATIN1.
            return Self::from_buf(s.into_bytes());
        }

        Self::from_utf8_inner(ascii, tail)
    }

    /// Creates a `WString` from UTF-8 bytes, treating invalid sequences
    /// as described in `DecodeAvmUtf8`, and reusing the allocation if possible.
    #[inline]
    pub fn from_utf8_bytes(b: Vec<u8>) -> Self {
        let (ascii, tail) = split_ascii_prefix_bytes(&b);
        if tail.is_empty() {
            // We can directly reinterpret ASCII bytes as LATIN1.
            return Self::from_buf(b);
        }

        Self::from_utf8_bytes_inner(ascii, tail)
    }

    pub(crate) fn from_utf8_inner(ascii: &[u8], tail: &str) -> Self {
        let is_wide = tail.find(|ch| ch > u8::MAX.into()).is_some();
        if is_wide {
            let mut buf = Vec::new();
            buf.extend(ascii.iter().map(|c| u16::from(*c)));
            buf.extend(tail.encode_utf16());
            Self::from_buf(buf)
        } else {
            let mut buf = Vec::new();
            buf.extend_from_slice(ascii);
            buf.extend(tail.chars().map(|c| c as u8));
            Self::from_buf(buf)
        }
    }

    pub(crate) fn from_utf8_bytes_inner(ascii: &str, tail: &[u8]) -> Self {
        let ascii = ascii.as_bytes();
        let is_wide = DecodeAvmUtf8::new(tail).any(|ch| ch > u8::MAX.into());
        if is_wide {
            let mut buf = Vec::new();
            buf.extend(ascii.iter().map(|c| u16::from(*c)));
            for ch in DecodeAvmUtf8::new(tail) {
                encode_raw_utf16(ch, &mut buf);
            }
            Self::from_buf(buf)
        } else {
            let mut buf = Vec::new();
            buf.extend_from_slice(ascii);
            buf.extend(DecodeAvmUtf8::new(tail).map(|c| c as u8));
            Self::from_buf(buf)
        }
    }

    /// Creates a `WString` from a single UCS2 code unit.
    #[inline]
    pub fn from_unit(c: u16) -> Self {
        let mut buf = Self::new();
        buf.push(c);
        buf
    }

    /// Creates a `WString` from a single unicode character.
    #[inline]
    pub fn from_char(c: char) -> Self {
        let mut buf = Self::new();
        buf.push_char(c);
        buf
    }

    /// Converts this `WString` into a string slice.
    #[inline]
    pub fn as_wstr(&self) -> &WStr {
        let wstr = ptr::from_raw_parts_mut(self.data.as_ptr(), self.meta);
        // SAFETY:`self` is immutably borrowed.
        unsafe { &*wstr }
    }

    /// Converts this `WString` into a mutable string slice.
    #[inline]
    pub fn as_wstr_mut(&mut self) -> &mut WStr {
        let wstr = ptr::from_raw_parts_mut(self.data.as_ptr(), self.meta);
        // SAFETY:`self` is mutably borrowed.
        unsafe { &mut *wstr }
    }

    /// Steals the internal buffer.
    ///
    /// # Safety
    ///
    /// - any future access to `self` (including drop) will invalidate the returned buffer.
    /// - the returned buffer shouldn't be dropped unless self is forgotten.
    #[inline]
    unsafe fn steal_buf(&mut self) -> ManuallyDrop<Units<Vec<u8>, Vec<u16>>> {
        let cap = self.capacity as usize;

        // SAFETY: we reconstruct the Vec<T> deconstructed in `Self::from_buf`.
        let buffer = if self.meta.is_wide() {
            Units::Wide(Vec::from_raw_parts(
                self.data.cast().as_ptr(),
                self.meta.len(),
                cap,
            ))
        } else {
            Units::Bytes(Vec::from_raw_parts(
                self.data.cast().as_ptr(),
                self.meta.len(),
                cap,
            ))
        };
        ManuallyDrop::new(buffer)
    }

    /// Cheaply converts the `WString` into its internal buffer.
    #[inline]
    pub fn into_buf(self) -> Units<Vec<u8>, Vec<u16>> {
        let mut this = ManuallyDrop::new(self);
        // SAFETY: `this` is never dropped, so we can take "true" ownership of the buffer.
        unsafe { ManuallyDrop::into_inner(this.steal_buf()) }
    }

    /// Decomposes the `WString` into its raw components, leaking the contents.
    ///
    /// This returns, in order:
    /// - the pointer to the actual buffer;
    /// - the length (number of codepoints) and wideness;
    /// - the total capacity, in term of codepoints (guaranteed to be less than `WStr::MAX_LEN`).
    ///
    /// `Self::from_raw_parts` can then be called later to rebuild the `WString` back.
    #[inline]
    pub fn into_raw_parts(self) -> (*mut (), WStrMetadata, u32) {
        // Don't drop the contnts; they are managed by the caller now.
        let this = ManuallyDrop::new(self);
        (this.data.as_ptr(), this.meta, this.capacity)
    }

    /// Rebuilds a `WString` from its raw components.
    ///
    /// # Safety
    /// The arguments must come from a previous call to `Self::into_raw_parts`.
    #[inline]
    pub unsafe fn from_raw_parts(ptr: *mut (), meta: WStrMetadata, capacity: u32) -> Self {
        Self {
            data: unsafe { NonNull::new_unchecked(ptr) },
            meta,
            capacity,
        }
    }

    // Modify the raw internal buffer.
    //
    // Panics if the resulting buffer has a length greater than `WStr::MAX_LEN`.
    fn with_buf<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Units<Vec<u8>, Vec<u16>>) -> R,
    {
        struct Guard<'a> {
            source: &'a mut WString,
            buffer: ManuallyDrop<Units<Vec<u8>, Vec<u16>>>,
        }

        impl<'a> Guard<'a> {
            fn init(source: &'a mut WString) -> Self {
                let buffer = unsafe { source.steal_buf() };
                Self { source, buffer }
            }

            fn commit(mut self) {
                // SAFETY: we disable the Drop impl, so we can put the ManuallyDrop'd buffer back
                unsafe {
                    let buffer = ManuallyDrop::take(&mut self.buffer);
                    core::ptr::write(self.source, WString::from_buf(buffer));
                    mem::forget(self);
                }
            }
        }

        impl Drop for Guard<'_> {
            fn drop(&mut self) {
                // SAFETY: something has gone wrong, replace the buffer with an empty one and drop it.
                unsafe {
                    core::ptr::write(self.source, WString::new());
                    ManuallyDrop::drop(&mut self.buffer);
                }
            }
        }

        let mut guard = Guard::init(self);
        let result = f(&mut guard.buffer);
        guard.commit();
        result
    }

    fn with_wide_buf_if<W, F, R>(&mut self, wide: W, f: F) -> R
    where
        W: FnOnce() -> bool,
        F: FnOnce(&mut Units<Vec<u8>, Vec<u16>>) -> R,
    {
        self.with_buf(|units| {
            if let Units::Bytes(buf) = units {
                // Convert into wide string if necessary.
                if wide() {
                    let buf = mem::take(buf);
                    *units = Units::Wide(buf.into_iter().map(|c| c.into()).collect());
                }
            }

            f(units)
        })
    }

    /// Truncates this `WString`, removing all contents.
    #[inline]
    pub fn clear(&mut self) {
        self.meta = ptr::WStrMetadata::new32(0, self.meta.is_wide());
    }

    /// Appends a UTF-16 code unit to `self`.
    ///
    /// This will convert this `WString` into its wide form if necessary.
    pub fn push(&mut self, ch: u16) {
        self.with_wide_buf_if(
            || ch > u8::MAX.into(),
            |units| match units {
                Units::Bytes(buf) => buf.push(ch as u8),
                Units::Wide(buf) => buf.push(ch),
            },
        )
    }

    // Appends a LATIN1 code unit to `self`.
    pub fn push_byte(&mut self, ch: u8) {
        self.with_buf(|units| match units {
            Units::Bytes(buf) => buf.push(ch),
            Units::Wide(buf) => buf.push(ch.into()),
        })
    }

    /// Appends a Unicode character to `self`.
    ///
    /// This will convert this `WString` into its wide form if necessary.
    pub fn push_char(&mut self, ch: char) {
        self.with_wide_buf_if(
            || ch as u32 > u8::MAX.into(),
            |units| match units {
                Units::Bytes(buf) => buf.push(ch as u8),
                Units::Wide(buf) => {
                    let mut tmp = [0; 2];
                    buf.extend_from_slice(ch.encode_utf16(&mut tmp));
                }
            },
        )
    }

    /// Appends a UTF-8 string to `self`.
    ///
    /// This will convert this `WString` into its wide form if necessary.
    pub fn push_utf8(&mut self, s: &str) {
        let (ascii, tail) = split_ascii_prefix(s);
        let is_wide = || tail.find(|ch| ch > u8::MAX.into()).is_some();

        self.with_wide_buf_if(is_wide, |units| match units {
            Units::Bytes(buf) => {
                buf.extend_from_slice(ascii);
                buf.extend(tail.encode_utf16().map(|ch| ch as u8));
            }
            Units::Wide(buf) => {
                buf.extend(ascii.iter().map(|c| u16::from(*c)));
                buf.extend(tail.encode_utf16());
            }
        });
    }

    /// Appends UTF-8 bytes to `self`, treating invalid sequences
    /// as described in `DecodeAvmUtf8`.
    ///
    /// This will convert this `WString` into its wide form if necessary.
    pub fn push_utf8_bytes(&mut self, utf8: &[u8]) {
        let (ascii, tail) = split_ascii_prefix_bytes(utf8);
        let ascii = ascii.as_bytes();
        let is_wide = || DecodeAvmUtf8::new(tail).any(|ch| ch > u8::MAX.into());

        self.with_wide_buf_if(is_wide, |units| match units {
            Units::Bytes(buf) => {
                buf.extend_from_slice(ascii);
                buf.extend(DecodeAvmUtf8::new(tail).map(|ch| ch as u8));
            }
            Units::Wide(buf) => {
                buf.extend(ascii.iter().map(|c| u16::from(*c)));
                for ch in DecodeAvmUtf8::new(tail) {
                    encode_raw_utf16(ch, buf);
                }
            }
        });
    }

    /// Appends another `WStr` to `self`.
    ///
    /// This will convert this `WString` into its wide form if necessary.
    pub fn push_str(&mut self, s: &WStr) {
        let other = s.units();
        let is_wide = || matches!(other, Units::Wide(_));
        self.with_wide_buf_if(is_wide, |units| match (units, other) {
            (Units::Bytes(buf), Units::Bytes(other)) => buf.extend_from_slice(other),
            (Units::Wide(buf), Units::Wide(other)) => buf.extend_from_slice(other),
            (Units::Wide(buf), Units::Bytes(other)) => {
                buf.extend(other.iter().map(|c| u16::from(*c)))
            }
            (Units::Bytes(_), Units::Wide(_)) => unreachable!(),
        })
    }
}

impl Drop for WString {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: `self` is gone after this line.
        unsafe {
            let mut buf = self.steal_buf();
            ManuallyDrop::drop(&mut buf);
        };
    }
}

impl Default for WString {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for WString {
    fn clone(&self) -> Self {
        let owned = match self.units() {
            Units::Bytes(us) => Units::Bytes(us.to_owned()),
            Units::Wide(us) => Units::Wide(us.to_owned()),
        };

        // SAFETY: We know the length isn't too big.
        unsafe { Self::from_buf_unchecked(owned) }
    }

    fn clone_from(&mut self, other: &Self) {
        if self.is_wide() != other.is_wide() {
            *self = other.clone();
            return;
        }
        self.with_buf(|buf| match (buf, other.units()) {
            (Units::Bytes(left), Units::Bytes(right)) => {
                left.clear();
                left.extend_from_slice(right);
            }
            (Units::Wide(left), Units::Wide(right)) => {
                left.clear();
                left.extend_from_slice(right);
            }
            _ => unreachable!(),
        })
    }
}

impl ToOwned for WStr {
    type Owned = WString;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        WString::from_wstr(self)
    }

    fn clone_into(&self, target: &mut Self::Owned) {
        target.clear();
        target.push_str(self);
    }
}

impl<'a> From<&'a WStr> for Cow<'a, WStr> {
    #[inline]
    fn from(s: &'a WStr) -> Self {
        Cow::Borrowed(s)
    }
}

impl From<WString> for Cow<'_, WStr> {
    #[inline]
    fn from(s: WString) -> Self {
        Cow::Owned(s)
    }
}

impl From<Cow<'_, WStr>> for WString {
    #[inline]
    fn from(s: Cow<'_, WStr>) -> Self {
        match s {
            Cow::Owned(s) => s,
            Cow::Borrowed(s) => Self::from_wstr(s),
        }
    }
}

impl Deref for WString {
    type Target = WStr;
    #[inline]
    fn deref(&self) -> &WStr {
        self.as_wstr()
    }
}

impl DerefMut for WString {
    #[inline]
    fn deref_mut(&mut self) -> &mut WStr {
        self.as_wstr_mut()
    }
}

impl<'a> From<&'a WStr> for WString {
    #[inline]
    fn from(s: &'a WStr) -> Self {
        s.to_owned()
    }
}

impl FromIterator<u16> for WString {
    fn from_iter<T: IntoIterator<Item = u16>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let (min_size, _) = iter.size_hint();
        let mut buf = Self::with_capacity(min_size, false);
        iter.for_each(|c| buf.push(c));
        buf
    }
}

impl AsMut<WStr> for WString {
    #[inline]
    fn as_mut(&mut self) -> &mut WStr {
        self.deref_mut()
    }
}

impl fmt::Write for WString {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_utf8(s);
        Ok(())
    }

    #[inline]
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.push_char(c);
        Ok(())
    }
}

wstr_impl_traits!(impl for WString);
