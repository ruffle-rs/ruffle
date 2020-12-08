#![allow(clippy::same_item_push, clippy::unknown_clippy_lints)]

//! Ruffle web frontend.
mod audio;
mod input;
mod locale;
mod log_adapter;
mod navigator;
mod storage;

use crate::log_adapter::WebLogBackend;
use crate::storage::LocalStorageBackend;
use crate::{
    audio::WebAudioBackend, input::WebInputBackend, locale::WebLocaleBackend,
    navigator::WebNavigatorBackend,
};
use generational_arena::{Arena, Index};
use js_sys::{Array, Function, Object, Uint8Array};
use ruffle_core::backend::input::InputBackend;
use ruffle_core::backend::render::RenderBackend;
use ruffle_core::backend::storage::MemoryStorageBackend;
use ruffle_core::backend::storage::StorageBackend;
use ruffle_core::context::UpdateContext;
use ruffle_core::events::{KeyCode, MouseWheelDelta};
use ruffle_core::external::{
    ExternalInterfaceMethod, ExternalInterfaceProvider, Value as ExternalValue, Value,
};
use ruffle_core::property_map::PropertyMap;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::PlayerEvent;
use ruffle_web_common::JsResult;
use std::collections::BTreeMap;
use std::sync::Once;
use std::sync::{Arc, Mutex};
use std::{cell::RefCell, error::Error, num::NonZeroI32};
use wasm_bindgen::{prelude::*, JsCast, JsValue};
use web_sys::{
    AddEventListenerOptions, Element, EventTarget, HtmlCanvasElement, HtmlElement, KeyboardEvent,
    PointerEvent, WheelEvent,
};

static RUFFLE_GLOBAL_PANIC: Once = Once::new();

thread_local! {
    /// We store the actual instances of the ruffle core in a static pool.
    /// This gives us a clear boundary between the JS side and Rust side, avoiding
    /// issues with lifetimes and type parameters (which cannot be exported with wasm-bindgen).
    static INSTANCES: RefCell<Arena<RefCell<RuffleInstance>>> = RefCell::new(Arena::new());

    static CURRENT_CONTEXT: RefCell<Option<*mut UpdateContext<'static, 'static, 'static>>> = RefCell::new(None);
}

type AnimationHandler = Closure<dyn FnMut(f64)>;

struct RuffleInstance {
    core: Arc<Mutex<ruffle_core::Player>>,
    js_player: JavascriptPlayer,
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
    player_mouse_down_callback: Option<Closure<dyn FnMut(PointerEvent)>>,
    window_mouse_down_callback: Option<Closure<dyn FnMut(PointerEvent)>>,
    mouse_wheel_callback: Option<Closure<dyn FnMut(WheelEvent)>>,
    key_down_callback: Option<Closure<dyn FnMut(KeyboardEvent)>>,
    key_up_callback: Option<Closure<dyn FnMut(KeyboardEvent)>>,
    has_focus: bool,
    trace_observer: Arc<RefCell<JsValue>>,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = Error)]
    type JsError;

    #[wasm_bindgen(constructor, js_class = "Error")]
    fn new(message: &str) -> JsError;
}

#[wasm_bindgen(module = "/packages/core/src/ruffle-player.ts")]
extern "C" {
    #[wasm_bindgen(extends = EventTarget)]
    #[derive(Clone)]
    pub type JavascriptPlayer;

    #[wasm_bindgen(method, js_name = "onCallbackAvailable")]
    fn on_callback_available(this: &JavascriptPlayer, name: &str);

    #[wasm_bindgen(method)]
    fn panic(this: &JavascriptPlayer, error: &JsError);
}

struct JavascriptInterface {
    js_player: JavascriptPlayer,
}

/// An opaque handle to a `RuffleInstance` inside the pool.
///
/// This type is exported to JS, and is used to interact with the library.
#[wasm_bindgen]
#[derive(Clone)]
pub struct Ruffle(Index);

#[wasm_bindgen]
impl Ruffle {
    #[wasm_bindgen(constructor)]
    pub fn new(
        parent: HtmlElement,
        js_player: JavascriptPlayer,
        allow_script_access: bool,
    ) -> Result<Ruffle, JsValue> {
        if RUFFLE_GLOBAL_PANIC.is_completed() {
            // If an actual panic happened, then we can't trust the state it left us in.
            // Prevent future players from loading so that they can inform the user about the error.
            return Err("Ruffle is panicking!".into());
        }
        set_panic_handler();
        Ruffle::new_internal(parent, js_player, allow_script_access)
            .map_err(|_| "Error creating player".into())
    }

    /// Stream an arbitrary movie file from (presumably) the Internet.
    ///
    /// This method should only be called once per player.
    pub fn stream_from(&mut self, movie_url: &str, parameters: &JsValue) -> Result<(), JsValue> {
        INSTANCES.with(|instances| {
            let instances = instances.borrow();
            let instance = instances.get(self.0).unwrap().borrow();
            let mut parameters_to_load = PropertyMap::new();
            populate_movie_parameters(&parameters, &mut parameters_to_load);
            instance
                .core
                .lock()
                .unwrap()
                .fetch_root_movie(movie_url, parameters_to_load);
            Ok(())
        })
    }

    /// Play an arbitrary movie on this instance.
    ///
    /// This method should only be called once per player.
    pub fn load_data(&mut self, swf_data: Uint8Array, parameters: &JsValue) -> Result<(), JsValue> {
        let movie = Arc::new({
            let mut data = vec![0; swf_data.length() as usize];
            swf_data.copy_to(&mut data[..]);
            let mut movie = SwfMovie::from_data(&data, None)
                .map_err(|e| format!("Error loading movie: {}", e))?;
            populate_movie_parameters(&parameters, movie.parameters_mut());
            movie
        });

        INSTANCES.with(|instances| {
            let instances = instances.borrow();
            let instance = instances.get(self.0).unwrap();
            instance.borrow().core.lock().unwrap().set_root_movie(movie);
        });

        Ok(())
    }

    pub fn play(&mut self) {
        // Remove instance from the active list.
        INSTANCES.with(|instances| {
            let instances = instances.borrow();
            let instance = instances.get(self.0).unwrap();
            instance.borrow().core.lock().unwrap().set_is_playing(true);
            log::info!("PLAY!");
        });
    }

    pub fn pause(&mut self) {
        // Remove instance from the active list.
        INSTANCES.with(|instances| {
            let instances = instances.borrow();
            let instance = instances.get(self.0).unwrap();
            instance.borrow().core.lock().unwrap().set_is_playing(false);
            log::info!("PAUSE!");
        });
    }

    pub fn destroy(&mut self) {
        // Remove instance from the active list.
        if let Some(instance) = INSTANCES.with(|instances| {
            if let Ok(mut instances) = instances.try_borrow_mut() {
                instances.remove(self.0)
            } else {
                // If we're being destroyed mid-panic, we won't mind not being able to remove this.
                None
            }
        }) {
            let mut instance = instance.borrow_mut();
            instance.canvas.remove();

            // Stop all audio playing from the instance
            instance.core.lock().unwrap().audio_mut().stop_all_sounds();

            // Clean up all event listeners.
            instance.key_down_callback = None;
            instance.key_up_callback = None;
            instance.mouse_down_callback = None;
            instance.mouse_move_callback = None;
            instance.mouse_up_callback = None;
            instance.player_mouse_down_callback = None;
            instance.window_mouse_down_callback = None;

            // Cancel the animation handler, if it's still active.
            if let Some(id) = instance.animation_handler_id {
                if let Some(window) = web_sys::window() {
                    let _ = window.cancel_animation_frame(id.into());
                }
            }
        }

        // Player is dropped at this point.
    }

    #[allow(clippy::boxed_local)] // for js_bind
    pub fn call_exposed_callback(&self, name: &str, args: Box<[JsValue]>) -> JsValue {
        let args: Vec<ExternalValue> = args.iter().map(js_to_external_value).collect();

        // Re-entrant callbacks need to return through the hole that was punched through for them
        // We record the context of external functions, and then if we get an internal callback
        // during the same call we'll reuse that.
        // This is unsafe by nature. I don't know any safe way to do this.
        if let Some(context) = CURRENT_CONTEXT.with(|v| *v.borrow()) {
            unsafe {
                if let Some(callback) = (*context).external_interface.get_callback(name) {
                    return external_to_js_value(callback.call(&mut *context, name, args));
                }
            }
        }

        INSTANCES.with(move |instances| {
            if let Ok(instances) = instances.try_borrow() {
                if let Some(instance) = instances.get(self.0) {
                    if let Ok(mut player) = instance.borrow().core.try_lock() {
                        return external_to_js_value(player.call_internal_interface(name, args));
                    }
                }
            }
            JsValue::NULL
        })
    }

    pub fn set_trace_observer(&self, observer: JsValue) {
        INSTANCES.with(move |instances| {
            if let Ok(instances) = instances.try_borrow() {
                if let Some(instance) = instances.get(self.0) {
                    *instance.borrow_mut().trace_observer.borrow_mut() = observer;
                }
            }
        })
    }

    /// Returns the web AudioContext used by this player.
    /// Returns `None` if the audio backend does not use Web Audio.
    pub fn audio_context(&self) -> Option<web_sys::AudioContext> {
        INSTANCES.with(move |instances| {
            if let Ok(instances) = instances.try_borrow() {
                if let Some(instance) = instances.get(self.0) {
                    let instance = instance.borrow_mut();
                    let player = instance.core.lock().unwrap();
                    return player
                        .audio()
                        .downcast_ref::<WebAudioBackend>()
                        .map(|audio| audio.audio_context().clone());
                }
            }
            None
        })
    }
}

impl Ruffle {
    fn new_internal(
        parent: HtmlElement,
        js_player: JavascriptPlayer,
        allow_script_access: bool,
    ) -> Result<Ruffle, Box<dyn Error>> {
        let _ = console_log::init_with_level(log::Level::Trace);

        let window = web_sys::window().ok_or("Expected window")?;
        let document = window.document().ok_or("Expected document")?;

        let (canvas, renderer) = create_renderer(&document)?;
        parent
            .append_child(&canvas.clone().into())
            .into_js_result()?;

        let audio = Box::new(WebAudioBackend::new()?);
        let navigator = Box::new(WebNavigatorBackend::new());
        let input = Box::new(WebInputBackend::new(&canvas));
        let locale = Box::new(WebLocaleBackend::new());

        let current_domain = window.location().href().unwrap();

        let local_storage = match window.local_storage() {
            Ok(Some(s)) => {
                Box::new(LocalStorageBackend::new(s, current_domain)) as Box<dyn StorageBackend>
            }
            err => {
                log::warn!("Unable to use localStorage: {:?}\nData will not save.", err);
                Box::new(MemoryStorageBackend::default())
            }
        };

        let trace_observer = Arc::new(RefCell::new(JsValue::UNDEFINED));
        let log = Box::new(WebLogBackend::new(trace_observer.clone()));

        let core = ruffle_core::Player::new(
            renderer,
            audio,
            navigator,
            input,
            local_storage,
            locale,
            log,
        )?;

        // Create instance.
        let instance = RuffleInstance {
            core,
            js_player: js_player.clone(),
            canvas: canvas.clone(),
            canvas_width: 0, // Initialize canvas width and height to 0 to force an initial canvas resize.
            canvas_height: 0,
            device_pixel_ratio: window.device_pixel_ratio(),
            animation_handler: None,
            animation_handler_id: None,
            mouse_move_callback: None,
            mouse_down_callback: None,
            player_mouse_down_callback: None,
            window_mouse_down_callback: None,
            mouse_up_callback: None,
            mouse_wheel_callback: None,
            key_down_callback: None,
            key_up_callback: None,
            timestamp: None,
            has_focus: false,
            trace_observer,
        };

        // Prevent touch-scrolling on canvas.
        canvas.style().set_property("touch-action", "none").unwrap();

        // Register the instance and create the animation frame closure.
        let mut ruffle = INSTANCES.with(move |instances| {
            let index = instances.borrow_mut().insert(RefCell::new(instance));
            let instances = instances.borrow();
            let ruffle = Ruffle(index);

            // Create the external interface
            if allow_script_access {
                let instance = instances.get(index).unwrap();
                let player = instance.borrow().js_player.clone();
                instance
                    .borrow()
                    .core
                    .lock()
                    .unwrap()
                    .add_external_interface(Box::new(JavascriptInterface::new(player)));
            }

            // Create the animation frame closure.
            {
                let mut ruffle = ruffle.clone();
                let instance = instances.get(index).unwrap();
                instance.borrow_mut().animation_handler =
                    Some(Closure::wrap(Box::new(move |timestamp: f64| {
                        ruffle.tick(timestamp);
                    }) as Box<dyn FnMut(f64)>));
            }

            // Create mouse move handler.
            {
                let mouse_move_callback = Closure::wrap(Box::new(move |js_event: PointerEvent| {
                    INSTANCES.with(move |instances| {
                        let instances = instances.borrow();
                        if let Some(instance) = instances.get(index) {
                            let instance = instance.borrow();
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
                let instance = instances.get(index).unwrap();
                instance.borrow_mut().mouse_move_callback = Some(mouse_move_callback);
            }

            // Create mouse down handler.
            {
                let mouse_down_callback = Closure::wrap(Box::new(move |js_event: PointerEvent| {
                    INSTANCES.with(move |instances| {
                        let instances = instances.borrow();
                        if let Some(instance) = instances.get(index) {
                            // Only fire player mouse event for left clicks.
                            if js_event.button() == 0 {
                                if let Some(target) = js_event.current_target() {
                                    let _ = target
                                        .unchecked_ref::<Element>()
                                        .set_pointer_capture(js_event.pointer_id());
                                }
                                let device_pixel_ratio = instance.borrow().device_pixel_ratio;
                                let event = PlayerEvent::MouseDown {
                                    x: f64::from(js_event.offset_x()) * device_pixel_ratio,
                                    y: f64::from(js_event.offset_y()) * device_pixel_ratio,
                                };
                                instance.borrow().core.lock().unwrap().handle_event(event);
                            }

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
                let instance = instances.get(index).unwrap();
                instance.borrow_mut().mouse_down_callback = Some(mouse_down_callback);
            }

            // Create player mouse down handler.
            {
                let player_mouse_down_callback =
                    Closure::wrap(Box::new(move |_js_event: PointerEvent| {
                        INSTANCES.with(move |instances| {
                            let instances = instances.borrow();
                            if let Some(instance) = instances.get(index) {
                                instance.borrow_mut().has_focus = true;
                            }
                        });
                    }) as Box<dyn FnMut(PointerEvent)>);

                let js_player_events: &EventTarget = js_player.as_ref();
                js_player_events
                    .add_event_listener_with_callback(
                        "pointerdown",
                        player_mouse_down_callback.as_ref().unchecked_ref(),
                    )
                    .unwrap();
                let instance = instances.get(index).unwrap();
                instance.borrow_mut().player_mouse_down_callback = Some(player_mouse_down_callback);
            }

            // Create window mouse down handler.
            {
                let window_mouse_down_callback =
                    Closure::wrap(Box::new(move |_js_event: PointerEvent| {
                        INSTANCES.with(|instances| {
                            let instances = instances.borrow();
                            if let Some(instance) = instances.get(index) {
                                // If we actually clicked on the player, this will be reset to true
                                // after the event bubbles down to the player.
                                instance.borrow_mut().has_focus = false;
                            }
                        });
                    }) as Box<dyn FnMut(PointerEvent)>);

                window
                    .add_event_listener_with_callback_and_bool(
                        "pointerdown",
                        window_mouse_down_callback.as_ref().unchecked_ref(),
                        true, // Use capture so this first *before* the player mouse down handler.
                    )
                    .unwrap();
                let instance = instances.get(index).unwrap();
                instance.borrow_mut().window_mouse_down_callback = Some(window_mouse_down_callback);
            }

            // Create mouse up handler.
            {
                let mouse_up_callback = Closure::wrap(Box::new(move |js_event: PointerEvent| {
                    INSTANCES.with(move |instances| {
                        let instances = instances.borrow();
                        if let Some(instance) = instances.get(index) {
                            let instance = instance.borrow();

                            // Only fire player mouse event for left clicks.
                            if js_event.button() == 0 {
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
                            }

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
                let instance = instances.get(index).unwrap();
                instance.borrow_mut().mouse_up_callback = Some(mouse_up_callback);
            }

            // Create mouse wheel handler.
            {
                let mouse_wheel_callback = Closure::wrap(Box::new(move |js_event: WheelEvent| {
                    INSTANCES.with(move |instances| {
                        let instances = instances.borrow();
                        if let Some(instance) = instances.get(index) {
                            let delta = match js_event.delta_mode() {
                                WheelEvent::DOM_DELTA_LINE => {
                                    MouseWheelDelta::Lines(-js_event.delta_y())
                                }
                                WheelEvent::DOM_DELTA_PIXEL => {
                                    MouseWheelDelta::Pixels(-js_event.delta_y())
                                }
                                _ => return,
                            };
                            let core = &instance.borrow().core;
                            let mut core_lock = core.lock().unwrap();
                            core_lock.handle_event(PlayerEvent::MouseWheel { delta });
                            if core_lock.should_prevent_scrolling() {
                                js_event.prevent_default();
                            }
                        }
                    });
                })
                    as Box<dyn FnMut(WheelEvent)>);

                let canvas_events: &EventTarget = canvas.as_ref();
                let mut options = AddEventListenerOptions::new();
                options.passive(false);
                canvas_events
                    .add_event_listener_with_callback_and_add_event_listener_options(
                        "wheel",
                        mouse_wheel_callback.as_ref().unchecked_ref(),
                        &options,
                    )
                    .unwrap();
                let instance = instances.get(index).unwrap();
                instance.borrow_mut().mouse_wheel_callback = Some(mouse_wheel_callback);
            }

            // Create keydown event handler.
            {
                let key_down_callback = Closure::wrap(Box::new(move |js_event: KeyboardEvent| {
                    INSTANCES.with(|instances| {
                        if let Some(instance) = instances.borrow().get(index) {
                            let instance = instance.borrow();
                            if instance.has_focus {
                                let mut core = instance.core.lock().unwrap();
                                let input =
                                    core.input_mut().downcast_mut::<WebInputBackend>().unwrap();
                                input.keydown(&js_event);

                                let key_code = input.last_key_code();
                                let key_char = input.last_key_char();

                                if key_code != KeyCode::Unknown {
                                    core.handle_event(PlayerEvent::KeyDown { key_code });
                                }

                                if let Some(codepoint) = key_char {
                                    core.handle_event(PlayerEvent::TextInput { codepoint });
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
                let instance = instances.get(index).unwrap();
                instance.borrow_mut().key_down_callback = Some(key_down_callback);
            }

            {
                let key_up_callback = Closure::wrap(Box::new(move |js_event: KeyboardEvent| {
                    js_event.prevent_default();
                    INSTANCES.with(|instances| {
                        if let Some(instance) = instances.borrow().get(index) {
                            let instance = instance.borrow();
                            if instance.has_focus {
                                let mut core = instance.core.lock().unwrap();
                                let input =
                                    core.input_mut().downcast_mut::<WebInputBackend>().unwrap();
                                input.keyup(&js_event);

                                let key_code = input.last_key_code();
                                if key_code != KeyCode::Unknown {
                                    core.handle_event(PlayerEvent::KeyUp { key_code });
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
                let mut instance = instances.get(index).unwrap().borrow_mut();
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
            let instances = instances.borrow();
            if let Some(instance) = instances.get(self.0) {
                let window = web_sys::window().unwrap();

                let mut mut_instance = instance.borrow_mut();
                // Calculate the dt from last tick.
                let dt = if let Some(prev_timestamp) = mut_instance.timestamp {
                    mut_instance.timestamp = Some(timestamp);
                    timestamp - prev_timestamp
                } else {
                    // Store the timestamp from the initial tick.
                    // (I tried to use Performance.now() to get the initial timestamp,
                    // but this didn't seem to be accurate and caused negative dts on
                    // Chrome.)
                    mut_instance.timestamp = Some(timestamp);
                    0.0
                };
                drop(mut_instance);

                let core = instance.borrow().core.clone();
                let mut core_lock = core.lock().unwrap();
                core_lock.tick(dt);
                let mut needs_render = core_lock.needs_render();

                // Check for canvas resize.
                let canvas = instance.borrow().canvas.to_owned();
                let canvas_width = canvas.client_width();
                let canvas_height = canvas.client_height();
                let device_pixel_ratio = window.device_pixel_ratio(); // Changes via user zooming.
                if instance.borrow().canvas_width != canvas_width
                    || instance.borrow().canvas_height != canvas_height
                    || (instance.borrow().device_pixel_ratio - device_pixel_ratio).abs()
                        >= std::f64::EPSILON
                {
                    let mut mut_instance = instance.borrow_mut();
                    // If a canvas resizes, its drawing context will get scaled. You must reset
                    // the width and height attributes of the canvas element to recreate the context.
                    // (NOT the CSS width/height!)
                    mut_instance.canvas_width = canvas_width;
                    mut_instance.canvas_height = canvas_height;
                    mut_instance.device_pixel_ratio = device_pixel_ratio;
                    drop(mut_instance);

                    // The actual viewport is scaled by DPI, bigger than CSS pixels.
                    let viewport_width = (f64::from(canvas_width) * device_pixel_ratio) as u32;
                    let viewport_height = (f64::from(canvas_height) * device_pixel_ratio) as u32;
                    canvas.set_width(viewport_width);
                    canvas.set_height(viewport_height);

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
                let mut instance = instance.borrow_mut();
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

struct JavascriptMethod {
    this: JsValue,
    function: JsValue,
}

impl ExternalInterfaceMethod for JavascriptMethod {
    fn call(
        &self,
        context: &mut UpdateContext<'_, '_, '_>,
        args: &[ExternalValue],
    ) -> ExternalValue {
        let old_context = CURRENT_CONTEXT.with(|v| {
            v.replace(Some(unsafe {
                std::mem::transmute::<
                    &mut UpdateContext,
                    &mut UpdateContext<'static, 'static, 'static>,
                >(context)
            } as *mut UpdateContext))
        });
        let result = if let Some(function) = self.function.dyn_ref::<Function>() {
            let args_array = Array::new();
            for arg in args {
                args_array.push(&external_to_js_value(arg.to_owned()));
            }
            if let Ok(result) = function.apply(&self.this, &args_array) {
                js_to_external_value(&result)
            } else {
                ExternalValue::Null
            }
        } else {
            ExternalValue::Null
        };
        CURRENT_CONTEXT.with(|v| v.replace(old_context));
        result
    }
}

impl JavascriptInterface {
    fn new(js_player: JavascriptPlayer) -> Self {
        Self { js_player }
    }

    fn find_method(&self, root: JsValue, name: &str) -> Option<JavascriptMethod> {
        let mut parent = JsValue::UNDEFINED;
        let mut value = root;
        for key in name.split('.') {
            parent = value;
            value = js_sys::Reflect::get(&parent, &JsValue::from_str(key)).ok()?;
        }
        if value.is_function() {
            Some(JavascriptMethod {
                this: parent,
                function: value,
            })
        } else {
            None
        }
    }
}

impl ExternalInterfaceProvider for JavascriptInterface {
    fn get_method(&self, name: &str) -> Option<Box<dyn ExternalInterfaceMethod>> {
        if let Some(method) = self.find_method(self.js_player.clone().into(), name) {
            return Some(Box::new(method));
        }
        if let Some(window) = web_sys::window() {
            if let Some(method) = self.find_method(window.into(), name) {
                return Some(Box::new(method));
            }
        }
        None
    }

    fn on_callback_available(&self, name: &str) {
        self.js_player.on_callback_available(name);
    }
}

fn js_to_external_value(js: &JsValue) -> ExternalValue {
    if let Some(value) = js.as_f64() {
        ExternalValue::Number(value)
    } else if let Some(value) = js.as_string() {
        ExternalValue::String(value)
    } else if let Some(value) = js.as_bool() {
        ExternalValue::Bool(value)
    } else if let Some(array) = js.dyn_ref::<Array>() {
        let mut values = Vec::new();
        for value in array.values() {
            if let Ok(value) = value {
                values.push(js_to_external_value(&value));
            }
        }
        ExternalValue::List(values)
    } else if let Some(object) = js.dyn_ref::<Object>() {
        let mut values = BTreeMap::new();
        for entry in Object::entries(&object).values() {
            if let Ok(entry) = entry.and_then(|v| v.dyn_into::<Array>()) {
                if let Some(key) = entry.get(0).as_string() {
                    values.insert(key, js_to_external_value(&entry.get(1)));
                }
            }
        }
        ExternalValue::Object(values)
    } else {
        ExternalValue::Null
    }
}

fn external_to_js_value(external: ExternalValue) -> JsValue {
    match external {
        Value::Null => JsValue::NULL,
        Value::Bool(value) => JsValue::from_bool(value),
        Value::Number(value) => JsValue::from_f64(value),
        Value::String(value) => JsValue::from_str(&value),
        Value::Object(object) => {
            let entries = Array::new();
            for (key, value) in object {
                entries.push(&Array::of2(
                    &JsValue::from_str(&key),
                    &external_to_js_value(value),
                ));
            }
            if let Ok(result) = Object::from_entries(&entries) {
                result.into()
            } else {
                JsValue::NULL
            }
        }
        Value::List(values) => {
            let array = Array::new();
            for value in values {
                array.push(&external_to_js_value(value));
            }
            array.into()
        }
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
        match ruffle_render_webgl::WebGlRenderBackend::new(&canvas) {
            Ok(renderer) => return Ok((canvas, Box::new(renderer))),
            Err(error) => log::error!("Error creating WebGL renderer: {}", error),
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
        match ruffle_render_canvas::WebCanvasRenderBackend::new(&canvas) {
            Ok(renderer) => return Ok((canvas, Box::new(renderer))),
            Err(error) => log::error!("Error creating canvas renderer: {}", error),
        }
    }

    Err("Unable to create renderer".into())
}

pub fn set_panic_handler() {
    static HOOK_HAS_BEEN_SET: Once = Once::new();
    HOOK_HAS_BEEN_SET.call_once(|| {
        std::panic::set_hook(Box::new(|info| {
            RUFFLE_GLOBAL_PANIC.call_once(|| {
                console_error_panic_hook::hook(info);

                let error = JsError::new(&info.to_string());
                let _ = INSTANCES.try_with(|instances| {
                    let mut players = Vec::new();

                    // We have to be super cautious to not panic here, and not hold any borrows for
                    // longer than we need to.
                    // We grab all of the JsPlayers out from the list and release our hold, as they
                    // may call back to destroy() - which will mutably borrow instances.

                    if let Ok(instances) = instances.try_borrow() {
                        for (_, instance) in instances.iter() {
                            if let Ok(player) = instance.try_borrow().map(|i| i.js_player.clone()) {
                                players.push(player);
                            }
                        }
                    }
                    for player in players {
                        player.panic(&error);
                    }
                });
            });
        }));
    });
}

fn populate_movie_parameters(input: &JsValue, output: &mut PropertyMap<String>) {
    if let Ok(keys) = js_sys::Reflect::own_keys(input) {
        for key in keys.values() {
            if let Ok(key) = key {
                if let Ok(value) = js_sys::Reflect::get(input, &key) {
                    if let (Some(key), Some(value)) = (key.as_string(), value.as_string()) {
                        output.insert(&key, value, false);
                    }
                }
            }
        }
    }
}
