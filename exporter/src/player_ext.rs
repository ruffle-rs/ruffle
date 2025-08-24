use std::{
    any::Any,
    sync::{Arc, Mutex},
};

use ruffle_core::Player;
use ruffle_render_wgpu::{backend::WgpuRenderBackend, target::TextureTarget};

pub trait PlayerExporterExt {
    fn capture_frame(&self) -> Option<image::RgbaImage>;

    fn total_frames(&self) -> u16;

    fn force_root_clip_play(&self);
}

impl PlayerExporterExt for Arc<Mutex<Player>> {
    fn capture_frame(&self) -> Option<image::RgbaImage> {
        let mut player = self.lock().unwrap();
        let renderer =
            <dyn Any>::downcast_mut::<WgpuRenderBackend<TextureTarget>>(player.renderer_mut())
                .unwrap();
        renderer.capture_frame()
    }

    fn total_frames(&self) -> u16 {
        self.lock().unwrap().mutate_with_update_context(|ctx| {
            let total_frames_from_root_clip = ctx
                .stage
                .root_clip()
                .and_then(|root_clip| root_clip.as_movie_clip())
                .map(|movie_clip| movie_clip.total_frames());

            // TODO Can we just use num_frames from the movie here?
            total_frames_from_root_clip.unwrap_or_else(|| ctx.root_swf.num_frames())
        })
    }

    fn force_root_clip_play(&self) {
        let mut player = self.lock().unwrap();

        // Check and resume if suspended
        if !player.is_playing() {
            player.set_is_playing(true);
        }

        // Also resume the root MovieClip if stopped
        player.mutate_with_update_context(|ctx| {
            if let Some(root_clip) = ctx.stage.root_clip() {
                if let Some(movie_clip) = root_clip.as_movie_clip() {
                    if !movie_clip.playing() {
                        movie_clip.play();
                    }
                }
            }
        });
    }
}
