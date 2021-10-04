use gc_arena::Collect;

use super::raw::WStrPtr;
use super::utils::split_ascii_prefix;
use super::{BorrowWStr, BorrowWStrMut, Units, WStr, WStrMut, MAX_STRING_LEN};

/// An owned, extensible UCS2 string, analoguous to `String`.
#[derive(Collect)]
#[collect(require_static)]
pub struct WString {
    // TODO: better packing on 64bit targets
    //
    // On 64bit targets, this struct is 24 bytes, but we could do better:
    // if `WStrPtr` packed its is_wide flag in the length, we could stuff the
    // capacity inside the unused 4 bytes of padding and make `WString` only
    // 16 bytes.
    slice: WStrPtr,
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
        let (ptr, len, capacity) = match &buf {
            Units::Bytes(buf) => (Units::Bytes(buf.as_ptr()), buf.len(), buf.capacity()),
            Units::Wide(buf) => (Units::Wide(buf.as_ptr()), buf.len(), buf.capacity()),
        };

        // SAFETY: forget the buffer to avoid a double free
        std::mem::forget(buf);
        let slice = WStrPtr::new(ptr, len);
        Self { slice, capacity }
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

    /// Steals the internal buffer.
    ///
    /// # Safety
    ///
    /// You must make sure that the returned buffer isn't dropped twice.
    #[inline]
    unsafe fn steal_buf(&mut self) -> Units<Vec<u8>, Vec<u16>> {
        let (len, capacity) = (self.slice.len(), self.capacity);

        // SAFETY: we reconstruct the Vec<T> deconstructed in `Self::from_buf`.
        match self.slice.data() {
            Units::Bytes(ptr) => Units::Bytes(Vec::from_raw_parts(ptr.as_ptr(), len, capacity)),
            Units::Wide(ptr) => Units::Wide(Vec::from_raw_parts(ptr.as_ptr(), len, capacity)),
        }
    }

    /// Cheaply converts the `WString` into its internal buffer.
    #[inline]
    pub fn into_buf(mut self) -> Units<Vec<u8>, Vec<u16>> {
        // SAFETY: `self` won't be dropped because it is forgotten.
        let buf = unsafe { self.steal_buf() };
        std::mem::forget(self);
        buf
    }

    impl_str_methods! {
        lifetime: '_;
        self: &Self;
        deref: self.borrow();
        pattern['a,]: 'a, &'a Self;
    }

    impl_str_mut_methods! {
        lifetime: '_;
        self: &mut Self;
        deref_mut: self.borrow_mut();
    }

    // Modify the raw internal buffer.
    //
    // Panics if the resulting buffer has a length greater than `MAX_STRING_LEN`.
    fn with_buf<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Units<Vec<u8>, Vec<u16>>) -> R,
    {
        struct Guard<'a>(&'a mut WString);

        impl<'a> Drop for Guard<'a> {
            fn drop(&mut self) {
                // SAFETY: something has gone wrong, replace the buffer with an empty one.
                unsafe {
                    std::ptr::write(self.0, WString::new());
                }
            }
        }

        // SAFETY:
        // - if `f` or `Self::from_buf` panics, we avoid the double free
        //   because `Guard` will replace `self` with an empty buffer;
        // - otherwise, we deactivate the guard by forgetting it.
        unsafe {
            let guard = Guard(self);
            let mut buf = guard.0.steal_buf();
            let result = f(&mut buf);
            std::ptr::write(guard.0, Self::from_buf(buf));
            std::mem::forget(guard);
            result
        }
    }

    fn with_wide_buf_if<W, F, R>(&mut self, wide: W, f: F) -> R
    where
        W: FnOnce() -> bool,
        F: FnOnce(Units<&mut Vec<u8>, &mut Vec<u16>>) -> R,
    {
        self.with_buf(|units| {
            if let Units::Bytes(buf) = units {
                // Convert into wide string if necessary.
                if wide() {
                    let buf = std::mem::take(buf);
                    *units = Units::Wide(buf.into_iter().map(|c| c.into()).collect());
                }
            }

            let units_ref = match units {
                Units::Bytes(buf) => Units::Bytes(buf),
                Units::Wide(buf) => Units::Wide(buf),
            };
            f(units_ref)
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
    pub fn push_str(&mut self, s: WStr<'_>) {
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
        if self.slice.is_wide() != other.slice.is_wide() {
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

impl BorrowWStr for WString {
    #[inline]
    fn borrow(&self) -> WStr<'_> {
        // SAFETY: `self` is immutably borrowed.
        unsafe { WStr::from_ptr(self.slice) }
    }
}

impl BorrowWStrMut for WString {
    #[inline]
    fn borrow_mut(&mut self) -> WStrMut<'_> {
        // SAFETY: `self` is immutably borrowed.
        unsafe { WStrMut::from_ptr(self.slice) }
    }
}

impl<'a> From<WStr<'a>> for WString {
    #[inline]
    fn from(s: WStr<'a>) -> Self {
        let mut buf = Self::new();
        buf.push_str(s.borrow());
        buf
    }
}

impl std::iter::FromIterator<u16> for WString {
    fn from_iter<T: IntoIterator<Item = u16>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let (min_size, _) = iter.size_hint();
        let mut buf = Self::with_capacity(min_size, false);
        iter.for_each(|c| buf.push(c));
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concat_bytes() {
        let mut s = WString::new();
        assert_eq!(s, WStr::from_units(b""));
        s.push_byte(b'a');
        assert_eq!(s, WStr::from_units(b"a"));
        s.push(b'b'.into());
        assert_eq!(s, WStr::from_units(b"ab"));
        s.push_utf8("cd");
        assert_eq!(s, WStr::from_units(b"abcd"));
        s.push_str(WStr::from_units(b"ef"));
        assert_eq!(s, WStr::from_units(b"abcdef"));
        s.push_char('g');
        assert_eq!(s, WStr::from_units(b"abcdefg"));
        assert!(matches!(s.units(), Units::Bytes(_)));
    }

    #[test]
    fn concat_wide() {
        macro_rules! wstr {
            ($($lit:literal),*) => {
                WStr::from_units(&[$($lit as u16),*])
            }
        }

        let mut s = WString::new();
        assert_eq!(s, WStr::from_units(b""));
        s.push_byte(b'a');
        assert_eq!(s, WStr::from_units(b"a"));
        s.push('€' as u16);
        assert_eq!(s, wstr!['a', '€']);
        s.push_utf8("😀");
        assert_eq!(s, wstr!['a', '€', 0xd83d, 0xde00]);
        s.push_str(WStr::from_units(b"!"));
        assert_eq!(s, wstr!['a', '€', 0xd83d, 0xde00, '!']);
        s.push_char('😀');
        assert_eq!(s, wstr!['a', '€', 0xd83d, 0xde00, '!', 0xd83d, 0xde00]);
        assert!(matches!(s.units(), Units::Wide(_)));
    }
}
