use image::RgbaImage;
use ruffle_test_framework::environment::{Environment, RenderBackend};

#[cfg(feature = "imgtests")]
use ruffle_test_framework::options::RenderOptions;

#[cfg(feature = "imgtests")]
use ruffle_render_wgpu::{
    backend::{request_adapter_and_device, WgpuRenderBackend},
    descriptors::Descriptors,
    target::TextureTarget,
    wgpu,
};

#[cfg(feature = "imgtests")]
use {std::sync::Arc, std::sync::OnceLock};

pub struct NativeEnvironment;

impl NativeEnvironment {
    #[cfg(feature = "imgtests")]
    fn descriptors(&self) -> Option<&Arc<Descriptors>> {
        WGPU.get_or_init(build_wgpu_descriptors).as_ref()
    }
}

impl Environment for NativeEnvironment {
    #[cfg(feature = "imgtests")]
    fn is_render_supported(&self, requirements: &RenderOptions) -> bool {
        if let Some(descriptors) = self.descriptors() {
            let adapter_info = descriptors.adapter.get_info();
            let is_warp =
                cfg!(windows) && adapter_info.vendor == 5140 && adapter_info.device == 140;

            !requirements.exclude_warp || !is_warp
        } else {
            false
        }
    }

    #[cfg(feature = "imgtests")]
    fn create_renderer(&self, width: u32, height: u32) -> Option<Box<dyn RenderBackend>> {
        if let Some(descriptors) = self.descriptors() {
            let target = TextureTarget::new(&descriptors.device, (width, height)).expect(
                "WGPU Texture Target creation must not fail, everything was checked ahead of time",
            );

            Some(Box::new(
                WgpuRenderBackend::new(descriptors.clone(), target)
                    .expect("WGPU Render backend creation must not fail, everything was checked ahead of time"),
            ))
        } else {
            None
        }
    }

    #[cfg(not(feature = "imgtests"))]
    fn name(&self) -> String {
        std::env::consts::OS.to_string()
    }

    #[cfg(feature = "imgtests")]
    fn name(&self) -> String {
        if let Some(descriptors) = self.descriptors() {
            let adapter_info = descriptors.adapter.get_info();
            format!("{}-{:?}", std::env::consts::OS, adapter_info.backend)
        } else {
            std::env::consts::OS.to_string()
        }
    }

    #[cfg(not(feature = "imgtests"))]
    fn capture_renderer(&self, _backend: &mut Box<dyn RenderBackend>) -> RgbaImage {
        panic!("Cannot capture renderer as imgtests are not enabled")
    }

    #[cfg(feature = "imgtests")]
    fn capture_renderer(&self, backend: &mut Box<dyn RenderBackend>) -> RgbaImage {
        let renderer = backend
            .downcast_mut::<WgpuRenderBackend<TextureTarget>>()
            .unwrap();

        renderer.capture_frame().expect("Failed to capture image")
    }
}

#[cfg(feature = "imgtests")]
static WGPU: OnceLock<Option<Arc<Descriptors>>> = OnceLock::new();

/*
   It can be expensive to construct WGPU, much less Descriptors, so we put it off as long as we can
   and share it across tests in the same process.

   Remember:
       `cargo test` will run all tests in the same process.
       `cargo nextest run` will create a different process per test.

   For `cargo test` it's relatively okay if we spend the time to construct descriptors once,
   but for `cargo nextest run` it's a big cost per test if it's not going to use it.
*/

#[cfg(feature = "imgtests")]
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

#[cfg(feature = "imgtests")]
fn build_wgpu_descriptors() -> Option<Arc<Descriptors>> {
    if let Some((instance, adapter, device, queue)) = create_wgpu_device() {
        Some(Arc::new(Descriptors::new(instance, adapter, device, queue)))
    } else {
        None
    }
}
