use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{Loader, static_loader};
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
    let text = TEXTS
        .try_lookup(locale, id)
        .map(Cow::Owned)
        .unwrap_or_else(|| {
            tracing::error!("Unknown desktop text id '{id}'");
            Cow::Owned(id.to_string())
        });
    reorder_bidi(locale, text)
}

pub fn optional_text(locale: &LanguageIdentifier, id: &str) -> Option<String> {
    TEXTS
        .lookup_single_language::<&str>(locale, id, None)
        .inspect_err(|e| tracing::trace!("Error looking up text: {e}"))
        .ok()
        .map(|s| reorder_bidi(locale, s.into()).into_owned())
}

pub fn available_languages() -> Vec<&'static LanguageIdentifier> {
    let mut result: Vec<_> = TEXTS.locales().collect();
    result.sort();
    result
}

pub fn text_with_args(
    locale: &LanguageIdentifier,
    id: &'static str,
    args: &HashMap<Cow<'static, str>, FluentValue>,
) -> Cow<'static, str> {
    let text = TEXTS
        .try_lookup_with_args(locale, id, args)
        .map(Cow::Owned)
        .unwrap_or_else(|| {
            tracing::error!("Unknown desktop text id '{id}'");
            Cow::Borrowed(id)
        });
    reorder_bidi(locale, text)
}

/// Reorder BiDi text so that RTL text runs are reversed.
/// The default direction is based on the locale.
///
/// TODO This is stupid, but it at least allows people to read RTL languages
///   somewhat correctly. Remove it when egui starts supporting RTL scripts.
fn reorder_bidi(locale: &LanguageIdentifier, text: Cow<'static, str>) -> Cow<'static, str> {
    let level = if locale.character_direction() == unic_langid::CharacterDirection::RTL {
        unicode_bidi::Level::rtl()
    } else {
        unicode_bidi::Level::ltr()
    };

    let bidi_info = unicode_bidi::BidiInfo::new(&text, Some(level));
    if !unicode_bidi::level::has_rtl(&bidi_info.levels) {
        // Fast path: no RTL text = no reordering
        return text;
    }

    // At this point we know something has to be reversed.
    let mut reordered_text = String::with_capacity(text.len());
    for para in &bidi_info.paragraphs {
        let (levels, runs) = bidi_info.visual_runs(para, para.range.clone());
        for run in runs {
            if levels[run.start].is_rtl() {
                reordered_text.extend(text[run].chars().rev().map(mirror_char));
            } else {
                reordered_text.push_str(&text[run]);
            }
        }
    }

    reordered_text.into()
}

/// Mirror the given character.
///
/// It's used as a very simple alternative to glyph mirroring, which we do not
/// support yet and allows basic characters to be rendered properly in RTL.
fn mirror_char(c: char) -> char {
    match c {
        '(' => ')',
        ')' => '(',
        '[' => ']',
        ']' => '[',
        '{' => '}',
        '}' => '{',
        '<' => '>',
        '>' => '<',
        _ => c,
    }
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
