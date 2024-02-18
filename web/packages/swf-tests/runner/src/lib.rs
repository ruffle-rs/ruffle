use image::RgbaImage;
use ruffle_render_wgpu::backend::WgpuRenderBackend;
use ruffle_render_wgpu::descriptors::Descriptors;
use ruffle_render_wgpu::target::TextureTarget;
use ruffle_render_wgpu::wgpu;
use ruffle_test_framework::environment::{Environment, RenderBackend, RenderInterface};
use ruffle_test_framework::options::{RenderOptions, TestOptions};
use ruffle_test_framework::runner::{TestRunner, TestStatus};
use ruffle_test_framework::test::Test;
use rust_embed::RustEmbed;
use std::sync::Arc;
use vfs::{AltrootFS, EmbeddedFS, VfsPath, VfsResult};
use wasm_bindgen::prelude::*;

#[derive(RustEmbed, Debug)]
#[folder = "../../../../tests/tests/swfs"]
#[exclude = "*.fla"]
struct TestAssets;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = onPanic)]
    fn handle_panic(error: JsError);
}

#[wasm_bindgen(start)]
fn start() {
    std::panic::set_hook(Box::new(|panic| {
        let error = JsError::new(&panic.to_string());
        handle_panic(error);
    }));
}

fn find_tests(output: &mut Vec<TestInfo>, folder: VfsPath) -> VfsResult<()> {
    for path in folder.read_dir()? {
        if path.is_dir()? {
            find_tests(output, path.clone())?;
            if let Some(test) = TestInfo::from_path(path) {
                output.push(test);
            }
        }
    }
    Ok(())
}

#[wasm_bindgen]
pub struct TestInfo(Test);

impl TestInfo {
    pub fn from_path(path: VfsPath) -> Option<Self> {
        if let Ok(options_file) = path.join("test.toml") {
            if options_file.is_file().ok()? {
                if let Ok(options) = TestOptions::read(&options_file) {
                    let name = path.as_str();
                    if let Ok(test) = Test::from_options(
                        options,
                        VfsPath::new(AltrootFS::new(path.clone())),
                        name.to_string(),
                    ) {
                        return Some(TestInfo(test));
                    }
                }
            }
        }
        None
    }
}

#[wasm_bindgen]
impl TestInfo {
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.0.name.to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn ignored(&self) -> bool {
        self.0.options.ignore
    }

    #[wasm_bindgen(getter)]
    pub fn known_failure(&self) -> bool {
        self.0.options.known_failure
    }

    #[wasm_bindgen(getter)]
    pub fn should_run(&self) -> bool {
        true //self.0.should_run(true, &WebEnvironment)
    }

    #[wasm_bindgen(getter)]
    pub fn wants_renderer(&self) -> String {
        match &self.0.options.player_options.with_renderer {
            None => "no",
            Some(r) if r.optional => "optional",
            Some(_) => "required",
        }
        .to_string()
    }

    pub async fn start(&self) -> Result<ActiveTest, String> {
        let environment = WgpuEnvironment::new(wgpu::Backends::GL).await;
        if !self.0.should_run(false, &environment) {
            return Ok(ActiveTest {
                runner: None,
                sleep: 0,
                error: None,
                finished: true,
                skipped: true,
            });
        }

        match self.0.create_test_runner(&environment) {
            Ok(runner) => Ok(ActiveTest {
                runner: Some(runner),
                sleep: 0,
                error: None,
                finished: false,
                skipped: false,
            }),
            Err(e) => Err(e.to_string()),
        }
    }
}

#[wasm_bindgen]
pub struct ActiveTest {
    runner: Option<TestRunner>,
    sleep: u32,
    error: Option<String>,
    finished: bool,
    skipped: bool,
}

#[wasm_bindgen]
impl ActiveTest {
    pub fn tick(&mut self) {
        if self.finished {
            return;
        }
        if let Some(runner) = &mut self.runner {
            runner.tick();
        }
    }

    pub fn run(&mut self) {
        if self.finished {
            return;
        }
        match self.runner.as_mut().map(|runner| runner.test()) {
            Some(Ok(TestStatus::Continue)) => {
                self.sleep = 0;
            }
            Some(Ok(TestStatus::Sleep(duration))) => {
                self.sleep = duration.as_millis() as u32;
            }
            Some(Ok(TestStatus::Finished)) => {
                self.error = None;
                self.finished = true;
            }
            Some(Err(e)) => {
                self.error = Some(e.to_string());
                self.finished = true;
            }
            None => {
                // test is none, but JS is trying to run it anyway... silly js developer
                self.finished = true;
            }
        }
    }

    #[wasm_bindgen(getter)]
    pub fn known_failure(&self) -> bool {
        self.runner
            .as_ref()
            .map(|runner| runner.options().known_failure)
            .unwrap_or_default()
    }

    #[wasm_bindgen(getter)]
    pub fn error(&self) -> Option<String> {
        self.error.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn finished(&self) -> bool {
        self.finished
    }

    #[wasm_bindgen(getter)]
    pub fn skipped(&self) -> bool {
        self.skipped
    }

    #[wasm_bindgen(getter)]
    pub fn sleep(&self) -> u32 {
        self.sleep
    }
}

#[wasm_bindgen]
pub fn list_tests() -> Vec<TestInfo> {
    let root = VfsPath::new(EmbeddedFS::<TestAssets>::new());
    let mut result = vec![];
    let _ = find_tests(&mut result, root);

    result
}

#[wasm_bindgen]
pub fn get_test(name: String) -> Option<TestInfo> {
    let root = VfsPath::new(EmbeddedFS::<TestAssets>::new());
    TestInfo::from_path(root.join(name).ok()?)
}

struct WgpuEnvironment {
    descriptors: Option<Arc<Descriptors>>,
}

impl WgpuEnvironment {
    #[cfg(target_family = "wasm")]
    pub async fn new(backends: wgpu::Backends) -> Self {
        if let Ok(canvas) = web_sys::OffscreenCanvas::new(10, 10) {
            if let Ok(backend) =
                WgpuRenderBackend::descriptors_for_offscreen_canvas(backends, canvas).await
            {
                return Self {
                    descriptors: Some(backend),
                };
            }
        }
        Self { descriptors: None }
    }

    #[cfg(not(target_family = "wasm"))]
    pub async fn new(_backends: wgpu::Backends) -> Self {
        Self { descriptors: None }
    }
}

impl Environment for WgpuEnvironment {
    fn is_render_supported(&self, _requirements: &RenderOptions) -> bool {
        self.descriptors.is_some()
    }

    fn create_renderer(
        &self,
        width: u32,
        height: u32,
    ) -> Option<(Box<dyn RenderInterface>, Box<dyn RenderBackend>)> {
        if let Some(descriptors) = self.descriptors.clone() {
            let target = TextureTarget::new(&descriptors.device, (width, height)).expect(
                "WGPU Texture Target creation must not fail, everything was checked ahead of time",
            );

            Some( (Box::new(WgpuRenderInterface(descriptors.clone())), Box::new(
                WgpuRenderBackend::new(descriptors, target)
                    .expect("WGPU Render backend creation must not fail, everything was checked ahead of time"),
            )))
        } else {
            None
        }
    }
}

struct WgpuRenderInterface(Arc<Descriptors>);

impl RenderInterface for WgpuRenderInterface {
    fn name(&self) -> String {
        let adapter_info = self.0.adapter.get_info();
        if adapter_info.backend == ruffle_render_wgpu::wgpu::Backend::Gl {
            "wgpu-webgl".to_string()
        } else {
            format!("{}-{:?}", std::env::consts::OS, adapter_info.backend)
        }
    }

    fn capture(&self, backend: &mut Box<dyn RenderBackend>) -> RgbaImage {
        let renderer = backend
            .downcast_mut::<WgpuRenderBackend<TextureTarget>>()
            .unwrap();

        renderer.capture_frame().expect("Failed to capture image")
    }
}
