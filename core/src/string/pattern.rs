//! Like [`std::str::Pattern`], but for [`WStr`].

// TODO: Is performance good? ideas for improvements:
//  - add some inlines?
//  - remove implicit bound checks?
//  - use memchr crate?

use super::{WStr, Units};

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

    fn into_searcher(self, haystack: WStr<'a>) -> Self::Searcher;
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

    fn into_searcher(self, haystack: WStr<'a>) -> Self::Searcher {
        match haystack.units() {
            Units::Bytes(h) => Either::Left(PredSearcher::new(true, h, self)),
            Units::Wide(h) => Either::Right(PredSearcher::new(true, h, self.into())),
        }
    }
}

impl<'a> Pattern<'a> for u16 {
    type Searcher = Either<PredSearcher<'a, u8, u8>, PredSearcher<'a, u16, u16>>;

    fn into_searcher(self, haystack: WStr<'a>) -> Self::Searcher {
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

    fn into_searcher(self, haystack: WStr<'a>) -> Self::Searcher {
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

    fn into_searcher(self, haystack: WStr<'a>) -> Self::Searcher {
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

    fn into_searcher(self, haystack: WStr<'a>) -> Self::Searcher {
        match haystack.units() {
            Units::Bytes(h) => Either::Left(PredSearcher::new(true, h, FnPred(self))),
            Units::Wide(h) => Either::Right(PredSearcher::new(true, h, FnPred(self))),
        }
    }
}

impl<'a> Pattern<'a> for WStr<'a> {
    #[allow(clippy::type_complexity)]
    type Searcher = Either<
        Either<Either<SliceSearcher<'a, u8>, SliceSearcher<'a, u16>>, StrSearcher<'a>>,
        EmptySearcher,
    >;

    fn into_searcher(self, haystack: WStr<'a>) -> Self::Searcher {
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
    range: std::ops::Range<usize>,
}

impl EmptySearcher {
    // The empty needle matches on every char boundary.
    fn new(len: usize) -> Self {
        Self {
            range: 0..(len + 1),
        }
    }
}

impl<'a> Searcher<'a> for EmptySearcher {
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
    fn is_match(&mut self, c: T) -> bool;
}

impl<T: Copy + Eq> Predicate<T> for T {
    fn is_match(&mut self, c: T) -> bool {
        *self == c
    }
}

pub struct AnyOf<'a, T>(&'a [T]);

impl<'a, T: Copy, U: Copy + Eq + TryFrom<T>> Predicate<T> for AnyOf<'a, U> {
    fn is_match(&mut self, c: T) -> bool {
        self.0.iter().any(|m| U::try_from(c).ok() == Some(*m))
    }
}

pub struct FnPred<F>(F);

impl<'a, T: Into<u16>, F: FnMut(u16) -> bool> Predicate<T> for FnPred<F> {
    fn is_match(&mut self, c: T) -> bool {
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
        if self.predicate.is_match(c) {
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
        if self.predicate.is_match(c) {
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
    haystack: WStr<'a>,
    needle: WStr<'a>,
    front: usize,
    back: usize,
}

impl<'a> StrSearcher<'a> {
    fn new(haystack: WStr<'a>, needle: WStr<'a>) -> Self {
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
        if self.haystack.slice(start..end) == self.needle {
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
        if self.haystack.slice(start..end) == self.needle {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;

    macro_rules! bstr {
        ($str:literal) => {
            WStr::from_units($str)
        };
    }

    macro_rules! wstr {
        ($($char:literal)*) => {
            WStr::from_units(&[$($char as u16),*])
        }
    }

    fn test_pattern<'a, P: Pattern<'a> + Clone + Debug>(
        haystack: WStr<'a>,
        pattern: P,
        forwards: &[(usize, usize)],
        backwards: Option<&[(usize, usize)]>,
    ) {
        let mut searcher = pattern.clone().into_searcher(haystack);
        let mut actual: Vec<_> = std::iter::from_fn(|| searcher.next_match()).collect();
        assert_eq!(
            actual, forwards,
            "incorrect forwards matching: haystack={:?}; pattern={:?}",
            haystack, pattern
        );

        searcher = pattern.clone().into_searcher(haystack);
        actual = std::iter::from_fn(|| searcher.next_match_back()).collect();
        actual.reverse();
        assert_eq!(
            actual,
            backwards.unwrap_or(forwards),
            "incorrect backwards matching: haystack={:?}; pattern={:?}",
            haystack,
            pattern
        );
    }

    #[test]
    fn char_patterns() {
        test_pattern(bstr!(b"a"), b'a', &[(0, 1)], None);

        let bytes = bstr!(b"abaabbcab");
        test_pattern(bytes, b'b', &[(1, 2), (4, 5), (5, 6), (8, 9)], None);
        test_pattern(bytes, b'd', &[], None);
        test_pattern(bytes, 'c' as u16, &[(6, 7)], None);
        test_pattern(bytes, '↓' as u16, &[], None);

        let wide = wstr!('↓''a''a''↓''a');
        test_pattern(wide, b'c', &[], None);
        test_pattern(wide, '↓' as u16, &[(0, 1), (3, 4)], None);
    }

    #[test]
    fn multi_char_patterns() {
        let bytes = bstr!(b"abcdabcd");
        let matches = &[(0, 1), (2, 3), (4, 5), (6, 7)];
        test_pattern(bytes, &[b'a', b'c'][..], matches, None);
        test_pattern(bytes, &['a' as u16, 'c' as u16][..], matches, None);

        let wide = wstr!('↓''a''b''↓''b''c');
        test_pattern(wide, &[b'a', b'b'][..], &[(1, 2), (2, 3), (4, 5)], None);
        test_pattern(wide, &['↓' as u16, '−' as u16][..], &[(0, 1), (3, 4)], None);

        // Don't test `FnMut(u16) -> bool` because it isn't `Debug`
    }

    #[test]
    fn str_patterns() {
        test_pattern(bstr!(b"aa"), bstr!(b""), &[(0, 0), (1, 1), (2, 2)], None);
        test_pattern(bstr!(b"abcde"), bstr!(b"abcde"), &[(0, 5)], None);

        let bytes = bstr!(b"bbabbbabbbba");
        let matches = &[(0, 2), (3, 5), (7, 9), (9, 11)];
        let matches_rev = &[(0, 2), (4, 6), (7, 9), (9, 11)];
        test_pattern(bytes, bstr!(b"bb"), matches, Some(matches_rev));
        test_pattern(bytes, wstr!('b''b'), matches, Some(matches_rev));

        let wide = wstr!('↓''↓''a''a''↓''↓''a''a''↓''↓');
        test_pattern(wide, bstr!(b"aa"), &[(2, 4), (6, 8)], None);
        test_pattern(wide, wstr!('↓''a'), &[(1, 3), (5, 7)], None);
    }
}
