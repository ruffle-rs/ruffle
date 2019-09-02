//! Navigator backend for web

use std::collections::HashMap;
use web_sys::window;
use ruffle_core::backend::navigator::{NavigatorBackend, NavigationMethod};

pub struct WebNavigatorBackend {

}

impl WebNavigatorBackend {
    pub fn new() -> Self {
        WebNavigatorBackend {

        }
    }
}

impl NavigatorBackend for WebNavigatorBackend {
    fn navigate_to_url(&self, url: String, window_spec: Option<String>, _vars_method: Option<(NavigationMethod, HashMap<String, String>)>) {
        if let Some(window) = window() {
            //TODO: Support `vars_method`
            //TODO: Should we return a result for failed opens? Does Flash care?
            #[allow(unused_must_use)]
            match window_spec {
                Some(ref window_name) if window_name != "" => { window.open_with_url_and_target(&url, window_name); },
                _ => { window.location().assign(&url); }
            };
        }
    }
}