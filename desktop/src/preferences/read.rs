use crate::preferences::SavedGlobalPreferences;
use ruffle_frontend_utils::parse::{
    DocumentHolder, ParseContext, ParseDetails, ParseWarning, ReadExt,
};
use toml_edit::DocumentMut;

/// Read the given preferences into a **guaranteed valid** `SavedGlobalPreferences`,
/// recording any possible warnings encountered along the way.
///
/// We wish to support backwards and forwards compatibility where possible,
/// so nothing is fatal in this function.
///
/// Default values are used wherever an unknown or invalid value is found;
/// this is to support the case of, for example, a later version having different supported
/// backends than an older version.
pub fn read_preferences(input: &str) -> ParseDetails<SavedGlobalPreferences> {
    let document = match input.parse::<DocumentMut>() {
        Ok(document) => document,
        Err(e) => {
            return ParseDetails {
                result: Default::default(),
                warnings: vec![ParseWarning::InvalidToml(e)],
            }
        }
    };

    let mut result = SavedGlobalPreferences::default();
    let mut cx = ParseContext::default();

    if let Some(value) = document.parse_from_str(&mut cx, "graphics_backend") {
        result.graphics_backend = value;
    };

    if let Some(value) = document.parse_from_str(&mut cx, "graphics_power_preference") {
        result.graphics_power_preference = value;
    };

    if let Some(value) = document.parse_from_str(&mut cx, "language") {
        result.language = value;
    };

    if let Some(value) = document.parse_from_str(&mut cx, "output_device") {
        result.output_device = Some(value);
    };

    if let Some(value) = document.get_float(&mut cx, "volume") {
        result.volume = value.clamp(0.0, 1.0) as f32;
    };

    if let Some(value) = document.get_bool(&mut cx, "mute") {
        result.mute = value;
    };

    if let Some(value) = document.get_bool(&mut cx, "enable_openh264") {
        result.enable_openh264 = value;
    };

    if let Some(value) = document.get_integer(&mut cx, "recent_limit") {
        result.recent_limit = value as usize;
    }

    if let Some(value) = document.parse_from_str(&mut cx, "theme") {
        result.theme_preference = value;
    }

    document.get_table_like(&mut cx, "log", |cx, log| {
        if let Some(value) = log.parse_from_str(cx, "filename_pattern") {
            result.log.filename_pattern = value;
        };
    });

    document.get_table_like(&mut cx, "storage", |cx, storage| {
        if let Some(value) = storage.parse_from_str(cx, "backend") {
            result.storage.backend = value;
        }
    });

    ParseDetails {
        warnings: cx.warnings,
        result: DocumentHolder::new(result, document),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::ThemePreference;
    use crate::log::FilenamePattern;
    use crate::preferences::{storage::StorageBackend, LogPreferences, StoragePreferences};
    use fluent_templates::loader::langid;
    use ruffle_render_wgpu::clap::{GraphicsBackend, PowerPreference};

    #[test]
    fn invalid_toml() {
        let result = read_preferences("~~INVALID~~");

        assert_eq!(&SavedGlobalPreferences::default(), result.values());
        assert_eq!(result.warnings.len(), 1);
        assert_eq!("Invalid TOML: TOML parse error at line 1, column 1\n  |\n1 | ~~INVALID~~\n  | ^\ninvalid key\n", result.warnings[0].to_string());
    }

    #[test]
    fn empty_toml() {
        let result = read_preferences("");

        assert_eq!(&SavedGlobalPreferences::default(), result.values());
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }

    #[test]
    fn invalid_backend_type() {
        let result = read_preferences("graphics_backend = 5");

        assert_eq!(&SavedGlobalPreferences::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "string",
                actual: "integer",
                path: "graphics_backend".to_string()
            }],
            result.warnings
        );
    }

    #[test]
    fn invalid_backend_value() {
        let result = read_preferences("graphics_backend = \"fast\"");

        assert_eq!(&SavedGlobalPreferences::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnsupportedValue {
                value: "fast".to_string(),
                path: "graphics_backend".to_string()
            }],
            result.warnings
        );
    }

    #[test]
    fn correct_backend_value() {
        let result = read_preferences("graphics_backend = \"vulkan\"");

        assert_eq!(
            &SavedGlobalPreferences {
                graphics_backend: GraphicsBackend::Vulkan,
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }

    #[test]
    fn invalid_power_type() {
        let result = read_preferences("graphics_power_preference = 5");

        assert_eq!(&SavedGlobalPreferences::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "string",
                actual: "integer",
                path: "graphics_power_preference".to_string()
            }],
            result.warnings
        );
    }

    #[test]
    fn invalid_power_value() {
        let result = read_preferences("graphics_power_preference = \"fast\"");

        assert_eq!(&SavedGlobalPreferences::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnsupportedValue {
                value: "fast".to_string(),
                path: "graphics_power_preference".to_string()
            }],
            result.warnings
        );
    }

    #[test]
    fn correct_power_value() {
        let result = read_preferences("graphics_power_preference = \"low\"");

        assert_eq!(
            &SavedGlobalPreferences {
                graphics_power_preference: PowerPreference::Low,
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }

    #[test]
    fn invalid_language_value() {
        let result = read_preferences("language = \"???\"");

        assert_eq!(&SavedGlobalPreferences::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnsupportedValue {
                value: "???".to_string(),
                path: "language".to_string()
            }],
            result.warnings
        );
    }

    #[test]
    fn correct_language_value() {
        let result = read_preferences("language = \"en-US\"");

        assert_eq!(
            &SavedGlobalPreferences {
                language: langid!("en-US"),
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }

    #[test]
    fn correct_output_device() {
        let result = read_preferences("output_device = \"Speakers\"");

        assert_eq!(
            &SavedGlobalPreferences {
                output_device: Some("Speakers".to_string()),
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }

    #[test]
    fn invalid_output_device() {
        let result = read_preferences("output_device = 5");

        assert_eq!(
            &SavedGlobalPreferences {
                output_device: None,
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "string",
                actual: "integer",
                path: "output_device".to_string()
            }],
            result.warnings
        );
    }

    #[test]
    fn mute() {
        let result = read_preferences("mute = \"false\"");
        assert_eq!(
            &SavedGlobalPreferences {
                mute: false,
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "boolean",
                actual: "string",
                path: "mute".to_string()
            }],
            result.warnings
        );

        let result = read_preferences("mute = true");
        assert_eq!(
            &SavedGlobalPreferences {
                mute: true,
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);

        let result = read_preferences("");
        assert_eq!(
            &SavedGlobalPreferences {
                mute: false,
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }

    #[test]
    fn volume() {
        let result = read_preferences("volume = \"0.5\"");
        assert_eq!(
            &SavedGlobalPreferences {
                volume: 1.0,
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "float",
                actual: "string",
                path: "volume".to_string()
            }],
            result.warnings
        );

        let result = read_preferences("volume = 0.5");
        assert_eq!(
            &SavedGlobalPreferences {
                volume: 0.5,
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);

        let result = read_preferences("volume = -1.0");
        assert_eq!(
            &SavedGlobalPreferences {
                volume: 0.0,
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }

    #[test]
    fn enable_openh264() {
        let result = read_preferences("enable_openh264 = \"true\"");
        assert_eq!(
            &SavedGlobalPreferences {
                enable_openh264: true,
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "boolean",
                actual: "string",
                path: "enable_openh264".to_string()
            }],
            result.warnings
        );

        let result = read_preferences("enable_openh264 = false");
        assert_eq!(
            &SavedGlobalPreferences {
                enable_openh264: false,
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);

        let result = read_preferences("");
        assert_eq!(
            &SavedGlobalPreferences {
                enable_openh264: true,
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }

    #[test]
    fn log_filename() {
        let result = read_preferences("log = {filename_pattern = 5}");
        assert_eq!(
            &SavedGlobalPreferences {
                log: LogPreferences {
                    ..Default::default()
                },
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "string",
                actual: "integer",
                path: "log.filename_pattern".to_string()
            }],
            result.warnings
        );

        let result = read_preferences("log = {filename_pattern = \"???\"}");
        assert_eq!(&SavedGlobalPreferences::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnsupportedValue {
                value: "???".to_string(),
                path: "log.filename_pattern".to_string()
            }],
            result.warnings
        );

        let result = read_preferences("log = {filename_pattern = \"with_timestamp\"}");
        assert_eq!(
            &SavedGlobalPreferences {
                log: LogPreferences {
                    filename_pattern: FilenamePattern::WithTimestamp,
                },
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }

    #[test]
    fn log() {
        let result = read_preferences("log = \"yes\"");
        assert_eq!(&SavedGlobalPreferences::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "table",
                actual: "string",
                path: "log".to_string()
            }],
            result.warnings
        );
    }

    #[test]
    fn storage_backend() {
        let result = read_preferences("storage = {backend = 5}");
        assert_eq!(&SavedGlobalPreferences::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "string",
                actual: "integer",
                path: "storage.backend".to_string()
            }],
            result.warnings
        );

        let result = read_preferences("storage = {backend = \"???\"}");
        assert_eq!(&SavedGlobalPreferences::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnsupportedValue {
                value: "???".to_string(),
                path: "storage.backend".to_string()
            }],
            result.warnings
        );

        let result = read_preferences("storage = {backend = \"memory\"}");
        assert_eq!(
            &SavedGlobalPreferences {
                storage: StoragePreferences {
                    backend: StorageBackend::Memory,
                },
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }

    #[test]
    fn storage() {
        let result = read_preferences("storage = \"no\"");
        assert_eq!(&SavedGlobalPreferences::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "table",
                actual: "string",
                path: "storage".to_string()
            }],
            result.warnings
        );
    }

    #[test]
    fn recent_limit() {
        let result = read_preferences("recent_limit = \"1\"");
        assert_eq!(
            &SavedGlobalPreferences {
                recent_limit: 10,
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "integer",
                actual: "string",
                path: "recent_limit".to_string(),
            }],
            result.warnings
        );

        let result = read_preferences("recent_limit = 0.5");
        assert_eq!(
            &SavedGlobalPreferences {
                recent_limit: 10,
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "integer",
                actual: "float",
                path: "recent_limit".to_string(),
            }],
            result.warnings
        );

        let result = read_preferences("recent_limit = 5");
        assert_eq!(
            &SavedGlobalPreferences {
                recent_limit: 5,
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }

    #[test]
    fn theme() {
        let result = read_preferences("theme = \"light\"");
        assert_eq!(
            &SavedGlobalPreferences {
                theme_preference: ThemePreference::Light,
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);

        let result = read_preferences("theme = \"dark\"");
        assert_eq!(
            &SavedGlobalPreferences {
                theme_preference: ThemePreference::Dark,
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);

        let result = read_preferences("theme = \"default\"");
        assert_eq!(
            &SavedGlobalPreferences {
                theme_preference: ThemePreference::System,
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(
            vec![ParseWarning::UnsupportedValue {
                value: "default".to_string(),
                path: "theme".to_string(),
            }],
            result.warnings
        );

        let result = read_preferences("theme = 9");
        assert_eq!(
            &SavedGlobalPreferences {
                theme_preference: ThemePreference::System,
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "string",
                actual: "integer",
                path: "theme".to_string(),
            }],
            result.warnings
        );
    }
}
