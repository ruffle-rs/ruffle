use fluster_core::backend::render::web_canvas::WebCanvasRenderBackend;
use js_sys::Uint8Array;
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
    fn new_internal(
        canvas: HtmlCanvasElement,
        swf_data: Uint8Array,
    ) -> Result<Player, Box<std::error::Error>> {
        let mut data = vec![0; swf_data.length() as usize];
        swf_data.copy_to(&mut data[..]);

        let renderer = WebCanvasRenderBackend::new(canvas)?;

        let player = fluster_core::Player::new(Box::new(renderer), data)?;
        Ok(Player(player))
    }
}
