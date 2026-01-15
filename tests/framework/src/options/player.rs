use crate::backends::TestAudioBackend;
use crate::environment::{Environment, RenderInterface};
use crate::options::RenderOptions;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::{PlayerBuilder, PlayerMode, PlayerRuntime};
use ruffle_render::backend::{RenderBackend, ViewportDimensions};
use ruffle_render::quality::StageQuality;
use serde::Deserialize;
use std::time::Duration;

#[derive(Clone, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct PlayerOptions {
    max_execution_duration: Option<Duration>,
    viewport_dimensions: Option<ViewportDimensions>,
    with_renderer: Option<RenderOptions>,
    with_audio: bool,
    with_video: bool,
    runtime: PlayerRuntime,
    version: Option<u8>,
    mode: Option<PlayerMode>,
    with_default_font: bool,
}

impl PlayerOptions {
    pub fn setup(&self, mut player_builder: PlayerBuilder) -> anyhow::Result<PlayerBuilder> {
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

        player_builder = player_builder
            .with_player_runtime(self.runtime)
            .with_player_version(self.version)
            // Assume flashplayerdebugger is used in tests
            .with_player_mode(self.mode.unwrap_or(PlayerMode::Debug))
            .with_default_font(self.with_default_font);

        if self.with_video {
            #[cfg(feature = "ruffle_video_external")]
            {
                let current_exe = std::env::current_exe()?;
                let directory = current_exe.parent().expect("Executable parent dir");

                use ruffle_video_external::{
                    backend::ExternalVideoBackend, decoder::openh264::OpenH264Codec,
                };
                let openh264 = OpenH264Codec::load(directory)
                    .map_err(|e| anyhow::anyhow!("Couldn't load OpenH264: {}", e))?;

                player_builder =
                    player_builder.with_video(ExternalVideoBackend::new_with_openh264(openh264));
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
