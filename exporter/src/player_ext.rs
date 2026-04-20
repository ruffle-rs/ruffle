use std::{
    any::Any,
    sync::{Arc, Mutex},
};

use ruffle_core::Player;
use ruffle_render_wgpu::{backend::WgpuRenderBackend, target::TextureTarget};

pub trait PlayerExporterExt {
    fn capture_frame(&self) -> Option<image::RgbaImage>;

    fn header_frames(&self) -> u16;

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

    fn header_frames(&self) -> u16 {
        self.lock()
            .unwrap()
            .mutate_with_update_context(|ctx| ctx.root_swf.num_frames())
    }

    fn force_root_clip_play(&self) {
        let mut player = self.lock().unwrap();

        // Check and resume if suspended
        if !player.is_playing() {
            player.set_is_playing(true);
        }

        // Also resume the root MovieClip if stopped
        player.mutate_with_update_context(|ctx| {
            if let Some(root_clip) = ctx.stage.root_clip()
                && let Some(movie_clip) = root_clip.as_movie_clip()
                && !movie_clip.playing()
            {
                movie_clip.play();
            }
        });
    }
}
