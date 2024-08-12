use crate::backends::TestAudioBackend;
use crate::environment::{Environment, RenderInterface};
use crate::image_trigger::ImageTrigger;
use crate::util::write_image;
use anyhow::{anyhow, Result};
use approx::relative_eq;
use image::ImageFormat;
use regex::Regex;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::{PlayerBuilder, PlayerRuntime, ViewportDimensions};
use ruffle_render::backend::RenderBackend;
use ruffle_render::quality::StageQuality;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use vfs::VfsPath;

#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TestOptions {
    pub num_frames: Option<u32>,
    pub num_ticks: Option<u32>,
    pub tick_rate: Option<f64>,
    pub output_path: String,
    pub sleep_to_meet_frame_rate: bool,
    pub image_comparisons: HashMap<String, ImageComparison>,
    pub ignore: bool,
    pub known_failure: bool,
    pub approximations: Option<Approximations>,
    pub player_options: PlayerOptions,
    pub log_fetch: bool,
    pub required_features: RequiredFeatures,
    pub fonts: HashMap<String, FontOptions>,
}

impl Default for TestOptions {
    fn default() -> Self {
        Self {
            num_frames: None,
            num_ticks: None,
            tick_rate: None,
            output_path: "output.txt".to_string(),
            sleep_to_meet_frame_rate: false,
            image_comparisons: Default::default(),
            ignore: false,
            known_failure: false,
            approximations: None,
            player_options: PlayerOptions::default(),
            log_fetch: false,
            required_features: RequiredFeatures::default(),
            fonts: Default::default(),
        }
    }
}

impl TestOptions {
    pub fn read(path: &VfsPath) -> Result<Self> {
        let result: Self = toml::from_str(&path.read_to_string()?)?;
        result.validate()?;
        Ok(result)
    }

    fn validate(&self) -> Result<()> {
        if !self.image_comparisons.is_empty() {
            let mut seen_triggers = HashSet::new();
            for comparison in self.image_comparisons.values() {
                if comparison.trigger != ImageTrigger::FsCommand
                    && !seen_triggers.insert(comparison.trigger)
                {
                    return Err(anyhow!(
                        "Multiple captures are set to trigger {:?}. This likely isn't intended!",
                        comparison.trigger
                    ));
                }
            }
        }

        Ok(())
    }

    pub fn output_path(&self, test_directory: &VfsPath) -> Result<VfsPath> {
        Ok(test_directory.join(&self.output_path)?)
    }
}

#[derive(Clone, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Approximations {
    number_patterns: Vec<String>,
    epsilon: Option<f64>,
    max_relative: Option<f64>,
}

impl Approximations {
    pub fn compare(&self, actual: f64, expected: f64) -> Result<()> {
        let result = match (self.epsilon, self.max_relative) {
            (Some(epsilon), Some(max_relative)) => relative_eq!(
                actual,
                expected,
                epsilon = epsilon,
                max_relative = max_relative
            ),
            (Some(epsilon), None) => relative_eq!(actual, expected, epsilon = epsilon),
            (None, Some(max_relative)) => {
                relative_eq!(actual, expected, max_relative = max_relative)
            }
            (None, None) => relative_eq!(actual, expected),
        };

        if result {
            Ok(())
        } else {
            Err(anyhow!(
                "Approximation failed: expected {}, found {}. Episilon = {:?}, Max Relative = {:?}",
                expected,
                actual,
                self.epsilon,
                self.max_relative
            ))
        }
    }

    pub fn number_patterns(&self) -> Vec<Regex> {
        self.number_patterns
            .iter()
            .map(|p| Regex::new(p).unwrap())
            .collect()
    }
}

#[derive(Clone, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct RequiredFeatures {
    lzma: bool,
    jpegxr: bool,
}

impl RequiredFeatures {
    pub fn can_run(&self) -> bool {
        (!self.lzma || cfg!(feature = "lzma")) && (!self.jpegxr || cfg!(feature = "jpegxr"))
    }
}

#[derive(Clone, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct PlayerOptions {
    max_execution_duration: Option<Duration>,
    viewport_dimensions: Option<ViewportDimensions>,
    with_renderer: Option<RenderOptions>,
    with_audio: bool,
    with_video: bool,
    runtime: PlayerRuntime,
}

impl PlayerOptions {
    pub fn setup(&self, mut player_builder: PlayerBuilder) -> Result<PlayerBuilder> {
        if let Some(max_execution_duration) = self.max_execution_duration {
            player_builder = player_builder.with_max_execution_duration(max_execution_duration);
        }

        if let Some(render_options) = &self.with_renderer {
            player_builder = player_builder.with_quality(match render_options.sample_count {
                16 => StageQuality::High16x16,
                8 => StageQuality::High8x8,
                4 => StageQuality::High,
                2 => StageQuality::Medium,
                _ => StageQuality::Low,
            });
        }

        if self.with_audio {
            player_builder = player_builder.with_audio(TestAudioBackend::default());
        }

        player_builder = player_builder.with_player_runtime(self.runtime);

        if self.with_video {
            #[cfg(feature = "ruffle_video_external")]
            {
                use ruffle_video_external::backend::ExternalVideoBackend;
                let openh264_path = ExternalVideoBackend::get_openh264()
                    .map_err(|e| anyhow!("Couldn't get OpenH264: {}", e))?;

                player_builder =
                    player_builder.with_video(ExternalVideoBackend::new(Some(openh264_path)));
            }

            #[cfg(all(
                not(feature = "ruffle_video_external"),
                feature = "ruffle_video_software"
            ))]
            {
                player_builder = player_builder
                    .with_video(ruffle_video_software::backend::SoftwareVideoBackend::new());
            }
        }

        Ok(player_builder)
    }

    pub fn can_run(&self, check_renderer: bool, environment: &impl Environment) -> bool {
        if let Some(render) = &self.with_renderer {
            // If we don't actually want to check the renderer (ie we're just listing potential tests),
            // don't spend the cost to create it
            if check_renderer && !render.optional && !environment.is_render_supported(render) {
                return false;
            }
        }
        true
    }

    pub fn viewport_dimensions(&self, movie: &SwfMovie) -> ViewportDimensions {
        self.viewport_dimensions
            .unwrap_or_else(|| ViewportDimensions {
                width: movie.width().to_pixels() as u32,
                height: movie.height().to_pixels() as u32,
                scale_factor: 1.0,
            })
    }

    pub fn create_renderer(
        &self,
        environment: &impl Environment,
        dimensions: ViewportDimensions,
    ) -> Option<(Box<dyn RenderInterface>, Box<dyn RenderBackend>)> {
        if self.with_renderer.is_some() {
            environment.create_renderer(dimensions.width, dimensions.height)
        } else {
            None
        }
    }
}

#[derive(Deserialize, Default, Clone, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct ImageComparison {
    tolerance: u8,
    max_outliers: usize,
    pub trigger: ImageTrigger,
}

fn calc_difference(lhs: u8, rhs: u8) -> u8 {
    (lhs as i16 - rhs as i16).unsigned_abs() as u8
}

impl ImageComparison {
    pub fn test(
        &self,
        name: &str,
        actual_image: image::RgbaImage,
        expected_image: image::RgbaImage,
        test_path: &VfsPath,
        environment_name: String,
        known_failure: bool,
    ) -> Result<()> {
        use anyhow::Context;

        let save_actual_image = || {
            if !known_failure {
                // If we're expecting failure, spamming files isn't productive.
                write_image(
                    &test_path.join(format!("{name}.actual-{environment_name}.png"))?,
                    &actual_image,
                    ImageFormat::Png,
                )
            } else {
                Ok(())
            }
        };

        if actual_image.width() != expected_image.width()
            || actual_image.height() != expected_image.height()
        {
            save_actual_image()?;
            return Err(anyhow!(
                "'{}' image is not the right size. Expected = {}x{}, actual = {}x{}.",
                name,
                expected_image.width(),
                expected_image.height(),
                actual_image.width(),
                actual_image.height()
            ));
        }

        let mut is_alpha_different = false;

        let difference_data: Vec<u8> = expected_image
            .as_raw()
            .chunks_exact(4)
            .zip(actual_image.as_raw().chunks_exact(4))
            .flat_map(|(cmp_chunk, data_chunk)| {
                if cmp_chunk[3] != data_chunk[3] {
                    is_alpha_different = true;
                }

                [
                    calc_difference(cmp_chunk[0], data_chunk[0]),
                    calc_difference(cmp_chunk[1], data_chunk[1]),
                    calc_difference(cmp_chunk[2], data_chunk[2]),
                    calc_difference(cmp_chunk[3], data_chunk[3]),
                ]
            })
            .collect();

        let outliers: usize = difference_data
            .chunks_exact(4)
            .map(|colors| {
                (colors[0] > self.tolerance) as usize
                    + (colors[1] > self.tolerance) as usize
                    + (colors[2] > self.tolerance) as usize
                    + (colors[3] > self.tolerance) as usize
            })
            .sum();

        let max_difference = difference_data
            .chunks_exact(4)
            .map(|colors| colors[0].max(colors[1]).max(colors[2]).max(colors[3]))
            .max()
            .unwrap();

        if outliers > self.max_outliers {
            save_actual_image()?;

            let mut difference_color = Vec::with_capacity(
                actual_image.width() as usize * actual_image.height() as usize * 3,
            );
            for p in difference_data.chunks_exact(4) {
                difference_color.extend_from_slice(&p[..3]);
            }

            if !known_failure {
                // If we're expecting failure, spamming files isn't productive.
                let difference_image = image::RgbImage::from_raw(
                    actual_image.width(),
                    actual_image.height(),
                    difference_color,
                )
                .context("Couldn't create color difference image")?;
                write_image(
                    &test_path.join(format!("{name}.difference-color-{environment_name}.png"))?,
                    &difference_image,
                    ImageFormat::Png,
                )?;
            }

            if is_alpha_different {
                let mut difference_alpha = Vec::with_capacity(
                    actual_image.width() as usize * actual_image.height() as usize,
                );
                for p in difference_data.chunks_exact(4) {
                    difference_alpha.push(p[3])
                }

                if !known_failure {
                    // If we're expecting failure, spamming files isn't productive.
                    let difference_image = image::GrayImage::from_raw(
                        actual_image.width(),
                        actual_image.height(),
                        difference_alpha,
                    )
                    .context("Couldn't create alpha difference image")?;
                    write_image(
                        &test_path
                            .join(format!("{name}.difference-alpha-{environment_name}.png"))?,
                        &difference_image,
                        ImageFormat::Png,
                    )?;
                }
            }

            return Err(anyhow!(
                "Image '{}' failed: Number of outliers ({}) is bigger than allowed limit of {}. Max difference is {}",
                name,
                outliers,
                self.max_outliers,
                max_difference
            ));
        } else {
            println!("Image '{name}' succeeded: {outliers} outliers found, max difference {max_difference}",);
        }

        Ok(())
    }
}

#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RenderOptions {
    optional: bool,
    pub sample_count: u32,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            optional: false,
            sample_count: 1,
        }
    }
}

#[derive(Deserialize, Default, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct FontOptions {
    pub family: String,
    pub path: String,
    pub bold: bool,
    pub italic: bool,
}
