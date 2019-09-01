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
    fn navigate_to_url(&self, url: String, _window_spec: Option<String>, _vars_method: Option<(NavigationMethod, HashMap<String, String>)>) {
        if let Some(window) = window() {
            //TODO: Support `window`
            //TODO: Support `vars_method`
            window.location().assign(&url);
        }
    }
}