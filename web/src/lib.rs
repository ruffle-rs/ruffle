use js_sys::Uint8Array;
use wasm_bindgen::{prelude::*, JsValue};

#[wasm_bindgen]
pub struct Player(fluster_core::Player);

#[wasm_bindgen]
impl Player {
    pub fn new(swf_data: Uint8Array) -> Result<Player, JsValue> {
        let mut data = vec![0; swf_data.length() as usize];
        swf_data.copy_to(&mut data[..]);

        let player = fluster_core::Player::new(data).map_err(|_| JsValue::null())?;
        Ok(Player(player))
    }

    pub fn tick(&mut self, dt: f64) {
        self.0.tick(dt);
    }
}
