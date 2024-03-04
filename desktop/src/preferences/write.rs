use crate::preferences::PreferencesAndDocument;
use ruffle_render_wgpu::clap::{GraphicsBackend, PowerPreference};
use toml_edit::value;
use unic_langid::LanguageIdentifier;

pub struct PreferencesWriter<'a>(&'a mut PreferencesAndDocument);

impl<'a> PreferencesWriter<'a> {
    pub(super) fn new(preferences: &'a mut PreferencesAndDocument) -> Self {
        Self(preferences)
    }

    pub fn set_graphics_backend(&mut self, backend: GraphicsBackend) {
        self.0.toml_document["graphics_backend"] = value(backend.as_str());
        self.0.values.graphics_backend = backend;
    }

    pub fn set_graphics_power_preference(&mut self, preference: PowerPreference) {
        self.0.toml_document["graphics_power_preference"] = value(preference.as_str());
        self.0.values.graphics_power_preference = preference;
    }

    pub fn set_language(&mut self, language: LanguageIdentifier) {
        self.0.toml_document["language"] = value(language.to_string());
        self.0.values.language = language;
    }

    pub fn set_output_device(&mut self, name: Option<String>) {
        if let Some(name) = &name {
            self.0.toml_document["output_device"] = value(name);
        } else {
            self.0.toml_document.remove("output_device");
        }
        self.0.values.output_device = name;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preferences::read::read_preferences;
    use fluent_templates::loader::langid;

    fn parse(input: &str) -> PreferencesAndDocument {
        let (result, document) = read_preferences(input);
        PreferencesAndDocument {
            toml_document: document,
            values: result.result,
        }
    }

    fn check_roundtrip(preferences: &mut PreferencesAndDocument) {
        let read_result = read_preferences(&preferences.toml_document.to_string());
        assert_eq!(
            preferences.values, read_result.0.result,
            "roundtrip failed: expected != actual"
        );
    }

    fn test(original: &str, fun: impl FnOnce(&mut PreferencesWriter), expected: &str) {
        let mut preferences = parse(original);
        let mut writer = PreferencesWriter::new(&mut preferences);
        fun(&mut writer);
        check_roundtrip(&mut preferences);
        assert_eq!(expected, preferences.toml_document.to_string());
    }

    #[test]
    fn set_graphics_backend() {
        test(
            "",
            |writer| writer.set_graphics_backend(GraphicsBackend::Default),
            "graphics_backend = \"default\"\n",
        );

        test(
            "graphics_backend = \"fast\"",
            |writer| writer.set_graphics_backend(GraphicsBackend::Vulkan),
            "graphics_backend = \"vulkan\"\n",
        );
    }

    #[test]
    fn set_graphics_power_preference() {
        test(
            "",
            |writer| writer.set_graphics_power_preference(PowerPreference::High),
            "graphics_power_preference = \"high\"\n",
        );

        test(
            "graphics_power_preference = \"fast\"",
            |writer| writer.set_graphics_power_preference(PowerPreference::Low),
            "graphics_power_preference = \"low\"\n",
        );
    }

    #[test]
    fn set_language() {
        test(
            "",
            |writer| writer.set_language(langid!("en-US")),
            "language = \"en-US\"\n",
        );

        test(
            "language = \"???\"",
            |writer| writer.set_language(langid!("en-Latn-US-valencia")),
            "language = \"en-Latn-US-valencia\"\n",
        );
    }

    #[test]
    fn set_output_device() {
        test(
            "",
            |writer| writer.set_output_device(Some("Speakers".to_string())),
            "output_device = \"Speakers\"\n",
        );

        test(
            "output_device = \"Speakers\"",
            |writer| writer.set_output_device(None),
            "",
        );
    }
}
