use anyhow::Result;
use approx::assert_relative_eq;
use regex::Regex;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::{PlayerBuilder, ViewportDimensions};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Deserialize)]
#[serde(default)]
pub struct TestOptions {
    pub num_frames: u32,
    pub output_path: PathBuf,
    pub sleep_to_meet_frame_rate: bool,
    pub image: bool,
    pub ignore: bool,
    pub approximations: Option<Approximations>,
    pub player_options: PlayerOptions,
}

impl Default for TestOptions {
    fn default() -> Self {
        Self {
            num_frames: 1,
            output_path: PathBuf::from("output.txt"),
            sleep_to_meet_frame_rate: false,
            image: false,
            ignore: false,
            approximations: None,
            player_options: PlayerOptions::default(),
        }
    }
}

impl TestOptions {
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut options: TestOptions = toml::from_str(&fs::read_to_string(path)?)?;

        // Use this opportunity to handle any implicit requirements
        if options.image {
            options.player_options.with_renderer = true;
        }

        Ok(options)
    }

    pub fn output_path(&self, test_directory: &Path) -> PathBuf {
        test_directory.join(&self.output_path)
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
    with_renderer: bool,
}

impl PlayerOptions {
    pub fn setup(
        &self,
        mut player_builder: PlayerBuilder,
        movie: &SwfMovie,
    ) -> Result<PlayerBuilder> {
        if let Some(max_execution_duration) = self.max_execution_duration {
            player_builder = player_builder.with_max_execution_duration(max_execution_duration);
        }

        #[cfg_attr(not(feature = "render"), allow(unused))]
        let (width, height) = if let Some(viewport_dimensions) = self.viewport_dimensions {
            player_builder = player_builder.with_viewport_dimensions(
                viewport_dimensions.width,
                viewport_dimensions.height,
                viewport_dimensions.scale_factor,
            );
            (viewport_dimensions.width, viewport_dimensions.height)
        } else {
            (
                movie.width().to_pixels() as u32,
                movie.height().to_pixels() as u32,
            )
        };

        #[cfg(feature = "render")]
        if self.with_renderer {
            use anyhow::anyhow;
            use ruffle_render_wgpu::backend::WgpuRenderBackend;
            use ruffle_render_wgpu::{target::TextureTarget, wgpu};
            use std::sync::Arc;

            let instance = wgpu::Instance::new(Default::default());

            let descriptors =
                futures::executor::block_on(WgpuRenderBackend::<TextureTarget>::build_descriptors(
                    wgpu::Backends::all(),
                    instance,
                    None,
                    Default::default(),
                    None,
                ))
                .map_err(|e| anyhow!(e.to_string()))?;

            let target = TextureTarget::new(&descriptors.device, (width, height))
                .map_err(|e| anyhow!(e.to_string()))?;

            player_builder = player_builder.with_renderer(
                WgpuRenderBackend::new(Arc::new(descriptors), target, 4)
                    .map_err(|e| anyhow!(e.to_string()))?,
            );
        }

        Ok(player_builder)
    }

    pub fn can_run(&self) -> bool {
        if self.with_renderer && !cfg!(feature = "render") {
            return false;
        }
        true
    }
}
