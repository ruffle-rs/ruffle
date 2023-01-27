use approx::assert_relative_eq;
use regex::Regex;
use ruffle_core::{Player, ViewportDimensions};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Deserialize)]
#[serde(default)]
pub struct TestOptions {
    pub num_frames: u32,
    pub sleep_to_meet_frame_rate: bool,
    pub image: bool,
    pub ignore: bool,
    pub approximations: Option<Approximations>,
    pub player_options: Option<PlayerOptions>,
}

impl Default for TestOptions {
    fn default() -> Self {
        Self {
            num_frames: 1,
            sleep_to_meet_frame_rate: false,
            image: false,
            ignore: false,
            approximations: None,
            player_options: None,
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct Approximations {
    number_patterns: Vec<String>,
    epsilon: Option<f64>,
    max_relative: Option<f64>,
}

impl Approximations {
    pub fn compare(&self, actual: f64, expected: f64) {
        match (self.epsilon, self.max_relative) {
            (Some(epsilon), Some(max_relative)) => assert_relative_eq!(
                actual,
                expected,
                epsilon = epsilon,
                max_relative = max_relative
            ),
            (Some(epsilon), None) => assert_relative_eq!(actual, expected, epsilon = epsilon),
            (None, Some(max_relative)) => {
                assert_relative_eq!(actual, expected, max_relative = max_relative)
            }
            (None, None) => assert_relative_eq!(actual, expected),
        }
    }

    pub fn number_patterns(&self) -> Vec<Regex> {
        self.number_patterns
            .iter()
            .map(|p| Regex::new(&p).unwrap())
            .collect()
    }
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct PlayerOptions {
    max_execution_duration: Option<Duration>,
    viewport_dimensions: Option<ViewportDimensions>,
}

impl PlayerOptions {
    pub fn setup(&self, player: Arc<Mutex<Player>>) {
        if let Some(max_execution_duration) = self.max_execution_duration {
            player
                .lock()
                .unwrap()
                .set_max_execution_duration(max_execution_duration);
        }
        if let Some(viewport_dimensions) = self.viewport_dimensions {
            player
                .lock()
                .unwrap()
                .set_viewport_dimensions(viewport_dimensions);
        }
    }
}
