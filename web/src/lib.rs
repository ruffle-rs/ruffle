mod audio;
mod render;
mod shape_utils;

use crate::{audio::WebAudioBackend, render::WebCanvasRenderBackend};
use generational_arena::{Arena, Index};
use js_sys::Uint8Array;
use std::cell::RefCell;
use std::error::Error;
use wasm_bindgen::{prelude::*, JsValue};
use web_sys::HtmlCanvasElement;

thread_local! {
    pub static PLAYERS: RefCell<Arena<ruffle_core::Player>> = RefCell::new(Arena::new());
}

#[wasm_bindgen]
pub struct Player(Index);

#[wasm_bindgen]
impl Player {
    pub fn new(canvas: HtmlCanvasElement, swf_data: Uint8Array) -> Result<Player, JsValue> {
        Player::new_internal(canvas, swf_data).map_err(|_| "Error creating player".into())
    }

    pub fn tick(&mut self, dt: f64) {
        PLAYERS.with(|players| {
            let mut players = players.borrow_mut();
            if let Some(player) = players.get_mut(self.0) {
                player.tick(dt);
            }
        });
    }

    pub fn destroy(&mut self) {
        PLAYERS.with(|players| {
            let mut players = players.borrow_mut();
            players.remove(self.0);
        });
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

        let player = ruffle_core::Player::new(Box::new(renderer), Box::new(audio), data)?;

        // Update canvas size to match player size.
        canvas.set_width(player.movie_width());
        canvas.set_height(player.movie_height());

        let style = canvas.style();
        style
            .set_property("width", &format!("{}px", player.movie_width()))
            .map_err(|_| "Unable to set style")?;
        style
            .set_property("height", &format!("{}px", player.movie_height()))
            .map_err(|_| "Unable to set style")?;

        let handle = PLAYERS.with(move |players| {
            let mut players = players.borrow_mut();
            let index = players.insert(player);
            Player(index)
        });

        Ok(handle)
    }
}
