use ruffle_test_framework::environment::{CompileMode, Environment};

#[cfg(feature = "imgtests")]
use ruffle_render_wgpu::descriptors::Descriptors;
#[cfg(feature = "imgtests")]
use std::sync::{Arc, LazyLock};

pub struct NativeEnvironment {
    pub compile_mode: CompileMode,
    #[cfg(feature = "imgtests")]
    descriptors: LazyLock<Option<Arc<Descriptors>>>,
}

impl NativeEnvironment {
    pub fn new(compile_mode: CompileMode) -> Self {
        Self {
            compile_mode,
            #[cfg(feature = "imgtests")]
            descriptors: LazyLock::new(renderer::build_wgpu_descriptors),
        }
    }

    /// Wait for any pending GPU work to finish, with a `timeout`.
    ///
    /// Call this before `main` ends so the driver's worker threads quiesce
    /// while the environment is still alive — otherwise libc's
    /// `__cxa_finalize` can run while a Mesa lavapipe JIT thread is still
    /// compiling a shader, racing the destructors and producing a
    /// `SIGSEGV`. The wait also keeps wgpu's `Queue::Drop` (with its
    /// hard-coded ~6.3 s budget that `panic!`s on timeout) happy on slow
    /// software renderers like DX12-WARP on the Windows GitHub-Actions
    /// runner.
    pub fn flush_gpu_with_timeout(&self, #[allow(unused)] timeout: std::time::Duration) {
        #[cfg(feature = "imgtests")]
        {
            // `LazyLock::get` peeks without forcing initialization, so tests
            // that never touched the renderer don't pay the cost of building
            // wgpu state just to flush it again at shutdown.
            if let Some(Some(descriptors)) = LazyLock::get(&self.descriptors) {
                use ruffle_render_wgpu::wgpu;

                let _ = descriptors.device.poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: Some(timeout),
                });
            }
        }
    }
}

impl Environment for NativeEnvironment {
    #[cfg(feature = "imgtests")]
    fn is_render_supported(
        &self,
        _requirements: &ruffle_test_framework::options::RenderOptions,
    ) -> bool {
        self.descriptors.is_some()
    }

    #[cfg(feature = "imgtests_glow")]
    fn is_render_supported(
        &self,
        _requirements: &ruffle_test_framework::options::RenderOptions,
    ) -> bool {
        true
    }

    #[cfg(any(feature = "imgtests", feature = "imgtests_glow"))]
    fn create_renderer(
        &self,
        width: u32,
        height: u32,
    ) -> Option<(
        Box<dyn ruffle_test_framework::environment::RenderInterface>,
        Box<dyn ruffle_test_framework::environment::RenderBackend>,
    )> {
        renderer::create_pair(self, width, height)
    }

    fn compile_mode(&self) -> CompileMode {
        self.compile_mode
    }
}

#[cfg(feature = "imgtests")]
mod renderer {
    use super::NativeEnvironment;
    use image::RgbaImage;
    use ruffle_render_wgpu::backend::{
        WgpuRenderBackend, create_wgpu_instance, request_adapter_and_device,
    };
    use ruffle_render_wgpu::descriptors::Descriptors;
    use ruffle_render_wgpu::target::TextureTarget;
    use ruffle_render_wgpu::wgpu;
    use ruffle_test_framework::environment::{RenderBackend, RenderInterface};
    use std::any::Any;
    use std::sync::Arc;

    pub struct NativeRenderInterface {
        name: String,
    }

    pub fn create_pair(
        env: &NativeEnvironment,
        width: u32,
        height: u32,
    ) -> Option<(Box<dyn RenderInterface>, Box<dyn RenderBackend>)> {
        let descriptors = env.descriptors.clone()?;
        let target = TextureTarget::new(&descriptors.device, (width, height)).expect(
            "WGPU Texture Target creation must not fail, everything was checked ahead of time",
        );

        let adapter_info = descriptors.adapter.get_info();
        let name = format!("{}-{:?}", std::env::consts::OS, adapter_info.backend);

        Some((
            Box::new(NativeRenderInterface { name }),
            Box::new(WgpuRenderBackend::new(descriptors, target).expect(
                "WGPU Render backend creation must not fail, everything was checked ahead of time",
            )),
        ))
    }

    impl RenderInterface for NativeRenderInterface {
        fn name(&self) -> String {
            self.name.clone()
        }

        fn capture(&self, backend: &mut dyn RenderBackend) -> RgbaImage {
            let renderer =
                <dyn Any>::downcast_mut::<WgpuRenderBackend<TextureTarget>>(backend).unwrap();

            renderer.capture_frame().expect("Failed to capture image")
        }
    }

    /*
       It can be expensive to construct WGPU, much less Descriptors, so we put it off as long as we can
       and share it across tests in the same process.

       Remember:
           `cargo test` will run all tests in the same process.
           `cargo nextest run` will create a different process per test.

       For `cargo test` it's relatively okay if we spend the time to construct descriptors once,
       but for `cargo nextest run` it's a big cost per test if it's not going to use it.

       The descriptors live on the `NativeEnvironment` (rather than a `static`)
       so that they get dropped when the environment is dropped at the end of
       `main`. Their `Drop` impl runs `vkDestroyDevice` / `vkDestroyInstance`,
       which joins driver worker threads (notably Mesa lavapipe's LLVM JIT
       pool) before libc tears the process down. Leaving the descriptors in a
       `static` left those workers racing with `__cxa_finalize`, producing a
       `SIGSEGV` after the test had already passed.
    */

    fn create_wgpu_device() -> Option<(wgpu::Instance, wgpu::Adapter, wgpu::Device, wgpu::Queue)> {
        let instance =
            create_wgpu_instance(wgpu::Backends::all(), wgpu::BackendOptions::default(), None);
        futures::executor::block_on(request_adapter_and_device(
            wgpu::Backends::all(),
            &instance,
            None,
            Default::default(),
        ))
        .ok()
        .map(|(adapter, device, queue)| (instance, adapter, device, queue))
    }

    pub(super) fn build_wgpu_descriptors() -> Option<Arc<Descriptors>> {
        let (instance, adapter, device, queue) = create_wgpu_device()?;

        Some(Arc::new(Descriptors::new(instance, adapter, device, queue)))
    }
}

#[cfg(feature = "imgtests_glow")]
mod renderer {
    use super::NativeEnvironment;
    use image::RgbaImage;
    use ruffle_render_glow::GlowRenderBackend;
    use ruffle_render_glow::glow::*;
    use ruffle_test_framework::environment::{RenderBackend, RenderInterface};
    use std::any::Any;
    use std::sync::Arc;
    extern crate khronos_egl as egl;

    pub struct NativeRenderInterface {
        name: String,
        gl: Arc<ruffle_render_glow::glow::Context>,
        _egl: egl::Instance<egl::Static>,
        _egl_display: egl::Display,
        _egl_surface: egl::Surface,
    }

    pub fn create_pair(
        _env: &NativeEnvironment,
        width: u32,
        height: u32,
    ) -> Option<(Box<dyn RenderInterface>, Box<dyn RenderBackend>)> {
        let egl = egl::Instance::new(egl::Static);
        let egl_display = unsafe { egl.get_display(egl::DEFAULT_DISPLAY).unwrap() };
        egl.initialize(egl_display).unwrap();
        let attributes = [
            egl::RED_SIZE,
            8,
            egl::GREEN_SIZE,
            8,
            egl::BLUE_SIZE,
            8,
            egl::ALPHA_SIZE,
            8,
            egl::STENCIL_SIZE,
            8,
            egl::DEPTH_SIZE,
            24,
            egl::NONE,
        ];
        let config = egl
            .choose_first_config(egl_display, &attributes)
            .unwrap()
            .unwrap();
        let context_attributes = [
            egl::CONTEXT_MAJOR_VERSION,
            3,
            egl::CONTEXT_MINOR_VERSION,
            2,
            egl::NONE,
        ];
        let egl_ctx = egl
            .create_context(egl_display, config, None, &context_attributes)
            .unwrap();
        let surface_attributes = [
            egl::WIDTH,
            width as i32,
            egl::HEIGHT,
            height as i32,
            egl::NONE,
        ];
        let egl_surface = egl
            .create_pbuffer_surface(egl_display, config, &surface_attributes)
            .unwrap();
        egl.make_current(
            egl_display,
            Some(egl_surface),
            Some(egl_surface),
            Some(egl_ctx),
        )
        .unwrap();
        let name = format!("{}-{:?}", std::env::consts::OS, "glow");

        unsafe {
            let gl = Arc::new(ruffle_render_glow::glow::Context::from_loader_function(
                |s| egl.get_proc_address(s).unwrap() as *const _,
            ));
            Some((
                Box::new(NativeRenderInterface {
                    name,
                    gl: gl.clone(),
                    _egl: egl,
                    _egl_display: egl_display,
                    _egl_surface: egl_surface,
                }),
                Box::new(GlowRenderBackend::new(gl.clone(), false, ruffle_render::quality::StageQuality::High).expect(
                    "glow Render backend creation must not fail, everything was checked ahead of time",
                )),
        ))
        }
    }

    impl RenderInterface for NativeRenderInterface {
        fn name(&self) -> String {
            self.name.clone()
        }

        fn capture(&self, backend: &mut dyn RenderBackend) -> RgbaImage {
            let renderer = <dyn Any>::downcast_mut::<GlowRenderBackend>(backend).unwrap();

            let width = renderer.viewport_dimensions().width;
            let height = renderer.viewport_dimensions().height;
            let buf_size: usize = usize::try_from(width * height * 4).unwrap();
            let mut pixels = vec![0; buf_size];

            unsafe {
                self.gl.finish();
                self.gl.read_pixels(
                    0,
                    0,
                    width as i32,
                    height as i32,
                    ruffle_render_glow::glow::RGBA,
                    ruffle_render_glow::glow::UNSIGNED_BYTE,
                    ruffle_render_glow::glow::PixelPackData::Slice(Some(&mut pixels[..])),
                );
            }
            let mut ret =
                image::RgbaImage::from_raw(width, height, pixels).expect("Failed to capture image");
            // glReadPixels returns the framebuffer flipped
            image::imageops::flip_vertical_in_place(&mut ret);
            ret
        }
    }
}
