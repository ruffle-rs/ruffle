use lyon::lyon_algorithms::path::Path;
use ruffle_core::shape_utils::DrawCommand;
use ruffle_core::swf;
use swf::{GradientSpread, Twips};
macro_rules! create_debug_label {
    ($($arg:tt)*) => (
        if cfg!(feature = "render_debug_labels") {
            Some(format!($($arg)*))
        } else {
            None
        }
    )
}

pub fn create_buffer_with_data(
    device: &wgpu::Device,
    data: &[u8],
    usage: wgpu::BufferUsage,
    label: Option<String>,
) -> wgpu::Buffer {
    let mapped = device.create_buffer_mapped(&wgpu::BufferDescriptor {
        size: data.len() as u64,
        usage,
        label: label.as_deref(),
    });
    mapped.data.copy_from_slice(data);
    mapped.finish()
}

pub fn point(x: Twips, y: Twips) -> lyon::math::Point {
    lyon::math::Point::new(x.to_pixels() as f32, y.to_pixels() as f32)
}

pub fn ruffle_path_to_lyon_path(commands: Vec<DrawCommand>, is_closed: bool) -> Path {
    let mut builder = Path::builder();
    for cmd in commands {
        match cmd {
            DrawCommand::MoveTo { x, y } => {
                builder.move_to(point(x, y));
            }
            DrawCommand::LineTo { x, y } => {
                builder.line_to(point(x, y));
            }
            DrawCommand::CurveTo { x1, y1, x2, y2 } => {
                builder.quadratic_bezier_to(point(x1, y1), point(x2, y2));
            }
        }
    }

    if is_closed {
        builder.close();
    }

    builder.build()
}

#[allow(clippy::many_single_char_names)]
pub fn swf_to_gl_matrix(m: swf::Matrix) -> [[f32; 4]; 4] {
    let tx = m.tx.get() as f32;
    let ty = m.ty.get() as f32;
    let det = m.a * m.d - m.c * m.b;
    let mut a = m.d / det;
    let mut b = -m.c / det;
    let mut c = -(tx * m.d - m.c * ty) / det;
    let mut d = -m.b / det;
    let mut e = m.a / det;
    let mut f = (tx * m.b - m.a * ty) / det;

    a *= 20.0 / 32768.0;
    b *= 20.0 / 32768.0;
    d *= 20.0 / 32768.0;
    e *= 20.0 / 32768.0;

    c /= 32768.0;
    f /= 32768.0;
    c += 0.5;
    f += 0.5;
    [
        [a, d, 0.0, 0.0],
        [b, e, 0., 0.0],
        [c, f, 1.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
    ]
}

#[allow(clippy::many_single_char_names)]
pub fn swf_bitmap_to_gl_matrix(
    m: swf::Matrix,
    bitmap_width: u32,
    bitmap_height: u32,
) -> [[f32; 4]; 4] {
    let bitmap_width = bitmap_width as f32;
    let bitmap_height = bitmap_height as f32;

    let tx = m.tx.get() as f32;
    let ty = m.ty.get() as f32;
    let det = m.a * m.d - m.c * m.b;
    let mut a = m.d / det;
    let mut b = -m.c / det;
    let mut c = -(tx * m.d - m.c * ty) / det;
    let mut d = -m.b / det;
    let mut e = m.a / det;
    let mut f = (tx * m.b - m.a * ty) / det;

    a *= 20.0 / bitmap_width;
    b *= 20.0 / bitmap_width;
    d *= 20.0 / bitmap_height;
    e *= 20.0 / bitmap_height;

    c /= bitmap_width;
    f /= bitmap_height;

    [
        [a, d, 0.0, 0.0],
        [b, e, 0.0, 0.0],
        [c, f, 1.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
    ]
}

pub fn build_view_matrix(viewport_width: u32, viewport_height: u32) -> [[f32; 4]; 4] {
    [
        [1.0 / (viewport_width as f32 / 2.0), 0.0, 0.0, 0.0],
        [0.0, -1.0 / (viewport_height as f32 / 2.0), 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [-1.0, 1.0, 0.0, 1.0],
    ]
}

/// Map for SWF gradient spread mode to the uniform value used by the gradient shader.
pub fn gradient_spread_mode_index(spread: GradientSpread) -> i32 {
    match spread {
        GradientSpread::Pad => 0,
        GradientSpread::Repeat => 1,
        GradientSpread::Reflect => 2,
    }
}
