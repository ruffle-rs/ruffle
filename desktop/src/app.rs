use crate::custom_event::RuffleEvent;
use crate::gui::{GuiController, MENU_HEIGHT};
use crate::player::{LaunchOptions, PlayerController};
use crate::preferences::GlobalPreferences;
use crate::util::{
    get_screen_size, gilrs_button_to_gamepad_button, parse_url, plot_stats_in_tracy,
    winit_to_ruffle_key_code, winit_to_ruffle_text_control,
};
use anyhow::{Context, Error};
use gilrs::{Event, EventType, Gilrs};
use ruffle_core::PlayerEvent;
use ruffle_render::backend::ViewportDimensions;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use url::Url;
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize, Size};
use winit::event::{ElementState, KeyEvent, Modifiers, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Fullscreen, Icon, Window, WindowBuilder};

pub struct App {
    preferences: GlobalPreferences,
    window: Arc<Window>,
    event_loop: Option<EventLoop<RuffleEvent>>,
    gui: Rc<RefCell<GuiController>>,
    player: PlayerController,
    min_window_size: LogicalSize<u32>,
    max_window_size: PhysicalSize<u32>,
    initial_movie_url: Option<Url>,
    no_gui: bool,
    preferred_width: Option<f64>,
    preferred_height: Option<f64>,
    start_fullscreen: bool,
}

impl App {
    pub async fn new(preferences: GlobalPreferences) -> Result<Self, Error> {
        let movie_url = preferences.cli.movie_url.clone();
        let icon_bytes = include_bytes!("../assets/favicon-32.rgba");
        let icon =
            Icon::from_rgba(icon_bytes.to_vec(), 32, 32).context("Couldn't load app icon")?;

        let event_loop = EventLoopBuilder::with_user_event().build()?;

        let no_gui = preferences.cli.no_gui;
        let min_window_size = (16, if no_gui { 16 } else { MENU_HEIGHT + 16 }).into();
        let max_window_size = get_screen_size(&event_loop);
        let preferred_width = preferences.cli.width;
        let preferred_height = preferences.cli.height;
        let start_fullscreen = preferences.cli.fullscreen;

        let window = WindowBuilder::new()
            .with_visible(false)
            .with_title("Ruffle")
            .with_window_icon(Some(icon))
            .with_min_inner_size(min_window_size)
            .with_max_inner_size(max_window_size)
            .build(&event_loop)?;
        let window = Arc::new(window);

        let mut font_database = fontdb::Database::default();
        font_database.load_system_fonts();

        let mut gui = GuiController::new(
            window.clone(),
            &event_loop,
            preferences.clone(),
            &font_database,
            movie_url.clone(),
            no_gui,
        )
        .await?;

        let mut player = PlayerController::new(
            event_loop.create_proxy(),
            window.clone(),
            gui.descriptors().clone(),
            font_database,
            preferences.clone(),
        );

        if let Some(movie_url) = &movie_url {
            gui.create_movie(
                &mut player,
                LaunchOptions::from(&preferences),
                movie_url.clone(),
            );
        } else {
            gui.show_open_dialog();
        }

        Ok(Self {
            preferences,
            window,
            event_loop: Some(event_loop),
            gui: Rc::new(RefCell::new(gui)),
            player,
            min_window_size,
            max_window_size,
            initial_movie_url: movie_url,
            no_gui,
            preferred_width,
            preferred_height,
            start_fullscreen,
        })
    }

    pub fn run(mut self) -> Result<(), Error> {
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
        let mut modifiers = Modifiers::default();

        if self.initial_movie_url.is_none() {
            // No SWF provided on command line; show window with dummy movie immediately.
            self.window.set_visible(true);
            loaded = LoadingState::Loaded;
        }

        let mut gilrs = Gilrs::new()
            .inspect_err(|err| {
                tracing::warn!("Gamepad support could not be initialized: {err}");
            })
            .ok();

        // Poll UI events.
        let event_loop = self.event_loop.take().expect("App already running");
        let event_loop_proxy = event_loop.create_proxy();
        event_loop.run(move |event, elwt| {
            let mut check_redraw = false;
            match event {
                winit::event::Event::LoopExiting => {
                    if let Some(mut player) = self.player.get() {
                        player.flush_shared_objects();
                    }
                    crate::shutdown();
                    return;
                }

                // Core loop
                // [NA] This used to be called `MainEventsCleared`, but I think the behaviour is different now.
                // We should look at changing our tick to happen somewhere else if we see any behavioural problems.
                winit::event::Event::AboutToWait if matches!(loaded, LoadingState::Loaded) => {
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
                winit::event::Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
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
                    let height_offset = if self.window.fullscreen().is_some() || self.no_gui {
                        0.0
                    } else {
                        MENU_HEIGHT as f64 * self.window.scale_factor()
                    };
                    match event {
                        WindowEvent::CloseRequested => {
                            elwt.exit();
                            return;
                        }
                        WindowEvent::Resized(size) => {
                            // TODO: Change this when winit adds a `Window::minimized` or `WindowEvent::Minimize`.
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
                                    LaunchOptions::from(&self.preferences),
                                    url,
                                );
                            }
                        }
                        WindowEvent::Focused(true) => {
                            self.player.handle_event(PlayerEvent::FocusGained);
                        }
                        WindowEvent::Focused(false) => {
                            self.player.handle_event(PlayerEvent::FocusLost);
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
                                _ => RuffleMouseButton::Unknown,
                            };
                            let event = match state {
                                // TODO We should get information about click index from the OS,
                                //   but winit does not support that yet.
                                ElementState::Pressed => PlayerEvent::MouseDown {
                                    x,
                                    y,
                                    button,
                                    index: None,
                                },
                                ElementState::Released => PlayerEvent::MouseUp { x, y, button },
                            };
                            let handled = self.player.handle_event(event);
                            if !handled
                                && state == ElementState::Pressed
                                && button == RuffleMouseButton::Right
                            {
                                // Show context menu.
                                if let Some(mut player) = self.player.get() {
                                    let context_menu = player.prepare_context_menu();

                                    // MouseUp event will be ignored when the context menu is shown,
                                    // but it has to be dispatched when the menu closes.
                                    let close_event = PlayerEvent::MouseUp {
                                        x,
                                        y,
                                        button: RuffleMouseButton::Right,
                                    };
                                    self.gui
                                        .borrow_mut()
                                        .show_context_menu(context_menu, close_event);
                                }
                            }
                            check_redraw = true;
                        }
                        WindowEvent::MouseWheel { delta, .. } => {
                            if self.gui.borrow_mut().is_context_menu_visible() {
                                return;
                            }

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
                        WindowEvent::KeyboardInput { event, .. } => {
                            if self.gui.borrow_mut().is_context_menu_visible() {
                                return;
                            }

                            // Handle escaping from fullscreen.
                            if let KeyEvent {
                                state: ElementState::Pressed,
                                logical_key: Key::Named(NamedKey::Escape),
                                ..
                            } = event
                            {
                                let _ = event_loop_proxy.send_event(RuffleEvent::ExitFullScreen);
                            }

                            let key_code = winit_to_ruffle_key_code(&event);
                            // [NA] TODO: This event used to give a single char. `last()` is functionally the same,
                            // but we may want to be better at this in the future.
                            let key_char = event.text.clone().and_then(|text| text.chars().last());

                            match (key_code, &event.state) {
                                (Some(key_code), ElementState::Pressed) => {
                                    self.player
                                        .handle_event(PlayerEvent::KeyDown { key_code, key_char });
                                    if let Some(control_code) =
                                        winit_to_ruffle_text_control(&event, &modifiers)
                                    {
                                        self.player.handle_event(PlayerEvent::TextControl {
                                            code: control_code,
                                        });
                                    } else if let Some(text) = event.text {
                                        for codepoint in text.chars() {
                                            self.player
                                                .handle_event(PlayerEvent::TextInput { codepoint });
                                        }
                                    }
                                }
                                (Some(key_code), ElementState::Released) => {
                                    self.player
                                        .handle_event(PlayerEvent::KeyUp { key_code, key_char });
                                }
                                _ => {}
                            };
                            check_redraw = true;
                        }
                        _ => (),
                    }
                }
                winit::event::Event::UserEvent(RuffleEvent::TaskPoll) => self.player.poll(),
                winit::event::Event::UserEvent(RuffleEvent::OnMetadata(swf_header)) => {
                    let height_offset = if self.window.fullscreen().is_some() || self.no_gui {
                        0.0
                    } else {
                        MENU_HEIGHT as f64
                    };

                    // To prevent issues like waiting on resize indefinitely (#11364) or desyncing the window state on Windows,
                    // do not resize while window is maximized.
                    let should_resize = !self.window.is_maximized();

                    let viewport_size = if should_resize {
                        let movie_width = swf_header.stage_size().width().to_pixels();
                        let movie_height = swf_header.stage_size().height().to_pixels();

                        let window_size: Size = match (self.preferred_width, self.preferred_height)
                        {
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

                        let viewport_size = self.window.inner_size();
                        let mut window_resize_denied = false;

                        if let Some(new_viewport_size) = self.window.request_inner_size(window_size)
                        {
                            if new_viewport_size != viewport_size {
                                self.gui.borrow_mut().resize(new_viewport_size);
                            } else {
                                tracing::warn!("Unable to resize window");
                                window_resize_denied = true;
                            }
                        }

                        let viewport_size = self.window.inner_size();

                        // On X11 (and possibly other platforms), the window size is not updated immediately.
                        // On a successful resize request, wait for the window to be resized to the requested size
                        // before we start running the SWF (which can observe the viewport size in "noScale" mode)
                        if !window_resize_denied && window_size != viewport_size.into() {
                            loaded = LoadingState::WaitingForResize;
                        } else {
                            loaded = LoadingState::Loaded;
                        }

                        viewport_size
                    } else {
                        self.window.inner_size()
                    };

                    self.window.set_fullscreen(if self.start_fullscreen {
                        Some(Fullscreen::Borderless(None))
                    } else {
                        None
                    });
                    self.window.set_visible(true);

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
                    let event_loop = event_loop_proxy.clone();
                    let picker = self.gui.borrow().file_picker();
                    tokio::spawn(async move {
                        if let Some(url) = picker
                            .pick_file(None)
                            .await
                            .and_then(|p| Url::from_file_path(p).ok())
                        {
                            let _ = event_loop.send_event(RuffleEvent::Open(url, options));
                        }
                    });
                }

                winit::event::Event::UserEvent(RuffleEvent::Open(url, options)) => {
                    self.gui
                        .borrow_mut()
                        .create_movie(&mut self.player, *options, url);
                }

                winit::event::Event::UserEvent(RuffleEvent::OpenDialog(descriptor)) => {
                    self.gui.borrow_mut().open_dialog(descriptor);
                }

                winit::event::Event::UserEvent(RuffleEvent::CloseFile) => {
                    self.window.set_title("Ruffle"); // Reset title since file has been closed.
                    self.player.destroy();
                }

                winit::event::Event::UserEvent(RuffleEvent::EnterFullScreen) => {
                    if let Some(mut player) = self.player.get() {
                        if player.is_playing() {
                            player.set_fullscreen(true);
                        }
                    }
                }

                winit::event::Event::UserEvent(RuffleEvent::ExitFullScreen) => {
                    if let Some(mut player) = self.player.get() {
                        if player.is_playing() {
                            player.set_fullscreen(false);
                        }
                    }
                }

                winit::event::Event::UserEvent(RuffleEvent::ExitRequested) => {
                    elwt.exit();
                    return;
                }

                _ => (),
            }

            if let Some(Event { event, .. }) = gilrs.as_mut().and_then(|gilrs| gilrs.next_event()) {
                match event {
                    EventType::ButtonPressed(button, _) => {
                        if let Some(button) = gilrs_button_to_gamepad_button(button) {
                            self.player
                                .handle_event(PlayerEvent::GamepadButtonDown { button });
                            check_redraw = true;
                        }
                    }
                    EventType::ButtonReleased(button, _) => {
                        if let Some(button) = gilrs_button_to_gamepad_button(button) {
                            self.player
                                .handle_event(PlayerEvent::GamepadButtonUp { button });
                            check_redraw = true;
                        }
                    }
                    _ => {}
                }
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
            elwt.set_control_flow(if matches!(loaded, LoadingState::Loaded) {
                if let Some(next_frame_time) = next_frame_time {
                    ControlFlow::WaitUntil(next_frame_time)
                } else {
                    // prevent 100% cpu use
                    // TODO: use set_request_repaint_callback to correctly get egui repaint requests.
                    ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(10))
                }
            } else {
                ControlFlow::Wait
            });
        })?;
        Ok(())
    }
}
