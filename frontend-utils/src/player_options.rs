mod read;
pub use read::read_player_options;

use ruffle_core::config::Letterbox;
use ruffle_core::{LoadBehavior, PlayerRuntime, StageAlign, StageScaleMode};
use ruffle_render::quality::StageQuality;
use std::time::Duration;
use url::Url;

#[derive(Default, Debug, PartialEq, Clone)]
pub struct PlayerOptions {
    pub parameters: Vec<(String, String)>,
    pub max_execution_duration: Option<Duration>,
    pub base: Option<Url>,
    pub quality: Option<StageQuality>,
    pub align: Option<StageAlign>,
    pub force_align: Option<bool>,
    pub scale: Option<StageScaleMode>,
    pub force_scale: Option<bool>,
    pub upgrade_to_https: Option<bool>,
    pub load_behavior: Option<LoadBehavior>,
    pub letterbox: Option<Letterbox>,
    pub spoof_url: Option<Url>,
    pub referer: Option<Url>,
    pub cookie: Option<String>,
    pub player_version: Option<u8>,
    pub player_runtime: Option<PlayerRuntime>,
    pub frame_rate: Option<f64>,
    pub dummy_external_interface: Option<bool>,
}

impl PlayerOptions {
    pub fn or(&self, other: &Self) -> Self {
        let mut parameters = other.parameters.clone();
        parameters.append(&mut self.parameters.clone());
        Self {
            parameters,
            max_execution_duration: self.max_execution_duration.or(other.max_execution_duration),
            base: self.base.clone().or_else(|| other.base.clone()),
            quality: self.quality.or(other.quality),
            align: self.align.or(other.align),
            force_align: self.force_align.or(other.force_align),
            scale: self.scale.or(other.scale),
            force_scale: self.force_scale.or(other.force_scale),
            upgrade_to_https: self.upgrade_to_https.or(other.upgrade_to_https),
            load_behavior: self.load_behavior.or(other.load_behavior),
            letterbox: self.letterbox.or(other.letterbox),
            spoof_url: self.spoof_url.clone().or_else(|| other.spoof_url.clone()),
            referer: self.referer.clone().or_else(|| other.referer.clone()),
            cookie: self.cookie.clone().or_else(|| other.cookie.clone()),
            player_version: self.player_version.or(other.player_version),
            player_runtime: self.player_runtime.or(other.player_runtime),
            frame_rate: self.frame_rate.or(other.frame_rate),
            dummy_external_interface: self
                .dummy_external_interface
                .or(other.dummy_external_interface),
        }
    }
}
