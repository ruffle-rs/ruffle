use std::borrow::Borrow;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

use gc_arena::Collect;

use super::utils::split_ascii_prefix;
use super::{Units, WStr, MAX_STRING_LEN};

/// An owned, extensible UCS2 string, analoguous to `String`.
#[derive(Collect)]
#[collect(require_static)]
pub struct WString {
    // TODO: better packing on 64bit targets
    //
    // On 64bit targets, this struct is 24 bytes, but we could do better by stuffing
    // capacity inside the unused bits in `NonNull<WStr>` and make `WString` only
    // 16 bytes.
    ptr: NonNull<WStr>,
    capacity: usize,
}

impl WString {
    /// Creates a new empty `WString`.
    #[inline]
    pub fn new() -> Self {
        Self::from_buf(Units::Bytes(Vec::new()))
    }

    /// Creates a new empty `WString` with the given capacity and wideness.
    #[inline]
    pub fn with_capacity(capacity: usize, wide: bool) -> Self {
        // SAFETY: the buffer is created empty.
        unsafe {
            Self::from_buf_unchecked(if wide {
                Units::Wide(Vec::with_capacity(capacity))
            } else {
                Units::Bytes(Vec::with_capacity(capacity))
            })
        }
    }

    /// Creates a `WString` from an owned buffer containing 1 or 2-bytes code units,
    /// without checking the length.
    ///
    /// # Safety
    ///
    /// The length cannot be greater than `MAX_STRING_LEN`.
    #[inline]
    pub unsafe fn from_buf_unchecked(buf: Units<Vec<u8>, Vec<u16>>) -> Self {
        // SAFETY: we take ownership of the buffer; avoid double frees
        let mut buf = ManuallyDrop::new(buf);
        let (capacity, ptr) = match buf.deref_mut() {
            Units::Bytes(buf) => (buf.capacity(), Units::Bytes(&mut buf[..] as *mut _)),
            Units::Wide(buf) => (buf.capacity(), Units::Wide(&mut buf[..] as *mut _)),
        };

        let ptr = NonNull::new_unchecked(super::ptr::from_units(ptr));
        Self { ptr, capacity }
    }

    /// Creates a `WString` from an owned buffer containing 1 or 2-bytes code units.
    #[inline]
    pub fn from_buf(buf: impl Into<Units<Vec<u8>, Vec<u16>>>) -> Self {
        let buf = buf.into();

        if buf.len() > MAX_STRING_LEN {
            super::panic_on_invalid_length(buf.len());
        }

        // SAFETY: the length was checked above.
        unsafe { Self::from_buf_unchecked(buf) }
    }

    /// Creates a `WString` from an UTF-8 `String`, reusing the allocation if possible.
    pub fn from_utf8_owned(s: String) -> Self {
        let (ascii, tail) = split_ascii_prefix(&s);
        if tail.is_empty() {
            // We can directly reinterpret ASCII bytes as LATIN1.
            return Self::from_buf(s.into_bytes());
        }

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

    /// Creates a `WString` from an UTF-8 `str`.
    #[inline]
    pub fn from_utf8(s: &str) -> Self {
        let mut buf = Self::new();
        buf.push_utf8(s);
        buf
    }

    /// Creates a `WString` from a single UCS2 code unit.
    #[inline]
    pub fn from_unit(c: u16) -> Self {
        let mut buf = Self::new();
        buf.push(c);
        buf
    }

    /// Creates a `StrBuf` from a single unicode character.
    #[inline]
    pub fn from_char(c: char) -> Self {
        let mut buf = Self::new();
        buf.push_char(c);
        buf
    }

    /// Converts this `WString` into a string slice.
    pub fn as_wstr(&self) -> &WStr {
        // SAFETY: `self` is immutably borrowed.
        unsafe { self.ptr.as_ref() }
    }

    /// Converts this `WString` into a mutable string slice.
    pub fn as_wstr_mut(&mut self) -> &mut WStr {
        // SAFETY: `self` is mutably borrowed.
        unsafe { self.ptr.as_mut() }
    }

    /// Steals the internal buffer.
    ///
    /// # Safety
    ///
    /// - any future access to `self` (including drop) will invalidate the returned buffer.
    /// - the returned buffer shouldn't be dropped, unless self is dropped.
    #[inline]
    unsafe fn steal_buf(&mut self) -> ManuallyDrop<Units<Vec<u8>, Vec<u16>>> {
        let ptr = self.ptr.as_ptr();
        let data = super::ptr::data(ptr);
        let len = super::ptr::len(ptr);
        let cap = self.capacity;

        // SAFETY: we reconstruct the Vec<T> deconstructed in `Self::from_buf`.
        let buffer = if super::ptr::is_wide(ptr) {
            Units::Wide(Vec::from_raw_parts(data as *mut u16, len, cap))
        } else {
            Units::Bytes(Vec::from_raw_parts(data as *mut u8, len, cap))
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

    // Modify the raw internal buffer.
    //
    // Panics if the resulting buffer has a length greater than `MAX_STRING_LEN`.
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
                    std::ptr::write(self.source, WString::from_buf(buffer));
                    std::mem::forget(self);
                }
            }
        }

        impl<'a> Drop for Guard<'a> {
            fn drop(&mut self) {
                // SAFETY: something has gone wrong, replace the buffer with an empty one and drop it.
                unsafe {
                    std::ptr::write(self.source, WString::new());
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
                    let buf = std::mem::take(buf);
                    *units = Units::Wide(buf.into_iter().map(|c| c.into()).collect());
                }
            }

            f(units)
        })
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
    fn drop(&mut self) {
        // SAFETY: `self` is gone after this line.
        let _ = unsafe { self.steal_buf() };
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

impl AsRef<WStr> for WString {
    #[inline]
    fn as_ref(&self) -> &WStr {
        self.deref()
    }
}

impl Borrow<WStr> for WString {
    #[inline]
    fn borrow(&self) -> &WStr {
        self.deref()
    }
}

impl<'a> From<&'a WStr> for WString {
    #[inline]
    fn from(s: &'a WStr) -> Self {
        let mut buf = Self::new();
        buf.push_str(s);
        buf
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

impl std::fmt::Write for WString {
    #[inline]
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.push_utf8(s);
        Ok(())
    }

    #[inline]
    fn write_char(&mut self, c: char) -> std::fmt::Result {
        self.push_char(c);
        Ok(())
    }
}
