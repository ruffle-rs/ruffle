use crate::cli::{GameModePreference, OpenUrlMode};
use crate::gui::ThemePreference;
use crate::log::FilenamePattern;
use crate::preferences::storage::StorageBackend;
use crate::preferences::{GlobalPreferencesWatchers, SavedGlobalPreferences};
use ruffle_frontend_utils::parse::DocumentHolder;
use ruffle_render_wgpu::clap::{GraphicsBackend, PowerPreference};
use toml_edit::value;
use unic_langid::LanguageIdentifier;

pub struct PreferencesWriter<'a>(
    &'a mut DocumentHolder<SavedGlobalPreferences>,
    Option<&'a GlobalPreferencesWatchers>,
);

impl<'a> PreferencesWriter<'a> {
    pub(super) fn new(preferences: &'a mut DocumentHolder<SavedGlobalPreferences>) -> Self {
        Self(preferences, None)
    }

    pub(super) fn set_watchers(&mut self, watchers: &'a GlobalPreferencesWatchers) {
        self.1 = Some(watchers);
    }

    pub fn set_graphics_backend(&mut self, backend: GraphicsBackend) {
        self.0.edit(|values, toml_document| {
            toml_document["graphics_backend"] = value(backend.as_str());
            values.graphics_backend = backend;
        })
    }

    pub fn set_graphics_power_preference(&mut self, preference: PowerPreference) {
        self.0.edit(|values, toml_document| {
            toml_document["graphics_power_preference"] = value(preference.as_str());
            values.graphics_power_preference = preference;
        })
    }

    pub fn set_language(&mut self, language: LanguageIdentifier) {
        self.0.edit(|values, toml_document| {
            toml_document["language"] = value(language.to_string());
            values.language = language;
        })
    }

    pub fn set_output_device(&mut self, name: Option<String>) {
        self.0.edit(|values, toml_document| {
            if let Some(name) = &name {
                toml_document["output_device"] = value(name);
            } else {
                toml_document.remove("output_device");
            }
            values.output_device = name;
        })
    }

    pub fn set_mute(&mut self, mute: bool) {
        self.0.edit(|values, toml_document| {
            toml_document["mute"] = value(mute);
            values.mute = mute;
        })
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.0.edit(|values, toml_document| {
            toml_document["volume"] = value(volume as f64);
            values.volume = volume;
        })
    }

    pub fn set_enable_openh264(&mut self, enable: bool) {
        self.0.edit(|values, toml_document| {
            toml_document["enable_openh264"] = value(enable);
            values.enable_openh264 = enable;
        })
    }

    pub fn set_log_filename_pattern(&mut self, pattern: FilenamePattern) {
        self.0.edit(|values, toml_document| {
            toml_document["log"]["filename_pattern"] = value(pattern.as_str());
            values.log.filename_pattern = pattern;
        })
    }

    pub fn set_storage_backend(&mut self, backend: StorageBackend) {
        self.0.edit(|values, toml_document| {
            toml_document["storage"]["backend"] = value(backend.as_str());
            values.storage.backend = backend;
        })
    }

    pub fn set_recent_limit(&mut self, limit: usize) {
        self.0.edit(|values, toml_document| {
            toml_document["recent_limit"] = value(limit as i64);
            values.recent_limit = limit;
        })
    }

    pub fn set_theme_preference(&mut self, theme_preference: ThemePreference) {
        self.0.edit(|values, toml_document| {
            if let Some(theme_preference) = theme_preference.as_str() {
                toml_document["theme"] = value(theme_preference);
            } else {
                toml_document.remove("theme");
            }
            values.theme_preference = theme_preference;
        });
        if let Some(watcher) = self.1.map(|w| &w.theme_preference_watcher) {
            let _ = watcher.send(theme_preference);
        }
    }

    pub fn set_gamemode_preference(&mut self, gamemode_preference: GameModePreference) {
        self.0.edit(|values, toml_document| {
            if let Some(gamemode_preference) = gamemode_preference.as_str() {
                toml_document["gamemode"] = value(gamemode_preference);
            } else {
                toml_document.remove("gamemode");
            }
            values.gamemode_preference = gamemode_preference;
        });
    }

    pub fn set_open_url_mode(&mut self, open_url_mode: OpenUrlMode) {
        self.0.edit(|values, toml_document| {
            if let Some(open_url_mode) = open_url_mode.as_str() {
                toml_document["open_url_mode"] = value(open_url_mode);
            } else {
                toml_document.remove("open_url_mode");
            }
            values.open_url_mode = open_url_mode;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preferences::read::read_preferences;
    use fluent_templates::loader::langid;

    ruffle_frontend_utils::define_serialization_test_helpers!(
        read_preferences,
        SavedGlobalPreferences,
        PreferencesWriter
    );

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

    #[test]
    fn set_volume() {
        test("", |writer| writer.set_volume(0.5), "volume = 0.5\n");
    }

    #[test]
    fn set_mute() {
        test("", |writer| writer.set_mute(true), "mute = true\n");
        test(
            "mute = true",
            |writer| writer.set_mute(false),
            "mute = false\n",
        );
    }

    #[test]
    fn set_enable_openh264() {
        test(
            "",
            |writer| writer.set_enable_openh264(false),
            "enable_openh264 = false\n",
        );
        test(
            "enable_openh264 = false",
            |writer| writer.set_enable_openh264(true),
            "enable_openh264 = true\n",
        );
    }

    #[test]
    fn set_log_filename_pattern() {
        test(
            "",
            |writer| writer.set_log_filename_pattern(FilenamePattern::WithTimestamp),
            "log = { filename_pattern = \"with_timestamp\" }\n",
        );
        test(
            "log = { filename_pattern = \"with_timestamp\" }\n",
            |writer| writer.set_log_filename_pattern(FilenamePattern::SingleFile),
            "log = { filename_pattern = \"single_file\" }\n",
        );
        test(
            "[log]\nfilename_pattern = \"with_timestamp\"\n",
            |writer| writer.set_log_filename_pattern(FilenamePattern::SingleFile),
            "[log]\nfilename_pattern = \"single_file\"\n",
        );
    }

    #[test]
    fn set_storage_backend() {
        test(
            "",
            |writer| writer.set_storage_backend(StorageBackend::Disk),
            "storage = { backend = \"disk\" }\n",
        );
        test(
            "storage = { backend = \"disk\" }\n",
            |writer| writer.set_storage_backend(StorageBackend::Memory),
            "storage = { backend = \"memory\" }\n",
        );
        test(
            "[storage]\nbackend = \"disk\"\n",
            |writer| writer.set_storage_backend(StorageBackend::Memory),
            "[storage]\nbackend = \"memory\"\n",
        );
    }

    #[test]
    fn set_recent_limit() {
        test(
            "",
            |writer| writer.set_recent_limit(40),
            "recent_limit = 40\n",
        );
        test(
            "recent_limit = 5",
            |writer| writer.set_recent_limit(15),
            "recent_limit = 15\n",
        );
    }

    #[test]
    fn set_theme() {
        test(
            "theme = 6\n",
            |writer| writer.set_theme_preference(ThemePreference::Dark),
            "theme = \"dark\"\n",
        );
        test(
            "theme = \"dark\"",
            |writer| writer.set_theme_preference(ThemePreference::System),
            "",
        );
    }

    #[test]
    fn set_gamemode() {
        test(
            "gamemode = 6\n",
            |writer| writer.set_gamemode_preference(GameModePreference::Off),
            "gamemode = \"off\"\n",
        );
        test(
            "gamemode = \"on\"",
            |writer| writer.set_gamemode_preference(GameModePreference::Default),
            "",
        );
    }

    #[test]
    fn set_open_url_mode() {
        test(
            "open_url_mode = 6\n",
            |writer| writer.set_open_url_mode(OpenUrlMode::Allow),
            "open_url_mode = \"allow\"\n",
        );
        test(
            "open_url_mode = \"deny\"",
            |writer| writer.set_open_url_mode(OpenUrlMode::Confirm),
            "",
        );
    }
}
