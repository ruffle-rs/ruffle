use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{static_loader, Loader};
use std::borrow::Cow;
use std::collections::HashMap;
use unic_langid::LanguageIdentifier;

static_loader! {
    static TEXTS = {
        locales: "./assets/texts",
        fallback_language: "en-US"
    };
}

pub fn text<'a>(locale: &LanguageIdentifier, id: &'a str) -> Cow<'a, str> {
    TEXTS
        .try_lookup(locale, id)
        .map(Cow::Owned)
        .unwrap_or_else(|| {
            tracing::error!("Unknown desktop text id '{id}'");
            Cow::Borrowed(id)
        })
}

pub fn optional_text(locale: &LanguageIdentifier, id: &str) -> Option<String> {
    TEXTS
        .lookup_single_language::<&str>(locale, id, None)
        .inspect_err(|e| tracing::trace!("Error looking up text: {e}"))
        .ok()
}

pub fn available_languages() -> Vec<&'static LanguageIdentifier> {
    let mut result: Vec<_> = TEXTS.locales().collect();
    result.sort();
    result
}

#[allow(dead_code)]
pub fn text_with_args(
    locale: &LanguageIdentifier,
    id: &'static str,
    args: &HashMap<Cow<'static, str>, FluentValue>,
) -> Cow<'static, str> {
    TEXTS
        .try_lookup_with_args(locale, id, args)
        .map(Cow::Owned)
        .unwrap_or_else(|| {
            tracing::error!("Unknown desktop text id '{id}'");
            Cow::Borrowed(id)
        })
}

pub enum LocalizableText {
    NonLocalizedText(Cow<'static, str>),
    LocalizedText(&'static str),
}

impl LocalizableText {
    pub fn localize(&self, locale: &LanguageIdentifier) -> Cow<'_, str> {
        match self {
            LocalizableText::NonLocalizedText(Cow::Borrowed(text)) => Cow::Borrowed(text),
            LocalizableText::NonLocalizedText(Cow::Owned(text)) => Cow::Borrowed(text),
            LocalizableText::LocalizedText(id) => text(locale, id),
        }
    }
}
