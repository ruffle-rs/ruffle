//! RegExp Structure

use crate::string::AvmString;
use bitflags::bitflags;
use gc_arena::Collect;
use regress::Regex;

#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct RegExp<'gc> {
    source: AvmString<'gc>,
    flags: RegExpFlags,
    last_index: usize,
}

bitflags! {
    #[derive(Collect)]
    #[collect(require_static)]
    struct RegExpFlags: u8 {
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
        }
    }

    pub fn source(&self) -> AvmString<'gc> {
        self.source
    }

    pub fn set_source<S>(&mut self, source: S)
    where
        S: Into<AvmString<'gc>>,
    {
        self.source = source.into();
    }

    pub fn last_index(&self) -> usize {
        self.last_index
    }

    pub fn set_last_index(&mut self, i: usize) {
        self.last_index = i;
    }

    pub fn dotall(&self) -> bool {
        self.flags.contains(RegExpFlags::DOTALL)
    }

    pub fn set_dotall(&mut self, value: bool) {
        self.flags.set(RegExpFlags::DOTALL, value);
    }

    pub fn extended(&self) -> bool {
        self.flags.contains(RegExpFlags::EXTENDED)
    }

    pub fn set_extended(&mut self, value: bool) {
        self.flags.set(RegExpFlags::EXTENDED, value);
    }

    pub fn global(&self) -> bool {
        self.flags.contains(RegExpFlags::GLOBAL)
    }

    pub fn set_global(&mut self, value: bool) {
        self.flags.set(RegExpFlags::GLOBAL, value);
    }

    pub fn ignore_case(&self) -> bool {
        self.flags.contains(RegExpFlags::IGNORE_CASE)
    }

    pub fn set_ignore_case(&mut self, value: bool) {
        self.flags.set(RegExpFlags::IGNORE_CASE, value);
    }

    pub fn multiline(&self) -> bool {
        self.flags.contains(RegExpFlags::MULTILINE)
    }

    pub fn set_multiline(&mut self, value: bool) {
        self.flags.set(RegExpFlags::MULTILINE, value);
    }

    pub fn test(&mut self, text: &str) -> bool {
        self.exec(text).is_some()
    }

    pub fn exec(&mut self, text: &str) -> Option<regress::Match> {
        if let Ok(re) = Regex::with_flags(
            &self.source,
            regress::Flags {
                icase: self.ignore_case(),
                multiline: self.multiline(),
                dot_all: self.dotall(),
                no_opt: false,
            },
        ) {
            let start = if self.global() { self.last_index } else { 0 };
            if let Some(matched) = re.find_from(text, start).next() {
                if self.global() {
                    self.last_index = matched.end();
                }

                return Some(matched);
            }
        }

        None
    }
}
