use bytemuck::{Pod, Zeroable};
use ruffle_core::backend::render::BitmapHandle;
use ruffle_core::backend::audio::swf::CharacterId;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GradientUniforms {
    pub colors: [[f32; 4]; 16],
    pub ratios: [f32; 16],
    pub gradient_type: i32,
    pub num_colors: u32,
    pub repeat_mode: i32,
    pub interpolation: i32,
    pub focal_point: f32,
}

#[derive(Debug)]
pub struct Mesh {
    pub draws: Vec<Draw>,
    pub shape_id: CharacterId,
}

#[derive(Debug)]
pub struct Draw {
    pub draw_type: DrawType,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

#[derive(Debug)]
pub enum DrawType {
    Color,
    Gradient {
        texture_transforms: wgpu::Buffer,
        gradient: wgpu::Buffer,
        bind_group: wgpu::BindGroup,
    },
    Bitmap {
        texture_transforms: wgpu::Buffer,
        texture_view: wgpu::TextureView,
        is_smoothed: bool,
        is_repeating: bool,
        bind_group: wgpu::BindGroup,
    },
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum IncompleteDrawType {
    Color,
    Gradient {
        texture_transform: [[f32; 4]; 4],
        gradient: GradientUniforms,
    },
    Bitmap {
        texture_transform: [[f32; 4]; 4],
        bitmap: BitmapHandle,
        is_smoothed: bool,
        is_repeating: bool,
    },
}

impl IncompleteDrawType {
    pub fn name(&self) -> &'static str {
        match self {
            IncompleteDrawType::Color => "Color",
            IncompleteDrawType::Gradient { .. } => "Gradient",
            IncompleteDrawType::Bitmap { .. } => "Bitmap",
        }
    }
}
