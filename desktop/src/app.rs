use crate::cli::Opt;
use crate::custom_event::RuffleEvent;
use crate::gui::{GuiController, MENU_HEIGHT};
use crate::player::{PlayerController, PlayerOptions};
use crate::util::{
    get_screen_size, parse_url, pick_file, plot_stats_in_tracy, winit_key_to_char,
    winit_to_ruffle_key_code, winit_to_ruffle_text_control,
};
use anyhow::{Context, Error};
use ruffle_core::{PlayerEvent, StageDisplayState};
use ruffle_render::backend::ViewportDimensions;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};
use url::Url;
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize, Size};
use winit::event::{ElementState, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use winit::window::{Fullscreen, Icon, Window, WindowBuilder};

pub struct App {
    opt: Opt,
    window: Rc<Window>,
    event_loop: Option<EventLoop<RuffleEvent>>,
    gui: Rc<RefCell<GuiController>>,
    player: PlayerController,
    min_window_size: LogicalSize<u32>,
    max_window_size: PhysicalSize<u32>,
}

impl App {
    pub fn new(opt: Opt) -> Result<Self, Error> {
        let movie_url = opt.movie_url.clone();
        let icon_bytes = include_bytes!("../assets/favicon-32.rgba");
        let icon =
            Icon::from_rgba(icon_bytes.to_vec(), 32, 32).context("Couldn't load app icon")?;

        let event_loop = EventLoopBuilder::with_user_event().build();

        let min_window_size = (16, if opt.no_gui { 16 } else { MENU_HEIGHT + 16 }).into();
        let max_window_size = get_screen_size(&event_loop);

        let window = WindowBuilder::new()
            .with_visible(false)
            .with_title("Ruffle")
            .with_window_icon(Some(icon))
            .with_min_inner_size(min_window_size)
            .with_max_inner_size(max_window_size)
            .build(&event_loop)?;
        let window = Rc::new(window);

        let mut gui = GuiController::new(window.clone(), &event_loop, &opt)?;

        let mut player = PlayerController::new(
            event_loop.create_proxy(),
            window.clone(),
            gui.descriptors().clone(),
        );

        if let Some(movie_url) = movie_url {
            gui.create_movie(&mut player, PlayerOptions::from(&opt), movie_url);
        } else {
            gui.show_open_dialog();
        }

        Ok(Self {
            opt,
            window,
            event_loop: Some(event_loop),
            gui: Rc::new(RefCell::new(gui)),
            player,
            min_window_size,
            max_window_size,
        })
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
        let mut next_frame_time = None;
        let mut minimized = false;
        let mut modifiers = ModifiersState::empty();
        let mut fullscreen_down = false;

        if self.opt.movie_url.is_none() {
            // No SWF provided on command line; show window with dummy movie immediately.
            self.window.set_visible(true);
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
                            next_frame_time = Some(new_time + player.time_til_next_frame());
                        } else {
                            next_frame_time = None;
                        }
                        check_redraw = true;
                    }
                }

                // Render
                winit::event::Event::RedrawRequested(_) => {
                    // Don't render when minimized to avoid potential swap chain errors in `wgpu`.
                    if !minimized {
                        if let Some(mut player) = self.player.get() {
                            // Even if the movie is paused, user interaction with debug tools can change the render output
                            player.render();
                            self.gui.borrow_mut().render(Some(player));
                        } else {
                            self.gui.borrow_mut().render(None);
                        }
                        plot_stats_in_tracy(&self.gui.borrow().descriptors().wgpu_instance);
                    }
                }

                winit::event::Event::WindowEvent { event, .. } => {
                    if self.gui.borrow_mut().handle_event(&event) {
                        // Event consumed by GUI.
                        return;
                    }
                    let height_offset = if self.window.fullscreen().is_some() || self.opt.no_gui {
                        0.0
                    } else {
                        MENU_HEIGHT as f64 * self.window.scale_factor()
                    };
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
                                    height: size.height - height_offset as u32,
                                    scale_factor: viewport_scale_factor,
                                });
                            }
                            self.window.request_redraw();
                            if matches!(loaded, LoadingState::WaitingForResize) {
                                loaded = LoadingState::Loaded;
                            }
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            if self.gui.borrow_mut().is_context_menu_visible() {
                                return;
                            }

                            mouse_pos = position;
                            let event = PlayerEvent::MouseMove {
                                x: position.x,
                                y: position.y - height_offset,
                            };
                            self.player.handle_event(event);
                            check_redraw = true;
                        }
                        WindowEvent::DroppedFile(file) => {
                            if let Ok(url) = parse_url(&file) {
                                self.gui.borrow_mut().create_movie(
                                    &mut self.player,
                                    PlayerOptions::from(&self.opt),
                                    url,
                                );
                            }
                        }
                        WindowEvent::MouseInput { button, state, .. } => {
                            if self.gui.borrow_mut().is_context_menu_visible() {
                                return;
                            }

                            use ruffle_core::events::MouseButton as RuffleMouseButton;
                            use winit::event::MouseButton;
                            let x = mouse_pos.x;
                            let y = mouse_pos.y - height_offset;
                            let button = match button {
                                MouseButton::Left => RuffleMouseButton::Left,
                                MouseButton::Right => RuffleMouseButton::Right,
                                MouseButton::Middle => RuffleMouseButton::Middle,
                                MouseButton::Other(_) => RuffleMouseButton::Unknown,
                            };
                            let event = match state {
                                ElementState::Pressed => PlayerEvent::MouseDown { x, y, button },
                                ElementState::Released => PlayerEvent::MouseUp { x, y, button },
                            };
                            if state == ElementState::Pressed && button == RuffleMouseButton::Right
                            {
                                // Show context menu.
                                // TODO: Should be squelched if player consumes the right click event.
                                if let Some(mut player) = self.player.get() {
                                    let context_menu = player.prepare_context_menu();
                                    self.gui.borrow_mut().show_context_menu(context_menu);
                                }
                            }
                            self.player.handle_event(event);
                            check_redraw = true;
                        }
                        WindowEvent::MouseWheel { delta, .. } => {
                            use ruffle_core::events::MouseWheelDelta;
                            use winit::event::MouseScrollDelta;
                            let delta = match delta {
                                MouseScrollDelta::LineDelta(_, dy) => {
                                    MouseWheelDelta::Lines(dy.into())
                                }
                                MouseScrollDelta::PixelDelta(pos) => MouseWheelDelta::Pixels(pos.y),
                            };
                            let event = PlayerEvent::MouseWheel { delta };
                            self.player.handle_event(event);
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
                            }
                            self.player.handle_event(PlayerEvent::MouseLeave);
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
                                        if player.is_playing() {
                                            player.update(|uc| {
                                                uc.stage.set_display_state(
                                                    uc,
                                                    StageDisplayState::Normal,
                                                );
                                            })
                                        }
                                    }
                                }
                                _ => (),
                            }

                            if let Some(key) = input.virtual_keycode {
                                let key_code = winit_to_ruffle_key_code(key);
                                let key_char = winit_key_to_char(key, modifiers.shift());
                                match input.state {
                                    ElementState::Pressed => {
                                        self.player.handle_event(PlayerEvent::KeyDown {
                                            key_code,
                                            key_char,
                                        });
                                        if let Some(control_code) =
                                            winit_to_ruffle_text_control(key, modifiers)
                                        {
                                            self.player.handle_event(PlayerEvent::TextControl {
                                                code: control_code,
                                            });
                                        }
                                    }
                                    ElementState::Released => {
                                        self.player.handle_event(PlayerEvent::KeyUp {
                                            key_code,
                                            key_char,
                                        });
                                    }
                                };
                                check_redraw = true;
                            }
                        }
                        WindowEvent::ReceivedCharacter(codepoint) => {
                            let event = PlayerEvent::TextInput { codepoint };
                            self.player.handle_event(event);
                            check_redraw = true;
                        }
                        _ => (),
                    }
                }
                winit::event::Event::UserEvent(RuffleEvent::TaskPoll) => self.player.poll(),
                winit::event::Event::UserEvent(RuffleEvent::OnMetadata(swf_header)) => {
                    let movie_width = swf_header.stage_size().width().to_pixels();
                    let movie_height = swf_header.stage_size().height().to_pixels();
                    let height_offset = if self.window.fullscreen().is_some() || self.opt.no_gui {
                        0.0
                    } else {
                        MENU_HEIGHT as f64
                    };

                    let window_size: Size = match (self.opt.width, self.opt.height) {
                        (None, None) => {
                            LogicalSize::new(movie_width, movie_height + height_offset).into()
                        }
                        (Some(width), None) => {
                            let scale = width / movie_width;
                            let height = movie_height * scale;
                            PhysicalSize::new(
                                width.max(1.0),
                                height.max(1.0) + height_offset * self.window.scale_factor(),
                            )
                            .into()
                        }
                        (None, Some(height)) => {
                            let scale = height / movie_height;
                            let width = movie_width * scale;
                            PhysicalSize::new(
                                width.max(1.0),
                                height.max(1.0) + height_offset * self.window.scale_factor(),
                            )
                            .into()
                        }
                        (Some(width), Some(height)) => PhysicalSize::new(
                            width.max(1.0),
                            height.max(1.0) + height_offset * self.window.scale_factor(),
                        )
                        .into(),
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
                            height: viewport_size.height - height_offset as u32,
                            scale_factor: viewport_scale_factor,
                        });
                    }
                }

                winit::event::Event::UserEvent(RuffleEvent::ContextMenuItemClicked(index)) => {
                    if let Some(mut player) = self.player.get() {
                        player.run_context_menu_callback(index);
                    }
                }

                winit::event::Event::UserEvent(RuffleEvent::BrowseAndOpen(options)) => {
                    if let Some(url) =
                        pick_file(false, None).and_then(|p| Url::from_file_path(p).ok())
                    {
                        self.gui
                            .borrow_mut()
                            .create_movie(&mut self.player, *options, url);
                    }
                }

                winit::event::Event::UserEvent(RuffleEvent::OpenURL(url, options)) => {
                    self.gui
                        .borrow_mut()
                        .create_movie(&mut self.player, *options, url);
                }

                winit::event::Event::UserEvent(RuffleEvent::CloseFile) => {
                    self.player.destroy();
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
                let gui = self.gui.borrow_mut();
                if player.map(|p| p.needs_render()).unwrap_or_default() || gui.needs_render() {
                    self.window.request_redraw();
                }
            }

            // After polling events, sleep the event loop until the next event or the next frame.
            *control_flow = if matches!(loaded, LoadingState::Loaded) {
                if let Some(next_frame_time) = next_frame_time {
                    ControlFlow::WaitUntil(next_frame_time)
                } else {
                    // prevent 100% cpu use
                    // TODO: use set_request_repaint_callback to correctly get egui repaint requests.
                    ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(10))
                }
            } else {
                ControlFlow::Wait
            };
        });
    }
}
