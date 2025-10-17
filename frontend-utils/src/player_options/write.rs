use std::time::Duration;

use crate::{parse::DocumentHolder, player_options::PlayerOptions};
use ruffle_core::{config::Letterbox, LoadBehavior, PlayerRuntime, StageAlign, StageScaleMode};
use ruffle_render::quality::StageQuality;
use toml_edit::value;
use url::Url;

pub struct PlayerOptionsWriter<'a>(&'a mut DocumentHolder<PlayerOptions>);

impl<'a> PlayerOptionsWriter<'a> {
    pub fn new(preferences: &'a mut DocumentHolder<PlayerOptions>) -> Self {
        Self(preferences)
    }

    pub fn remove_parameters(&mut self) {
        self.0.edit(|options, toml_document| {
            toml_document.remove("parameters");
            options.parameters.clear();
        })
    }

    pub fn add_parameter(&mut self, key: impl Into<String>, string_value: impl Into<String>) {
        let key = key.into();
        let string_value = string_value.into();
        self.0.edit(|options, toml_document| {
            toml_document["parameters"][&key] = value(&string_value);

            let index = options.parameters.iter().position(|(k, _)| k == &key);
            if let Some(index) = index {
                let _ = std::mem::replace(&mut options.parameters[index], (key, string_value));
            } else {
                options.parameters.push((key, string_value));
            }
        })
    }

    pub fn set_max_execution_duration(&mut self, max_execution_duration: Option<Duration>) {
        self.0.edit(|options, toml_document| {
            if let Some(max_execution_duration) = max_execution_duration.map(|d| d.as_secs_f64()) {
                toml_document["script_timeout"] = value(max_execution_duration);
            } else {
                toml_document.remove("script_timeout");
            }

            options.max_execution_duration = max_execution_duration;
        })
    }

    pub fn set_base(&mut self, base: Option<Url>) {
        self.0.edit(|options, toml_document| {
            if let Some(base) = &base {
                toml_document["base_url"] = value(base.to_string());
            } else {
                toml_document.remove("base_url");
            }

            options.base = base;
        })
    }

    pub fn set_quality(&mut self, quality: Option<StageQuality>) {
        self.0.edit(|options, toml_document| {
            if let Some(quality) = quality {
                toml_document["quality"] = value(quality.to_string());
            } else {
                toml_document.remove("quality");
            }

            options.quality = quality;
        })
    }

    pub fn set_align(&mut self, align: Option<StageAlign>) {
        self.0.edit(|options, toml_document| {
            if let Some(align) = align {
                toml_document["align"] = value(align.to_string());
            } else {
                toml_document.remove("align");
            }

            options.align = align;
        })
    }

    pub fn set_force_align(&mut self, force_align: Option<bool>) {
        self.0.edit(|options, toml_document| {
            if let Some(force_align) = force_align {
                toml_document["force_align"] = value(force_align);
            } else {
                toml_document.remove("force_align");
            }

            options.force_align = force_align;
        })
    }

    pub fn set_scale(&mut self, scale: Option<StageScaleMode>) {
        self.0.edit(|options, toml_document| {
            if let Some(scale) = scale {
                toml_document["scale_mode"] = value(scale.to_string());
            } else {
                toml_document.remove("scale_mode");
            }

            options.scale = scale;
        })
    }

    pub fn set_force_scale(&mut self, force_scale: Option<bool>) {
        self.0.edit(|options, toml_document| {
            if let Some(force_scale) = force_scale {
                toml_document["force_scale_mode"] = value(force_scale);
            } else {
                toml_document.remove("force_scale_mode");
            }

            options.force_scale = force_scale;
        })
    }

    pub fn set_upgrade_to_https(&mut self, upgrade_to_https: Option<bool>) {
        self.0.edit(|options, toml_document| {
            if let Some(upgrade_to_https) = upgrade_to_https {
                toml_document["upgrade_http_to_https"] = value(upgrade_to_https);
            } else {
                toml_document.remove("upgrade_http_to_https");
            }

            options.upgrade_to_https = upgrade_to_https;
        })
    }

    pub fn set_load_behavior(&mut self, load_behavior: Option<LoadBehavior>) {
        self.0.edit(|options, toml_document| {
            if let Some(load_behavior) = load_behavior {
                toml_document["load_behavior"] = value(load_behavior.to_string());
            } else {
                toml_document.remove("load_behavior");
            }

            options.load_behavior = load_behavior;
        })
    }

    pub fn set_letterbox(&mut self, letterbox: Option<Letterbox>) {
        self.0.edit(|options, toml_document| {
            if let Some(letterbox) = letterbox {
                toml_document["letterbox"] = value(letterbox.to_string());
            } else {
                toml_document.remove("letterbox");
            }

            options.letterbox = letterbox;
        })
    }

    pub fn set_spoof_url(&mut self, spoof_url: Option<Url>) {
        self.0.edit(|options, toml_document| {
            if let Some(spoof_url) = &spoof_url {
                toml_document["spoof_url"] = value(spoof_url.to_string());
            } else {
                toml_document.remove("spoof_url");
            }

            options.spoof_url = spoof_url;
        })
    }

    pub fn set_player_version(&mut self, player_version: Option<u8>) {
        self.0.edit(|options, toml_document| {
            if let Some(player_version) = player_version {
                toml_document["version"] = value(player_version as i64);
            } else {
                toml_document.remove("version");
            }

            options.player_version = player_version;
        })
    }

    pub fn set_player_runtime(&mut self, player_runtime: Option<PlayerRuntime>) {
        self.0.edit(|options, toml_document| {
            if let Some(player_runtime) = player_runtime {
                toml_document["runtime"] = value(player_runtime.to_string());
            } else {
                toml_document.remove("runtime");
            }

            options.player_runtime = player_runtime;
        })
    }

    pub fn set_frame_rate(&mut self, frame_rate: Option<f64>) {
        self.0.edit(|options, toml_document| {
            if let Some(frame_rate) = frame_rate {
                toml_document["frame_rate"] = value(frame_rate);
            } else {
                toml_document.remove("frame_rate");
            }

            options.frame_rate = frame_rate;
        })
    }

    pub fn set_dummy_external_interface(&mut self, dummy_external_interface: Option<bool>) {
        self.0.edit(|options, toml_document| {
            if let Some(dummy_external_interface) = dummy_external_interface {
                toml_document["mock_external_interface"] = value(dummy_external_interface);
            } else {
                toml_document.remove("mock_external_interface");
            }

            options.dummy_external_interface = dummy_external_interface;
        })
    }
}

pub fn write_player_options(writer: &mut PlayerOptionsWriter, options: &PlayerOptions) {
    writer.remove_parameters();
    for (key, value) in &options.parameters {
        writer.add_parameter(key, value);
    }

    writer.set_max_execution_duration(options.max_execution_duration);
    writer.set_base(options.base.clone());
    writer.set_quality(options.quality);
    writer.set_align(options.align);
    writer.set_force_align(options.force_align);
    writer.set_scale(options.scale);
    writer.set_force_scale(options.force_scale);
    writer.set_upgrade_to_https(options.upgrade_to_https);
    writer.set_load_behavior(options.load_behavior);
    writer.set_letterbox(options.letterbox);
    writer.set_spoof_url(options.spoof_url.clone());
    writer.set_player_version(options.player_version);
    writer.set_player_runtime(options.player_runtime);
    writer.set_frame_rate(options.frame_rate);
    writer.set_dummy_external_interface(options.dummy_external_interface);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{ParseContext, ParseDetails};
    use crate::player_options::read_player_options;
    use toml_edit::DocumentMut;

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

    crate::define_serialization_test_helpers!(read, PlayerOptions, PlayerOptionsWriter);

    #[test]
    fn parameters_add() {
        test(
            "",
            |writer| writer.add_parameter("param-a", "value-a"),
            "parameters = { param-a = \"value-a\" }\n",
        );
    }

    #[test]
    fn parameters_add2() {
        test(
            "[parameters]\nparam-a = \"value-a\"\n",
            |writer| writer.add_parameter("param-b", "value-b"),
            "[parameters]\nparam-a = \"value-a\"\nparam-b = \"value-b\"\n",
        );
    }

    #[test]
    fn max_execution_duration() {
        test(
            "",
            |writer| writer.set_max_execution_duration(Some(Duration::from_millis(1337))),
            "script_timeout = 1.337\n",
        );
    }

    #[test]
    fn max_execution_duration_remove() {
        test(
            "script_timeout = 1.337\n",
            |writer| writer.set_max_execution_duration(None),
            "",
        );
    }

    #[test]
    fn max_execution_duration_replace() {
        test(
            "script_timeout = \"what\"\n",
            |writer| writer.set_max_execution_duration(Some(Duration::from_millis(42))),
            "script_timeout = 0.042\n",
        );
    }

    #[test]
    fn base() {
        let url = Url::parse("https://example.com/test").unwrap();
        test(
            "",
            |writer| writer.set_base(Some(url)),
            "base_url = \"https://example.com/test\"\n",
        );
    }

    #[test]
    fn base_remove() {
        test(
            "base_url = \"https://example.com/test\"\n",
            |writer| writer.set_base(None),
            "",
        );
    }

    #[test]
    fn quality() {
        test(
            "",
            |writer| writer.set_quality(Some(StageQuality::Low)),
            "quality = \"low\"\n",
        );
    }

    #[test]
    fn quality_variants() {
        macro_rules! test_variant {
            ($actual:expr, $expected:expr) => {
                test(
                    "",
                    |writer| writer.set_quality(Some($actual)),
                    &format!("quality = \"{}\"\n", $expected),
                );
            };
        }

        test_variant!(StageQuality::Low, "low");
        test_variant!(StageQuality::Medium, "medium");
        test_variant!(StageQuality::High, "high");
        test_variant!(StageQuality::High8x8, "8x8");
        test_variant!(StageQuality::High8x8Linear, "8x8linear");
        test_variant!(StageQuality::High16x16, "16x16");
        test_variant!(StageQuality::High16x16Linear, "16x16linear");
    }

    #[test]
    fn quality_remove() {
        test(
            "quality = \"unknown\"\n",
            |writer| writer.set_quality(None),
            "",
        );
    }

    #[test]
    fn align() {
        test(
            "",
            |writer| writer.set_align(Some(StageAlign::RIGHT | StageAlign::TOP)),
            "align = \"top_right\"\n",
        );
    }

    #[test]
    fn align_variants() {
        macro_rules! test_variant {
            ($actual:expr, $expected:expr) => {
                test(
                    "",
                    |writer| writer.set_align(Some($actual)),
                    &format!("align = \"{}\"\n", $expected),
                );
            };
        }

        test_variant!(StageAlign::TOP, "top");
        test_variant!(StageAlign::BOTTOM, "bottom");
        test_variant!(StageAlign::RIGHT, "right");
        test_variant!(StageAlign::LEFT, "left");
        test_variant!(StageAlign::empty(), "center");
        test_variant!(StageAlign::TOP | StageAlign::LEFT, "top_left");
        test_variant!(StageAlign::TOP | StageAlign::RIGHT, "top_right");
        test_variant!(StageAlign::BOTTOM | StageAlign::LEFT, "bottom_left");
        test_variant!(StageAlign::BOTTOM | StageAlign::RIGHT, "bottom_right");
    }

    #[test]
    fn align_remove() {
        test("align = \"unknown\"\n", |writer| writer.set_align(None), "");
    }

    #[test]
    fn force_align() {
        test(
            "",
            |writer| writer.set_force_align(Some(true)),
            "force_align = true\n",
        );
    }

    #[test]
    fn force_align_remove() {
        test(
            "force_align = false\n",
            |writer| writer.set_force_align(None),
            "",
        );
    }

    #[test]
    fn scale() {
        test(
            "",
            |writer| writer.set_scale(Some(StageScaleMode::NoBorder)),
            "scale_mode = \"no_border\"\n",
        );
    }

    #[test]
    fn scale_variants() {
        macro_rules! test_variant {
            ($actual:expr, $expected:expr) => {
                test(
                    "",
                    |writer| writer.set_scale(Some($actual)),
                    &format!("scale_mode = \"{}\"\n", $expected),
                );
            };
        }

        test_variant!(StageScaleMode::NoBorder, "no_border");
        test_variant!(StageScaleMode::ExactFit, "exact_fit");
        test_variant!(StageScaleMode::NoScale, "no_scale");
        test_variant!(StageScaleMode::ShowAll, "show_all");
    }

    #[test]
    fn scale_remove() {
        test(
            "scale_mode = \"unknown\"\n",
            |writer| writer.set_scale(None),
            "",
        );
    }

    #[test]
    fn force_scale() {
        test(
            "",
            |writer: &mut PlayerOptionsWriter<'_>| writer.set_force_scale(Some(true)),
            "force_scale_mode = true\n",
        );
    }

    #[test]
    fn force_scale_remove() {
        test(
            "force_scale_mode = false\n",
            |writer| writer.set_force_scale(None),
            "",
        );
    }

    #[test]
    fn upgrade_to_https() {
        test(
            "",
            |writer: &mut PlayerOptionsWriter<'_>| writer.set_upgrade_to_https(Some(true)),
            "upgrade_http_to_https = true\n",
        );
    }

    #[test]
    fn upgrade_to_https_remove() {
        test(
            "upgrade_http_to_https = false\n",
            |writer| writer.set_upgrade_to_https(None),
            "",
        );
    }

    #[test]
    fn load_behavior() {
        test(
            "",
            |writer| writer.set_load_behavior(Some(LoadBehavior::Streaming)),
            "load_behavior = \"streaming\"\n",
        );
    }

    #[test]
    fn load_behavior_variants() {
        macro_rules! test_variant {
            ($actual:expr, $expected:expr) => {
                test(
                    "",
                    |writer| writer.set_load_behavior(Some($actual)),
                    &format!("load_behavior = \"{}\"\n", $expected),
                );
            };
        }

        test_variant!(LoadBehavior::Streaming, "streaming");
        test_variant!(LoadBehavior::Delayed, "delayed");
        test_variant!(LoadBehavior::Blocking, "blocking");
    }

    #[test]
    fn load_behavior_remove() {
        test(
            "load_behavior = \"unknown\"\n",
            |writer| writer.set_load_behavior(None),
            "",
        );
    }

    #[test]
    fn letterbox() {
        test(
            "",
            |writer| writer.set_letterbox(Some(Letterbox::Fullscreen)),
            "letterbox = \"fullscreen\"\n",
        );
    }

    #[test]
    fn letterbox_variants() {
        macro_rules! test_variant {
            ($actual:expr, $expected:expr) => {
                test(
                    "",
                    |writer| writer.set_letterbox(Some($actual)),
                    &format!("letterbox = \"{}\"\n", $expected),
                );
            };
        }

        test_variant!(Letterbox::On, "on");
        test_variant!(Letterbox::Off, "off");
        test_variant!(Letterbox::Fullscreen, "fullscreen");
    }

    #[test]
    fn letterbox_remove() {
        test(
            "letterbox = \"unknown\"\n",
            |writer| writer.set_letterbox(None),
            "",
        );
    }

    #[test]
    fn spoof_url() {
        let url = Url::parse("https://example.com/test").unwrap();
        test(
            "",
            |writer| writer.set_spoof_url(Some(url)),
            "spoof_url = \"https://example.com/test\"\n",
        );
    }

    #[test]
    fn spoof_url_remove() {
        test(
            "spoof_url = \"https://example.com/test\"\n",
            |writer| writer.set_spoof_url(None),
            "",
        );
    }

    #[test]
    fn player_version() {
        test(
            "",
            |writer| writer.set_player_version(Some(18)),
            "version = 18\n",
        );
    }

    #[test]
    fn player_version_remove() {
        test(
            "version = \"unknown\"\n",
            |writer| writer.set_player_version(None),
            "",
        );
    }

    #[test]
    fn player_runtime_air() {
        test(
            "",
            |writer| writer.set_player_runtime(Some(PlayerRuntime::AIR)),
            "runtime = \"air\"\n",
        );
    }

    #[test]
    fn player_runtime_fp() {
        test(
            "",
            |writer: &mut PlayerOptionsWriter<'_>| {
                writer.set_player_runtime(Some(PlayerRuntime::FlashPlayer))
            },
            "runtime = \"flash_player\"\n",
        );
    }

    #[test]
    fn player_runtime_remove() {
        test(
            "runtime = \"unknown\"\n",
            |writer| writer.set_player_runtime(None),
            "",
        );
    }

    #[test]
    fn frame_rate() {
        test(
            "",
            |writer| writer.set_frame_rate(Some(29.987)),
            "frame_rate = 29.987\n",
        );
    }

    #[test]
    fn frame_rate_remove() {
        test(
            "frame_rate = \"unknown\"\n",
            |writer| writer.set_frame_rate(None),
            "",
        );
    }

    #[test]
    fn dummy_external_interface() {
        test(
            "",
            |writer: &mut PlayerOptionsWriter<'_>| writer.set_dummy_external_interface(Some(true)),
            "mock_external_interface = true\n",
        );
    }

    #[test]
    fn dummy_external_interface_remove() {
        test(
            "mock_external_interface = false\n",
            |writer| writer.set_dummy_external_interface(None),
            "",
        );
    }
}
