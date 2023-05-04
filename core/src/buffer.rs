//! Shared-ownership buffer types

use gc_arena::Collect;
use std::fmt::Debug;
use std::ops::{Bound, Deref, RangeBounds};
use std::sync::{Arc, RwLock, RwLockReadGuard};

/// A shared data buffer.
///
/// `Buffer` is intended to mirror the API of a `Vec<u8>`, but with shared
/// ownership. Mutability is partially supported: you may append data to the
/// end of the buffer, but not change or remove bytes already added to the
/// buffer.
///
/// Buffer data may also be sliced to yield references to the underlying data.
/// See `Slice` for more info.
#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct Buffer(Arc<RwLock<Vec<u8>>>);

impl Buffer {
    pub fn new() -> Self {
        Buffer(Arc::new(RwLock::new(Vec::new())))
    }

    pub fn with_capacity(cap: usize) -> Self {
        Buffer(Arc::new(RwLock::new(Vec::with_capacity(cap))))
    }

    pub fn capacity(&self) -> usize {
        self.0.read().expect("unlock read").capacity()
    }

    pub fn len(&self) -> usize {
        self.0.read().expect("unlock read").len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.read().expect("unlock read").is_empty()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.0.write().expect("unlock write").reserve(additional)
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        self.0
            .write()
            .expect("unlock write")
            .reserve_exact(additional)
    }

    pub fn as_slice(&self) -> Slice {
        let end = self.0.read().expect("unlock read").len();
        Slice {
            buf: self.clone(),
            start: 0,
            end,
        }
    }

    pub fn append(&mut self, other: &mut Vec<u8>) {
        self.0.write().expect("unlock write").append(other)
    }

    pub fn extend_from_slice(&mut self, other: &[u8]) {
        self.0
            .write()
            .expect("unlock write")
            .extend_from_slice(other)
    }

    pub fn get<T: RangeBounds<usize>>(&self, range: T) -> Option<Slice> {
        let s = self.0.read().expect("unlock read");

        let start = match range.start_bound() {
            Bound::Included(u) => *u,
            Bound::Excluded(u) => *u + 1,
            Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            Bound::Included(u) => *u + 1,
            Bound::Excluded(u) => *u,
            Bound::Unbounded => s.len(),
        };

        if start <= s.len() && end <= s.len() && start <= end {
            Some(Slice {
                buf: Self(self.0.clone()),
                start: start,
                end: end,
            })
        } else {
            None
        }
    }

    pub fn to_full_slice(&self) -> Slice {
        self.get(0..).expect("full slices are always valid")
    }

    pub fn to_empty_slice(&self) -> Slice {
        self.get(0..0).expect("empty slices are always valid")
    }
}

impl From<Vec<u8>> for Buffer {
    fn from(val: Vec<u8>) -> Self {
        Self(Arc::new(RwLock::new(val)))
    }
}

/// A reference into a shared data buffer.
///
/// `Slice` is intended to mirror the API of a `&[u8]`, but without retaining
/// a borrow. Ownership of a `Slice` keeps the underlying buffer data alive,
/// and you can read the buffer data at any time.
///
/// Slice bounds are interpreted as a [start..end] pair, i.e. inclusive on the
/// start bound and exclusive on the end.
#[derive(Clone, Debug)]
pub struct Slice {
    buf: Buffer,
    start: usize,
    end: usize,
}

impl Slice {
    /// Create a subslice of this buffer slice.
    ///
    /// The parameter `slice` must be derived from the same buffer this slice
    /// was, and must also be within bounds of this slice. If not, then the
    /// returned slice will be empty.
    pub fn to_subslice(&self, slice: &[u8]) -> Self {
        let self_guard = self.buf.0.read().expect("unlock read");
        let self_pval = self_guard.as_ptr() as usize;
        let slice_pval = slice.as_ptr() as usize;

        if (self_pval + self.start) <= slice_pval && slice_pval < (self_pval + self.end) {
            Self {
                buf: self.buf.clone(),
                start: slice_pval - self_pval,
                end: (slice_pval - self_pval) + slice.len(),
            }
        } else {
            self.buf.to_empty_slice()
        }
    }

    /// Create a subslice of this buffer slice, without bounds checking.
    ///
    /// The parameter `slice` must be derived from the same buffer this slice
    /// was, otherwise the returned slice will be empty
    ///
    /// This emulates "unbounded reads" in file formats that don't bounds-check
    /// things properly.
    pub fn to_unbounded_subslice(&self, slice: &[u8]) -> Self {
        let self_guard = self.buf.0.read().expect("unlock read");
        let self_pval = self_guard.as_ptr() as usize;
        let self_len = self.buf.len();
        let slice_pval = slice.as_ptr() as usize;

        if self_pval <= slice_pval && slice_pval < (self_pval + self_len) {
            Slice {
                buf: self.buf.clone(),
                start: slice_pval - self_pval,
                end: (slice_pval - self_pval) + slice.len(),
            }
        } else {
            self.buf.to_empty_slice()
        }
    }

    /// Construct a new Slice from a start and an end.
    ///
    /// The start and end values will be relative to the current slice.
    /// Furthermore, this function will yield an empty slice if the calculated
    /// slice would be invalid (e.g. negative length) or would extend past the
    /// end of the current slice.
    pub fn to_start_and_end(&self, start: usize, end: usize) -> Self {
        let new_start = self.start + start;
        let new_end = self.start + end;

        if new_start <= new_end && new_end < self.end {
            if let Some(result) = self.buf.get(new_start..new_end) {
                result
            } else {
                self.buf.to_empty_slice()
            }
        } else {
            self.buf.to_empty_slice()
        }
    }

    /// Get a subslice of this slice.
    ///
    /// Normal subslicing bounds rules will be respected. If you want to get a
    /// slice outside the bounds of this one, use `to_unbounded_subslice`.
    pub fn get<T: RangeBounds<usize>>(&self, range: T) -> Option<Slice> {
        let s = self.buf.0.read().expect("unlock read");

        let start = match range.start_bound() {
            Bound::Included(u) => *u,
            Bound::Excluded(u) => *u + 1,
            Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            Bound::Included(u) => *u + 1,
            Bound::Excluded(u) => *u,
            Bound::Unbounded => s.len(),
        };

        if start <= s.len() && end <= s.len() && start <= end {
            Some(Slice {
                buf: self.buf.clone(),
                start: self.start + start,
                end: self.start + end,
            })
        } else {
            None
        }
    }

    /// Checks if this slice is empty
    pub fn is_empty(&self) -> bool {
        self.end == self.start
    }

    /// Get the length of the Slice.
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn data(&self) -> SliceRef {
        SliceRef {
            guard: self.buf.0.read().expect("unlock read"),
            start: self.start,
            end: self.end,
        }
    }
}

#[derive(Debug)]
pub struct SliceRef<'a> {
    guard: RwLockReadGuard<'a, Vec<u8>>,
    start: usize,
    end: usize,
}

impl Deref for SliceRef<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.guard[self.start..self.end]
    }
}

impl PartialEq for SliceRef<'_> {
    fn eq(&self, other: &SliceRef<'_>) -> bool {
        self.guard.as_ptr() == other.guard.as_ptr()
            && self.start == other.start
            && self.end == other.end
    }
}

#[cfg(test)]
mod test {
    use crate::buffer::Buffer;

    #[test]
    fn buf_slice() {
        let buf = Buffer::from(vec![0, 1, 2, 3, 4, 5]);
        let slice = buf.get(2..4).expect("valid slice");

        assert_eq!(&*slice.data(), &[2, 3]);
    }

    #[test]
    fn buf_slice_append() {
        let mut buf = Buffer::from(vec![0, 1, 2, 3, 4, 5]);
        let slice = buf.get(2..4).expect("valid slice");

        assert_eq!(&*slice.data(), &[2, 3]);

        buf.append(&mut vec![6, 7, 8, 9]);

        assert_eq!(&*slice.data(), &[2, 3]);
    }
}
