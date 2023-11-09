use ruffle_render_wgpu::backend::request_adapter_and_device;
use ruffle_render_wgpu::descriptors::Descriptors;
use ruffle_render_wgpu::wgpu;
use std::sync::{Arc, OnceLock};

/*
   It can be expensive to construct WGPU, much less Descriptors, so we put it off as long as we can
   and share it across tests in the same process.

   Remember:
       `cargo test` will run all tests in the same process.
       `cargo nextest run` will create a different process per test.

   For `cargo test` it's relatively okay if we spend the time to construct descriptors once,
   but for `cargo nextest run` it's a big cost per test if it's not going to use it.
*/

fn create_wgpu_device() -> Option<(wgpu::Instance, wgpu::Adapter, wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(Default::default());
    futures::executor::block_on(request_adapter_and_device(
        wgpu::Backends::all(),
        &instance,
        None,
        Default::default(),
        None,
    ))
    .ok()
    .map(|(adapter, device, queue)| (instance, adapter, device, queue))
}

fn build_wgpu_descriptors() -> Option<Arc<Descriptors>> {
    if let Some((instance, adapter, device, queue)) = create_wgpu_device() {
        Some(Arc::new(Descriptors::new(instance, adapter, device, queue)))
    } else {
        None
    }
}

pub fn wgpu_descriptors() -> Option<&'static Arc<Descriptors>> {
    // TODO: Use `std::sync::LazyLock` once it's stabilized?
    static WGPU: OnceLock<Option<Arc<Descriptors>>> = OnceLock::new();
    WGPU.get_or_init(build_wgpu_descriptors).as_ref()
}
