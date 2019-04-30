use fluster_core::backend::{
    audio::web::WebAudioBackend, render::web_canvas::WebCanvasRenderBackend,
};
use js_sys::Uint8Array;
use std::error::Error;
use wasm_bindgen::{prelude::*, JsValue};
use web_sys::HtmlCanvasElement;

#[wasm_bindgen]
pub struct Player(fluster_core::Player);

#[wasm_bindgen]
impl Player {
    pub fn new(canvas: HtmlCanvasElement, swf_data: Uint8Array) -> Result<Player, JsValue> {
        Player::new_internal(canvas, swf_data).map_err(|_| "Error creating player".into())
    }

    pub fn tick(&mut self, dt: f64) {
        self.0.tick(dt);
    }
}

impl Player {
    fn new_internal(canvas: HtmlCanvasElement, swf_data: Uint8Array) -> Result<Player, Box<Error>> {
        console_error_panic_hook::set_once();
        console_log::init_with_level(log::Level::Trace)?;

        let mut data = vec![0; swf_data.length() as usize];
        swf_data.copy_to(&mut data[..]);

        let renderer = WebCanvasRenderBackend::new(&canvas)?;
        let audio = WebAudioBackend::new()?;

        let player = fluster_core::Player::new(Box::new(renderer), Box::new(audio), data)?;

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

        Ok(Player(player))
    }
}
