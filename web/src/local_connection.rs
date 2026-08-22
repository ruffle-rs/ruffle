use ruffle_core::backend::local_connection::{
    ExternalLocalConnectionMessage, LocalConnectionBackend,
};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

/// Prefix for localStorage keys used to track active LocalConnection listeners.
const LS_KEY_PREFIX: &str = "__ruffle_lc:";

/// How often the heartbeat refreshes localStorage timestamps, in seconds.
const HEARTBEAT_INTERVAL_SECS: f64 = 10.0;

/// Entries older than this are considered stale (1.5x heartbeat interval), in milliseconds.
const STALENESS_THRESHOLD_MS: f64 = HEARTBEAT_INTERVAL_SECS * 1.5 * 1000.0;

/// Web implementation of LocalConnectionBackend using the BroadcastChannel API
/// and localStorage for listener registry/discovery.
///
/// This enables cross-tab/cross-iframe LocalConnection communication between
/// different Ruffle player instances on the same origin.
///
/// - **BroadcastChannel** is used for actual message delivery.
/// - **localStorage** is used as a shared registry of active listeners,
///   enabling cross-tab connect uniqueness and delivery status reporting.
pub struct WebLocalConnectionBackend {
    channel: web_sys::BroadcastChannel,
    incoming: Rc<RefCell<Vec<ExternalLocalConnectionMessage>>>,
    _on_message: Closure<dyn FnMut(web_sys::MessageEvent)>,

    /// Connection names registered by this instance (for cleanup on Drop / beforeunload).
    registered_names: Rc<RefCell<Vec<String>>>,

    /// The `beforeunload` event listener closure, stored to prevent GC.
    _on_beforeunload: Option<Closure<dyn FnMut(web_sys::Event)>>,

    /// Last time we refreshed heartbeat timestamps (ms since epoch).
    last_heartbeat: f64,

    /// Reference to the core Player, used to wake it up upon receiving a message.
    player: Rc<RefCell<Option<std::sync::Weak<std::sync::Mutex<ruffle_core::Player>>>>>,
}

impl WebLocalConnectionBackend {
    pub fn new() -> Self {
        let channel = web_sys::BroadcastChannel::new("__ruffle_local_connection__")
            .expect("Failed to create BroadcastChannel for LocalConnection");

        let incoming: Rc<RefCell<Vec<ExternalLocalConnectionMessage>>> =
            Rc::new(RefCell::new(Vec::new()));

        let player: Rc<RefCell<Option<std::sync::Weak<std::sync::Mutex<ruffle_core::Player>>>>> =
            Rc::new(RefCell::new(None));

        let incoming_clone = incoming.clone();
        let player_clone = player.clone();
        let on_message = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
            let data = event.data();
            match Self::parse_message(&data) {
                Ok(msg) => {
                    incoming_clone.borrow_mut().push(msg);
                    if let Some(player) = player_clone.borrow().as_ref().and_then(|p| p.upgrade())
                        && let Ok(mut player_lock) = player.try_lock()
                    {
                        player_lock.update_local_connections();
                    }
                }
                Err(()) => {
                    tracing::warn!(
                        "Failed to parse incoming LocalConnection BroadcastChannel message; dropping"
                    );
                }
            }
        }) as Box<dyn FnMut(web_sys::MessageEvent)>);

        channel.set_onmessage(Some(on_message.as_ref().unchecked_ref()));

        // Set up a beforeunload handler to clean up localStorage entries
        // in case the tab is closed without explicit close() calls.
        let registered_names: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let cleanup_ref = registered_names.clone();
        let on_beforeunload = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            let names = cleanup_ref.borrow();
            if let Some(storage) = Self::get_local_storage() {
                for name in names.iter() {
                    let key = format!("{LS_KEY_PREFIX}{name}");
                    let _ = storage.remove_item(&key);
                }
            }
        }) as Box<dyn FnMut(web_sys::Event)>);

        let beforeunload_attached = if let Some(window) = web_sys::window() {
            window
                .add_event_listener_with_callback(
                    "beforeunload",
                    on_beforeunload.as_ref().unchecked_ref(),
                )
                .is_ok()
        } else {
            false
        };

        Self {
            channel,
            incoming,
            _on_message: on_message,
            registered_names,
            _on_beforeunload: if beforeunload_attached {
                Some(on_beforeunload)
            } else {
                None
            },
            last_heartbeat: 0.0,
            player,
        }
    }

    /// Get the browser's localStorage, if available.
    fn get_local_storage() -> Option<web_sys::Storage> {
        web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
    }

    /// Get the current timestamp in milliseconds since epoch.
    fn now_ms() -> f64 {
        js_sys::Date::now()
    }

    /// Parse a JS message event data into an ExternalLocalConnectionMessage.
    fn parse_message(data: &JsValue) -> Result<ExternalLocalConnectionMessage, ()> {
        let connection_name = js_sys::Reflect::get(data, &JsValue::from_str("connectionName"))
            .map_err(|_| ())?
            .as_string()
            .ok_or(())?;

        let method_name = js_sys::Reflect::get(data, &JsValue::from_str("methodName"))
            .map_err(|_| ())?
            .as_string()
            .ok_or(())?;

        let amf_data_js =
            js_sys::Reflect::get(data, &JsValue::from_str("amfData")).map_err(|_| ())?;
        let amf_data_array = amf_data_js
            .dyn_into::<js_sys::Uint8Array>()
            .map_err(|_| ())?;
        let amf_data = amf_data_array.to_vec();

        Ok(ExternalLocalConnectionMessage {
            connection_name,
            method_name,
            amf_data,
        })
    }

    /// Create a JS object to send via postMessage.
    fn create_message(
        connection_name: &str,
        method_name: &str,
        amf_data: &[u8],
    ) -> Result<JsValue, JsValue> {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("connectionName"),
            &JsValue::from_str(connection_name),
        )?;
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("methodName"),
            &JsValue::from_str(method_name),
        )?;
        let amf_array = js_sys::Uint8Array::from(amf_data);
        js_sys::Reflect::set(&obj, &JsValue::from_str("amfData"), &amf_array)?;
        Ok(obj.into())
    }

    /// Refresh heartbeat timestamps for all registered names.
    fn refresh_heartbeat(&mut self) {
        let now = Self::now_ms();
        if now - self.last_heartbeat < HEARTBEAT_INTERVAL_SECS * 1000.0 {
            return; // Not time yet.
        }
        self.last_heartbeat = now;

        if let Some(storage) = Self::get_local_storage() {
            let timestamp = now.to_string();
            let names = self.registered_names.borrow();
            for name in names.iter() {
                let key = format!("{LS_KEY_PREFIX}{name}");
                let _ = storage.set_item(&key, &timestamp);
            }
        }
    }
}

impl LocalConnectionBackend for WebLocalConnectionBackend {
    fn register_listener(&mut self, connection_name: &str) {
        if let Some(storage) = Self::get_local_storage() {
            let key = format!("{LS_KEY_PREFIX}{connection_name}");
            let timestamp = Self::now_ms().to_string();
            let _ = storage.set_item(&key, &timestamp);
        }
        self.registered_names
            .borrow_mut()
            .push(connection_name.to_string());
        // Reset heartbeat so next poll will refresh at the right interval.
        self.last_heartbeat = Self::now_ms();
    }

    fn unregister_listener(&mut self, connection_name: &str) {
        if let Some(storage) = Self::get_local_storage() {
            let key = format!("{LS_KEY_PREFIX}{connection_name}");
            let _ = storage.remove_item(&key);
        }
        self.registered_names
            .borrow_mut()
            .retain(|n| n != connection_name);
    }

    fn has_remote_listener(&self, connection_name: &str) -> bool {
        if let Some(storage) = Self::get_local_storage() {
            let key = format!("{LS_KEY_PREFIX}{connection_name}");
            if let Ok(Some(value)) = storage.get_item(&key) {
                // Check staleness: if the timestamp is too old, the listener is dead.
                if let Ok(timestamp) = value.parse::<f64>() {
                    let age_ms = Self::now_ms() - timestamp;
                    if age_ms <= STALENESS_THRESHOLD_MS {
                        return true;
                    }
                    // Stale entry — clean it up.
                    let _ = storage.remove_item(&key);
                }
            }
        }
        false
    }

    fn send_message(&mut self, connection_name: &str, method_name: &str, amf_data: &[u8]) {
        match Self::create_message(connection_name, method_name, amf_data) {
            Ok(msg) => {
                if let Err(e) = self.channel.post_message(&msg) {
                    tracing::error!(
                        "Failed to broadcast LocalConnection message via BroadcastChannel: {:?}",
                        e
                    );
                }
            }
            Err(e) => {
                tracing::error!(
                    "Failed to create LocalConnection broadcast message: {:?}",
                    e
                );
            }
        }
    }

    fn poll_incoming(&mut self) -> Vec<ExternalLocalConnectionMessage> {
        // Refresh heartbeat timestamps periodically.
        self.refresh_heartbeat();

        let mut incoming = self.incoming.borrow_mut();
        std::mem::take(&mut *incoming)
    }

    fn set_player(&mut self, player: std::sync::Weak<std::sync::Mutex<ruffle_core::Player>>) {
        *self.player.borrow_mut() = Some(player);
    }
}

impl Drop for WebLocalConnectionBackend {
    fn drop(&mut self) {
        self.channel.close();

        // Clean up all localStorage entries this instance registered.
        if let Some(storage) = Self::get_local_storage() {
            let names = self.registered_names.borrow();
            for name in names.iter() {
                let key = format!("{LS_KEY_PREFIX}{name}");
                let _ = storage.remove_item(&key);
            }
        }

        // Remove beforeunload listener.
        if let Some(ref closure) = self._on_beforeunload
            && let Some(window) = web_sys::window()
        {
            let _ = window.remove_event_listener_with_callback(
                "beforeunload",
                closure.as_ref().unchecked_ref(),
            );
        }
    }
}
