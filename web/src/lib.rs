//! Ruffle web frontend.
mod audio;
mod render;
mod navigator;
mod utils;

use crate::{audio::WebAudioBackend, render::WebCanvasRenderBackend, navigator::WebNavigatorBackend};
use generational_arena::{Arena, Index};
use js_sys::Uint8Array;
use ruffle_core::{backend::render::RenderBackend, PlayerEvent};
use std::{cell::RefCell, error::Error, num::NonZeroI32};
use wasm_bindgen::{prelude::*, JsCast, JsValue};
use web_sys::{Event, EventTarget, HtmlCanvasElement, MouseEvent};

thread_local! {
    /// We store the actual instances of the ruffle core in a static pool.
    /// This gives us a clear boundary between the JS side and Rust side, avoiding
    /// issues with lifetimes and type paramters (which cannot be exported with wasm-bindgen).
    static INSTANCES: RefCell<Arena<RuffleInstance>> = RefCell::new(Arena::new());
}

type AnimationHandler = Closure<dyn FnMut(f64)>;

struct RuffleInstance {
    core: ruffle_core::Player<WebAudioBackend, WebCanvasRenderBackend, WebNavigatorBackend>,
    canvas: HtmlCanvasElement,
    canvas_width: i32,
    canvas_height: i32,
    device_pixel_ratio: f64,
    timestamp: Option<f64>,
    animation_handler: Option<AnimationHandler>, // requestAnimationFrame callback
    animation_handler_id: Option<NonZeroI32>,    // requestAnimationFrame id
    #[allow(dead_code)]
    click_callback: Option<Closure<dyn FnMut(Event)>>,
    mouse_move_callback: Option<Closure<dyn FnMut(MouseEvent)>>,
    mouse_down_callback: Option<Closure<dyn FnMut(MouseEvent)>>,
    mouse_up_callback: Option<Closure<dyn FnMut(MouseEvent)>>,
}

/// An opaque handle to a `RuffleInstance` inside the pool.
///
/// This type is exported to JS, and is used to interact with the library.
#[wasm_bindgen]
#[derive(Clone)]
pub struct Ruffle(Index);

#[wasm_bindgen]
impl Ruffle {
    pub fn new(canvas: HtmlCanvasElement, swf_data: Uint8Array) -> Result<Ruffle, JsValue> {
        Ruffle::new_internal(canvas, swf_data).map_err(|_| "Error creating player".into())
    }

    pub fn destroy(&mut self) -> Result<(), JsValue> {
        // Remove instance from the active list.
        if let Some(instance) = INSTANCES.with(|instances| {
            let mut instances = instances.borrow_mut();
            instances.remove(self.0)
        }) {
            // Cancel the animation handler, if it's still active.
            if let Some(id) = instance.animation_handler_id {
                if let Some(window) = web_sys::window() {
                    return window.cancel_animation_frame(id.into());
                }
            }
        }

        // Player is dropped at this point.
        Ok(())
    }
}

impl Ruffle {
    fn new_internal(
        canvas: HtmlCanvasElement,
        swf_data: Uint8Array,
    ) -> Result<Ruffle, Box<dyn Error>> {
        console_error_panic_hook::set_once();
        let _ = console_log::init_with_level(log::Level::Trace);

        let mut data = vec![0; swf_data.length() as usize];
        swf_data.copy_to(&mut data[..]);

        let window = web_sys::window().ok_or_else(|| "Expected window")?;
        let renderer = WebCanvasRenderBackend::new(&canvas)?;
        let audio = WebAudioBackend::new()?;
        let navigator = WebNavigatorBackend::new();

        let core = ruffle_core::Player::new(renderer, audio, navigator, data)?;

        // Create instance.
        let instance = RuffleInstance {
            core,
            canvas: canvas.clone(),
            canvas_width: 0, // Intiailize canvas width and height to 0 to force an initial canvas resize.
            canvas_height: 0,
            device_pixel_ratio: window.device_pixel_ratio(),
            animation_handler: None,
            animation_handler_id: None,
            click_callback: None,
            mouse_move_callback: None,
            mouse_down_callback: None,
            mouse_up_callback: None,
            timestamp: None,
        };

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
                let mouse_move_callback = Closure::wrap(Box::new(move |js_event: MouseEvent| {
                    INSTANCES.with(move |instances| {
                        let mut instances = instances.borrow_mut();
                        if let Some(instance) = instances.get_mut(index) {
                            let event = PlayerEvent::MouseMove {
                                x: f64::from(js_event.offset_x()) * instance.device_pixel_ratio,
                                y: f64::from(js_event.offset_y()) * instance.device_pixel_ratio,
                            };
                            instance.core.handle_event(event);
                        }
                    });
                })
                    as Box<dyn FnMut(MouseEvent)>);
                let canvas_events: &EventTarget = canvas.as_ref();
                canvas_events
                    .add_event_listener_with_callback(
                        "mousemove",
                        mouse_move_callback.as_ref().unchecked_ref(),
                    )
                    .unwrap();
                let instance = instances.get_mut(index).unwrap();
                instance.mouse_move_callback = Some(mouse_move_callback);
            }

            // Create mouse down handler.
            {
                let mouse_down_callback = Closure::wrap(Box::new(move |js_event: MouseEvent| {
                    INSTANCES.with(move |instances| {
                        let mut instances = instances.borrow_mut();
                        if let Some(instance) = instances.get_mut(index) {
                            let event = PlayerEvent::MouseDown {
                                x: f64::from(js_event.offset_x()) * instance.device_pixel_ratio,
                                y: f64::from(js_event.offset_y()) * instance.device_pixel_ratio,
                            };
                            instance.core.handle_event(event);
                        }
                    });
                })
                    as Box<dyn FnMut(MouseEvent)>);
                let canvas_events: &EventTarget = canvas.as_ref();
                canvas_events
                    .add_event_listener_with_callback(
                        "mousedown",
                        mouse_down_callback.as_ref().unchecked_ref(),
                    )
                    .unwrap();
                let instance = instances.get_mut(index).unwrap();
                instance.mouse_down_callback = Some(mouse_down_callback);
            }

            // Create mouse up handler.
            {
                let mouse_up_callback = Closure::wrap(Box::new(move |js_event: MouseEvent| {
                    INSTANCES.with(move |instances| {
                        let mut instances = instances.borrow_mut();
                        if let Some(instance) = instances.get_mut(index) {
                            let event = PlayerEvent::MouseUp {
                                x: f64::from(js_event.offset_x()) * instance.device_pixel_ratio,
                                y: f64::from(js_event.offset_y()) * instance.device_pixel_ratio,
                            };
                            instance.core.handle_event(event);
                        }
                    });
                })
                    as Box<dyn FnMut(MouseEvent)>);
                let canvas_events: &EventTarget = canvas.as_ref();
                canvas_events
                    .add_event_listener_with_callback(
                        "mouseup",
                        mouse_up_callback.as_ref().unchecked_ref(),
                    )
                    .unwrap();
                let instance = instances.get_mut(index).unwrap();
                instance.mouse_up_callback = Some(mouse_up_callback);
            }

            // Create click event handler.
            {
                let click_callback = Closure::wrap(Box::new(move |_| {
                    INSTANCES.with(move |instances| {
                        let mut instances = instances.borrow_mut();
                        if let Some(instance) = instances.get_mut(index) {
                            instance.core.set_is_playing(true);
                        }
                    });
                }) as Box<dyn FnMut(Event)>);
                let canvas_events: &EventTarget = canvas.as_ref();
                canvas_events
                    .add_event_listener_with_callback(
                        "click",
                        click_callback.as_ref().unchecked_ref(),
                    )
                    .unwrap();
                canvas.style().set_property("cursor", "pointer").unwrap();
                let instance = instances.get_mut(index).unwrap();
                instance.click_callback = Some(click_callback);
                // Do initial render to render "click-to-play".
                instance.core.render();
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

                instance.core.tick(dt);

                // Check for canvas resize.
                let canvas_width = instance.canvas.client_width();
                let canvas_height = instance.canvas.client_height();
                if instance.canvas_width != canvas_width || instance.canvas_height != canvas_height
                {
                    // If a canvas resizes, it's drawing context will get scaled. You must reset
                    // the width and height attributes of the canvas element to recreate the context.
                    // (NOT the CSS width/height!)
                    instance.canvas_width = canvas_width;
                    instance.canvas_height = canvas_height;

                    // The actual viewport is scaled by DPI, bigger than CSS pixels.
                    let viewport_width =
                        (f64::from(canvas_width) * instance.device_pixel_ratio) as u32;
                    let viewport_height =
                        (f64::from(canvas_height) * instance.device_pixel_ratio) as u32;
                    instance.canvas.set_width(viewport_width);
                    instance.canvas.set_height(viewport_height);
                    instance
                        .core
                        .set_viewport_dimensions(viewport_width, viewport_height);
                    instance
                        .core
                        .renderer_mut()
                        .set_viewport_dimensions(viewport_width, viewport_height);

                    // Force a re-render if we resize.
                    instance.core.render();
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
