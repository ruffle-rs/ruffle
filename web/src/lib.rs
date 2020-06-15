//! Ruffle web frontend.
mod audio;
mod input;
mod navigator;
mod storage;

use crate::storage::LocalStorageBackend;
use crate::{audio::WebAudioBackend, input::WebInputBackend, navigator::WebNavigatorBackend};
use generational_arena::{Arena, Index};
use js_sys::Uint8Array;
use ruffle_core::backend::render::RenderBackend;
use ruffle_core::backend::storage::MemoryStorageBackend;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::PlayerEvent;
use ruffle_web_common::JsResult;
use std::mem::drop;
use std::sync::{Arc, Mutex};
use std::{cell::RefCell, error::Error, num::NonZeroI32};
use wasm_bindgen::{prelude::*, JsCast, JsValue};
use web_sys::{Element, EventTarget, HtmlCanvasElement, HtmlElement, KeyboardEvent, PointerEvent};
use ruffle_core::backend::storage::StorageBackend;


thread_local! {
    /// We store the actual instances of the ruffle core in a static pool.
    /// This gives us a clear boundary between the JS side and Rust side, avoiding
    /// issues with lifetimes and type paramters (which cannot be exported with wasm-bindgen).
    static INSTANCES: RefCell<Arena<RuffleInstance>> = RefCell::new(Arena::new());
}

type AnimationHandler = Closure<dyn FnMut(f64)>;

struct RuffleInstance {
    core: Arc<Mutex<ruffle_core::Player>>,
    canvas: HtmlCanvasElement,
    canvas_width: i32,
    canvas_height: i32,
    device_pixel_ratio: f64,
    timestamp: Option<f64>,
    animation_handler: Option<AnimationHandler>, // requestAnimationFrame callback
    animation_handler_id: Option<NonZeroI32>,    // requestAnimationFrame id
    #[allow(dead_code)]
    mouse_move_callback: Option<Closure<dyn FnMut(PointerEvent)>>,
    mouse_down_callback: Option<Closure<dyn FnMut(PointerEvent)>>,
    mouse_up_callback: Option<Closure<dyn FnMut(PointerEvent)>>,
    window_mouse_down_callback: Option<Closure<dyn FnMut(PointerEvent)>>,
    key_down_callback: Option<Closure<dyn FnMut(KeyboardEvent)>>,
    key_up_callback: Option<Closure<dyn FnMut(KeyboardEvent)>>,
    has_focus: bool,
}

/// An opaque handle to a `RuffleInstance` inside the pool.
///
/// This type is exported to JS, and is used to interact with the library.
#[wasm_bindgen]
#[derive(Clone)]
pub struct Ruffle(Index);

#[wasm_bindgen]
impl Ruffle {
    pub fn new(parent: HtmlElement, swf_data: Uint8Array) -> Result<Ruffle, JsValue> {
        Ruffle::new_internal(parent, swf_data).map_err(|_| "Error creating player".into())
    }

    pub fn play(&mut self) {
        // Remove instance from the active list.
        INSTANCES.with(|instances| {
            let mut instances = instances.borrow_mut();
            let instance = instances.get_mut(self.0).unwrap();
            instance.core.lock().unwrap().set_is_playing(true);
            log::info!("PLAY!");
        });
    }

    pub fn destroy(&mut self) -> Result<(), JsValue> {
        // Remove instance from the active list.
        if let Some(mut instance) = INSTANCES.with(|instances| {
            let mut instances = instances.borrow_mut();
            instances.remove(self.0)
        }) {
            instance.canvas.remove();

            // Stop all audio playing from the instance
            let mut player = instance.core.lock().unwrap();
            let audio = player.audio_mut();
            audio.stop_all_sounds();

            // Clean up all event listeners.
            instance.key_down_callback = None;
            instance.key_up_callback = None;
            instance.mouse_down_callback = None;
            instance.mouse_move_callback = None;
            instance.mouse_up_callback = None;
            instance.window_mouse_down_callback = None;

            // Cancel the animation handler, if it's still active.
            if let Some(id) = instance.animation_handler_id {
                if let Some(window) = web_sys::window() {
                    return window.cancel_animation_frame(id.into());
                }
            } }

        // Player is dropped at this point.
        Ok(())
    }
}

impl Ruffle {
    fn new_internal(parent: HtmlElement, swf_data: Uint8Array) -> Result<Ruffle, Box<dyn Error>> {
        console_error_panic_hook::set_once();
        let _ = console_log::init_with_level(log::Level::Trace);

        let movie = {
            let mut data = vec![0; swf_data.length() as usize];
            swf_data.copy_to(&mut data[..]);
            SwfMovie::from_data(&data)?
        };

        let window = web_sys::window().ok_or_else(|| "Expected window")?;
        let document = window.document().ok_or("Expected document")?;

        let (canvas, renderer) = create_renderer(&document)?;
        parent
            .append_child(&canvas.clone().into())
            .into_js_result()?;

        let audio = Box::new(WebAudioBackend::new()?);
        let navigator = Box::new(WebNavigatorBackend::new());
        let input = Box::new(WebInputBackend::new(&canvas));

        let current_domain = window.location().href().unwrap();

        let local_storage = window
            .local_storage()
            .unwrap()
            .map(|s| Box::new(LocalStorageBackend::new(s, current_domain)) as Box<dyn StorageBackend>)
            .unwrap_or_else(|| Box::new(MemoryStorageBackend::default()));

        let core =
            ruffle_core::Player::new(renderer, audio, navigator, input, movie, local_storage)?;
        let mut core_lock = core.lock().unwrap();
        let frame_rate = core_lock.frame_rate();
        core_lock.audio_mut().set_frame_rate(frame_rate);
        drop(core_lock);

        // Create instance.
        let instance = RuffleInstance {
            core,
            canvas: canvas.clone(),
            canvas_width: 0, // Intiailize canvas width and height to 0 to force an initial canvas resize.
            canvas_height: 0,
            device_pixel_ratio: window.device_pixel_ratio(),
            animation_handler: None,
            animation_handler_id: None,
            mouse_move_callback: None,
            mouse_down_callback: None,
            window_mouse_down_callback: None,
            mouse_up_callback: None,
            key_down_callback: None,
            key_up_callback: None,
            timestamp: None,
            has_focus: false,
        };

        // Prevent touch-scrolling on canvas.
        canvas.style().set_property("touch-action", "none").unwrap();

        // Register the instance and create the animation frame closure.
        let mut ruffle = INSTANCES.with(move |instances| {
            let mut instances = instances.borrow_mut();
            let index = instances.insert(instance);
            let ruffle = Ruffle(index);

            // Create the animation frame closure.
            {
                let mut ruffle = ruffle.clone();
                let instance = instances.get_mut(index).unwrap();
                instance.animation_handler = Some(Closure::wrap(Box::new(move |timestamp: f64| {
                    ruffle.tick(timestamp);
                })
                    as Box<dyn FnMut(f64)>));
            }

            // Create mouse move handler.
            {
                let mouse_move_callback = Closure::wrap(Box::new(move |js_event: PointerEvent| {
                    INSTANCES.with(move |instances| {
                        let mut instances = instances.borrow_mut();
                        if let Some(instance) = instances.get_mut(index) {
                            let event = PlayerEvent::MouseMove {
                                x: f64::from(js_event.offset_x()) * instance.device_pixel_ratio,
                                y: f64::from(js_event.offset_y()) * instance.device_pixel_ratio,
                            };
                            instance.core.lock().unwrap().handle_event(event);
                            if instance.has_focus {
                                js_event.prevent_default();
                            }
                        }
                    });
                })
                    as Box<dyn FnMut(PointerEvent)>);
                let canvas_events: &EventTarget = canvas.as_ref();
                canvas_events
                    .add_event_listener_with_callback(
                        "pointermove",
                        mouse_move_callback.as_ref().unchecked_ref(),
                    )
                    .unwrap();
                let instance = instances.get_mut(index).unwrap();
                instance.mouse_move_callback = Some(mouse_move_callback);
            }

            // Create mouse down handler.
            {
                let mouse_down_callback = Closure::wrap(Box::new(move |js_event: PointerEvent| {
                    INSTANCES.with(move |instances| {
                        let mut instances = instances.borrow_mut();
                        if let Some(instance) = instances.get_mut(index) {
                            instance.has_focus = true;
                            if let Some(target) = js_event.current_target() {
                                let _ = target
                                    .unchecked_ref::<Element>()
                                    .set_pointer_capture(js_event.pointer_id());
                            }
                            let event = PlayerEvent::MouseDown {
                                x: f64::from(js_event.offset_x()) * instance.device_pixel_ratio,
                                y: f64::from(js_event.offset_y()) * instance.device_pixel_ratio,
                            };
                            instance.core.lock().unwrap().handle_event(event);
                            js_event.prevent_default();
                        }
                    });
                })
                    as Box<dyn FnMut(PointerEvent)>);
                let canvas_events: &EventTarget = canvas.as_ref();
                canvas_events
                    .add_event_listener_with_callback(
                        "pointerdown",
                        mouse_down_callback.as_ref().unchecked_ref(),
                    )
                    .unwrap();
                let instance = instances.get_mut(index).unwrap();
                instance.mouse_down_callback = Some(mouse_down_callback);
            }

            // Create window mouse down handler.
            {
                let window_mouse_down_callback =
                    Closure::wrap(Box::new(move |_js_event: PointerEvent| {
                        INSTANCES.with(|instances| {
                            let mut instances = instances.borrow_mut();
                            if let Some(instance) = instances.get_mut(index) {
                                // If we actually clicked on the canvas, this will be reset to true
                                // after the event bubbles down to the canvas.
                                instance.has_focus = false;
                            }
                        });
                    }) as Box<dyn FnMut(PointerEvent)>);

                window
                    .add_event_listener_with_callback_and_bool(
                        "pointerdown",
                        window_mouse_down_callback.as_ref().unchecked_ref(),
                        true, // Use capture so this first *before* the canvas mouse down handler.
                    )
                    .unwrap();
                let instance = instances.get_mut(index).unwrap();
                instance.window_mouse_down_callback = Some(window_mouse_down_callback);
            }

            // Create mouse up handler.
            {
                let mouse_up_callback = Closure::wrap(Box::new(move |js_event: PointerEvent| {
                    INSTANCES.with(move |instances| {
                        let mut instances = instances.borrow_mut();
                        if let Some(instance) = instances.get_mut(index) {
                            if let Some(target) = js_event.current_target() {
                                let _ = target
                                    .unchecked_ref::<Element>()
                                    .release_pointer_capture(js_event.pointer_id());
                            }
                            let event = PlayerEvent::MouseUp {
                                x: f64::from(js_event.offset_x()) * instance.device_pixel_ratio,
                                y: f64::from(js_event.offset_y()) * instance.device_pixel_ratio,
                            };
                            instance.core.lock().unwrap().handle_event(event);
                            if instance.has_focus {
                                js_event.prevent_default();
                            }
                        }
                    });
                })
                    as Box<dyn FnMut(PointerEvent)>);
                let canvas_events: &EventTarget = canvas.as_ref();
                canvas_events
                    .add_event_listener_with_callback(
                        "pointerup",
                        mouse_up_callback.as_ref().unchecked_ref(),
                    )
                    .unwrap();
                let instance = instances.get_mut(index).unwrap();
                instance.mouse_up_callback = Some(mouse_up_callback);
            }

            // Create click event handler.
            // {
            //     let click_callback = Closure::wrap(Box::new(move |_| {
            //         INSTANCES.with(move |instances| {
            //             let mut instances = instances.borrow_mut();
            //             if let Some(instance) = instances.get_mut(index) {
            //                 instance.core.lock().unwrap().set_is_playing(true);
            //             }
            //         });
            //     }) as Box<dyn FnMut(Event)>);
            //     let canvas_events: &EventTarget = canvas.as_ref();
            //     canvas_events
            //         .add_event_listener_with_callback(
            //             "click",
            //             click_callback.as_ref().unchecked_ref(),
            //         )
            //         .unwrap();
            //     canvas.style().set_property("cursor", "pointer").unwrap();
            //     let instance = instances.get_mut(index).unwrap();
            //     instance.click_callback = Some(click_callback);
            //     // Do initial render to render "click-to-play".
            //     instance.core.render();
            // }

            // Create keydown event handler.
            {
                let key_down_callback = Closure::wrap(Box::new(move |js_event: KeyboardEvent| {
                    INSTANCES.with(|instances| {
                        if let Some(instance) = instances.borrow_mut().get_mut(index) {
                            if instance.has_focus {
                                let code = js_event.code();
                                instance
                                    .core
                                    .lock()
                                    .unwrap()
                                    .input_mut()
                                    .downcast_mut::<WebInputBackend>()
                                    .unwrap()
                                    .keydown(code.clone());

                                if let Some(codepoint) =
                                    input::web_key_to_codepoint(&js_event.key())
                                {
                                    instance
                                        .core
                                        .lock()
                                        .unwrap()
                                        .handle_event(PlayerEvent::TextInput { codepoint });
                                }

                                if let Some(key_code) = input::web_to_ruffle_key_code(&code) {
                                    instance
                                        .core
                                        .lock()
                                        .unwrap()
                                        .handle_event(PlayerEvent::KeyDown { key_code });
                                }

                                js_event.prevent_default();
                            }
                        }
                    });
                })
                    as Box<dyn FnMut(KeyboardEvent)>);

                window
                    .add_event_listener_with_callback(
                        "keydown",
                        key_down_callback.as_ref().unchecked_ref(),
                    )
                    .unwrap();
                let instance = instances.get_mut(index).unwrap();
                instance.key_down_callback = Some(key_down_callback);
            }

            {
                let key_up_callback = Closure::wrap(Box::new(move |js_event: KeyboardEvent| {
                    js_event.prevent_default();
                    INSTANCES.with(|instances| {
                        if let Some(instance) = instances.borrow_mut().get_mut(index) {
                            if instance.has_focus {
                                let code = js_event.code();
                                instance
                                    .core
                                    .lock()
                                    .unwrap()
                                    .input_mut()
                                    .downcast_mut::<WebInputBackend>()
                                    .unwrap()
                                    .keyup(code.clone());

                                if let Some(key_code) = input::web_to_ruffle_key_code(&code) {
                                    instance
                                        .core
                                        .lock()
                                        .unwrap()
                                        .handle_event(PlayerEvent::KeyUp { key_code });
                                }

                                js_event.prevent_default();
                            }
                        }
                    });
                })
                    as Box<dyn FnMut(KeyboardEvent)>);
                window
                    .add_event_listener_with_callback(
                        "keyup",
                        key_up_callback.as_ref().unchecked_ref(),
                    )
                    .unwrap();
                let instance = instances.get_mut(index).unwrap();
                instance.key_up_callback = Some(key_up_callback);
            }

            ruffle
        });

        // Set initial timestamp and do initial tick to start animation loop.
        ruffle.tick(0.0);

        Ok(ruffle)
    }

    fn tick(&mut self, timestamp: f64) {
        INSTANCES.with(|instances| {
            let mut instances = instances.borrow_mut();
            if let Some(instance) = instances.get_mut(self.0) {
                let window = web_sys::window().unwrap();

                // Calculate the dt from last tick.
                let dt = if let Some(prev_timestamp) = instance.timestamp {
                    instance.timestamp = Some(timestamp);
                    timestamp - prev_timestamp
                } else {
                    // Store the timestamp from the initial tick.
                    // (I tried to use Performance.now() to get the initial timestamp,
                    // but this didn't seem to be accurate and caused negative dts on
                    // Chrome.)
                    instance.timestamp = Some(timestamp);
                    0.0
                };

                let mut core_lock = instance.core.lock().unwrap();
                core_lock.tick(dt);
                let mut needs_render = core_lock.needs_render();

                // Check for canvas resize.
                let canvas_width = instance.canvas.client_width();
                let canvas_height = instance.canvas.client_height();
                let device_pixel_ratio = window.device_pixel_ratio(); // Changes via user zooming.
                if instance.canvas_width != canvas_width
                    || instance.canvas_height != canvas_height
                    || (instance.device_pixel_ratio - device_pixel_ratio).abs() >= std::f64::EPSILON
                {
                    // If a canvas resizes, it's drawing context will get scaled. You must reset
                    // the width and height attributes of the canvas element to recreate the context.
                    // (NOT the CSS width/height!)
                    instance.canvas_width = canvas_width;
                    instance.canvas_height = canvas_height;
                    instance.device_pixel_ratio = device_pixel_ratio;

                    // The actual viewport is scaled by DPI, bigger than CSS pixels.
                    let viewport_width =
                        (f64::from(canvas_width) * instance.device_pixel_ratio) as u32;
                    let viewport_height =
                        (f64::from(canvas_height) * instance.device_pixel_ratio) as u32;
                    instance.canvas.set_width(viewport_width);
                    instance.canvas.set_height(viewport_height);

                    core_lock.set_viewport_dimensions(viewport_width, viewport_height);
                    core_lock
                        .renderer_mut()
                        .set_viewport_dimensions(viewport_width, viewport_height);

                    // Force a re-render if we resize.
                    needs_render = true;
                }

                if needs_render {
                    core_lock.render();
                }

                // Request next animation frame.
                if let Some(handler) = &instance.animation_handler {
                    let window = web_sys::window().unwrap();
                    let id = window
                        .request_animation_frame(handler.as_ref().unchecked_ref())
                        .unwrap();
                    instance.animation_handler_id = NonZeroI32::new(id);
                } else {
                    instance.animation_handler_id = None;
                }
            }
        });
    }
}

fn create_renderer(
    document: &web_sys::Document,
) -> Result<(HtmlCanvasElement, Box<dyn RenderBackend>), Box<dyn Error>> {
    #[cfg(not(any(feature = "canvas", feature = "webgl")))]
    std::compile_error!("You must enable one of the render backend features (e.g., webgl).");

    // Try to create a backend, falling through to the next backend on failure.
    // We must recreate the canvas each attempt, as only a single context may be created per canvas
    // with `getContext`.
    #[cfg(feature = "webgl")]
    {
        log::info!("Creating WebGL renderer...");
        let canvas: HtmlCanvasElement = document
            .create_element("canvas")
            .into_js_result()?
            .dyn_into()
            .map_err(|_| "Expected HtmlCanvasElement")?;
        if let Ok(renderer) = ruffle_render_webgl::WebGlRenderBackend::new(&canvas) {
            return Ok((canvas, Box::new(renderer)));
        }
    }

    #[cfg(feature = "canvas")]
    {
        log::info!("Falling back to Canvas renderer...");
        let canvas: HtmlCanvasElement = document
            .create_element("canvas")
            .into_js_result()?
            .dyn_into()
            .map_err(|_| "Expected HtmlCanvasElement")?;
        if let Ok(renderer) = ruffle_render_canvas::WebCanvasRenderBackend::new(&canvas) {
            return Ok((canvas, Box::new(renderer)));
        }
    }

    Err("Unable to create renderer".into())
}
