use crate::preferences::SavedGlobalPreferences;
use ruffle_frontend_utils::bookmarks::{Bookmark, Bookmarks};
use ruffle_frontend_utils::parse::{DocumentHolder, ParseContext, ParseDetails, ReadExt};
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
                warnings: vec![format!("Invalid TOML: {e}")],
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
        result: DocumentHolder::new(result, document),
        warnings: cx.warnings,
    }
}

pub fn read_bookmarks(input: &str) -> ParseDetails<Bookmarks> {
    let document = match input.parse::<DocumentMut>() {
        Ok(document) => document,
        Err(e) => {
            return ParseDetails {
                result: Default::default(),
                warnings: vec![format!("Invalid TOML: {e}")],
            }
        }
    };

    let mut result = Vec::new();
    let mut cx = ParseContext::default();

    document.get_array_of_tables(&mut cx, "bookmark", |cx, bookmarks| {
        for bookmark in bookmarks.iter() {
            let url = match bookmark.parse_from_str(cx, "url") {
                Some(value) => value,
                None => url::Url::parse(ruffle_frontend_utils::bookmarks::INVALID_URL)
                    .expect("Url is constant and valid"),
            };

            let name = match bookmark.parse_from_str(cx, "name") {
                Some(value) => value,
                // Fallback to using the URL as the name.
                None => ruffle_frontend_utils::url_to_readable_name(&url).into_owned(),
            };

            result.push(Bookmark { url, name });
        }
    });

    ParseDetails {
        result: DocumentHolder::new(result, document),
        warnings: cx.warnings,
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::FilenamePattern;
    use crate::preferences::{storage::StorageBackend, LogPreferences, StoragePreferences};
    use fluent_templates::loader::langid;
    use ruffle_render_wgpu::clap::{GraphicsBackend, PowerPreference};
    use std::str::FromStr;
    use url::Url;

    #[test]
    fn invalid_toml() {
        let result = read_preferences("~~INVALID~~");

        assert_eq!(&SavedGlobalPreferences::default(), result.values());
        assert_eq!(vec!["Invalid TOML: TOML parse error at line 1, column 1\n  |\n1 | ~~INVALID~~\n  | ^\ninvalid key\n".to_string()], result.warnings);
    }

    #[test]
    fn empty_toml() {
        let result = read_preferences("");

        assert_eq!(&SavedGlobalPreferences::default(), result.values());
        assert_eq!(Vec::<String>::new(), result.warnings);
    }

    #[test]
    fn invalid_backend_type() {
        let result = read_preferences("graphics_backend = 5");

        assert_eq!(&SavedGlobalPreferences::default(), result.values());
        assert_eq!(
            vec!["Invalid graphics_backend: expected string but found integer".to_string()],
            result.warnings
        );
    }

    #[test]
    fn invalid_backend_value() {
        let result = read_preferences("graphics_backend = \"fast\"");

        assert_eq!(&SavedGlobalPreferences::default(), result.values());
        assert_eq!(
            vec!["Invalid graphics_backend: unsupported value \"fast\"".to_string()],
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
        assert_eq!(Vec::<String>::new(), result.warnings);
    }

    #[test]
    fn invalid_power_type() {
        let result = read_preferences("graphics_power_preference = 5");

        assert_eq!(&SavedGlobalPreferences::default(), result.values());
        assert_eq!(
            vec![
                "Invalid graphics_power_preference: expected string but found integer".to_string()
            ],
            result.warnings
        );
    }

    #[test]
    fn invalid_power_value() {
        let result = read_preferences("graphics_power_preference = \"fast\"");

        assert_eq!(&SavedGlobalPreferences::default(), result.values());
        assert_eq!(
            vec!["Invalid graphics_power_preference: unsupported value \"fast\"".to_string()],
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
        assert_eq!(Vec::<String>::new(), result.warnings);
    }

    #[test]
    fn invalid_language_value() {
        let result = read_preferences("language = \"???\"");

        assert_eq!(&SavedGlobalPreferences::default(), result.values());
        assert_eq!(
            vec!["Invalid language: unsupported value \"???\"".to_string()],
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
        assert_eq!(Vec::<String>::new(), result.warnings);
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
        assert_eq!(Vec::<String>::new(), result.warnings);
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
            vec!["Invalid output_device: expected string but found integer".to_string()],
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
            vec!["Invalid mute: expected boolean but found string".to_string()],
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
        assert_eq!(Vec::<String>::new(), result.warnings);

        let result = read_preferences("");
        assert_eq!(
            &SavedGlobalPreferences {
                mute: false,
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<String>::new(), result.warnings);
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
            vec!["Invalid volume: expected float but found string".to_string()],
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
        assert_eq!(Vec::<String>::new(), result.warnings);

        let result = read_preferences("volume = -1.0");
        assert_eq!(
            &SavedGlobalPreferences {
                volume: 0.0,
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<String>::new(), result.warnings);
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
            vec!["Invalid log.filename_pattern: expected string but found integer".to_string()],
            result.warnings
        );

        let result = read_preferences("log = {filename_pattern = \"???\"}");
        assert_eq!(&SavedGlobalPreferences::default(), result.values());
        assert_eq!(
            vec!["Invalid log.filename_pattern: unsupported value \"???\"".to_string()],
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
        assert_eq!(Vec::<String>::new(), result.warnings);
    }

    #[test]
    fn log() {
        let result = read_preferences("log = \"yes\"");
        assert_eq!(&SavedGlobalPreferences::default(), result.values());
        assert_eq!(
            vec!["Invalid log: expected table but found string".to_string()],
            result.warnings
        );
    }

    #[test]
    fn storage_backend() {
        let result = read_preferences("storage = {backend = 5}");
        assert_eq!(&SavedGlobalPreferences::default(), result.values());
        assert_eq!(
            vec!["Invalid storage.backend: expected string but found integer".to_string()],
            result.warnings
        );

        let result = read_preferences("storage = {backend = \"???\"}");
        assert_eq!(&SavedGlobalPreferences::default(), result.values());
        assert_eq!(
            vec!["Invalid storage.backend: unsupported value \"???\"".to_string()],
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
        assert_eq!(Vec::<String>::new(), result.warnings);
    }

    #[test]
    fn storage() {
        let result = read_preferences("storage = \"no\"");
        assert_eq!(&SavedGlobalPreferences::default(), result.values());
        assert_eq!(
            vec!["Invalid storage: expected table but found string".to_string()],
            result.warnings
        );
    }

    #[test]
    fn bookmark() {
        let result = read_bookmarks("[bookmark]");
        assert_eq!(&Vec::<Bookmark>::new(), result.values());
        assert_eq!(
            vec!["Invalid bookmark: expected array of tables but found table".to_string()],
            result.warnings
        );

        let result = read_bookmarks("[[bookmark]]");
        assert_eq!(
            &vec![Bookmark {
                url: Url::parse(ruffle_frontend_utils::bookmarks::INVALID_URL).unwrap(),
                name: "".to_string(),
            }],
            result.values()
        );
        assert_eq!(Vec::<String>::new(), result.warnings);

        let result = read_bookmarks("[[bookmark]]\nurl = \"invalid\"");
        assert_eq!(
            &vec![Bookmark {
                url: Url::parse(ruffle_frontend_utils::bookmarks::INVALID_URL).unwrap(),
                name: "".to_string(),
            }],
            result.values()
        );
        assert_eq!(
            vec!["Invalid bookmark.url: unsupported value \"invalid\"".to_string()],
            result.warnings
        );

        let result = read_bookmarks(
            "[[bookmark]]\nurl = \"https://ruffle.rs/logo-anim.swf\"\nname = \"Logo SWF\"",
        );
        assert_eq!(
            &vec![Bookmark {
                url: Url::parse("https://ruffle.rs/logo-anim.swf").unwrap(),
                name: "Logo SWF".to_string(),
            }],
            result.values()
        );
        assert_eq!(Vec::<String>::new(), result.warnings);
    }

    #[test]
    fn multiple_bookmarks() {
        let result = read_bookmarks(
            r#"
            [[bookmark]]
            url = "file:///home/user/example.swf"

            [[bookmark]]
            url = "https://ruffle.rs/logo-anim.swf"
            "#,
        );
        assert_eq!(
            &vec![
                Bookmark {
                    url: Url::from_str("file:///home/user/example.swf").unwrap(),
                    name: "example.swf".to_string(),
                },
                Bookmark {
                    url: Url::from_str("https://ruffle.rs/logo-anim.swf").unwrap(),
                    name: "logo-anim.swf".to_string(),
                }
            ],
            result.values()
        );
        assert_eq!(Vec::<String>::new(), result.warnings);

        let result = read_bookmarks(
            r#"
            [[bookmark]]
            url = "file:///home/user/example.swf"

            [[bookmark]]
            url = "invalid"

            [[bookmark]]

            [[bookmark]]
            url = "https://ruffle.rs/logo-anim.swf"
            "#,
        );
        assert_eq!(
            &vec![
                Bookmark {
                    url: Url::from_str("file:///home/user/example.swf").unwrap(),
                    name: "example.swf".to_string(),
                },
                Bookmark {
                    url: Url::parse(ruffle_frontend_utils::bookmarks::INVALID_URL).unwrap(),
                    name: "".to_string(),
                },
                Bookmark {
                    url: Url::parse(ruffle_frontend_utils::bookmarks::INVALID_URL).unwrap(),
                    name: "".to_string(),
                },
                Bookmark {
                    url: Url::from_str("https://ruffle.rs/logo-anim.swf").unwrap(),
                    name: "logo-anim.swf".to_string(),
                }
            ],
            result.values()
        );
        assert_eq!(
            vec!["Invalid bookmark.url: unsupported value \"invalid\"".to_string()],
            result.warnings
        );
    }
}
