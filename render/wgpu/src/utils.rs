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
