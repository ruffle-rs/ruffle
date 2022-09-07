use crate::frame::Frame;
use crate::pipelines::BlendMode as ActualBlendMode;
use crate::{ColorAdjustments, DrawType, MaskState, Mesh, RegistryData};
use fnv::FnvHashMap;
use ruffle_render::backend::ShapeHandle;
use ruffle_render::bitmap::BitmapHandle;
use ruffle_render::commands::CommandHandler;
use ruffle_render::transform::Transform;
use swf::{BlendMode, Color};

pub struct CommandRenderer<'a, 'b> {
    frame: &'b mut Frame<'a>,
    bitmap_registry: &'a FnvHashMap<BitmapHandle, RegistryData>,
    meshes: &'a Vec<Mesh>,
    quad_vertices: wgpu::BufferSlice<'a>,
    quad_indices: wgpu::BufferSlice<'a>,
    blend_modes: Vec<BlendMode>,
}

impl<'a, 'b> CommandRenderer<'a, 'b> {
    pub fn new(
        frame: &'b mut Frame<'a>,
        meshes: &'a Vec<Mesh>,
        bitmap_registry: &'a FnvHashMap<BitmapHandle, RegistryData>,
        quad_vertices: wgpu::BufferSlice<'a>,
        quad_indices: wgpu::BufferSlice<'a>,
    ) -> Self {
        Self {
            frame,
            bitmap_registry,
            meshes,
            quad_vertices,
            quad_indices,
            blend_modes: vec![BlendMode::Normal],
        }
    }
}

impl<'a, 'b> CommandHandler for CommandRenderer<'a, 'b> {
    fn render_bitmap(&mut self, bitmap: BitmapHandle, transform: &Transform, smoothing: bool) {
        if let Some(entry) = self.bitmap_registry.get(&bitmap) {
            let texture = &entry.texture_wrapper;

            self.frame.apply_transform(
                &(transform.matrix
                    * ruffle_render::matrix::Matrix {
                        a: texture.width as f32,
                        d: texture.height as f32,
                        ..Default::default()
                    }),
                ColorAdjustments::from(transform.color_transform),
            );

            self.frame.draw_bitmap(
                self.quad_vertices,
                self.quad_indices,
                6,
                &texture.bind_group,
                false,
                smoothing,
            );
        }
    }

    fn render_shape(&mut self, shape: ShapeHandle, transform: &Transform) {
        self.frame.apply_transform(
            &transform.matrix,
            ColorAdjustments::from(transform.color_transform),
        );

        let mesh = &self.meshes[shape.0];
        let mask_state = self.frame.mask_state();
        for draw in &mesh.draws {
            let num_indices = if mask_state != MaskState::DrawMaskStencil
                && mask_state != MaskState::ClearMaskStencil
            {
                draw.num_indices
            } else {
                // Omit strokes when drawing a mask stencil.
                draw.num_mask_indices
            };
            if num_indices == 0 {
                continue;
            }

            match &draw.draw_type {
                DrawType::Color => {
                    self.frame.draw_color(
                        draw.vertex_buffer.slice(..),
                        draw.index_buffer.slice(..),
                        num_indices,
                    );
                }
                DrawType::Gradient { bind_group, .. } => {
                    self.frame.draw_gradient(
                        draw.vertex_buffer.slice(..),
                        draw.index_buffer.slice(..),
                        num_indices,
                        bind_group,
                    );
                }
                DrawType::Bitmap {
                    is_repeating,
                    is_smoothed,
                    bind_group,
                    ..
                } => {
                    self.frame.draw_bitmap(
                        draw.vertex_buffer.slice(..),
                        draw.index_buffer.slice(..),
                        num_indices,
                        bind_group,
                        *is_repeating,
                        *is_smoothed,
                    );
                }
            }
        }
    }

    fn draw_rect(&mut self, color: Color, matrix: &ruffle_render::matrix::Matrix) {
        self.frame.apply_transform(
            &matrix,
            ColorAdjustments {
                mult_color: [
                    f32::from(color.r) / 255.0,
                    f32::from(color.g) / 255.0,
                    f32::from(color.b) / 255.0,
                    f32::from(color.a) / 255.0,
                ],
                add_color: [0.0, 0.0, 0.0, 0.0],
            },
        );

        self.frame
            .draw_color(self.quad_vertices, self.quad_indices, 6);
    }

    fn push_mask(&mut self) {
        debug_assert!(
            self.frame.mask_state() == MaskState::NoMask
                || self.frame.mask_state() == MaskState::DrawMaskedContent
        );
        self.frame.set_mask_state(MaskState::DrawMaskStencil);
        self.frame.set_mask_count(self.frame.num_masks() + 1);
    }

    fn activate_mask(&mut self) {
        debug_assert!(
            self.frame.num_masks() > 0 && self.frame.mask_state() == MaskState::DrawMaskStencil
        );
        self.frame.set_mask_state(MaskState::DrawMaskedContent);
    }

    fn deactivate_mask(&mut self) {
        debug_assert!(
            self.frame.num_masks() > 0 && self.frame.mask_state() == MaskState::DrawMaskedContent
        );
        self.frame.set_mask_state(MaskState::ClearMaskStencil);
    }

    fn pop_mask(&mut self) {
        debug_assert!(
            self.frame.num_masks() > 0 && self.frame.mask_state() == MaskState::ClearMaskStencil
        );
        let num_masks = self.frame.num_masks() - 1;
        self.frame.set_mask_count(num_masks);
        if num_masks == 0 {
            self.frame.set_mask_state(MaskState::NoMask);
        } else {
            self.frame.set_mask_state(MaskState::DrawMaskedContent);
        };
    }

    fn push_blend_mode(&mut self, blend: BlendMode) {
        self.blend_modes.push(blend);
        self.frame.set_blend_mode(blend.into());
    }

    fn pop_blend_mode(&mut self) {
        self.blend_modes.pop();
        self.frame.set_blend_mode(
            self.blend_modes
                .last()
                .map(|b| ActualBlendMode::from(*b))
                .unwrap_or(ActualBlendMode::Normal),
        );
    }
}
