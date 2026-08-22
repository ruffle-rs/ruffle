use crate::bitmap::bitmap_data::{BitmapData, IBitmapDrawable};
use crate::bitmap::operations;
use crate::context::UpdateContext;
use crate::debug_ui::{ItemToSave, Message};
use crate::display_object::{BoundsMode, DisplayObject, TDisplayObject};
use ruffle_render::matrix::Matrix;
use ruffle_render::quality::StageQuality;
use ruffle_render::transform::Transform;
use std::fmt::{Debug, Formatter};
use swf::{BlendMode, Twips};

/// A snapshot of a display object rendered on demand for the "Preview" panel.
///
/// The raw RGBA pixels are kept alongside the uploaded texture so the image
/// can be exported (e.g. saved as a PNG) without needing to read the pixels
/// back from the GPU-owned egui texture.
pub struct RenderedPreview {
    texture: egui::TextureHandle,
    width: u32,
    height: u32,
    rgba_premultiplied: Vec<u8>,
}

impl Debug for RenderedPreview {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderedPreview")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("rgba", &self.rgba_premultiplied.len())
            .finish()
    }
}

impl RenderedPreview {
    pub fn texture(&self) -> &egui::TextureHandle {
        &self.texture
    }

    /// Computes the effective scale (after clamping to `max_dimension`) and
    /// resulting pixel dimensions a preview of `object` would have, without
    /// actually rendering it.
    ///
    /// `max_dimension` caps the width and height of the resulting texture,
    /// regardless of `scale`, as a safety net against accidentally
    /// allocating an enormous texture.
    pub fn size_and_scale_for(
        object: DisplayObject,
        scale: f32,
        max_dimension: u32,
    ) -> Option<(u32, u32, f64)> {
        let bounds = object.bounds(BoundsMode::Engine);
        if !bounds.is_valid() {
            return None;
        }

        let width_px = bounds.width().to_pixels();
        let height_px = bounds.height().to_pixels();
        if width_px <= 0.0 || height_px <= 0.0 {
            return None;
        }

        let scale = (scale as f64).min(max_dimension as f64 / width_px.max(height_px));
        let out_width = ((width_px * scale).ceil() as u32).clamp(1, max_dimension);
        let out_height = ((height_px * scale).ceil() as u32).clamp(1, max_dimension);

        Some((out_width, out_height, scale))
    }

    /// Renders a `DisplayObject` (and its children) offscreen, in its own local
    /// coordinate space, and uploads the result as an egui texture.
    pub fn render<'gc>(
        context: &mut UpdateContext<'gc>,
        object: DisplayObject<'gc>,
        egui_ctx: &egui::Context,
        scale: f32,
        quality: StageQuality,
        max_dimension: u32,
    ) -> Option<Self> {
        let bounds = object.bounds(BoundsMode::Engine);
        let (out_width, out_height, scale) =
            Self::size_and_scale_for(object, scale, max_dimension)?;

        let transform = Transform {
            matrix: Matrix {
                a: scale as f32,
                b: 0.0,
                c: 0.0,
                d: scale as f32,
                tx: Twips::from_pixels(-bounds.x_min.to_pixels() * scale),
                ty: Twips::from_pixels(-bounds.y_min.to_pixels() * scale),
            },
            ..Default::default()
        };

        let target = BitmapData::new(context.gc(), out_width, out_height, true, 0);

        operations::draw(
            context,
            target,
            IBitmapDrawable::DisplayObject(object),
            transform,
            true,
            BlendMode::Normal,
            None,
            quality,
        )
        .ok()?;

        let data = target.sync(context.renderer);
        let data = data.borrow();
        let width = data.width();
        let height = data.height();
        let rgba_premultiplied = data.pixels_rgba().to_vec();
        let image = egui::ColorImage::from_rgba_premultiplied(
            [width as usize, height as usize],
            &rgba_premultiplied,
        );

        let texture = egui_ctx.load_texture(
            format!("do-preview-{:p}", object.as_ptr()),
            image,
            Default::default(),
        );

        Some(Self {
            texture,
            width,
            height,
            rgba_premultiplied,
        })
    }

    /// Encodes this preview as a PNG and queues it to be saved via a native
    /// file dialog.
    pub fn save_as_png(&self, object: DisplayObject, messages: &mut Vec<Message>) {
        let mut straight_rgba = self.rgba_premultiplied.clone();
        ruffle_render::utils::unmultiply_alpha_rgba(&mut straight_rgba);

        let mut data = Vec::new();
        let mut encoder = png::Encoder::new(&mut data, self.width, self.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        match encoder
            .write_header()
            .and_then(|mut w| w.write_image_data(&straight_rgba))
        {
            Ok(()) => messages.push(Message::SaveFile(ItemToSave {
                suggested_name: format!("{:p}.png", object.as_ptr()),
                data,
            })),
            Err(e) => tracing::error!("Couldn't create png: {e}"),
        }
    }
}
