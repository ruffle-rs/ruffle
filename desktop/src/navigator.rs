//! Navigator backend for web

use std::collections::HashMap;
use ruffle_core::backend::navigator::{NavigatorBackend, NavigationMethod};
use webbrowser;
use log;
use url::Url;

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
    fn navigate_to_url(&self, url: String, _window_spec: Option<String>, vars_method: Option<(NavigationMethod, HashMap<String, String>)>) {
        //TODO: Should we return a result for failed opens? Does Flash care?
        
        //NOTE: Flash desktop players / projectors ignore the window parameter,
        //      unless it's a `_layer`, and we shouldn't handle that anyway.
        let mut parsed_url = match Url::parse(&url) {
            Ok(parsed_url) => parsed_url,
            Err(e) => {
                log::error!("Could not parse URL because of {}, the corrupt URL was: {}", e, url);
                return;
            }
        };

        let modified_url = match vars_method {
            Some((_, query_pairs)) => {
                { //lifetime limiter because we don't have NLL yet
                    let mut modifier = parsed_url.query_pairs_mut();

                    for (k, v) in query_pairs.iter() {
                        modifier.append_pair(k, v);
                    }
                }

                parsed_url.into_string()
            },
            None => url
        };
        
        match webbrowser::open(&modified_url) {
            Ok(_output) => {},
            Err(e) => log::error!("Could not open URL {}: {}", modified_url, e)
        };
    }
}