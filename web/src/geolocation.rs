use super::JavascriptPlayer;
use ruffle_core::events::PermissionStatus;
use ruffle_core::{backend::geolocation::GeolocationBackend, PlayerEvent};
use serde::{Deserialize, Serialize};
use web_sys::window;

// Default value on Android devices
pub const DEFAULT_UPDATE_INTERVAL: f64 = 10000.0;

pub struct WebGeolocationBackend {
    js_player: JavascriptPlayer,
    is_geolocation_supported: bool,
    geo_status: PermissionStatus,
    geo_update_interval: f64,
}

impl WebGeolocationBackend {
    pub fn new(js_player: JavascriptPlayer) -> Self {
        let mut backend = Self {
            js_player,
            is_geolocation_supported: false,
            geo_status: PermissionStatus::Unknown,
            geo_update_interval: DEFAULT_UPDATE_INTERVAL,
        };
        // Check if Geolocation object exists. If it does then geolocation
        // is supposedly supported.
        if let Some(w) = window() {
            if w.navigator().geolocation().is_ok() {
                backend.is_geolocation_supported = true;
            }
        }
        backend
    }
}

impl GeolocationBackend for WebGeolocationBackend {
    fn is_geolocation_supported(&mut self) -> bool {
        self.is_geolocation_supported
    }

    fn request_geolocation_permission(&self) {
        self.js_player.request_geolocation_permission();
    }

    fn geolocation_permission_status(&self) -> PermissionStatus {
        self.geo_status
    }

    fn set_geolocation_permission_status(&mut self, status: String) {
        self.geo_status = match status.as_str() {
            "granted" => PermissionStatus::Granted,
            "denied" => PermissionStatus::Denied,
            _ => PermissionStatus::Unknown,
        }
    }

    fn set_geolocation_update_interval(&mut self, interval: f64) {
        self.geo_update_interval = interval;
        self.js_player.set_geolocation_update_interval(interval);
    }

    fn geolocation_update_interval(&self) -> f64 {
        self.geo_update_interval
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct GeolocationPositionJS {
    pub coords: GeolocationPositionJSCoordinates,
    pub timestamp: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct GeolocationPositionJSCoordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>, // might be null
    pub accuracy: f64,
    #[serde(rename(deserialize = "altitudeAccuracy"))]
    pub altitude_accuracy: Option<f64>, // might be null
    pub speed: Option<f64>,   // might be null
    pub heading: Option<f64>, // might be null
}

impl From<GeolocationPositionJS> for PlayerEvent {
    fn from(pos: GeolocationPositionJS) -> Self {
        PlayerEvent::GeolocationUpdate {
            latitude: pos.coords.latitude,
            longitude: pos.coords.longitude,
            altitude: match pos.coords.altitude {
                Some(x) => x,
                None => f64::NAN,
            },
            horizontal_accuracy: pos.coords.accuracy,
            vertical_accuracy: match pos.coords.altitude_accuracy {
                Some(x) => x,
                None => f64::NAN,
            },
            speed: match pos.coords.speed {
                Some(x) => x,
                None => f64::NAN,
            },
            heading: match pos.coords.heading {
                Some(x) => x,
                None => f64::NAN,
            },
            timestamp: pos.timestamp,
        }
    }
}
