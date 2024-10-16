use alloc::vec::Vec;
use core::ops::{Bound, Index, IndexMut, Range, RangeBounds};

use super::{ptr, FromWStr, Pattern, WString};

/// A UCS2 string slice, analogous to `&'a str`.
#[repr(transparent)]
pub struct WStr {
    /// See the `ptr` module for more details.
    _repr: [()],
}

#[cold]
pub fn panic_on_invalid_length(len: usize) -> ! {
    panic!("Too many code units in Ruffle string (len = {})", len)
}

/// A raw string buffer containing `u8` or `u16` code units.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub enum Units<T, U> {
    /// A buffer containing `u8` code units, interpreted as LATIN-1.
    Bytes(T),
    /// A buffer containing `u16` code units, interpreted as UTF-16
    /// but allowing unpaired surrogates.
    Wide(U),
}

/// Generate `From` implementations for `Units` type.
macro_rules! units_from {
    (impl[$($generics:tt)*] Units<$ty_bytes:ty, $ty_wide:ty>; $($rest:tt)*) => {
        units_from! {
            impl[$($generics)*] Units<$ty_bytes, $ty_wide> {
                units: Units<$ty_bytes, $ty_wide> => units
            } $($rest)*
        }
    };

    (impl[$($generics:tt)*] Units<$ty_bytes:ty, $ty_wide:ty> {
        $units:ident : Units<$from_bytes:ty, $from_wide:ty> => $expr:expr
    } $($rest:tt)*) => {
        impl<$($generics)*> From<$from_bytes> for Units<$ty_bytes, $ty_wide> {
            #[inline]
            fn from($units: $from_bytes) -> Self {
                Units::Bytes($expr)
            }
        }

        impl<$($generics)*> From<$from_wide> for Units<$ty_bytes, $ty_wide> {
            #[inline]
            fn from($units: $from_wide) -> Self {
                Units::Wide($expr)
            }
        }

        units_from! { $($rest)* }
    };

    () => {};
}

units_from! {
    impl['a] Units<&'a [u8], &'a [u16]>;

    impl['a] Units<&'a mut [u8], &'a mut [u16]>;

    impl['a, const N: usize] Units<&'a [u8], &'a [u16]> {
        units: Units<&'a [u8; N], &'a [u16; N]> => &units[..]
    }

    impl['a, const N: usize] Units<&'a mut [u8], &'a mut [u16]> {
        units: Units<&'a mut [u8; N], &'a mut [u16; N]> => &mut units[..]
    }

    impl[] Units<Vec<u8>, Vec<u16>>;
}

impl WStr {
    /// The maximum string length, equals to 2³¹-1.
    pub const MAX_LEN: usize = 0x7FFF_FFFF;

    /// Creates a `&WStr` from a buffer containing 1 or 2-bytes code units.
    pub fn from_units<'a>(units: impl Into<Units<&'a [u8], &'a [u16]>>) -> &'a Self {
        let (ptr, len) = match units.into() {
            Units::Bytes(us) => (Units::Bytes(us as *const _), us.len()),
            Units::Wide(us) => (Units::Wide(us as *const _), us.len()),
        };

        if len > WStr::MAX_LEN {
            super::panic_on_invalid_length(len);
        }

        // SAFETY: we validated the slice length above, and the shared borrow is valid for 'a.
        unsafe { &*ptr::from_units(ptr) }
    }

    /// Creates a `&mut WStr` from a mutable buffer containing 1 or 2-bytes code units.
    pub fn from_units_mut<'a>(
        units: impl Into<Units<&'a mut [u8], &'a mut [u16]>>,
    ) -> &'a mut Self {
        let (ptr, len) = match units.into() {
            Units::Bytes(us) => (Units::Bytes(us as *mut _), us.len()),
            Units::Wide(us) => (Units::Wide(us as *mut _), us.len()),
        };

        if len > WStr::MAX_LEN {
            super::panic_on_invalid_length(len);
        }

        // SAFETY: we validated the slice length above, and the mutable borrow is valid for 'a.
        unsafe { &mut *ptr::from_units_mut(ptr) }
    }

    /// Creates an empty string.
    #[inline]
    pub fn empty<'a>() -> &'a WStr {
        WStr::from_units(Units::<&'a [u8], _>::Bytes(&[]))
    }

    /// Creates an empty mutable string.
    #[inline]
    pub fn empty_mut<'a>() -> &'a mut WStr {
        WStr::from_units_mut(Units::<&'a mut [u8], _>::Bytes(&mut []))
    }

    /// Provides access to the underlying buffer.
    #[inline]
    pub fn units(&self) -> Units<&[u8], &[u16]> {
        // SAFETY: `self` is a valid `WStr` borrowed immutably, so we can deref. the buffers.
        unsafe {
            match ptr::units(self) {
                Units::Bytes(us) => Units::Bytes(&*us),
                Units::Wide(us) => Units::Wide(&*us),
            }
        }
    }

    /// Provides mutable access to the underlying buffer.
    #[inline]
    pub fn units_mut(&mut self) -> super::Units<&mut [u8], &mut [u16]> {
        // SAFETY: `self` is a valid `WStr` borrowed mutably, so we can mut. deref. the buffers.
        unsafe {
            match ptr::units_mut(self) {
                Units::Bytes(us) => Units::Bytes(&mut *us),
                Units::Wide(us) => Units::Wide(&mut *us),
            }
        }
    }

    /// Returns `true` if `self` is a wide string.
    #[inline]
    pub fn is_wide(&self) -> bool {
        // SAFETY: `self` is a valid `WStr`.
        unsafe { ptr::WStrMetadata::of(self).is_wide() }
    }

    /// Returns the number of code units.
    #[inline]
    pub fn len(&self) -> usize {
        // SAFETY: `self` is a valid `WStr`.
        unsafe { ptr::WStrMetadata::of(self).len() }
    }

    /// Returns `true` if `self` contains no code units.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the `i`th code unit of `self`; panics if the index is out of range.
    #[inline]
    pub fn at(&self, i: usize) -> u16 {
        self.get(i).expect("string index out of bounds")
    }

    /// Returns the `i`th code unit of `self`, or `None` if the index is out of range.
    #[inline]
    pub fn get(&self, i: usize) -> Option<u16> {
        if i < self.len() {
            // SAFETY: `self` is a valid `WStr` and `i` is a valid index.
            Some(unsafe { ptr::read_at(self, i) })
        } else {
            None
        }
    }

    /// Returns the `i`th code unit of `self` without doing bound checks.
    ///
    /// # Safety
    /// `i` must be less than `self.len()`.
    #[inline]
    pub unsafe fn get_unchecked(&self, i: usize) -> u16 {
        ptr::read_at(self, i)
    }

    #[inline(always)]
    fn check_range<R: RangeBounds<usize>>(&self, range: R) -> Option<Range<usize>> {
        let len = self.len();
        let min = match range.start_bound() {
            Bound::Included(n) => *n,
            Bound::Excluded(n) => n.checked_add(1)?,
            Bound::Unbounded => 0,
        };

        let max = match range.end_bound() {
            Bound::Included(n) => n.checked_add(1)?,
            Bound::Excluded(n) => *n,
            Bound::Unbounded => len,
        };

        if min <= max && max <= len {
            Some(min..max)
        } else {
            None
        }
    }

    /// Returns a subslice of `self`, or `None` if the slice indices are out of range.
    ///
    /// Use indexing for a panicking version.
    #[inline]
    pub fn slice<R: RangeBounds<usize>>(&self, range: R) -> Option<&Self> {
        self.check_range(range).map(|r| {
            // SAFETY: `self` is a valid `WStr` and `r` is a valid slice range.
            unsafe { &*ptr::slice(self, r) }
        })
    }

    /// Returns a mutable subslice of `self`, or `None` if the slice indices are out of range.
    ///
    /// Use indexing for a panicking version.
    #[inline]
    pub fn slice_mut<R: RangeBounds<usize>>(&mut self, range: R) -> Option<&mut Self> {
        self.check_range(range).map(|r| {
            // SAFETY: `self` is a valid `WStr` and `r` is a valid slice range.
            unsafe { &mut *ptr::slice_mut(self, r) }
        })
    }

    /// Returns a subslice of `self` without doing bound checks.
    ///
    /// # Safety
    /// The range indices must be less than or equal to `self.len()`.
    #[inline]
    pub unsafe fn slice_unchecked<R: RangeBounds<usize>>(&self, range: R) -> &Self {
        self.slice(range)
            .unwrap_or_else(|| core::hint::unreachable_unchecked())
    }

    /// Returns a mutable subslice of `self` without doing bound checks.
    ///
    /// # Safety
    /// The range indices must be less than or equal to `self.len()`.
    #[inline]
    pub unsafe fn slice_unchecked_mut<R: RangeBounds<usize>>(&mut self, range: R) -> &Self {
        self.slice_mut(range)
            .unwrap_or_else(|| core::hint::unreachable_unchecked())
    }

    /// Iterates over the code units of `self`.
    #[inline]
    pub fn iter(&self) -> super::ops::Iter<'_> {
        super::ops::str_iter(self)
    }

    /// Iterates over the unicode characters of `self`.
    #[inline]
    pub fn chars(&self) -> super::ops::Chars<'_> {
        core::char::decode_utf16(super::ops::str_iter(self))
    }

    /// Iterates over the unicode characters of `self`, together with their indices.
    #[inline]
    pub fn char_indices(&self) -> crate::ops::CharIndices<'_> {
        super::ops::str_char_indices(self)
    }

    /// Returns the offset of `self` in `other`, if `self` is a substring of `other`.
    ///
    /// This is the value such that `self == other.slice(offset..offset + self.len())`.
    #[inline]
    pub fn offset_in(&self, other: &WStr) -> Option<usize> {
        super::ops::str_offset_in(self, other)
    }

    #[inline]
    /// Compares two strings for equality, ignoring case as done by the Flash Player.
    /// Note that the case mapping is different than Rust's case mapping.
    pub fn eq_ignore_case(&self, other: &WStr) -> bool {
        super::ops::str_eq_ignore_case(self, other)
    }

    #[inline]
    /// Compares two strings with the specified case sensitivity.
    /// Note that the case mapping is different than Rust's case mapping.
    pub fn eq_with_case<'a>(
        &self,
        other: impl Into<Units<&'a [u8], &'a [u16]>>,
        case_sensitive: bool,
    ) -> bool {
        if case_sensitive {
            self == WStr::from_units(other.into())
        } else {
            self.eq_ignore_case(WStr::from_units(other.into()))
        }
    }

    #[inline]
    /// Compares two strings, ignoring case as done by the Flash Player.
    /// Note that the case mapping is different than Rust's case mapping.
    pub fn cmp_ignore_case(&self, other: &WStr) -> core::cmp::Ordering {
        super::ops::str_cmp_ignore_case(self, other)
    }

    /// Parses the given string into another type.
    #[inline]
    pub fn parse<T: FromWStr>(&self) -> Result<T, T::Err> {
        T::from_wstr(self)
    }

    /// Returns `true` is the string contains only LATIN1 characters.
    ///
    /// Note that this doesn't necessarily mean that `self.is_wide()` is `false`.
    #[inline]
    pub fn is_latin1(&self) -> bool {
        super::ops::str_is_latin1(self)
    }

    /// Converts this string to an UTF8 `String`.
    ///
    /// Unpaired surrogates are replaced by the replacement character.
    #[inline]
    pub fn to_utf8_lossy(&self) -> alloc::borrow::Cow<'_, str> {
        super::ops::WStrToUtf8::new(self).to_utf8_lossy()
    }

    /// Returns a new string with all ASCII characters mapped to their lowercase equivalent.
    #[inline]
    pub fn to_ascii_lowercase(&self) -> WString {
        super::ops::str_to_ascii_lowercase(self)
    }

    /// Converts this string to its ASCII lower case equivalent in-place.
    #[inline]
    pub fn make_ascii_lowercase(&mut self) {
        super::ops::str_make_ascii_lowercase(self)
    }

    /// Returns a new string with all ASCII characters mapped to their uppercase equivalent.
    #[inline]
    pub fn to_ascii_uppercase(&self) -> WString {
        super::ops::str_to_ascii_uppercase(self)
    }

    /// Converts this string to its ASCII uppercase equivalent in-place.
    #[inline]
    pub fn make_ascii_uppercase(&mut self) {
        super::ops::str_make_ascii_uppercase(self)
    }

    /// Analogue of [`str::replace`].
    #[inline]
    pub fn replace<'a, P: Pattern<'a>>(&'a self, pattern: P, with: &WStr) -> WString {
        super::ops::str_replace(self, pattern, with)
    }

    /// Analogue of [`str::find`].
    #[inline]
    pub fn find<'a, P: Pattern<'a>>(&'a self, pattern: P) -> Option<usize> {
        super::ops::str_find(self, pattern)
    }

    /// Analogue of [`str::rfind`].
    #[inline]
    pub fn rfind<'a, P: Pattern<'a>>(&'a self, pattern: P) -> Option<usize> {
        super::ops::str_rfind(self, pattern)
    }

    /// Analogue of [`str::contains`].
    #[inline]
    pub fn contains<'a, P: Pattern<'a>>(&'a self, pattern: P) -> bool {
        self.find(pattern).is_some()
    }

    /// Analogue of [`str::split`].
    #[inline]
    pub fn split<'a, P: Pattern<'a>>(&'a self, separator: P) -> super::ops::Split<'a, P> {
        super::ops::str_split(self, separator)
    }

    /// Analogue of [`str::split_at`].
    #[inline]
    pub fn split_at(&self, index: usize) -> (&WStr, &WStr) {
        (&self[..index], &self[index..])
    }

    /// Analogue of [`str::split_once`].
    #[inline]
    pub fn split_once<'a, P: Pattern<'a>>(&'a self, pattern: P) -> Option<(&'a WStr, &'a WStr)> {
        super::ops::str_split_once(self, pattern)
    }

    /// Analogue of [`str::rsplit_once`].
    #[inline]
    pub fn rsplit_once<'a, P: Pattern<'a>>(&'a self, pattern: P) -> Option<(&'a WStr, &'a WStr)> {
        super::ops::str_rsplit_once(self, pattern)
    }

    /// Analogue of [`str::trim_matches`].
    #[inline]
    pub fn trim_matches<'a, P: Pattern<'a>>(&'a self, pattern: P) -> &'a WStr {
        super::ops::str_trim_matches(self, pattern)
    }

    /// Analogue of [`str::trim_start_matches`].
    #[inline]
    pub fn trim_start_matches<'a, P: Pattern<'a>>(&'a self, pattern: P) -> &'a WStr {
        super::ops::str_trim_start_matches(self, pattern)
    }

    /// Analogue of [`str::trim_end_matches`].
    #[inline]
    pub fn trim_end_matches<'a, P: Pattern<'a>>(&'a self, pattern: P) -> &'a WStr {
        super::ops::str_trim_end_matches(self, pattern)
    }

    /// Analogue of [`str::trim`], but uses Flash's definition of whitespace.
    #[inline]
    pub fn trim(&self) -> &WStr {
        self.trim_matches(super::utils::swf_is_whitespace)
    }

    /// Analogue of [`str::trim_start`], but uses Flash's definition of whitespace.
    #[inline]
    pub fn trim_start(&self) -> &WStr {
        self.trim_start_matches(super::utils::swf_is_whitespace)
    }

    /// Analogue of [`str::trim_end`], but uses Flash's definition of whitespace.
    #[inline]
    pub fn trim_end(&self) -> &WStr {
        self.trim_end_matches(super::utils::swf_is_whitespace)
    }

    /// Analogue of [`str::starts_with`].
    #[inline]
    pub fn starts_with<'a, P: Pattern<'a>>(&'a self, pattern: P) -> bool {
        super::ops::starts_with(self, pattern)
    }

    /// Analogue of [`str::ends_with`].
    #[inline]
    pub fn ends_with<'a, P: Pattern<'a>>(&'a self, pattern: P) -> bool {
        super::ops::ends_with(self, pattern)
    }

    /// Analogue of [`str::strip_prefix`].
    #[inline]
    pub fn strip_prefix<'a, P: Pattern<'a>>(&'a self, pattern: P) -> Option<&'a WStr> {
        super::ops::strip_prefix(self, pattern)
    }

    /// Analogue of [`str::strip_suffix`].
    #[inline]
    pub fn strip_suffix<'a, P: Pattern<'a>>(&'a self, pattern: P) -> Option<&'a WStr> {
        super::ops::strip_suffix(self, pattern)
    }

    /// Analogue of [`str::repeat`]
    #[inline]
    pub fn repeat(&self, count: usize) -> WString {
        super::ops::str_repeat(self, count)
    }
}

impl Default for &WStr {
    #[inline]
    fn default() -> Self {
        WStr::empty()
    }
}

impl Default for &mut WStr {
    #[inline]
    fn default() -> Self {
        WStr::empty_mut()
    }
}

impl<R: RangeBounds<usize>> Index<R> for WStr {
    type Output = WStr;

    #[inline]
    fn index(&self, idx: R) -> &Self::Output {
        self.slice(idx).expect("string indices out of bounds")
    }
}

impl<R: RangeBounds<usize>> IndexMut<R> for WStr {
    #[inline]
    fn index_mut(&mut self, idx: R) -> &mut Self::Output {
        self.slice_mut(idx).expect("string indices out of bounds")
    }
}

impl core::cmp::PartialEq for WStr {
    #[inline]
    fn eq(&self, other: &WStr) -> bool {
        super::ops::str_eq(self, other)
    }
}

impl core::cmp::Eq for WStr {}

impl core::cmp::Ord for WStr {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        super::ops::str_cmp(self, other)
    }
}

impl core::cmp::PartialOrd for WStr {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl core::hash::Hash for WStr {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        super::ops::str_hash(self, state)
    }
}

impl core::fmt::Display for WStr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        super::ops::str_fmt(self, f)
    }
}

impl core::fmt::Debug for WStr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        super::ops::str_debug_fmt(self, f)
    }
}

impl<'a> core::iter::IntoIterator for &'a WStr {
    type Item = u16;
    type IntoIter = super::Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        super::ops::str_iter(self)
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __wstr_impl_internal {
    (@eq_ord_units [$($generics:tt)*] for $lhs:ty, $rhs:ty) => {
        impl<$($generics)*> ::core::cmp::PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                ::core::cmp::PartialEq::eq(&self[..], $crate::WStr::from_units(other))
            }
        }

        impl<$($generics)*> ::core::cmp::PartialOrd<$rhs> for $lhs {
            #[inline]
            fn partial_cmp(&self, other: &$rhs) -> Option<::core::cmp::Ordering> {
                Some(::core::cmp::Ord::cmp(&self[..], $crate::WStr::from_units(other)))
            }
        }

        impl<$($generics)*> ::core::cmp::PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                ::core::cmp::PartialEq::eq(WStr::from_units(self), &other[..])
            }
        }

        impl<$($generics)*> ::core::cmp::PartialOrd<$lhs> for $rhs {
            #[inline]
            fn partial_cmp(&self, other: &$lhs) -> Option<::core::cmp::Ordering> {
                Some(::core::cmp::Ord::cmp(WStr::from_units(self), &other[..]))
            }
        }
    };

    (@eq_ord [$($generics:tt)*] for $lhs:ty, $rhs:ty) => {
        impl<$($generics)*> ::core::cmp::PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                ::core::cmp::PartialEq::eq(&self[..], &other[..])
            }
        }

        impl<$($generics)*> ::core::cmp::PartialOrd<$lhs> for $rhs {
            #[inline]
            fn partial_cmp(&self, other: &$lhs) -> Option<::core::cmp::Ordering> {
                Some(::core::cmp::Ord::cmp(&self[..], &other[..]))
            }
        }
    };

    (@eq_ord_self [$($generics:tt)*] for $ty:ty) => {
        $crate::__wstr_impl_internal! { @eq_ord_units [$($generics)* const N: usize] for $ty, [u8; N] }
        $crate::__wstr_impl_internal! { @eq_ord_units [$($generics)* const N: usize] for $ty, [u16; N] }
        $crate::__wstr_impl_internal! { @eq_ord_units [$($generics)*] for $ty, [u8] }
        $crate::__wstr_impl_internal! { @eq_ord_units [$($generics)*] for $ty, [u16] }
    };

    (@eq_base [$($generics:tt)*] for $ty:ty) => {
        impl<$($generics)*> ::core::cmp::PartialEq<$ty> for $ty {
            #[inline]
            fn eq(&self, other: &$ty) -> bool {
                ::core::cmp::PartialEq::eq(::core::ops::Deref::deref(self), ::core::ops::Deref::deref(other))
            }
        }

        impl<$($generics)*> ::core::cmp::Eq for $ty {}
    };

    (@base [$($generics:tt)*] for $ty:ty) => {
        impl<$($generics)*> ::core::convert::AsRef<$crate::WStr> for $ty {
            #[inline]
            fn as_ref(&self) -> &$crate::WStr {
                ::core::ops::Deref::deref(self)
            }
        }

        impl<$($generics)*> ::core::borrow::Borrow<$crate::WStr> for $ty {
            #[inline]
            fn borrow(&self) -> &$crate::WStr {
                ::core::ops::Deref::deref(self)
            }
        }

        #[automatically_derived]
        impl<$($generics)*> ::core::cmp::Ord for $ty {
            #[inline]
            fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
                ::core::cmp::Ord::cmp(::core::ops::Deref::deref(self), ::core::ops::Deref::deref(other))
            }
        }

        #[automatically_derived]
        impl<$($generics)*> ::core::cmp::PartialOrd<$ty> for $ty {
            #[inline]
            fn partial_cmp(&self, other: &$ty) -> Option<::core::cmp::Ordering> {
                Some(::core::cmp::Ord::cmp(::core::ops::Deref::deref(self), other))
            }
        }

        impl<$($generics)*> ::core::hash::Hash for $ty {
            #[inline]
            fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                ::core::hash::Hash::hash(::core::ops::Deref::deref(self), state)
            }
        }

        impl<$($generics)*> ::core::fmt::Display for $ty {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::Display::fmt(::core::ops::Deref::deref(self), f)
            }
        }

        impl<$($generics)*> ::core::fmt::Debug for $ty {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::Debug::fmt(::core::ops::Deref::deref(self), f)
            }
        }

        impl<'_0, $($generics)*> ::core::iter::IntoIterator for &'_0 $ty {
            type Item = u16;
            type IntoIter = $crate::Iter<'_0>;

            #[inline]
            fn into_iter(self) -> Self::IntoIter {
                ::core::iter::IntoIterator::into_iter(::core::ops::Deref::deref(self))
            }
        }

        impl<'_0, $($generics)*> $crate::Pattern<'_0> for &'_0 $ty {
            type Searcher = <&'_0 WStr as $crate::Pattern<'_0>>::Searcher;

            #[inline]
            fn into_searcher(self, haystack: &'_0 $crate::WStr) -> Self::Searcher {
                $crate::Pattern::into_searcher(::core::ops::Deref::deref(self), haystack)
            }
        }
    };

    (@full_no_eq [$($generics:tt)*] for $ty:ty) => {
        $crate::__wstr_impl_internal!(@base [$($generics)*] for $ty);
        $crate::__wstr_impl_internal!(@eq_ord_self [$($generics)*] for $ty);
        $crate::__wstr_impl_internal!(@eq_ord [$($generics)*] for $ty, $crate::WStr);
        $crate::__wstr_impl_internal!(@eq_ord [$($generics)*] for $crate::WStr, $ty);
        $crate::__wstr_impl_internal!(@eq_ord [$($generics)* 'a,] for $ty, &'a $crate::WStr);
        $crate::__wstr_impl_internal!(@eq_ord [$($generics)* 'a,] for &'a $crate::WStr, $ty);
    };
}

/// Implements standard traits for `WStr`-like types.
///
/// This macro requires a pre-existing [`Deref<Target=WStr>`][core::ops::Deref] impl, and will emit
/// delegating impls for the following traits:
///
///   - [`AsRef<WStr>`], [`Borrow<WStr>`][core::borrow::Borrow];
///   - [`Eq`], [`PartialEq`] (this can be disabled with `manual_eq`);
///   - [`Ord`], [`PartialOrd`],
///   - [`PartialEq<_>`], [`PartialOrd<_>`] for [`WStr`], [`&WStr`][WStr],
///     `[u8]`, `[u16]`, `[u8; N]` and `[u16; N]`;
///   - [`Display`][core::fmt::Display], [`Debug`][core::fmt::Debug];
///   - [`Hash`][core::hash::Hash];
///   - [`IntoIterator<Item=u16>`][IntoIterator].
#[macro_export]
macro_rules! wstr_impl_traits {
    (impl manual_eq for $ty_name:ty) => {
        $crate::__wstr_impl_internal!(@full_no_eq [] for $ty_name);
    };
    (impl for $ty_name:ty) => {
        $crate::__wstr_impl_internal!(@full_no_eq [] for $ty_name);
        $crate::__wstr_impl_internal!(@eq_base [] for $ty_name);
    };
    (impl[$($generics:tt)+] manual_eq for $ty_name:ty) => {
        $crate::__wstr_impl_internal!(@full_no_eq [$($generics)*,] for $ty_name);
    };
    (impl [$($generics:tt)+] for $ty_name:ty) => {
        $crate::__wstr_impl_internal!(@full_no_eq [$($generics)*,] for $ty_name);
        $crate::__wstr_impl_internal!(@eq_base [$($generics)*,] for $ty_name);

    };
}

__wstr_impl_internal!(@eq_ord_self [] for WStr);
