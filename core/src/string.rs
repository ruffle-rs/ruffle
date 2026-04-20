use std::borrow::Cow;

pub use ruffle_wstr::*;

pub use ruffle_common::avm_string::*;

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
