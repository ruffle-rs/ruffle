use crate::preferences::SavedGlobalPreferences;
use std::str::FromStr;
use toml_edit::{Document, Item};

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

/// Read the given preferences into a **guaranteed valid** `SavedGlobalPreferences`,
/// recording any possible warnings encountered along the way.
///
/// We wish to support backwards and forwards compatibility where possible,
/// so nothing is fatal in this function.
///
/// Default values are used wherever an unknown or invalid value is found;
/// this is to support the case of, for example, a later version having different supported
/// backends than an older version.
pub fn read_preferences(input: &str) -> (ParseResult, Document) {
    let mut result = ParseResult {
        result: Default::default(),
        warnings: vec![],
    };
    let document = match input.parse::<Document>() {
        Ok(document) => document,
        Err(e) => {
            result.add_warning(format!("Invalid TOML: {e}"));
            return (result, Document::default());
        }
    };

    match parse_item_from_str(document.get("graphics_backend")) {
        Ok(Some(value)) => result.result.graphics_backend = value,
        Ok(None) => {}
        Err(e) => result.add_warning(format!("Invalid graphics_backend: {e}")),
    };

    match parse_item_from_str(document.get("graphics_power_preference")) {
        Ok(Some(value)) => result.result.graphics_power_preference = value,
        Ok(None) => {}
        Err(e) => result.add_warning(format!("Invalid graphics_power_preference: {e}")),
    };

    match parse_item_from_str(document.get("language")) {
        Ok(Some(value)) => result.result.language = value,
        Ok(None) => {}
        Err(e) => result.add_warning(format!("Invalid language: {e}")),
    };

    match parse_item_from_str(document.get("output_device")) {
        Ok(Some(value)) => result.result.output_device = Some(value),
        Ok(None) => {}
        Err(e) => result.add_warning(format!("Invalid output_device: {e}")),
    };

    (result, document)
}

fn parse_item_from_str<T: FromStr + Default>(item: Option<&Item>) -> Result<Option<T>, String> {
    if let Some(item) = item {
        if let Some(str) = item.as_str() {
            if let Ok(value) = str.parse::<T>() {
                Ok(Some(value))
            } else {
                Err(format!("unsupported value {str:?}"))
            }
        } else {
            Err(format!("expected string but found {}", item.type_name()))
        }
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
