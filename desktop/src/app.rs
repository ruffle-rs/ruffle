use crate::cli::Opt;
use crate::custom_event::RuffleEvent;
use crate::executor::GlutinAsyncExecutor;
use crate::gui::{GuiController, MovieView};
use crate::player::PlayerController;
use crate::util::{
    get_screen_size, parse_url, pick_file, winit_key_to_char, winit_to_ruffle_key_code,
    winit_to_ruffle_text_control,
};
use crate::{audio, navigator, storage, ui, CALLSTACK, RENDER_INFO, SWF_INFO};
use anyhow::{anyhow, Context, Error};
use ruffle_core::backend::audio::AudioBackend;
use ruffle_core::{PlayerBuilder, PlayerEvent, StageDisplayState};
use ruffle_render::backend::{RenderBackend, ViewportDimensions};
use ruffle_render_wgpu::backend::WgpuRenderBackend;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use url::Url;
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize, Size};
use winit::event::{ElementState, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy};
use winit::window::{Fullscreen, Icon, Window, WindowBuilder};

pub struct App {
    opt: Opt,
    window: Rc<Window>,
    event_loop: Option<EventLoop<RuffleEvent>>,
    event_loop_proxy: EventLoopProxy<RuffleEvent>,
    executor: Arc<Mutex<GlutinAsyncExecutor>>,
    gui: Arc<Mutex<GuiController>>,
    player: PlayerController,
    min_window_size: LogicalSize<u32>,
    max_window_size: PhysicalSize<u32>,
}

impl App {
    pub fn new(opt: Opt) -> Result<Self, Error> {
        let movie_url = if let Some(path) = &opt.input_path {
            Some(parse_url(path).context("Couldn't load specified path")?)
        } else {
            None
        };

        let icon_bytes = include_bytes!("../assets/favicon-32.rgba");
        let icon =
            Icon::from_rgba(icon_bytes.to_vec(), 32, 32).context("Couldn't load app icon")?;

        let event_loop = EventLoopBuilder::with_user_event().build();

        let min_window_size = (16, 16).into();
        let max_window_size = get_screen_size(&event_loop);

        let window = WindowBuilder::new()
            .with_visible(false)
            .with_title("Ruffle")
            .with_window_icon(Some(icon))
            .with_min_inner_size(min_window_size)
            .with_max_inner_size(max_window_size)
            .build(&event_loop)?;

        let mut builder = PlayerBuilder::new();

        match audio::CpalAudioBackend::new() {
            Ok(mut audio) => {
                audio.set_volume(opt.volume);
                builder = builder.with_audio(audio);
            }
            Err(e) => {
                tracing::error!("Unable to create audio device: {}", e);
            }
        };

        let (executor, channel) = GlutinAsyncExecutor::new(event_loop.create_proxy());
        let navigator = navigator::ExternalNavigatorBackend::new(
            opt.base.to_owned().unwrap_or(
                movie_url
                    .clone()
                    .unwrap_or_else(|| Url::parse("file:///empty").expect("Dummy Url")),
            ),
            channel,
            event_loop.create_proxy(),
            opt.proxy.clone(),
            opt.upgrade_to_https,
            opt.open_url_mode,
        );

        let window = Rc::new(window);

        if cfg!(feature = "software_video") {
            builder =
                builder.with_video(ruffle_video_software::backend::SoftwareVideoBackend::new());
        }

        let gui = GuiController::new(
            window.clone(),
            &event_loop,
            opt.trace_path(),
            opt.graphics.into(),
            opt.power.into(),
        )?;

        let renderer = WgpuRenderBackend::new(gui.descriptors().clone(), gui.create_movie_view())
            .map_err(|e| anyhow!(e.to_string()))
            .context("Couldn't create wgpu rendering backend")?;
        RENDER_INFO.with(|i| *i.borrow_mut() = Some(renderer.debug_info().to_string()));

        builder = builder
            .with_navigator(navigator)
            .with_renderer(renderer)
            .with_storage(storage::DiskStorageBackend::new()?)
            .with_ui(ui::DesktopUiBackend::new(window.clone())?)
            .with_autoplay(true)
            .with_letterbox(opt.letterbox)
            .with_max_execution_duration(Duration::from_secs_f64(opt.max_execution_duration))
            .with_quality(opt.quality)
            .with_warn_on_unsupported_content(!opt.dont_warn_on_unsupported_content)
            .with_scale_mode(opt.scale, opt.force_scale)
            .with_fullscreen(opt.fullscreen)
            .with_load_behavior(opt.load_behavior)
            .with_spoofed_url(opt.spoof_url.clone().map(|url| url.to_string()))
            .with_player_version(opt.player_version)
            .with_frame_rate(opt.frame_rate);

        let player = PlayerController::new(builder.build());

        let mut app = Self {
            opt,
            window,
            event_loop_proxy: event_loop.create_proxy(),
            event_loop: Some(event_loop),
            executor,
            gui: Arc::new(Mutex::new(gui)),
            player,
            min_window_size,
            max_window_size,
        };

        if let Some(movie_url) = movie_url {
            app.load_swf(movie_url)?;
        }

        Ok(app)
    }

    fn load_swf(&mut self, url: Url) -> Result<(), Error> {
        let filename = url
            .path_segments()
            .and_then(|segments| segments.last())
            .unwrap_or_else(|| url.as_str());
        let title = format!("Ruffle - {filename}");
        self.window.set_title(&title);
        SWF_INFO.with(|i| *i.borrow_mut() = Some(filename.to_string()));

        let event_loop_proxy = self.event_loop_proxy.clone();
        let on_metadata = move |swf_header: &ruffle_core::swf::HeaderExt| {
            let _ = event_loop_proxy.send_event(RuffleEvent::OnMetadata(swf_header.clone()));
        };

        let mut parameters: Vec<(String, String)> = url.query_pairs().into_owned().collect();
        parameters.extend(self.opt.parameters());

        if let Some(mut player) = self.player.get() {
            CALLSTACK.with(|callstack| {
                *callstack.borrow_mut() = Some(player.callstack());
            });

            player.fetch_root_movie(url.to_string(), parameters, Box::new(on_metadata));
        } else {
            unimplemented!("TODO: get_or_create player");
        }

        Ok(())
    }

    pub fn run(mut self) -> ! {
        enum LoadingState {
            Loading,
            WaitingForResize,
            Loaded,
        }
        let mut loaded = LoadingState::Loading;
        let mut mouse_pos = PhysicalPosition::new(0.0, 0.0);
        let mut time = Instant::now();
        let mut next_frame_time = Instant::now();
        let mut minimized = false;
        let mut modifiers = ModifiersState::empty();
        let mut fullscreen_down = false;

        if self.opt.input_path.is_none() {
            // No SWF provided on command line; show window with dummy movie immediately.
            self.window.set_visible(true);
            self.gui.lock().expect("Gui lock").set_ui_visible(true);
            loaded = LoadingState::Loaded;
        }

        // Poll UI events.
        let event_loop = self.event_loop.take().expect("App already running");
        event_loop.run(move |event, _window_target, control_flow| {
            let mut check_redraw = false;
            match event {
                winit::event::Event::LoopDestroyed => {
                    if let Some(mut player) = self.player.get() {
                        player.flush_shared_objects();
                    }
                    crate::shutdown();
                    return;
                }

                // Core loop
                winit::event::Event::MainEventsCleared
                    if matches!(loaded, LoadingState::Loaded) =>
                {
                    let new_time = Instant::now();
                    let dt = new_time.duration_since(time).as_micros();
                    if dt > 0 {
                        time = new_time;
                        if let Some(mut player) = self.player.get() {
                            player.tick(dt as f64 / 1000.0);
                            next_frame_time = new_time + player.time_til_next_frame();
                        }
                        check_redraw = true;
                    }
                }

                // Render
                winit::event::Event::RedrawRequested(_) => {
                    // Don't render when minimized to avoid potential swap chain errors in `wgpu`.
                    if !minimized {
                        if let Some(mut player) = self.player.get() {
                            player.render();
                            let renderer = player
                                .renderer_mut()
                                .downcast_mut::<WgpuRenderBackend<MovieView>>()
                                .expect("Renderer must be correct type");
                            self.gui
                                .lock()
                                .expect("Gui lock")
                                .render(Some(renderer.target()));
                        } else {
                            self.gui.lock().expect("Gui lock").render(None);
                        }
                        #[cfg(feature = "tracy")]
                        tracing_tracy::client::Client::running()
                            .expect("tracy client must be running")
                            .frame_mark();
                    }
                }

                winit::event::Event::WindowEvent { event, .. } => {
                    if self.gui.lock().expect("Gui lock").handle_event(&event) {
                        // Event consumed by GUI.
                        return;
                    }
                    match event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                            return;
                        }
                        WindowEvent::Resized(size) => {
                            // TODO: Change this when winit adds a `Window::minimzed` or `WindowEvent::Minimize`.
                            minimized = size.width == 0 && size.height == 0;

                            if let Some(mut player) = self.player.get() {
                                let viewport_scale_factor = self.window.scale_factor();
                                player.set_viewport_dimensions(ViewportDimensions {
                                    width: size.width,
                                    height: size.height,
                                    scale_factor: viewport_scale_factor,
                                });
                            }
                            self.window.request_redraw();
                            if matches!(loaded, LoadingState::WaitingForResize) {
                                loaded = LoadingState::Loaded;
                            }
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            if self.gui.lock().expect("Gui lock").is_context_menu_visible() {
                                return;
                            }

                            if let Some(mut player) = self.player.get() {
                                mouse_pos = position;
                                let event = PlayerEvent::MouseMove {
                                    x: position.x,
                                    y: position.y,
                                };
                                player.handle_event(event);
                            }
                            check_redraw = true;
                        }
                        WindowEvent::MouseInput { button, state, .. } => {
                            if self.gui.lock().expect("Gui lock").is_context_menu_visible() {
                                return;
                            }

                            use ruffle_core::events::MouseButton as RuffleMouseButton;
                            use winit::event::MouseButton;
                            if let Some(mut player) = self.player.get() {
                                let x = mouse_pos.x;
                                let y = mouse_pos.y;
                                let button = match button {
                                    MouseButton::Left => RuffleMouseButton::Left,
                                    MouseButton::Right => RuffleMouseButton::Right,
                                    MouseButton::Middle => RuffleMouseButton::Middle,
                                    MouseButton::Other(_) => RuffleMouseButton::Unknown,
                                };
                                let event = match state {
                                    ElementState::Pressed => {
                                        PlayerEvent::MouseDown { x, y, button }
                                    }
                                    ElementState::Released => PlayerEvent::MouseUp { x, y, button },
                                };
                                if state == ElementState::Pressed
                                    && button == RuffleMouseButton::Right
                                {
                                    // Show context menu.
                                    // TODO: Should be squelched if player consumes the right click event.
                                    let context_menu = player.prepare_context_menu();
                                    self.gui
                                        .lock()
                                        .expect("Gui lock")
                                        .show_context_menu(context_menu);
                                }
                                player.handle_event(event);
                            }
                            check_redraw = true;
                        }
                        WindowEvent::MouseWheel { delta, .. } => {
                            use ruffle_core::events::MouseWheelDelta;
                            use winit::event::MouseScrollDelta;
                            if let Some(mut player) = self.player.get() {
                                let delta = match delta {
                                    MouseScrollDelta::LineDelta(_, dy) => {
                                        MouseWheelDelta::Lines(dy.into())
                                    }
                                    MouseScrollDelta::PixelDelta(pos) => {
                                        MouseWheelDelta::Pixels(pos.y)
                                    }
                                };
                                let event = PlayerEvent::MouseWheel { delta };
                                player.handle_event(event);
                            }
                            check_redraw = true;
                        }
                        WindowEvent::CursorEntered { .. } => {
                            if let Some(mut player) = self.player.get() {
                                player.set_mouse_in_stage(true);
                                if player.needs_render() {
                                    self.window.request_redraw();
                                }
                            }
                        }
                        WindowEvent::CursorLeft { .. } => {
                            if let Some(mut player) = self.player.get() {
                                player.set_mouse_in_stage(false);
                                player.handle_event(PlayerEvent::MouseLeave);
                            }
                            check_redraw = true;
                        }
                        WindowEvent::ModifiersChanged(new_modifiers) => {
                            modifiers = new_modifiers;
                        }
                        WindowEvent::KeyboardInput { input, .. } => {
                            // Handle fullscreen keyboard shortcuts: Alt+Return, Escape.
                            match input {
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Return),
                                    ..
                                } if modifiers.alt() => {
                                    if !fullscreen_down {
                                        if let Some(mut player) = self.player.get() {
                                            player.update(|uc| {
                                                uc.stage.toggle_display_state(uc);
                                            });
                                        }
                                    }
                                    fullscreen_down = true;
                                    return;
                                }
                                KeyboardInput {
                                    state: ElementState::Released,
                                    virtual_keycode: Some(VirtualKeyCode::Return),
                                    ..
                                } if fullscreen_down => {
                                    fullscreen_down = false;
                                }
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                } => {
                                    if let Some(mut player) = self.player.get() {
                                        player.update(|uc| {
                                            uc.stage
                                                .set_display_state(uc, StageDisplayState::Normal);
                                        })
                                    }
                                }
                                _ => (),
                            }

                            if let Some(mut player) = self.player.get() {
                                if let Some(key) = input.virtual_keycode {
                                    let key_code = winit_to_ruffle_key_code(key);
                                    let key_char = winit_key_to_char(key, modifiers.shift());
                                    match input.state {
                                        ElementState::Pressed => {
                                            player.handle_event(PlayerEvent::KeyDown {
                                                key_code,
                                                key_char,
                                            });
                                            if let Some(control_code) =
                                                winit_to_ruffle_text_control(key, modifiers)
                                            {
                                                player.handle_event(PlayerEvent::TextControl {
                                                    code: control_code,
                                                });
                                            }
                                        }
                                        ElementState::Released => {
                                            player.handle_event(PlayerEvent::KeyUp {
                                                key_code,
                                                key_char,
                                            });
                                        }
                                    };
                                    check_redraw = true;
                                }
                            }
                        }
                        WindowEvent::ReceivedCharacter(codepoint) => {
                            if let Some(mut player) = self.player.get() {
                                let event = PlayerEvent::TextInput { codepoint };
                                player.handle_event(event);
                            }
                            check_redraw = true;
                        }
                        _ => (),
                    }
                }
                winit::event::Event::UserEvent(RuffleEvent::TaskPoll) => self
                    .executor
                    .lock()
                    .expect("active executor reference")
                    .poll_all(),
                winit::event::Event::UserEvent(RuffleEvent::OnMetadata(swf_header)) => {
                    let movie_width = swf_header.stage_size().width().to_pixels();
                    let movie_height = swf_header.stage_size().height().to_pixels();

                    let window_size: Size = match (self.opt.width, self.opt.height) {
                        (None, None) => LogicalSize::new(movie_width, movie_height).into(),
                        (Some(width), None) => {
                            let scale = width / movie_width;
                            let height = movie_height * scale;
                            PhysicalSize::new(width.max(1.0), height.max(1.0)).into()
                        }
                        (None, Some(height)) => {
                            let scale = height / movie_height;
                            let width = movie_width * scale;
                            PhysicalSize::new(width.max(1.0), height.max(1.0)).into()
                        }
                        (Some(width), Some(height)) => {
                            PhysicalSize::new(width.max(1.0), height.max(1.0)).into()
                        }
                    };

                    let window_size = Size::clamp(
                        window_size,
                        self.min_window_size.into(),
                        self.max_window_size.into(),
                        self.window.scale_factor(),
                    );

                    self.window.set_inner_size(window_size);
                    self.window.set_fullscreen(if self.opt.fullscreen {
                        Some(Fullscreen::Borderless(None))
                    } else {
                        None
                    });
                    self.window.set_visible(true);

                    let viewport_size = self.window.inner_size();

                    // On X11 (and possibly other platforms), the window size is not updated immediately.
                    // Wait for the window to be resized to the requested size before we start running
                    // the SWF (which can observe the viewport size in "noScale" mode)
                    if window_size != viewport_size.into() {
                        loaded = LoadingState::WaitingForResize;
                    } else {
                        loaded = LoadingState::Loaded;
                    }

                    let viewport_scale_factor = self.window.scale_factor();
                    if let Some(mut player) = self.player.get() {
                        player.set_viewport_dimensions(ViewportDimensions {
                            width: viewport_size.width,
                            height: viewport_size.height,
                            scale_factor: viewport_scale_factor,
                        });
                    }
                }

                winit::event::Event::UserEvent(RuffleEvent::ContextMenuItemClicked(index)) => {
                    if let Some(mut player) = self.player.get() {
                        player.run_context_menu_callback(index);
                    }
                }

                winit::event::Event::UserEvent(RuffleEvent::OpenFile) => {
                    if let Some(path) = pick_file() {
                        // TODO: Show dialog on error.
                        let url = parse_url(&path).expect("Couldn't load specified path");
                        let _ = self.load_swf(url);
                        self.gui.lock().expect("Gui lock").set_ui_visible(false);
                    }
                }

                winit::event::Event::UserEvent(RuffleEvent::OpenURL(url)) => {
                    let _ = self.load_swf(url);
                    self.gui.lock().expect("Gui lock").set_ui_visible(false);
                }

                winit::event::Event::UserEvent(RuffleEvent::ExitRequested) => {
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                _ => (),
            }

            // Check for a redraw request.
            if check_redraw {
                let player = self.player.get();
                let gui = self.gui.lock().expect("Gui lock");
                if player.map(|p| p.needs_render()).unwrap_or_default() || gui.needs_render() {
                    self.window.request_redraw();
                }
            }

            // After polling events, sleep the event loop until the next event or the next frame.
            *control_flow = if matches!(loaded, LoadingState::Loaded) {
                ControlFlow::WaitUntil(next_frame_time)
            } else {
                ControlFlow::Wait
            };
        });
    }
}
