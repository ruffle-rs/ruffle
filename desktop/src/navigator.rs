//! Navigator backend for web

use std::collections::HashMap;
use ruffle_core::backend::navigator::{NavigatorBackend, NavigationMethod};
use webbrowser;
use log;

/// Implementation of `NavigatorBackend` for non-web environments that can call
/// out to a web browser.
pub struct ExternalNavigatorBackend {

}

impl ExternalNavigatorBackend {
    pub fn new() -> Self {
        ExternalNavigatorBackend {

        }
    }
}

impl NavigatorBackend for ExternalNavigatorBackend {
    fn navigate_to_url(&self, url: String, _window_spec: Option<String>, _vars_method: Option<(NavigationMethod, HashMap<String, String>)>) {
        //TODO: How does Flash interpret the target on desktop?
        //TODO: Explicitly reject relative URLs, since `webbrowser` sometimes loads them
        //TODO: Support `vars_method`
        //TODO: Should we return a result for failed opens? Does Flash care?
        
        match webbrowser::open(&url) {
            Ok(_output) => {},
            Err(e) => log::error!("Could not open URL {}: {}", url, e)
        };
    }
}