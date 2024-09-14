use ruffle_test_framework::environment::Environment;

pub struct NativeEnvironment;

impl Environment for NativeEnvironment {
    #[cfg(feature = "imgtests")]
    fn is_render_supported(
        &self,
        requirements: &ruffle_test_framework::options::RenderOptions,
    ) -> bool {
        renderer::is_supported(requirements)
    }

    #[cfg(feature = "imgtests")]
    fn create_renderer(
        &self,
        width: u32,
        height: u32,
    ) -> Option<(
        Box<dyn ruffle_test_framework::environment::RenderInterface>,
        Box<dyn ruffle_test_framework::environment::RenderBackend>,
    )> {
        renderer::NativeRenderInterface::create_pair(width, height)
    }
}

#[cfg(feature = "imgtests")]
mod renderer {
    use image::RgbaImage;
    use ruffle_render_wgpu::backend::{request_adapter_and_device, WgpuRenderBackend};
    use ruffle_render_wgpu::descriptors::Descriptors;
    use ruffle_render_wgpu::target::TextureTarget;
    use ruffle_render_wgpu::wgpu;
    use ruffle_test_framework::environment::{RenderBackend, RenderInterface};
    use ruffle_test_framework::options::RenderOptions;
    use {std::sync::Arc, std::sync::OnceLock};

    pub struct NativeRenderInterface;

    impl NativeRenderInterface {
        pub fn create_pair(
            width: u32,
            height: u32,
        ) -> Option<(Box<dyn RenderInterface>, Box<dyn RenderBackend>)> {
            if let Some(descriptors) = descriptors() {
                let target = TextureTarget::new(&descriptors.device, (width, height)).expect(
                    "WGPU Texture Target creation must not fail, everything was checked ahead of time",
                );

                Some( (Box::new(Self), Box::new(
                    WgpuRenderBackend::new(descriptors.clone(), target)
                        .expect("WGPU Render backend creation must not fail, everything was checked ahead of time"),
                )))
            } else {
                None
            }
        }
    }

    impl RenderInterface for NativeRenderInterface {
        fn name(&self) -> String {
            if let Some(descriptors) = descriptors() {
                let adapter_info = descriptors.adapter.get_info();
                format!("{}-{:?}", std::env::consts::OS, adapter_info.backend)
            } else {
                std::env::consts::OS.to_string()
            }
        }

        fn capture(&self, backend: &mut Box<dyn RenderBackend>) -> RgbaImage {
            let renderer = backend
                .downcast_mut::<WgpuRenderBackend<TextureTarget>>()
                .unwrap();

            renderer.capture_frame().expect("Failed to capture image")
        }
    }

    pub fn is_supported(_requirements: &RenderOptions) -> bool {
        descriptors().is_some()
    }

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

    fn descriptors() -> Option<&'static Arc<Descriptors>> {
        WGPU.get_or_init(build_wgpu_descriptors).as_ref()
    }
}
