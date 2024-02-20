use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{static_loader, LanguageIdentifier, Loader};
use std::collections::HashMap;

static_loader! {
    static TEXTS = {
        locales: "./assets/texts",
        fallback_language: "en-US"
    };
}

pub fn core_text(language: &LanguageIdentifier, id: &str) -> String {
    TEXTS.try_lookup(language, id).unwrap_or_else(|| {
        tracing::error!("Unknown core text id '{id}'");
        id.to_string()
    })
}

pub fn core_text_with_args<T: AsRef<str>>(
    language: &LanguageIdentifier,
    id: &str,
    args: &HashMap<T, FluentValue>,
) -> String {
    TEXTS
        .try_lookup_with_args(language, id, args)
        .unwrap_or_else(|| {
            tracing::error!("Unknown core text id '{id}'");
            id.to_string()
        })
}
