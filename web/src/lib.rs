//! Ruffle web frontend.
mod audio;
mod render;

use crate::{audio::WebAudioBackend, render::WebCanvasRenderBackend};
use generational_arena::{Arena, Index};
use js_sys::Uint8Array;
use std::{cell::RefCell, error::Error, num::NonZeroI32};
use wasm_bindgen::{prelude::*, JsCast, JsValue};
use web_sys::{Event, EventTarget, HtmlCanvasElement};

thread_local! {
    /// We store the actual instances of the ruffle core in a static pool.
    /// This gives us a clear boundary between the JS side and Rust side, avoiding
    /// issues with lifetimes and type paramters (which cannot be exported with wasm-bindgen).
    static INSTANCES: RefCell<Arena<RuffleInstance>> = RefCell::new(Arena::new());
}

type AnimationHandler = Closure<FnMut(f64)>;

struct RuffleInstance {
    core: ruffle_core::Player,
    timestamp: f64,
    animation_handler: Option<AnimationHandler>, // requestAnimationFrame callback
    animation_handler_id: Option<NonZeroI32>,    // requestAnimationFrame id
    #[allow(dead_code)]
    click_callback: Option<Closure<FnMut(Event)>>,
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
    fn new_internal(canvas: HtmlCanvasElement, swf_data: Uint8Array) -> Result<Ruffle, Box<Error>> {
        console_error_panic_hook::set_once();
        let _ = console_log::init_with_level(log::Level::Trace);

        let mut data = vec![0; swf_data.length() as usize];
        swf_data.copy_to(&mut data[..]);

        let renderer = WebCanvasRenderBackend::new(&canvas)?;
        let audio = WebAudioBackend::new()?;

        let core = ruffle_core::Player::new(Box::new(renderer), Box::new(audio), data)?;

        // Update canvas size to match player size.
        canvas.set_width(core.movie_width());
        canvas.set_height(core.movie_height());

        let style = canvas.style();
        style
            .set_property("width", &format!("{}px", core.movie_width()))
            .map_err(|_| "Unable to set style")?;
        style
            .set_property("height", &format!("{}px", core.movie_height()))
            .map_err(|_| "Unable to set style")?;

        let window = web_sys::window().ok_or_else(|| "Expected window")?;
        let timestamp = window
            .performance()
            .ok_or_else(|| "Expected performance")?
            .now();

        // Create instance.
        let instance = RuffleInstance {
            core,
            animation_handler: None,
            animation_handler_id: None,
            click_callback: None,
            timestamp,
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
                    as Box<FnMut(f64)>));
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
                }) as Box<FnMut(Event)>);
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

                // Do an initial render for the pause overlay.
                instance.core.render();
            }

            ruffle
        });

        // Do an initial tick to start the animation loop.
        ruffle.tick(timestamp);

        Ok(ruffle)
    }

    fn tick(&mut self, timestamp: f64) {
        INSTANCES.with(|instances| {
            let mut instances = instances.borrow_mut();
            if let Some(instance) = instances.get_mut(self.0) {
                let dt = timestamp - instance.timestamp;
                instance.timestamp = timestamp;

                instance.core.tick(dt);

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
