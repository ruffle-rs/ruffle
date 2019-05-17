mod audio;
mod render;
mod shape_utils;

use crate::{audio::WebAudioBackend, render::WebCanvasRenderBackend};
use generational_arena::{Arena, Index};
use js_sys::Uint8Array;
use std::{cell::RefCell, error::Error, num::NonZeroI32};
use wasm_bindgen::{prelude::*, JsCast, JsValue};
use web_sys::HtmlCanvasElement;

thread_local! {
    static PLAYERS: RefCell<Arena<PlayerInstance>> = RefCell::new(Arena::new());
}

type AnimationHandler = Closure<FnMut(f64)>;

struct PlayerInstance {
    core: ruffle_core::Player,
    timestamp: f64,
    animation_handler: Option<AnimationHandler>, // requestAnimationFrame callback
    animation_handler_id: Option<NonZeroI32>,    // requestAnimationFrame id
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Player(Index);

#[wasm_bindgen]
impl Player {
    pub fn new(canvas: HtmlCanvasElement, swf_data: Uint8Array) -> Result<Player, JsValue> {
        Player::new_internal(canvas, swf_data).map_err(|_| "Error creating player".into())
    }

    pub fn destroy(&mut self) -> Result<(), JsValue> {
        // Remove instance from the active list.
        if let Some(player_instance) = PLAYERS.with(|players| {
            let mut players = players.borrow_mut();
            players.remove(self.0)
        }) {
            // Cancel the animation handler, if it's still active.
            if let Some(id) = player_instance.animation_handler_id {
                if let Some(window) = web_sys::window() {
                    return window.cancel_animation_frame(id.into());
                }
            }
        }

        // Player is dropped at this point.
        Ok(())
    }
}

impl Player {
    fn new_internal(canvas: HtmlCanvasElement, swf_data: Uint8Array) -> Result<Player, Box<Error>> {
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
        let instance = PlayerInstance {
            core,
            animation_handler: None,
            animation_handler_id: None,
            timestamp,
        };

        // Register the instance and create the animation frame closure.
        let mut player = PLAYERS.with(move |players| {
            let mut players = players.borrow_mut();
            let index = players.insert(instance);
            let player = Player(index);

            // Create the animation frame closure.
            {
                let mut player = player.clone();
                let instance = players.get_mut(index).unwrap();
                instance.animation_handler = Some(Closure::wrap(Box::new(move |timestamp: f64| {
                    player.tick(timestamp);
                })
                    as Box<FnMut(f64)>));
            }

            player
        });

        // Do an initial tick to start the animation loop.
        player.tick(timestamp);

        Ok(player)
    }

    fn tick(&mut self, timestamp: f64) {
        PLAYERS.with(|players| {
            let mut players = players.borrow_mut();
            if let Some(player_instance) = players.get_mut(self.0) {
                let dt = timestamp - player_instance.timestamp;
                player_instance.timestamp = timestamp;
                player_instance.core.tick(dt);

                // Request next animation frame.
                if let Some(handler) = &player_instance.animation_handler {
                    let window = web_sys::window().unwrap();
                    let id = window
                        .request_animation_frame(handler.as_ref().unchecked_ref())
                        .unwrap();
                    player_instance.animation_handler_id = NonZeroI32::new(id);
                } else {
                    player_instance.animation_handler_id = None;
                }
            }
        });
    }
}
