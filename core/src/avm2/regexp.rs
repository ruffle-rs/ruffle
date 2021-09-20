//! RegExp Structure

use std::borrow::Cow;

use crate::string::{AvmString, Units, WStrToUtf8};
use bitflags::bitflags;
use gc_arena::Collect;

#[derive(Collect, Debug)]
#[collect(no_drop)]
pub struct RegExp<'gc> {
    source: AvmString<'gc>,
    flags: RegExpFlags,
    last_index: usize,

    #[collect(require_static)]
    cached_regex: Option<Result<regress::Regex, ()>>,
    cached_text: Option<CachedText<'gc>>,
}

impl<'gc> Clone for RegExp<'gc> {
    fn clone(&self) -> Self {
        Self {
            source: self.source,
            flags: self.flags,
            last_index: self.last_index,
            cached_regex: None,
            cached_text: None,
        }
    }
}

bitflags! {
    #[derive(Collect)]
    #[collect(require_static)]
    pub struct RegExpFlags: u8 {
        const GLOBAL       = 1 << 0;
        const IGNORE_CASE  = 1 << 1;
        const MULTILINE    = 1 << 2;
        const DOTALL       = 1 << 3;
        const EXTENDED     = 1 << 4;
    }
}

impl<'gc> RegExp<'gc> {
    pub fn new<S>(source: S) -> Self
    where
        S: Into<AvmString<'gc>>,
    {
        Self {
            source: source.into(),
            flags: RegExpFlags::empty(),
            last_index: 0,
            cached_regex: None,
            cached_text: None,
        }
    }

    pub fn source(&self) -> AvmString<'gc> {
        self.source
    }

    pub fn set_source<S>(&mut self, source: S)
    where
        S: Into<AvmString<'gc>>,
    {
        self.cached_regex = None;
        self.source = source.into();
    }

    pub fn flags(&self) -> RegExpFlags {
        self.flags
    }

    pub fn set_flags(&mut self, flags: RegExpFlags) {
        self.cached_regex = None;
        self.flags = flags;
    }

    pub fn last_index(&self) -> usize {
        self.last_index
    }

    pub fn set_last_index(&mut self, i: usize) {
        self.last_index = i;
    }

    fn find_utf8_match_at<T, F>(&mut self, text: AvmString<'gc>, start: usize, f: F) -> Option<T>
    where
        F: FnOnce(&mut CachedText<'gc>, regress::Match) -> T,
    {
        if self.cached_regex.is_none() {
            let re = regress::Regex::with_flags(
                &self.source.to_utf8_lossy(),
                regress::Flags {
                    icase: self.flags.contains(RegExpFlags::IGNORE_CASE),
                    multiline: self.flags.contains(RegExpFlags::MULTILINE),
                    dot_all: self.flags.contains(RegExpFlags::DOTALL),
                    no_opt: false,
                },
            );
            self.cached_regex = Some(re.map_err(drop));
        }

        let regex = match self.cached_regex.as_mut() {
            Some(Ok(re)) => re,
            Some(Err(_)) => return None,
            None => unreachable!(),
        };

        let cached = self
            .cached_text
            .as_ref()
            .filter(|cached| AvmString::ptr_eq(&cached.text, &text))
            .is_some();
        if !cached {
            self.cached_text = Some(CachedText::new(text));
        }
        let text = self.cached_text.as_mut().unwrap();

        let start = text.utf8_index(start)?;
        let re_match = regex.find_from(text.utf8(), start).next()?;
        Some(f(text, re_match))
    }

    pub fn test(&mut self, text: AvmString<'gc>) -> bool {
        let global = self.flags.contains(RegExpFlags::GLOBAL);
        let start = if global { self.last_index } else { 0 };
        let matched_idx = self.find_utf8_match_at(text, start, |text, re_match| {
            if global {
                text.utf16_index(re_match.end())
            } else {
                None
            }
        });

        match matched_idx {
            Some(Some(idx)) => {
                self.last_index = idx;
                true
            }
            Some(None) => true,
            None => false,
        }
    }

    pub fn exec(&mut self, text: AvmString<'gc>) -> Option<regress::Match> {
        let global = self.flags.contains(RegExpFlags::GLOBAL);
        let start = if global { self.last_index } else { 0 };
        let re_match = self.find_utf8_match_at(text, start, |text, mut re_match| {
            // Sort the capture endpoints by increasing index, so that CachedText::utf16_index is efficient.
            let mut utf8_indices = re_match
                .captures
                .iter_mut()
                .filter_map(Option::as_mut)
                .chain(std::iter::once(&mut re_match.range))
                .flat_map(|capture| [&mut capture.start, &mut capture.end])
                .collect::<Vec<_>>();
            utf8_indices.sort_by_key(|i| **i);

            // Map UTF8 indices back to UTF16.
            for i in utf8_indices {
                *i = text.utf16_index(*i).unwrap();
            }

            re_match
        })?;

        if global {
            self.last_index = re_match.end();
        }

        Some(re_match)
    }
}

#[derive(Collect, Debug)]
#[collect(no_drop)]
struct CachedText<'gc> {
    text: AvmString<'gc>,
    // None means that `text` is already a valid utf8 string.
    utf8: Option<String>,
    utf8_prefix_len: usize,

    // Cached values of the last `{utf8, utf16}_index` call,
    // to avoid unnecessary recomputation when calling these methods
    // with increasing indices.
    cur_utf8_index: usize,
    cur_utf16_index: usize,
}

impl<'gc> CachedText<'gc> {
    fn new(text: AvmString<'gc>) -> Self {
        let to_utf8 = WStrToUtf8::new(text.as_ucs2());
        let utf8 = to_utf8.to_utf8_lossy();
        let utf8_prefix_len = if utf8.len() == text.len() {
            // Identical len means the string is fully utf8,
            // even if `utf8_prefix` is empty.
            text.len()
        } else {
            to_utf8.prefix().len()
        };

        Self {
            text,
            utf8: match utf8 {
                Cow::Owned(s) => Some(s),
                Cow::Borrowed(_) => None,
            },
            utf8_prefix_len,
            cur_utf8_index: utf8_prefix_len,
            cur_utf16_index: utf8_prefix_len,
        }
    }

    fn utf8(&self) -> &str {
        self.utf8
            .as_deref()
            .unwrap_or_else(|| match self.text.units() {
                // SAFETY: because `self.utf8` is None, we know `text` contains
                // a valid UTF8 string.
                Units::Bytes(s) => unsafe { std::str::from_utf8_unchecked(s) },
                _ => unreachable!(),
            })
    }

    fn reset(&mut self) {
        self.cur_utf8_index = self.utf8_prefix_len;
        self.cur_utf16_index = self.utf8_prefix_len;
    }

    fn advance(&mut self) -> Option<()> {
        let c = self.utf8()[self.cur_utf8_index..].chars().next()?;
        self.cur_utf8_index += c.len_utf8();
        self.cur_utf16_index += c.len_utf16();
        Some(())
    }

    /// Returns the UTF8 index corresponding to the given UTF16 index.
    ///
    /// If `utf16_index` is out of bounds, return `None`.
    /// If `utf16_index` isn't on a char boundary, return the index
    /// of the next char.
    fn utf8_index(&mut self, utf16_index: usize) -> Option<usize> {
        if utf16_index <= self.utf8_prefix_len {
            return Some(utf16_index);
        }

        if utf16_index < self.cur_utf16_index {
            self.reset();
        }

        while self.cur_utf16_index < utf16_index {
            self.advance()?;
        }

        Some(self.cur_utf8_index)
    }

    /// Returns the UTF16 index corresponding to the given UTF8 index.
    ///
    /// If `utf8_index` is out of bounds, return `None`.
    /// If `utf8_index` isn't on a char boundary, return the index
    /// of the next char.
    fn utf16_index(&mut self, utf8_index: usize) -> Option<usize> {
        if utf8_index <= self.utf8_prefix_len {
            return Some(utf8_index);
        }

        if utf8_index < self.cur_utf8_index {
            self.reset();
        }

        while self.cur_utf8_index < utf8_index {
            self.advance()?;
        }

        Some(self.cur_utf16_index)
    }
}
