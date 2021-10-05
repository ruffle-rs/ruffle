use std::marker::PhantomData;
use std::ops::RangeBounds;

use gc_arena::Collect;

use super::raw::WStrPtr;
use super::{BorrowWStr, BorrowWStrMut, Units, MAX_STRING_LEN};

/// A UCS2 string slice, analoguous to `&'a str`.
#[derive(Copy, Clone, Collect)]
#[collect(require_static)]
pub struct WStr<'a> {
    ptr: WStrPtr,
    // for covariance in 'a
    _marker: PhantomData<&'a ()>,
}

impl<'a> WStr<'a> {
    /// # Safety
    ///
    /// `ptr` must point to a string with the correct lifetime.
    #[inline]
    pub(super) unsafe fn from_ptr(ptr: WStrPtr) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub(super) fn to_ptr(self) -> WStrPtr {
        self.ptr
    }

    /// Creates a `WStr<'a>` from a buffer containing 1 or 2-bytes code units.
    pub fn from_units(units: impl Into<Units<&'a [u8], &'a [u16]>>) -> Self {
        let (data, len) = match units.into() {
            Units::Bytes(us) => (Units::Bytes(us.as_ptr()), us.len()),
            Units::Wide(us) => (Units::Wide(us.as_ptr()), us.len()),
        };

        if len > MAX_STRING_LEN {
            super::panic_on_invalid_length(len);
        }

        // SAFETY: we validated the slice length above, and the shared borrow is valid for 'a.
        unsafe { Self::from_ptr(WStrPtr::new(data, len)) }
    }

    /// Creates an empty string.
    #[inline]
    pub fn empty() -> Self {
        // SAFETY: the pointer is non-null.
        unsafe { Self::from_ptr(WStrPtr::new(Units::Bytes([].as_ptr()), 0)) }
    }

    impl_str_methods! {
        lifetime: 'a;
        self: Self;
        deref: self;
        pattern[]: 'a, Self;
    }
}

impl<'a> Default for WStr<'a> {
    #[inline]
    fn default() -> Self {
        Self::from_units(Units::<&'a [u8], _>::Bytes(&[]))
    }
}

impl<'a> BorrowWStr for WStr<'a> {
    #[inline]
    fn borrow(&self) -> WStr<'_> {
        *self
    }
}

/// A mutable UCS2 string slice, analoguous to `&'a mut str`.
#[derive(Collect)]
#[collect(require_static)]
pub struct WStrMut<'a> {
    ptr: WStrPtr,
    // for invariance in 'a
    _marker: PhantomData<&'a mut ()>,
}

impl<'a> WStrMut<'a> {
    /// # Safety
    ///
    /// `ptr` must point to a string with the correct lifetime.
    #[inline]
    pub(super) unsafe fn from_ptr(ptr: WStrPtr) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Creates a `WStrMut<'a>` from a mutable buffer containing 1 or 2-bytes code units.
    pub fn from_units(units: impl Into<Units<&'a mut [u8], &'a mut [u16]>>) -> Self {
        let (data, len) = match units.into() {
            Units::Bytes(us) => (Units::Bytes(us.as_ptr()), us.len()),
            Units::Wide(us) => (Units::Wide(us.as_ptr()), us.len()),
        };

        if len > MAX_STRING_LEN {
            super::panic_on_invalid_length(len);
        }

        // SAFETY: we validated the slice length above, and the mutable borrow is valid for 'a.
        unsafe { Self::from_ptr(WStrPtr::new(data, len)) }
    }

    /// Creates an empty string.
    #[inline]
    pub fn empty() -> Self {
        // SAFETY: the pointer is non-null.
        unsafe { Self::from_ptr(WStrPtr::new(Units::Bytes([].as_ptr()), 0)) }
    }

    /// Converts into a immutable reference.
    pub fn into_ref(self) -> WStr<'a> {
        // SAFETY: `WStr<'a>` is strictly less powerful that `WStrMut<'a>`
        unsafe { WStr::from_ptr(self.ptr) }
    }

    impl_str_methods! {
        lifetime: '_;
        self: &Self;
        deref: self.borrow();
        pattern['b,]: 'b, &'b Self;
    }

    impl_str_mut_methods! {
        lifetime: 'a;
        self: Self;
        deref_mut: self;
    }
}

impl<'a> Default for WStrMut<'a> {
    #[inline]
    fn default() -> Self {
        Self::from_units(Units::<&'a mut [u8], _>::Bytes(&mut []))
    }
}

impl<'a> BorrowWStr for WStrMut<'a> {
    #[inline]
    fn borrow(&self) -> WStr<'_> {
        // SAFETY: 'a outlives '_, so `WStr<'_>` is strictly less powerful that `WStrMut<'a>`
        unsafe { WStr::from_ptr(self.ptr) }
    }
}

impl<'a> BorrowWStrMut for WStrMut<'a> {
    #[inline]
    fn borrow_mut(&mut self) -> WStrMut<'_> {
        // SAFETY: 'a outlives '_, so `WStrMut<'_>` is strictly less powerful that `WStrMut<'a>`
        unsafe { WStrMut::from_ptr(self.ptr) }
    }
}

// Basic conversions; these aren't directly on WStr/WStrMut
// because we don't want to duplicate documentation with the methods
// added by impl_str[_mut]_methods.

#[inline]
pub(super) fn raw_str(s: WStr<'_>) -> WStrPtr {
    s.ptr
}

#[inline]
pub(super) fn units(s: WStr<'_>) -> Units<&[u8], &[u16]> {
    // SAFETY: `s` is borrowed immutably for '_
    unsafe {
        match s.ptr.units() {
            Units::Bytes(us) => Units::Bytes(us.as_ref()),
            Units::Wide(us) => Units::Wide(us.as_ref()),
        }
    }
}

#[inline]
pub(super) fn units_mut(s: WStrMut<'_>) -> Units<&mut [u8], &mut [u16]> {
    // SAFETY: `s` is borrowed mutably for '_
    unsafe {
        match s.ptr.units() {
            Units::Bytes(mut us) => Units::Bytes(us.as_mut()),
            Units::Wide(mut us) => Units::Wide(us.as_mut()),
        }
    }
}

#[inline]
pub(super) fn try_index(s: WStr<'_>, i: usize) -> Option<u16> {
    if i < s.ptr.len() {
        // SAFETY: the index is in-bounds.
        Some(unsafe { s.ptr.get(i) })
    } else {
        None
    }
}

#[inline]
pub(super) fn try_slice<R: RangeBounds<usize>>(s: WStr<'_>, range: R) -> Option<WStr<'_>> {
    // SAFETY: `s` is borrowed immutably for '_
    s.ptr
        .try_slice(range)
        .map(|ptr| unsafe { WStr::from_ptr(ptr) })
}

#[inline]
pub(super) fn try_slice_mut<R: RangeBounds<usize>>(
    s: WStrMut<'_>,
    range: R,
) -> Option<WStrMut<'_>> {
    // SAFETY: `s` is borrowed mutably for '_
    s.ptr
        .try_slice(range)
        .map(|ptr| unsafe { WStrMut::from_ptr(ptr) })
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! wstr {
        ($lit:literal) => {{
            const LEN: usize = $lit.len();
            const BUF: [u16; LEN] = {
                let mut b = [0; LEN];
                let mut i = 0;
                while i < LEN {
                    b[i] = $lit[i] as u16;
                    i += 1;
                }
                b
            };
            &BUF
        }};
    }

    #[test]
    fn roundtrip() {
        fn test<'a>(units: impl Into<Units<&'a [u8], &'a [u16]>>) {
            let units = units.into();
            let s = WStr::from_units(units);
            let conv = s.units();
            let eq = match (units, conv) {
                (Units::Bytes(a), Units::Bytes(b)) => a == b,
                (Units::Wide(a), Units::Wide(b)) => a == b,
                _ => false,
            };

            assert!(eq, "expected {:?}, got {:?}", units, conv);
        }

        test(b"");
        test(wstr!(b""));
        test(b"Hello world!");
        test(wstr!(b"Hello world!"));
    }

    #[test]
    #[rustfmt::skip]
    #[allow(clippy::eq_op)]
    fn eq() {
        let a1 = WStr::from_units(b"hello");
        let b1 = WStr::from_units(b"world");
        let a2 = WStr::from_units(wstr!(b"hello"));
        let b2 = WStr::from_units(wstr!(b"world"));

        assert_eq!(a1, a1); assert_eq!(a2, a1); assert_ne!(b1, a1); assert_ne!(b2, a1);
        assert_eq!(a1, a2); assert_eq!(a2, a2); assert_ne!(b1, a2); assert_ne!(b2, a2);
        assert_ne!(a1, b1); assert_ne!(a2, b1); assert_eq!(b1, b1); assert_eq!(b2, b1);
        assert_ne!(a1, b2); assert_ne!(a2, b2); assert_eq!(b1, b2); assert_eq!(b2, b2);
    }

    #[test]
    #[rustfmt::skip]
    #[allow(clippy::eq_op)]
    fn cmp() {
        let a1 = WStr::from_units(b"hello");
        let b1 = WStr::from_units(b"world");
        let a2 = WStr::from_units(wstr!(b"hello"));
        let b2 = WStr::from_units(wstr!(b"world"));

        assert!(a1 == a1); assert!(a2 == a1); assert!(b1 >  a1); assert!(b2 >  a1);
        assert!(a1 == a2); assert!(a2 == a2); assert!(b1 >  a2); assert!(b2 >  a2);
        assert!(a1 <  b1); assert!(a2 <  b1); assert!(b1 == b1); assert!(b2 == b1);
        assert!(a1 <  b2); assert!(a2 <  b2); assert!(b1 == b2); assert!(b2 == b2);
    }

    #[test]
    fn fmt() {
        let a = WStr::from_units(b"Hello world!");
        let b = WStr::from_units(wstr!(b"Hello world!"));
        let c = WStr::from_units(b"\t\n\x03");
        let d = WStr::from_units(&[0x202d_u16, 0x202e_u16]);

        assert_eq!(format!("{}", a), "Hello world!");
        assert_eq!(format!("{}", b), "Hello world!");
        assert_eq!(format!("{}", c), "\t\n\x03");
        assert_eq!(format!("{}", d), "\u{202d}\u{202e}");

        assert_eq!(format!("{:?}", a), "\"Hello world!\"");
        assert_eq!(format!("{:?}", b), "\"Hello world!\"");
        assert_eq!(format!("{:?}", c), "\"\\t\\n\\u{3}\"");
        assert_eq!(format!("{:?}", d), "\"\\u{202d}\\u{202e}\"");
    }
}
