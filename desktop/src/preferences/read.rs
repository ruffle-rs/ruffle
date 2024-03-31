use crate::preferences::SavedGlobalPreferences;
use std::str::FromStr;
use toml_edit::{DocumentMut, Item, TableLike};

#[derive(Debug, PartialEq)]
pub struct ParseResult {
    pub result: SavedGlobalPreferences,
    pub warnings: Vec<String>,
}

impl ParseResult {
    fn add_warning(&mut self, message: String) {
        self.warnings.push(message);
    }
}

#[derive(Default)]
pub struct ParseContext {
    warnings: Vec<String>,
    /// Path of the current item being parsed
    path: Vec<&'static str>,
}

impl ParseContext {
    pub fn push_key(&mut self, key: &'static str) {
        self.path.push(key);
    }

    pub fn pop_key(&mut self) {
        let _ = self.path.pop();
    }

    pub fn path(&self) -> String {
        self.path.join(".")
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

pub trait ReadExt<'a> {
    fn get_impl(&'a self, key: &str) -> Option<&'a Item>;

    fn get_table_like(
        &'a self,
        cx: &mut ParseContext,
        key: &'static str,
        fun: impl FnOnce(&mut ParseContext, &dyn TableLike),
    ) {
        if let Some(item) = self.get_impl(key) {
            cx.push_key(key);

            if let Some(table) = item.as_table_like() {
                fun(cx, table);
            } else {
                cx.add_warning(format!(
                    "Invalid {}: expected table but found {}",
                    cx.path(),
                    item.type_name()
                ));
            }

            cx.pop_key();
        }
    }

    fn parse_from_str<T: FromStr>(&'a self, cx: &mut ParseContext, key: &'static str) -> Option<T> {
        cx.push_key(key);

        let res = if let Some(item) = self.get_impl(key) {
            if let Some(str) = item.as_str() {
                if let Ok(value) = str.parse::<T>() {
                    Some(value)
                } else {
                    cx.add_warning(format!("Invalid {}: unsupported value {str:?}", cx.path()));
                    None
                }
            } else {
                cx.add_warning(format!(
                    "Invalid {}: expected string but found {}",
                    cx.path(),
                    item.type_name()
                ));
                None
            }
        } else {
            None
        };

        cx.pop_key();

        res
    }

    fn get_bool(&'a self, cx: &mut ParseContext, key: &'static str) -> Option<bool> {
        cx.push_key(key);

        let res = if let Some(item) = self.get_impl(key) {
            if let Some(value) = item.as_bool() {
                Some(value)
            } else {
                cx.add_warning(format!(
                    "Invalid {}: expected boolean but found {}",
                    cx.path(),
                    item.type_name()
                ));
                None
            }
        } else {
            None
        };

        cx.pop_key();

        res
    }

    fn get_float(&'a self, cx: &mut ParseContext, key: &'static str) -> Option<f64> {
        cx.push_key(key);

        let res = if let Some(item) = self.get_impl(key) {
            if let Some(value) = item.as_float() {
                Some(value)
            } else {
                cx.add_warning(format!(
                    "Invalid {}: expected float but found {}",
                    cx.path(),
                    item.type_name()
                ));
                None
            }
        } else {
            None
        };

        cx.pop_key();

        res
    }
}

// Implementations for toml_edit types.

impl<'a> ReadExt<'a> for DocumentMut {
    fn get_impl(&'a self, key: &str) -> Option<&'a Item> {
        self.get(key)
    }
}

impl<'a> ReadExt<'a> for Item {
    fn get_impl(&'a self, key: &str) -> Option<&'a Item> {
        self.get(key)
    }
}

impl<'a> ReadExt<'a> for dyn TableLike + 'a {
    fn get_impl(&'a self, key: &str) -> Option<&'a Item> {
        self.get(key)
    }
}

/// Read the given preferences into a **guaranteed valid** `SavedGlobalPreferences`,
/// recording any possible warnings encountered along the way.
///
/// We wish to support backwards and forwards compatibility where possible,
/// so nothing is fatal in this function.
///
/// Default values are used wherever an unknown or invalid value is found;
/// this is to support the case of, for example, a later version having different supported
/// backends than an older version.
pub fn read_preferences(input: &str) -> (ParseResult, DocumentMut) {
    let mut result = ParseResult {
        result: Default::default(),
        warnings: vec![],
    };
    let document = match input.parse::<DocumentMut>() {
        Ok(document) => document,
        Err(e) => {
            result.add_warning(format!("Invalid TOML: {e}"));
            return (result, DocumentMut::default());
        }
    };

    let mut cx = ParseContext::default();

    if let Some(value) = document.parse_from_str(&mut cx, "graphics_backend") {
        result.result.graphics_backend = value;
    };

    if let Some(value) = document.parse_from_str(&mut cx, "graphics_power_preference") {
        result.result.graphics_power_preference = value;
    };

    if let Some(value) = document.parse_from_str(&mut cx, "language") {
        result.result.language = value;
    };

    if let Some(value) = document.parse_from_str(&mut cx, "output_device") {
        result.result.output_device = Some(value);
    };

    if let Some(value) = document.get_float(&mut cx, "volume") {
        result.result.volume = value.clamp(0.0, 1.0) as f32;
    };

    if let Some(value) = document.get_bool(&mut cx, "mute") {
        result.result.mute = value;
    };

    document.get_table_like(&mut cx, "log", |cx, log| {
        if let Some(value) = log.parse_from_str(cx, "filename_pattern") {
            result.result.log.filename_pattern = value;
        };
    });

    result.warnings = cx.warnings;
    (result, document)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::FilenamePattern;
    use crate::preferences::LogPreferences;
    use fluent_templates::loader::langid;
    use ruffle_render_wgpu::clap::{GraphicsBackend, PowerPreference};

    #[test]
    fn invalid_toml() {
        let result = read_preferences("~~INVALID~~").0;

        assert_eq!(ParseResult{result: Default::default(), warnings:
            vec![
                "Invalid TOML: TOML parse error at line 1, column 1\n  |\n1 | ~~INVALID~~\n  | ^\ninvalid key\n".to_string()
            ]}, result
        );
    }

    #[test]
    fn empty_toml() {
        let result = read_preferences("").0;

        assert_eq!(
            ParseResult {
                result: Default::default(),
                warnings: vec![]
            },
            result
        );
    }

    #[test]
    fn invalid_backend_type() {
        let result = read_preferences("graphics_backend = 5").0;

        assert_eq!(
            ParseResult {
                result: Default::default(),
                warnings: vec![
                    "Invalid graphics_backend: expected string but found integer".to_string()
                ]
            },
            result
        );
    }

    #[test]
    fn invalid_backend_value() {
        let result = read_preferences("graphics_backend = \"fast\"").0;

        assert_eq!(
            ParseResult {
                result: Default::default(),
                warnings: vec!["Invalid graphics_backend: unsupported value \"fast\"".to_string()]
            },
            result
        );
    }

    #[test]
    fn correct_backend_value() {
        let result = read_preferences("graphics_backend = \"vulkan\"").0;

        assert_eq!(
            ParseResult {
                result: SavedGlobalPreferences {
                    graphics_backend: GraphicsBackend::Vulkan,
                    ..Default::default()
                },
                warnings: vec![]
            },
            result
        );
    }

    #[test]
    fn invalid_power_type() {
        let result = read_preferences("graphics_power_preference = 5").0;

        assert_eq!(
            ParseResult {
                result: Default::default(),
                warnings: vec![
                    "Invalid graphics_power_preference: expected string but found integer"
                        .to_string()
                ]
            },
            result
        );
    }

    #[test]
    fn invalid_power_value() {
        let result = read_preferences("graphics_power_preference = \"fast\"").0;

        assert_eq!(
            ParseResult {
                result: Default::default(),
                warnings: vec![
                    "Invalid graphics_power_preference: unsupported value \"fast\"".to_string()
                ]
            },
            result
        );
    }

    #[test]
    fn correct_power_value() {
        let result = read_preferences("graphics_power_preference = \"low\"").0;

        assert_eq!(
            ParseResult {
                result: SavedGlobalPreferences {
                    graphics_power_preference: PowerPreference::Low,
                    ..Default::default()
                },
                warnings: vec![]
            },
            result
        );
    }

    #[test]
    fn invalid_language_value() {
        let result = read_preferences("language = \"???\"").0;

        assert_eq!(
            ParseResult {
                result: Default::default(),
                warnings: vec!["Invalid language: unsupported value \"???\"".to_string()]
            },
            result
        );
    }

    #[test]
    fn correct_language_value() {
        let result = read_preferences("language = \"en-US\"").0;

        assert_eq!(
            ParseResult {
                result: SavedGlobalPreferences {
                    language: langid!("en-US"),
                    ..Default::default()
                },
                warnings: vec![]
            },
            result
        );
    }

    #[test]
    fn correct_output_device() {
        let result = read_preferences("output_device = \"Speakers\"").0;

        assert_eq!(
            ParseResult {
                result: SavedGlobalPreferences {
                    output_device: Some("Speakers".to_string()),
                    ..Default::default()
                },
                warnings: vec![]
            },
            result
        );
    }

    #[test]
    fn invalid_output_device() {
        let result = read_preferences("output_device = 5").0;

        assert_eq!(
            ParseResult {
                result: SavedGlobalPreferences {
                    output_device: None,
                    ..Default::default()
                },
                warnings: vec![
                    "Invalid output_device: expected string but found integer".to_string()
                ]
            },
            result
        );
    }

    #[test]
    fn mute() {
        assert_eq!(
            ParseResult {
                result: SavedGlobalPreferences {
                    mute: false,
                    ..Default::default()
                },
                warnings: vec!["Invalid mute: expected boolean but found string".to_string()]
            },
            read_preferences("mute = \"false\"").0
        );

        assert_eq!(
            ParseResult {
                result: SavedGlobalPreferences {
                    mute: true,
                    ..Default::default()
                },
                warnings: vec![]
            },
            read_preferences("mute = true").0
        );

        assert_eq!(
            ParseResult {
                result: SavedGlobalPreferences {
                    mute: false,
                    ..Default::default()
                },
                warnings: vec![]
            },
            read_preferences("").0
        );
    }

    #[test]
    fn volume() {
        assert_eq!(
            ParseResult {
                result: SavedGlobalPreferences {
                    volume: 1.0,
                    ..Default::default()
                },
                warnings: vec!["Invalid volume: expected float but found string".to_string()]
            },
            read_preferences("volume = \"0.5\"").0
        );

        assert_eq!(
            ParseResult {
                result: SavedGlobalPreferences {
                    volume: 0.5,
                    ..Default::default()
                },
                warnings: vec![]
            },
            read_preferences("volume = 0.5").0
        );

        assert_eq!(
            ParseResult {
                result: SavedGlobalPreferences {
                    volume: 0.0,
                    ..Default::default()
                },
                warnings: vec![]
            },
            read_preferences("volume = -1.0").0
        );
    }

    #[test]
    fn log_filename() {
        assert_eq!(
            ParseResult {
                result: SavedGlobalPreferences {
                    log: LogPreferences {
                        ..Default::default()
                    },
                    ..Default::default()
                },
                warnings: vec![
                    "Invalid log.filename_pattern: expected string but found integer".to_string()
                ]
            },
            read_preferences("log = {filename_pattern = 5}").0
        );

        assert_eq!(
            ParseResult {
                result: SavedGlobalPreferences {
                    log: LogPreferences {
                        ..Default::default()
                    },
                    ..Default::default()
                },
                warnings: vec![
                    "Invalid log.filename_pattern: unsupported value \"???\"".to_string()
                ]
            },
            read_preferences("log = {filename_pattern = \"???\"}").0
        );

        assert_eq!(
            ParseResult {
                result: SavedGlobalPreferences {
                    log: LogPreferences {
                        filename_pattern: FilenamePattern::WithTimestamp,
                    },
                    ..Default::default()
                },
                warnings: vec![]
            },
            read_preferences("log = {filename_pattern = \"with_timestamp\"}").0
        );
    }

    #[test]
    fn log() {
        assert_eq!(
            ParseResult {
                result: SavedGlobalPreferences {
                    ..Default::default()
                },
                warnings: vec!["Invalid log: expected table but found string".to_string()]
            },
            read_preferences("log = \"yes\"").0
        );
    }
}
