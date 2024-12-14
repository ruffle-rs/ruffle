//! Shared-ownership buffer types

use gc_arena::Collect;
use std::cmp::min;
use std::fmt::{Debug, Formatter, LowerHex, UpperHex};
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Read, Result as IoResult};
use std::ops::{Bound, Deref, RangeBounds};
use std::sync::{Arc, RwLock, RwLockReadGuard};
use thiserror::Error;

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
                start,
                end,
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

impl Default for Buffer {
    fn default() -> Self {
        Buffer::new()
    }
}

impl From<Vec<u8>> for Buffer {
    fn from(val: Vec<u8>) -> Self {
        Self(Arc::new(RwLock::new(val)))
    }
}

impl PartialEq for Buffer {
    fn eq(&self, other: &Buffer) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
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

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn data(&self) -> SliceRef {
        SliceRef {
            guard: self.buf.0.read().expect("unlock read"),
            start: self.start,
            end: self.end,
        }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buf
    }

    /// Create a readable cursor into the `Slice`.
    pub fn as_cursor(&self) -> SliceCursor {
        SliceCursor {
            slice: self.clone(),
            pos: 0,
        }
    }
}

impl LowerHex for Slice {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        LowerHex::fmt(&self.data(), f)
    }
}

impl UpperHex for Slice {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        UpperHex::fmt(&self.data(), f)
    }
}

/// A readable cursor into a buffer slice.
pub struct SliceCursor {
    slice: Slice,
    pos: usize,
}

impl Read for SliceCursor {
    fn read(&mut self, data: &mut [u8]) -> IoResult<usize> {
        let copy_count = min(data.len(), self.slice.len() - self.pos);
        let slice = self
            .slice
            .get(self.pos..self.pos + copy_count)
            .expect("Slice offsets are always valid");
        let slice_data = slice.data();

        data[..copy_count].copy_from_slice(&slice_data);
        self.pos += copy_count;
        Ok(copy_count)
    }
}

#[derive(Debug, Error)]
pub enum SubstreamError {
    #[error("Attempted to add substream chunk from a foreign buffer")]
    ForeignBuffer,
}

/// A list of multiple slices of the same buffer.
///
/// `Substream` represents a substream of the underlying `Buffer`. `Slice`s can
/// be appended to the chunks list in order to extend the substream, in the
/// same way that the underlying `Buffer` can be extended. All `Slice`s must be
/// backed by the same `Buffer` in order to be part of the same `Substream`.
///
/// Clones of a `Substream` share a single chunk list, and appending chunks
/// will extend all clones of the `Substream`.
#[derive(Clone, Debug)]
pub struct Substream {
    buf: Buffer,

    /// Shared list of chunks. Chunks are stored as (start, end) pairs.
    chunks: Arc<RwLock<Vec<(usize, usize)>>>,
}

impl Substream {
    pub fn new(buf: Buffer) -> Self {
        Self {
            buf,
            chunks: Arc::new(RwLock::new(vec![])),
        }
    }

    /// Append another `Slice` onto the end of the `Substream`.
    ///
    /// Appended chunks will be present in all clones of the `Substream`.
    pub fn append(&mut self, slice: Slice) -> Result<(), SubstreamError> {
        let mut chunks = self.chunks.write().unwrap();
        if self.buf == slice.buf {
            chunks.push((slice.start, slice.end));

            Ok(())
        } else {
            Err(SubstreamError::ForeignBuffer)
        }
    }

    /// Calculate the number of chunks in the `Substream`.
    pub fn num_chunks(&self) -> usize {
        self.chunks.read().unwrap().len()
    }

    /// Create a readable cursor into the `Substream`.
    ///
    /// The returned cursor clones the `Substream` and thus shares a chunk list
    /// with it.
    pub fn as_cursor(&self) -> SubstreamCursor {
        SubstreamCursor {
            substream: self.clone(),
            chunk_pos: 0,
            bytes_pos: 0,
        }
    }

    /// Calculate the number of bytes in the `Substream`.
    pub fn len(&self) -> usize {
        let mut tally = 0;
        let chunks = self.chunks.read().unwrap();
        for (chunk_start, chunk_end) in chunks.iter() {
            tally += chunk_end - chunk_start;
        }

        tally
    }

    /// Determine if the `Substream` is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Create a chunk iterator into the `Substream`.
    ///
    /// The returned iterator clones the `Substream` and thus shares a chunk
    /// list with it.
    pub fn iter_chunks(&self) -> SubstreamChunksIter {
        SubstreamChunksIter {
            substream: self.clone(),
            next_buf: 0,
        }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buf
    }

    pub fn first_chunk(&self) -> Option<Slice> {
        if let Some((start, end)) = self.chunks.read().unwrap().first() {
            Some(Slice {
                buf: self.buf.clone(),
                start: *start,
                end: *end,
            })
        } else {
            None
        }
    }

    pub fn last_chunk(&self) -> Option<Slice> {
        if let Some((start, end)) = self.chunks.read().unwrap().last() {
            Some(Slice {
                buf: self.buf.clone(),
                start: *start,
                end: *end,
            })
        } else {
            None
        }
    }
}

impl From<Buffer> for Substream {
    fn from(buf: Buffer) -> Self {
        Self {
            buf,
            chunks: Arc::new(RwLock::new(vec![])),
        }
    }
}

impl From<Slice> for Substream {
    fn from(slice: Slice) -> Self {
        Self {
            buf: slice.buf,
            chunks: Arc::new(RwLock::new(vec![(slice.start, slice.end)])),
        }
    }
}

/// A readable cursor into a buffer substream.
///
/// Reads from the cursor (via `Read` etc) will be filled as if the substream
/// referred to in the chunks list is a single contiguous stream of bytes with
/// no other data in between. Code using a `SubstreamCursor` can thus work with
/// both multiplexed and unmultiplexed data in the same way.
pub struct SubstreamCursor {
    substream: Substream,
    chunk_pos: usize,
    bytes_pos: usize,
}

impl Read for SubstreamCursor {
    fn read(&mut self, data: &mut [u8]) -> IoResult<usize> {
        let mut out_count = 0;
        let buf_owned = self.substream.buf.clone();
        let buf = buf_owned.0.read().map_err(|_| {
            IoError::new(
                IoErrorKind::Other,
                "the underlying substream is locked by a panicked process",
            )
        })?;

        let chunks = self.substream.chunks.read().unwrap();

        while out_count < data.len() {
            let cur_chunk = chunks.get(self.chunk_pos);
            if cur_chunk.is_none() {
                //out of chunks to read
                return Ok(out_count);
            }

            let cur_chunk = cur_chunk.expect("cur_chunk should never be None");

            let chunk_len = cur_chunk.1 - cur_chunk.0;
            let copy_count = min(data.len() - out_count, chunk_len - self.bytes_pos);

            data[out_count..out_count + copy_count].copy_from_slice(
                buf.get(cur_chunk.0 + self.bytes_pos..cur_chunk.0 + self.bytes_pos + copy_count)
                    .expect("Slice offsets are always valid"),
            );

            self.bytes_pos += copy_count;
            out_count += copy_count;

            if self.bytes_pos < chunk_len {
                //`data` is full
                break;
            }

            //`data` not full, move onto next chunk
            self.chunk_pos += 1;
            self.bytes_pos = 0;
        }

        Ok(out_count)
    }
}

/// Iterator for substream chunks
pub struct SubstreamChunksIter {
    substream: Substream,
    next_buf: usize,
}

impl Iterator for SubstreamChunksIter {
    type Item = Slice;

    fn next(&mut self) -> Option<Slice> {
        if let Some((start, end)) = self.substream.chunks.read().unwrap().get(self.next_buf) {
            self.next_buf += 1;
            return Some(Slice {
                buf: self.substream.buf.clone(),
                start: *start,
                end: *end,
            });
        }

        None
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

impl LowerHex for SliceRef<'_> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "[")?;
        for (index, byte) in self[..].iter().enumerate() {
            if index > 0 {
                write!(f, " ")?;
            }
            LowerHex::fmt(byte, f)?;
        }
        write!(f, "]")?;

        Ok(())
    }
}

impl UpperHex for SliceRef<'_> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "[")?;
        for (index, byte) in self[..].iter().enumerate() {
            if index > 0 {
                write!(f, " ")?;
            }
            UpperHex::fmt(byte, f)?;
        }
        write!(f, "]")?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::buffer::{Buffer, Substream};
    use std::io::Read;

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

    #[test]
    fn slice_cursor_read() {
        let buf = Buffer::from(vec![
            38, 26, 99, 1, 1, 1, 1, 38, 12, 14, 1, 1, 93, 86, 1, 88,
        ]);
        let slice = buf.to_full_slice();
        let mut cursor = slice.as_cursor();
        let refdata = slice.data();

        let mut data = vec![0; 1];

        for byte in &refdata[..] {
            let result = cursor.read(&mut data);
            assert_eq!(result.unwrap(), 1);
            assert_eq!(data[0], *byte);
        }
    }

    #[test]
    fn slice_cursor_read_all() {
        let buf = Buffer::from(vec![
            38, 26, 99, 1, 1, 1, 1, 38, 12, 14, 1, 1, 93, 86, 1, 88,
        ]);
        let slice = buf.to_full_slice();
        let mut cursor = slice.as_cursor();
        let refdata = slice.data();

        let mut data = vec![0; slice.len() + 32];

        let result = cursor.read(&mut data);
        assert_eq!(result.unwrap(), slice.len());
        assert_eq!(&data[..slice.len()], &refdata[..]);
    }

    #[test]
    fn substream_cursor_read_inside() {
        let buf = Buffer::from(vec![
            38, 26, 99, 1, 1, 1, 1, 38, 12, 14, 1, 1, 93, 86, 1, 88,
        ]);
        let mut substream = Substream::new(buf.clone());

        substream.append(buf.get(3..7).unwrap()).unwrap();
        substream.append(buf.get(10..12).unwrap()).unwrap();
        substream.append(buf.get(14..15).unwrap()).unwrap();

        let mut cursor = substream.as_cursor();
        let mut data = vec![0; 7];
        let result = cursor.read(&mut data);
        assert_eq!(result.unwrap(), 7);
        assert_eq!(data, vec![1; 7]);
    }

    #[test]
    fn substream_cursor_read_outside() {
        let buf = Buffer::from(vec![
            38, 26, 99, 1, 1, 1, 1, 38, 12, 14, 1, 1, 93, 86, 1, 88,
        ]);
        let mut substream = Substream::new(buf.clone());

        substream.append(buf.get(0..3).unwrap()).unwrap();
        substream.append(buf.get(7..10).unwrap()).unwrap();
        substream.append(buf.get(12..14).unwrap()).unwrap();
        substream.append(buf.get(15..).unwrap()).unwrap();

        let mut cursor = substream.as_cursor();
        let mut data = vec![0; 7];
        let result = cursor.read(&mut data);
        assert_eq!(result.unwrap(), 7);
        assert_eq!(data, vec![38, 26, 99, 38, 12, 14, 93]);

        let mut data = vec![0; 2];
        let result = cursor.read(&mut data);
        assert_eq!(result.unwrap(), 2);
        assert_eq!(data, vec![86, 88]);

        let mut cursor = substream.as_cursor();
        let mut data = vec![0; 8];
        let result = cursor.read(&mut data);
        assert_eq!(result.unwrap(), 8);
        assert_eq!(data, vec![38, 26, 99, 38, 12, 14, 93, 86]);

        let mut data = vec![0; 1];
        let result = cursor.read(&mut data);
        assert_eq!(result.unwrap(), 1);
        assert_eq!(data, vec![88]);

        let mut cursor = substream.as_cursor();
        let mut data = vec![0; 9];
        let result = cursor.read(&mut data);
        assert_eq!(result.unwrap(), 9);
        assert_eq!(data, vec![38, 26, 99, 38, 12, 14, 93, 86, 88]);
    }
}
