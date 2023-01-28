use once_cell::sync::Lazy;
use ruffle_render_wgpu::backend::WgpuRenderBackend;
use ruffle_render_wgpu::descriptors::Descriptors;
use ruffle_render_wgpu::target::TextureTarget;
use ruffle_render_wgpu::wgpu;
use std::sync::Arc;

/*
   It can be expensive to construct WGPU, much less Descriptors, so we put it off as long as we can
   and share it across tests in the same process.

   Remember:
       `cargo test` will run all tests in the same process.
       `cargo nextest run` will create a different process per test.

   For `cargo test` it's relatively okay if we spend the time to construct descriptors once,
   but for `cargo nextest run` it's a big cost per test if it's not going to use it.
*/

fn create_wgpu_device() -> Option<(wgpu::Adapter, wgpu::Device, wgpu::Queue)> {
    futures::executor::block_on(WgpuRenderBackend::<TextureTarget>::request_device(
        wgpu::Backends::all(),
        wgpu::Instance::new(Default::default()),
        None,
        Default::default(),
        None,
    ))
    .ok()
}

fn build_wgpu_descriptors() -> Option<Arc<Descriptors>> {
    if let Some((adapter, device, queue)) = create_wgpu_device() {
        Some(Arc::new(Descriptors::new(adapter, device, queue)))
    } else {
        None
    }
}

pub static WGPU: Lazy<Option<Arc<Descriptors>>> = Lazy::new(build_wgpu_descriptors);
