//! Like [`core::str::pattern::Pattern`], but for [`WStr`].

// TODO: Is performance good? ideas for improvements:
//  - add some inlines?
//  - remove implicit bound checks?
//  - use memchr crate?

use super::{Units, WStr};

/// A pattern that can be searched in a [`WStr`].
///
/// - `WStr` searches for the given string.
/// - `u8` searches for a single LATIN1 code unit.
/// - `u16` searches for a single UCS2 code unit.
/// - `&[u8]` searches for any of the given LATIN1 code units.
/// - `&[u16]` searches for any of the given UCS2 code units.
/// - `FnMut(u16) -> bool` searches for code units matching the predicate.
pub trait Pattern<'a> {
    type Searcher: Searcher<'a>;

    fn into_searcher(self, haystack: &'a WStr) -> Self::Searcher;
}

pub enum SearchStep {
    Match(usize, usize),
    Reject(usize, usize),
    Done,
}

pub trait Searcher<'a> {
    fn next(&mut self) -> SearchStep;

    fn next_back(&mut self) -> SearchStep;

    fn next_match(&mut self) -> Option<(usize, usize)> {
        loop {
            break match self.next() {
                SearchStep::Match(i, j) => Some((i, j)),
                SearchStep::Reject(_, _) => continue,
                SearchStep::Done => None,
            };
        }
    }

    fn next_match_back(&mut self) -> Option<(usize, usize)> {
        loop {
            break match self.next_back() {
                SearchStep::Match(i, j) => Some((i, j)),
                SearchStep::Reject(_, _) => continue,
                SearchStep::Done => None,
            };
        }
    }

    fn next_reject(&mut self) -> Option<(usize, usize)> {
        loop {
            break match self.next() {
                SearchStep::Match(_, _) => continue,
                SearchStep::Reject(i, j) => Some((i, j)),
                SearchStep::Done => None,
            };
        }
    }

    fn next_reject_back(&mut self) -> Option<(usize, usize)> {
        loop {
            break match self.next_back() {
                SearchStep::Match(_, _) => continue,
                SearchStep::Reject(i, j) => Some((i, j)),
                SearchStep::Done => None,
            };
        }
    }
}

impl<'a> Pattern<'a> for u8 {
    type Searcher = Either<PredSearcher<'a, u8, u8>, PredSearcher<'a, u16, u16>>;

    fn into_searcher(self, haystack: &'a WStr) -> Self::Searcher {
        match haystack.units() {
            Units::Bytes(h) => Either::Left(PredSearcher::new(true, h, self)),
            Units::Wide(h) => Either::Right(PredSearcher::new(true, h, self.into())),
        }
    }
}

impl<'a> Pattern<'a> for u16 {
    type Searcher = Either<PredSearcher<'a, u8, u8>, PredSearcher<'a, u16, u16>>;

    fn into_searcher(self, haystack: &'a WStr) -> Self::Searcher {
        let is_latin1 = self <= u8::MAX as u16;
        match haystack.units() {
            Units::Bytes(h) => Either::Left(PredSearcher::new(is_latin1, h, self as u8)),
            Units::Wide(h) => Either::Right(PredSearcher::new(true, h, self)),
        }
    }
}

impl<'a> Pattern<'a> for &'a [u8] {
    type Searcher =
        Either<PredSearcher<'a, u8, AnyOf<'a, u8>>, PredSearcher<'a, u16, AnyOf<'a, u8>>>;

    fn into_searcher(self, haystack: &'a WStr) -> Self::Searcher {
        let can_match = !self.is_empty();
        match haystack.units() {
            Units::Bytes(h) => Either::Left(PredSearcher::new(can_match, h, AnyOf(self))),
            Units::Wide(h) => Either::Right(PredSearcher::new(can_match, h, AnyOf(self))),
        }
    }
}

impl<'a> Pattern<'a> for &'a [u16] {
    type Searcher =
        Either<PredSearcher<'a, u8, AnyOf<'a, u16>>, PredSearcher<'a, u16, AnyOf<'a, u16>>>;

    fn into_searcher(self, haystack: &'a WStr) -> Self::Searcher {
        let can_match =
            !self.is_empty() && (haystack.is_wide() || self.iter().any(|c| *c <= u8::MAX as u16));
        match haystack.units() {
            Units::Bytes(h) => Either::Left(PredSearcher::new(can_match, h, AnyOf(self))),
            Units::Wide(h) => Either::Right(PredSearcher::new(can_match, h, AnyOf(self))),
        }
    }
}

impl<'a, F: FnMut(u16) -> bool> Pattern<'a> for F {
    type Searcher = Either<PredSearcher<'a, u8, FnPred<F>>, PredSearcher<'a, u16, FnPred<F>>>;

    fn into_searcher(self, haystack: &'a WStr) -> Self::Searcher {
        match haystack.units() {
            Units::Bytes(h) => Either::Left(PredSearcher::new(true, h, FnPred(self))),
            Units::Wide(h) => Either::Right(PredSearcher::new(true, h, FnPred(self))),
        }
    }
}

impl<'a> Pattern<'a> for &'a WStr {
    #[allow(clippy::type_complexity)]
    type Searcher = Either<
        Either<Either<SliceSearcher<'a, u8>, SliceSearcher<'a, u16>>, StrSearcher<'a>>,
        EmptySearcher,
    >;

    fn into_searcher(self, haystack: &'a WStr) -> Self::Searcher {
        if self.is_empty() {
            return Either::Right(EmptySearcher::new(haystack.len()));
        }

        let s = match (haystack.units(), self.units()) {
            (Units::Bytes(h), Units::Bytes(n)) => Either::Left(SliceSearcher::new(h, n)),
            (Units::Wide(h), Units::Wide(n)) => Either::Right(SliceSearcher::new(h, n)),
            (Units::Bytes(_), _) if self.len() > haystack.len() || !self.is_latin1() => {
                Either::Left(SliceSearcher::new(&[], &[0]))
            }
            _ => return Either::Left(Either::Right(StrSearcher::new(haystack, self))),
        };

        Either::Left(Either::Left(s))
    }
}

pub enum Either<T, U> {
    Left(T),
    Right(U),
}

impl<'a, T: Searcher<'a>, U: Searcher<'a>> Searcher<'a> for Either<T, U> {
    fn next(&mut self) -> SearchStep {
        match self {
            Either::Left(s) => s.next(),
            Either::Right(s) => s.next(),
        }
    }

    fn next_back(&mut self) -> SearchStep {
        match self {
            Either::Left(s) => s.next_back(),
            Either::Right(s) => s.next_back(),
        }
    }

    fn next_match(&mut self) -> Option<(usize, usize)> {
        match self {
            Either::Left(s) => s.next_match(),
            Either::Right(s) => s.next_match(),
        }
    }

    fn next_match_back(&mut self) -> Option<(usize, usize)> {
        match self {
            Either::Left(s) => s.next_match_back(),
            Either::Right(s) => s.next_match_back(),
        }
    }

    fn next_reject(&mut self) -> Option<(usize, usize)> {
        match self {
            Either::Left(s) => s.next_reject(),
            Either::Right(s) => s.next_reject(),
        }
    }

    fn next_reject_back(&mut self) -> Option<(usize, usize)> {
        match self {
            Either::Left(s) => s.next_reject_back(),
            Either::Right(s) => s.next_reject_back(),
        }
    }
}

pub struct EmptySearcher {
    range: core::ops::Range<usize>,
}

impl EmptySearcher {
    // The empty needle matches on every char boundary.
    fn new(len: usize) -> Self {
        Self {
            range: 0..(len + 1),
        }
    }
}

impl Searcher<'_> for EmptySearcher {
    fn next(&mut self) -> SearchStep {
        match self.range.next() {
            Some(i) => SearchStep::Match(i, i),
            None => SearchStep::Done,
        }
    }

    fn next_back(&mut self) -> SearchStep {
        match self.range.next_back() {
            Some(i) => SearchStep::Match(i, i),
            None => SearchStep::Done,
        }
    }
}

pub struct PredSearcher<'a, T, P> {
    haystack: &'a [T],
    predicate: P,
    front: usize,
}

pub trait Predicate<T> {
    fn matches(&mut self, c: T) -> bool;
}

impl<T: Copy + Eq> Predicate<T> for T {
    fn matches(&mut self, c: T) -> bool {
        *self == c
    }
}

pub struct AnyOf<'a, T>(&'a [T]);

impl<T: Copy, U: Copy + Eq + TryFrom<T>> Predicate<T> for AnyOf<'_, U> {
    fn matches(&mut self, c: T) -> bool {
        self.0.iter().any(|m| U::try_from(c).ok() == Some(*m))
    }
}

pub struct FnPred<F>(F);

impl<T: Into<u16>, F: FnMut(u16) -> bool> Predicate<T> for FnPred<F> {
    fn matches(&mut self, c: T) -> bool {
        (self.0)(c.into())
    }
}

impl<'a, T: Copy, P: Predicate<T>> PredSearcher<'a, T, P> {
    #[inline]
    fn new(can_match: bool, haystack: &'a [T], predicate: P) -> Self {
        Self {
            haystack,
            predicate,
            front: if can_match { 0 } else { haystack.len() },
        }
    }
}

impl<'a, T: Copy, M: Predicate<T>> Searcher<'a> for PredSearcher<'a, T, M> {
    fn next(&mut self) -> SearchStep {
        let c = match self.haystack.get(self.front) {
            None => return SearchStep::Done,
            Some(c) => *c,
        };

        let i = self.front;
        self.front += 1;
        if self.predicate.matches(c) {
            SearchStep::Match(i, i + 1)
        } else {
            SearchStep::Reject(i, i + 1)
        }
    }

    fn next_back(&mut self) -> SearchStep {
        let len = self.haystack.len();
        if self.front >= len {
            return SearchStep::Done;
        }
        let c = self.haystack[len - 1];
        self.haystack = &self.haystack[..len - 1];
        if self.predicate.matches(c) {
            SearchStep::Match(len - 1, len)
        } else {
            SearchStep::Reject(len - 1, len)
        }
    }
}

pub struct SliceSearcher<'a, T> {
    haystack: &'a [T],
    needle: &'a [T],
    front: usize,
    back: usize,
}

impl<'a, T> SliceSearcher<'a, T> {
    fn new(haystack: &'a [T], needle: &'a [T]) -> Self {
        debug_assert!(!needle.is_empty());
        let (front, back) = match haystack.len().checked_sub(needle.len()) {
            Some(i) => (0, i),
            None => (1, 0),
        };
        Self {
            haystack,
            needle,
            front,
            back,
        }
    }
}

impl<'a, T: Eq> Searcher<'a> for SliceSearcher<'a, T> {
    fn next(&mut self) -> SearchStep {
        if self.front > self.back {
            return SearchStep::Done;
        }

        let start = self.front;
        let end = self.front + self.needle.len();
        if &self.haystack[start..end] == self.needle {
            self.front = end;
            SearchStep::Match(start, end)
        } else {
            self.front += 1;
            SearchStep::Reject(start, start + 1)
        }
    }

    fn next_back(&mut self) -> SearchStep {
        if self.front > self.back {
            return SearchStep::Done;
        }

        let start = self.back;
        let end = self.back + self.needle.len();
        if &self.haystack[start..end] == self.needle {
            if let Some(back) = start.checked_sub(self.needle.len()) {
                self.back = back;
            } else {
                self.front = 1;
                self.back = 0;
            }
            SearchStep::Match(start, end)
        } else {
            if self.back == 0 {
                self.front = 1;
            } else {
                self.back -= 1;
            }
            SearchStep::Reject(end - 1, end)
        }
    }
}

pub struct StrSearcher<'a> {
    haystack: &'a WStr,
    needle: &'a WStr,
    front: usize,
    back: usize,
}

impl<'a> StrSearcher<'a> {
    fn new(haystack: &'a WStr, needle: &'a WStr) -> Self {
        debug_assert!(!needle.is_empty());
        let (front, back) = match haystack.len().checked_sub(needle.len()) {
            Some(i) => (0, i),
            None => (1, 0),
        };
        Self {
            haystack,
            needle,
            front,
            back,
        }
    }
}

impl<'a> Searcher<'a> for StrSearcher<'a> {
    fn next(&mut self) -> SearchStep {
        if self.front > self.back {
            return SearchStep::Done;
        }

        let start = self.front;
        let end = self.front + self.needle.len();
        if &self.haystack[start..end] == self.needle {
            self.front = end;
            SearchStep::Match(start, end)
        } else {
            self.front += 1;
            SearchStep::Reject(start, start + 1)
        }
    }

    fn next_back(&mut self) -> SearchStep {
        if self.front > self.back {
            return SearchStep::Done;
        }

        let start = self.back;
        let end = start + self.needle.len();
        if &self.haystack[start..end] == self.needle {
            if let Some(back) = start.checked_sub(self.needle.len()) {
                self.back = back;
            } else {
                self.front = 1;
                self.back = 0;
            }
            SearchStep::Match(start, end)
        } else {
            if self.back == 0 {
                self.front = 1;
            } else {
                self.back -= 1;
            }
            SearchStep::Reject(end - 1, end)
        }
    }
}
