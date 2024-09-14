use crate::parse::{ItemExt, ParseContext, ReadExt};
use crate::player_options::PlayerOptions;
use std::time::Duration;
use toml_edit::TableLike;

pub fn read_player_options<'a>(
    cx: &mut ParseContext<'a>,
    table: &'a dyn TableLike,
) -> PlayerOptions {
    let mut result = PlayerOptions::default();

    // Parameters (FlashVars) key-value table, all values must be strings.
    table.get_table_like(cx, "parameters", |cx, parameters| {
        for (key, value) in parameters.iter() {
            cx.push_key(key);

            if let Some(value) = value.as_str_or_warn(cx) {
                result.parameters.push((key.to_owned(), value.to_owned()));
            }

            cx.pop_key();
        }
    });

    // Script timeout in seconds (fractional-values are allowed)
    result.max_execution_duration = table
        .get_float_like(cx, "script_timeout")
        .map(Duration::from_secs_f64);

    // Base Url
    result.base = table.parse_from_str(cx, "base_url");

    // Quality
    result.quality = table.parse_from_str(cx, "quality");

    // Align
    result.align = table.parse_from_str(cx, "align");

    // Force Align
    result.force_align = table.get_bool(cx, "force_align");

    // Scale Mode
    result.scale = table.parse_from_str(cx, "scale_mode");

    // Force Scale Mode
    result.force_scale = table.get_bool(cx, "force_scale_mode");

    // Upgrade HTTP to HTTPS
    result.upgrade_to_https = table.get_bool(cx, "upgrade_http_to_https");

    // Load Behavior
    result.load_behavior = table.parse_from_str(cx, "load_behavior");

    // Letterbox
    result.letterbox = table.parse_from_str(cx, "letterbox");

    // Spoof Url
    result.spoof_url = table.parse_from_str(cx, "spoof_url");

    // Player version
    result.player_version = table.get_integer(cx, "version").map(|x| x as u8);

    // Player runtime
    result.player_runtime = table.parse_from_str(cx, "runtime");

    // Frame rate
    result.frame_rate = table.get_float_like(cx, "frame_rate");

    // Mock external interface
    result.dummy_external_interface = table.get_bool(cx, "mock_external_interface");

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{DocumentHolder, ParseDetails, ParseWarning};
    use ruffle_core::config::Letterbox;
    use ruffle_core::{LoadBehavior, PlayerRuntime, StageAlign, StageScaleMode};
    use ruffle_render::quality::StageQuality;
    use toml_edit::DocumentMut;
    use url::Url;

    fn read(input: &str) -> ParseDetails<PlayerOptions> {
        let doc = input
            .parse::<DocumentMut>()
            .expect("Test input should be valid TOML");
        let mut cx = ParseContext::default();

        let options = read_player_options(&mut cx, doc.as_table());

        ParseDetails {
            warnings: cx.warnings,
            result: DocumentHolder::new(options, doc),
        }
    }

    #[test]
    fn empty() {
        let result = read("");
        assert_eq!(&PlayerOptions::default(), result.values());
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }

    #[test]
    fn parameters() {
        let result = read("[parameters]");
        assert_eq!(&PlayerOptions::default(), result.values());
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);

        let result = read(
            r#"
        [parameters]
        value1 = "a_very_interesting_value"
        value2 = true
        "#,
        );
        assert_eq!(
            &PlayerOptions {
                parameters: vec![("value1".to_string(), "a_very_interesting_value".to_string())],
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "string",
                actual: "boolean",
                path: "parameters.value2".to_string()
            }],
            result.warnings
        );
    }

    #[test]
    fn script_timeout() {
        let result = read("script_timeout = true");
        assert_eq!(&PlayerOptions::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "float or integer",
                actual: "boolean",
                path: "script_timeout".to_string()
            }],
            result.warnings
        );

        let result = read("script_timeout = 5");
        assert_eq!(
            &PlayerOptions {
                max_execution_duration: Some(Duration::from_secs(5)),
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);

        let result = read("script_timeout = 1.5");
        assert_eq!(
            &PlayerOptions {
                max_execution_duration: Some(Duration::from_secs_f64(1.5)),
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }

    #[test]
    fn base_url() {
        let result = read("base_url = false");
        assert_eq!(&PlayerOptions::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "string",
                actual: "boolean",
                path: "base_url".to_string()
            }],
            result.warnings
        );

        let result = read("base_url = \"invalid\"");
        assert_eq!(&PlayerOptions::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnsupportedValue {
                value: "invalid".to_string(),
                path: "base_url".to_string(),
            }],
            result.warnings
        );

        let result = read("base_url = \"file:///example/path/\"");
        assert_eq!(
            &PlayerOptions {
                base: Some(Url::parse("file:///example/path/").unwrap()),
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }

    #[test]
    fn quality() {
        let result = read("quality = false");
        assert_eq!(&PlayerOptions::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "string",
                actual: "boolean",
                path: "quality".to_string()
            }],
            result.warnings
        );

        let result = read("quality = \"fabulous\"");
        assert_eq!(&PlayerOptions::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnsupportedValue {
                value: "fabulous".to_string(),
                path: "quality".to_string(),
            }],
            result.warnings
        );

        fn assert_variant(variant: &str, quality: StageQuality) {
            let result = read(&format!("quality = \"{variant}\""));
            assert_eq!(
                &PlayerOptions {
                    quality: Some(quality),
                    ..Default::default()
                },
                result.values()
            );
            assert_eq!(Vec::<ParseWarning>::new(), result.warnings);

            // Check case-sensitivity.
            let variant = variant.to_ascii_uppercase();
            let result = read(&format!("quality = \"{variant}\""));
            assert_eq!(
                &PlayerOptions {
                    quality: None,
                    ..Default::default()
                },
                result.values()
            );
            assert_eq!(
                vec![ParseWarning::UnsupportedValue {
                    value: variant,
                    path: "quality".to_string()
                }],
                result.warnings
            );
        }
        assert_variant("high", StageQuality::High);
        assert_variant("medium", StageQuality::Medium);
        assert_variant("low", StageQuality::Low);
    }

    #[test]
    fn align() {
        let result = read("align = 1.0");
        assert_eq!(&PlayerOptions::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "string",
                actual: "float",
                path: "align".to_string()
            }],
            result.warnings
        );

        let result = read("align = \"corner\"");
        assert_eq!(&PlayerOptions::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnsupportedValue {
                value: "corner".to_string(),
                path: "align".to_string(),
            }],
            result.warnings
        );

        fn assert_variant(variant: &str, align: StageAlign) {
            let result = read(&format!("align = \"{variant}\""));
            assert_eq!(
                &PlayerOptions {
                    align: Some(align),
                    ..Default::default()
                },
                result.values()
            );
            assert_eq!(Vec::<ParseWarning>::new(), result.warnings);

            // Check case-sensitivity.
            let variant = variant.to_ascii_uppercase();
            let result: ParseDetails<PlayerOptions> = read(&format!("align = \"{variant}\""));
            assert_eq!(
                &PlayerOptions {
                    align: None,
                    ..Default::default()
                },
                result.values()
            );
            assert_eq!(
                vec![ParseWarning::UnsupportedValue {
                    value: variant,
                    path: "align".to_string()
                }],
                result.warnings
            );
        }
        assert_variant("bottom", StageAlign::BOTTOM);
        assert_variant("bottom_left", StageAlign::BOTTOM | StageAlign::LEFT);
        assert_variant("bottom_right", StageAlign::BOTTOM | StageAlign::RIGHT);
        assert_variant("left", StageAlign::LEFT);
        assert_variant("right", StageAlign::RIGHT);
        assert_variant("top", StageAlign::TOP);
        assert_variant("top_left", StageAlign::TOP | StageAlign::LEFT);
        assert_variant("top_right", StageAlign::TOP | StageAlign::RIGHT);
        assert_variant("center", StageAlign::empty())
    }

    #[test]
    fn force_align() {
        let result = read("force_align = 1");
        assert_eq!(&PlayerOptions::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "boolean",
                actual: "integer",
                path: "force_align".to_string()
            }],
            result.warnings
        );

        let result = read("force_align = true");
        assert_eq!(
            &PlayerOptions {
                force_align: Some(true),
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }

    #[test]
    fn scale_mode() {
        let result = read("scale_mode = true");
        assert_eq!(&PlayerOptions::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "string",
                actual: "boolean",
                path: "scale_mode".to_string()
            }],
            result.warnings
        );

        let result = read("scale_mode = \"invalid\"");
        assert_eq!(&PlayerOptions::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnsupportedValue {
                value: "invalid".to_string(),
                path: "scale_mode".to_string(),
            }],
            result.warnings
        );

        fn assert_variant(variant: &str, scale: StageScaleMode) {
            let result = read(&format!("scale_mode = \"{variant}\""));
            assert_eq!(
                &PlayerOptions {
                    scale: Some(scale),
                    ..Default::default()
                },
                result.values()
            );
            assert_eq!(Vec::<ParseWarning>::new(), result.warnings);

            // Check case-sensitivity.
            let variant = variant.to_ascii_uppercase();
            let result = read(&format!("scale_mode = \"{variant}\""));
            assert_eq!(
                &PlayerOptions {
                    scale: None,
                    ..Default::default()
                },
                result.values()
            );
            assert_eq!(
                vec![ParseWarning::UnsupportedValue {
                    value: variant,
                    path: "scale_mode".to_string()
                }],
                result.warnings
            );
        }
        assert_variant("exact_fit", StageScaleMode::ExactFit);
        assert_variant("no_border", StageScaleMode::NoBorder);
        assert_variant("no_scale", StageScaleMode::NoScale);
        assert_variant("show_all", StageScaleMode::ShowAll);
    }

    #[test]
    fn force_scale_mode() {
        let result = read("force_scale_mode = 1");
        assert_eq!(&PlayerOptions::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "boolean",
                actual: "integer",
                path: "force_scale_mode".to_string()
            }],
            result.warnings
        );

        let result = read("force_scale_mode = true");
        assert_eq!(
            &PlayerOptions {
                force_scale: Some(true),
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }

    #[test]
    fn upgrade_http_to_https() {
        let result = read("upgrade_http_to_https = 1");
        assert_eq!(&PlayerOptions::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "boolean",
                actual: "integer",
                path: "upgrade_http_to_https".to_string()
            }],
            result.warnings
        );

        let result = read("upgrade_http_to_https = true");
        assert_eq!(
            &PlayerOptions {
                upgrade_to_https: Some(true),
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }

    #[test]
    fn load_behavior() {
        let result = read("load_behavior = true");
        assert_eq!(&PlayerOptions::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "string",
                actual: "boolean",
                path: "load_behavior".to_string()
            }],
            result.warnings
        );

        let result = read("load_behavior = \"fast\"");
        assert_eq!(&PlayerOptions::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnsupportedValue {
                value: "fast".to_string(),
                path: "load_behavior".to_string(),
            }],
            result.warnings
        );

        fn assert_variant(variant: &str, behavior: LoadBehavior) {
            let result = read(&format!("load_behavior = \"{variant}\""));
            assert_eq!(
                &PlayerOptions {
                    load_behavior: Some(behavior),
                    ..Default::default()
                },
                result.values()
            );
            assert_eq!(Vec::<ParseWarning>::new(), result.warnings);

            // Check case-sensitivity.
            let variant = variant.to_ascii_uppercase();
            let result = read(&format!("load_behavior = \"{variant}\""));
            assert_eq!(
                &PlayerOptions {
                    load_behavior: None,
                    ..Default::default()
                },
                result.values()
            );
            assert_eq!(
                vec![ParseWarning::UnsupportedValue {
                    value: variant,
                    path: "load_behavior".to_string()
                }],
                result.warnings
            );
        }
        assert_variant("streaming", LoadBehavior::Streaming);
        assert_variant("delayed", LoadBehavior::Delayed);
        assert_variant("blocking", LoadBehavior::Blocking);
    }

    #[test]
    fn letterbox() {
        let result = read("letterbox = true");
        assert_eq!(&PlayerOptions::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "string",
                actual: "boolean",
                path: "letterbox".to_string()
            }],
            result.warnings
        );

        let result = read("letterbox = \"invalid\"");
        assert_eq!(&PlayerOptions::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnsupportedValue {
                value: "invalid".to_string(),
                path: "letterbox".to_string(),
            }],
            result.warnings
        );

        fn assert_variant(variant: &str, letterbox: Letterbox) {
            let result = read(&format!("letterbox = \"{variant}\""));
            assert_eq!(
                &PlayerOptions {
                    letterbox: Some(letterbox),
                    ..Default::default()
                },
                result.values()
            );
            assert_eq!(Vec::<ParseWarning>::new(), result.warnings);

            // Check case-sensitivity.
            let variant = variant.to_ascii_uppercase();
            let result = read(&format!("letterbox = \"{variant}\""));
            assert_eq!(
                &PlayerOptions {
                    letterbox: None,
                    ..Default::default()
                },
                result.values()
            );
            assert_eq!(
                vec![ParseWarning::UnsupportedValue {
                    value: variant,
                    path: "letterbox".to_string()
                }],
                result.warnings
            );
        }
        assert_variant("on", Letterbox::On);
        assert_variant("off", Letterbox::Off);
        assert_variant("fullscreen", Letterbox::Fullscreen);
    }

    #[test]
    fn spoof_url() {
        let result = read("spoof_url = false");
        assert_eq!(&PlayerOptions::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "string",
                actual: "boolean",
                path: "spoof_url".to_string()
            }],
            result.warnings
        );

        let result = read("spoof_url = \"invalid\"");
        assert_eq!(&PlayerOptions::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnsupportedValue {
                value: "invalid".to_string(),
                path: "spoof_url".to_string(),
            }],
            result.warnings
        );

        let result = read("spoof_url = \"https://ruffle.rs/spoofed_file.swf\"");
        assert_eq!(
            &PlayerOptions {
                spoof_url: Some(Url::parse("https://ruffle.rs/spoofed_file.swf").unwrap()),
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }

    #[test]
    fn version() {
        let result = read("version = \"air\"");
        assert_eq!(&PlayerOptions::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "integer",
                actual: "string",
                path: "version".to_string()
            }],
            result.warnings
        );

        let result = read("version = 26");
        assert_eq!(
            &PlayerOptions {
                player_version: Some(26),
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }

    #[test]
    fn runtime() {
        let result = read("runtime = true");
        assert_eq!(&PlayerOptions::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "string",
                actual: "boolean",
                path: "runtime".to_string()
            }],
            result.warnings
        );

        let result = read("runtime = \"invalid\"");
        assert_eq!(&PlayerOptions::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnsupportedValue {
                value: "invalid".to_string(),
                path: "runtime".to_string(),
            }],
            result.warnings
        );

        fn assert_variant(variant: &str, runtime: PlayerRuntime) {
            let result = read(&format!("runtime = \"{variant}\""));
            assert_eq!(
                &PlayerOptions {
                    player_runtime: Some(runtime),
                    ..Default::default()
                },
                result.values()
            );
            assert_eq!(Vec::<ParseWarning>::new(), result.warnings);

            // Check case-sensitivity.
            let variant = variant.to_ascii_uppercase();
            let result = read(&format!("runtime = \"{variant}\""));
            assert_eq!(
                &PlayerOptions {
                    player_runtime: None,
                    ..Default::default()
                },
                result.values()
            );
            assert_eq!(
                vec![ParseWarning::UnsupportedValue {
                    value: variant,
                    path: "runtime".to_string()
                }],
                result.warnings
            );
        }
        assert_variant("flash_player", PlayerRuntime::FlashPlayer);
        assert_variant("air", PlayerRuntime::AIR);
    }

    #[test]
    fn frame_rate() {
        let result = read("frame_rate = true");
        assert_eq!(&PlayerOptions::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "float or integer",
                actual: "boolean",
                path: "frame_rate".to_string()
            }],
            result.warnings
        );

        let result = read("frame_rate = 30");
        assert_eq!(
            &PlayerOptions {
                frame_rate: Some(30.0),
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);

        let result = read("frame_rate = 5.0");
        assert_eq!(
            &PlayerOptions {
                frame_rate: Some(5.0),
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }

    #[test]
    fn mock_external_interface() {
        let result = read("mock_external_interface = 1");
        assert_eq!(&PlayerOptions::default(), result.values());
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "boolean",
                actual: "integer",
                path: "mock_external_interface".to_string()
            }],
            result.warnings
        );

        let result = read("mock_external_interface = true");
        assert_eq!(
            &PlayerOptions {
                dummy_external_interface: Some(true),
                ..Default::default()
            },
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }
}
