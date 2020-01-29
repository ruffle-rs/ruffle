//! Navigator backend for web

use crate::custom_event::RuffleEvent;
use glutin::event_loop::EventLoopProxy;
use log;
use ruffle_core::backend::navigator::{
    Error, NavigationMethod, NavigatorBackend, OwnedFuture, RequestOptions,
};
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use url::Url;
use webbrowser;

/// Implementation of `NavigatorBackend` for non-web environments that can call
/// out to a web browser.
pub struct ExternalNavigatorBackend {
    /// Sink for tasks sent to us through `spawn_future`.
    channel: Sender<OwnedFuture<(), Error>>,

    /// Event sink to trigger a new task poll.
    event_loop: EventLoopProxy<RuffleEvent>,
}

impl ExternalNavigatorBackend {
    pub fn new(
        channel: Sender<OwnedFuture<(), Error>>,
        event_loop: EventLoopProxy<RuffleEvent>,
    ) -> Self {
        Self {
            channel,
            event_loop,
        }
    }
}

impl NavigatorBackend for ExternalNavigatorBackend {
    fn navigate_to_url(
        &self,
        url: String,
        _window_spec: Option<String>,
        vars_method: Option<(NavigationMethod, HashMap<String, String>)>,
    ) {
        //TODO: Should we return a result for failed opens? Does Flash care?

        //NOTE: Flash desktop players / projectors ignore the window parameter,
        //      unless it's a `_layer`, and we shouldn't handle that anyway.
        let mut parsed_url = match Url::parse(&url) {
            Ok(parsed_url) => parsed_url,
            Err(e) => {
                log::error!(
                    "Could not parse URL because of {}, the corrupt URL was: {}",
                    e,
                    url
                );
                return;
            }
        };

        let modified_url = match vars_method {
            Some((_, query_pairs)) => {
                {
                    //lifetime limiter because we don't have NLL yet
                    let mut modifier = parsed_url.query_pairs_mut();

                    for (k, v) in query_pairs.iter() {
                        modifier.append_pair(k, v);
                    }
                }

                parsed_url.into_string()
            }
            None => url,
        };

        match webbrowser::open(&modified_url) {
            Ok(_output) => {}
            Err(e) => log::error!("Could not open URL {}: {}", modified_url, e),
        };
    }

    fn fetch(&self, _url: String, _options: RequestOptions) -> OwnedFuture<Vec<u8>, Error> {
        Box::pin(async { Err("Fetch not implemented on desktop!".into()) })
    }

    fn spawn_future(&mut self, future: OwnedFuture<(), Error>) {
        self.channel.send(future).expect("working channel send");

        if self.event_loop.send_event(RuffleEvent::TaskPoll).is_err() {
            log::warn!(
                "A task was queued on an event loop that has already ended. It will not be polled."
            );
        }
    }
}
