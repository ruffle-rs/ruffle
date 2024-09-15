//! RegExp Structure

use std::borrow::Cow;

use crate::avm2::activation::Activation;
use crate::avm2::object::FunctionObject;
use crate::avm2::object::TObject;
use crate::avm2::Error;
use crate::avm2::{ArrayObject, ArrayStorage, Object, Value};
use crate::string::WString;
use crate::string::{AvmString, Units, WStrToUtf8};
use bitflags::bitflags;
use gc_arena::Collect;
use ruffle_wstr::WStr;

use super::object::RegExpObject;

#[derive(Collect, Debug)]
#[collect(no_drop)]
pub struct RegExp<'gc> {
    source: AvmString<'gc>,
    #[collect(require_static)]
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
    #[derive(Clone, Copy, Debug)]
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
                    unicode: false,
                    unicode_sets: false,
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

    /// Helper for replace_string. Evaluates the special $-sequences
    /// in `replacement`.
    fn effective_replacement<'a>(
        replacement: &'a AvmString<'gc>,
        text: &AvmString<'gc>,
        m: &regress::Match,
    ) -> Cow<'a, WStr> {
        if !replacement.contains(b'$') {
            // Nothing to do if there's no $ replacement symbols
            return Cow::Borrowed(replacement.as_wstr());
        }
        let mut ret = WString::new();
        let s = replacement.as_wstr();
        let mut chars = s.chars().peekable();
        while let Some(Ok(c)) = chars.next() {
            if c != '$' {
                ret.push_char(c);
                continue;
            }
            match chars.next() {
                Some(Ok('$')) => ret.push_char('$'),
                Some(Ok('&')) => ret.push_str(&text[m.range.start..m.range.end]),
                Some(Ok('`')) => ret.push_str(&text[..m.range.start]),
                Some(Ok('\'')) => ret.push_str(&text[m.range.end..]),
                Some(Ok(n)) => {
                    if let Some(d) = n.to_digit(10) {
                        let d_u = usize::try_from(d).unwrap_or(0);
                        if d_u > m.captures.len() {
                            ret.push_char('$');
                            ret.push_char(n);
                            continue;
                        }
                        let mut grp_index = d_u;
                        if let Some(Ok(next_char)) = chars.peek() {
                            if let Some(d1) = next_char.to_digit(10) {
                                let d1_u = usize::try_from(d1).unwrap_or(0);
                                let two_digit_index = d_u * 10 + d1_u;
                                if two_digit_index <= m.captures.len() {
                                    chars.next();
                                    grp_index = two_digit_index
                                }
                            }
                        }
                        if let Some(Some(r)) = m.captures.get(grp_index - 1) {
                            ret.push_str(&text[r.start..r.end])
                        }
                        continue;
                    }

                    ret.push_char('$');
                    ret.push_char(n);
                }
                _ => ret.push_char('$'),
            }
        }
        Cow::Owned(ret)
    }

    /// Implements string.replace(regex, replacement) where the replacement is
    /// a function.
    pub fn replace_fn(
        regexp: RegExpObject<'gc>,
        activation: &mut Activation<'_, 'gc>,
        text: AvmString<'gc>,
        f: &FunctionObject<'gc>,
    ) -> Result<AvmString<'gc>, Error<'gc>> {
        Self::replace_with_fn(regexp, activation, &text, |activation, txt, m| {
            let args = std::iter::once(Some(&m.range))
                .chain((m.captures.iter()).map(|x| x.as_ref()))
                .map(|o| match o {
                    Some(r) => {
                        AvmString::new(activation.context.gc_context, &txt[r.start..r.end]).into()
                    }
                    None => "".into(),
                })
                .chain(std::iter::once(m.range.start.into()))
                .chain(std::iter::once((*txt).into()))
                .collect::<Vec<_>>();
            let r = f.call(Value::Null, &args, activation)?;
            return Ok(Cow::Owned(WString::from(
                r.coerce_to_string(activation)?.as_wstr(),
            )));
        })
    }

    /// Implements string.replace(regex, replacement) where the replacement may be
    /// a string with $-sequences.
    pub fn replace_string(
        regexp: RegExpObject<'gc>,
        activation: &mut Activation<'_, 'gc>,
        text: AvmString<'gc>,
        replacement: AvmString<'gc>,
    ) -> Result<AvmString<'gc>, Error<'gc>> {
        RegExp::replace_with_fn(regexp, activation, &text, |_activation, txt, m| {
            Ok(Self::effective_replacement(&replacement, txt, m))
        })
    }

    // Helper for replace_string and replace_function.
    //
    // Replaces occurrences of regex with results of f(activation, &text, &match)
    // Panics if regexp isn't a regexp
    fn replace_with_fn<'a, F>(
        regexp: RegExpObject<'gc>,
        activation: &mut Activation<'_, 'gc>,
        text: &AvmString<'gc>,
        mut f: F,
    ) -> Result<AvmString<'gc>, Error<'gc>>
    where
        F: FnMut(
            &mut Activation<'_, 'gc>,
            &AvmString<'gc>,
            &regress::Match,
        ) -> Result<Cow<'a, WStr>, Error<'gc>>,
    {
        let mut start = 0;

        let (is_global, mut m) = {
            // we only hold onto a mutable lock on the regular expression
            // for a small window, because f might refer to the RegExp
            // (See https://github.com/ruffle-rs/ruffle/issues/17899)
            let mut re = regexp.as_regexp_mut(activation.gc()).unwrap();
            let global_flag = re.flags().contains(RegExpFlags::GLOBAL);

            (global_flag, re.find_utf16_match(*text, start))
        };
        if m.is_none() {
            // Nothing to do; short circuit and just return the original string, to avoid any allocs or functions
            return Ok(*text);
        }

        let mut ret = WString::new();
        while let Some(segment) = m {
            ret.push_str(&text[start..segment.range.start]);
            ret.push_str(&f(activation, text, &segment)?);

            start = segment.range.end;

            if segment.range.is_empty() {
                if start == text.len() {
                    break;
                }
                ret.push_str(&text[start..start + 1]);
                start += 1;
            }

            if !is_global {
                break;
            }
            // Again, here we only hold onto a mutable lock for
            // the RegExp long enough to do our matching, so that
            // when we call f we don't have a lock
            m = regexp
                .as_regexp_mut(activation.gc())
                .unwrap()
                .find_utf16_match(*text, start);
        }

        ret.push_str(&text[start..]);
        Ok(AvmString::new(activation.context.gc_context, ret))
    }

    pub fn split(
        &mut self,
        activation: &mut Activation<'_, 'gc>,
        text: AvmString<'gc>,
        limit: usize,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let mut storage = ArrayStorage::new(0);
        // The empty regex is a special case which splits into characters.
        if self.source.is_empty() {
            let mut it = text.chars().take(limit);
            while let Some(Ok(c)) = it.next() {
                storage.push(
                    AvmString::new(activation.context.gc_context, WString::from_char(c)).into(),
                );
            }
            return ArrayObject::from_storage(activation, storage);
        }

        let mut start = 0;
        while let Some(m) = self.find_utf16_match(text, start) {
            if m.range.end == start {
                break;
            }
            storage.push(
                AvmString::new(activation.context.gc_context, &text[start..m.range.start]).into(),
            );
            if storage.length() >= limit {
                break;
            }
            for c in m.captures.iter().filter_map(Option::as_ref) {
                storage.push(
                    AvmString::new(activation.context.gc_context, &text[c.start..c.end]).into(),
                );
                if storage.length() >= limit {
                    break; // Intentional bug to match Flash.
                           // Causes adding parts past limit.
                }
            }

            start = m.range.end;
        }
        if storage.length() < limit {
            storage.push(AvmString::new(activation.context.gc_context, &text[start..]).into());
        }
        ArrayObject::from_storage(activation, storage)
    }

    pub fn find_utf16_match(
        &mut self,
        text: AvmString<'gc>,
        start: usize,
    ) -> Option<regress::Match> {
        self.find_utf8_match_at(text, start, |text, mut re_match| {
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
        })
    }
    pub fn exec(&mut self, text: AvmString<'gc>) -> Option<regress::Match> {
        let global = self.flags.contains(RegExpFlags::GLOBAL);
        let start = if global { self.last_index } else { 0 };
        let re_match = self.find_utf16_match(text, start)?;
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
    // TODO WStrToUtf8 implements UTF-8/UTF-16 index mapping, merge it if possible
    cur_utf8_index: usize,
    cur_utf16_index: usize,
}

impl<'gc> CachedText<'gc> {
    fn new(text: AvmString<'gc>) -> Self {
        let to_utf8 = WStrToUtf8::new(&text);
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
                // NOTES: The only case where a wide string could be valid UTF8 is if it's empty.
                Units::Wide([]) => "",
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
