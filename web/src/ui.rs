use super::JavascriptPlayer;
use ruffle_core::backend::ui::UiBackend;

pub struct WebUiBackend {
    js_player: JavascriptPlayer,
}

impl WebUiBackend {
    pub fn new(js_player: JavascriptPlayer) -> Self {
        Self { js_player }
    }
}

impl UiBackend for WebUiBackend {
    fn is_fullscreen(&self) -> bool {
        self.js_player.is_fullscreen()
    }

    fn message(&self, message: &str) {
        self.js_player.display_message(message);
    }
}
