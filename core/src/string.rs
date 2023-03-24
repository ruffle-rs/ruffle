use std::borrow::{Borrow, Cow};
use std::ops::Deref;

use gc_arena::Collect;

mod avm_string;
mod interner;

pub use ruffle_wstr::*;

pub use avm_string::AvmString;
pub use interner::AvmStringInterner;

pub trait SwfStrExt {
    /// Converts a SWF-encoded string into a `WStr`.
    fn decode(&self, encoding: &'static swf::Encoding) -> Cow<'_, WStr>;
}

impl SwfStrExt for swf::SwfStr {
    fn decode(&self, encoding: &'static swf::Encoding) -> Cow<'_, WStr> {
        match self.to_str_lossy(encoding) {
            Cow::Borrowed(utf8) => from_utf8(utf8),
            Cow::Owned(utf8) => WString::from_utf8_owned(utf8).into(),
        }
    }
}

/// This type only exists because `WString` doesn't implement `Collect`
#[derive(Collect, Eq, PartialEq, Hash)]
#[collect(require_static)]
struct OwnedWStr(WString);

impl Deref for OwnedWStr {
    type Target = WStr;

    #[inline(always)]
    fn deref(&self) -> &WStr {
        &self.0
    }
}

impl Borrow<WStr> for OwnedWStr {
    #[inline(always)]
    fn borrow(&self) -> &WStr {
        &self.0
    }
}

impl Default for OwnedWStr {
    #[inline(always)]
    fn default() -> Self {
        OwnedWStr(WString::new())
    }
}
