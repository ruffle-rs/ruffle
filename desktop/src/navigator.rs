//! Navigator backend for web

use crate::custom_event::RuffleEvent;
use ruffle_core::backend::navigator::{
    NavigationMethod, NavigatorBackend, OwnedFuture, RequestOptions,
};
use ruffle_core::loader::Error;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use url::{ParseError, Url};
use winit::event_loop::EventLoopProxy;

/// Implementation of `NavigatorBackend` for non-web environments that can call
/// out to a web browser.
pub struct ExternalNavigatorBackend {
    /// Sink for tasks sent to us through `spawn_future`.
    channel: Sender<OwnedFuture<(), Error>>,

    /// Event sink to trigger a new task poll.
    event_loop: EventLoopProxy<RuffleEvent>,

    /// The base path for all relative fetches.
    relative_base_path: PathBuf,

    /// The time that the SWF was launched.
    start_time: Instant,
}

impl ExternalNavigatorBackend {
    #[allow(dead_code)]
    pub fn new(
        channel: Sender<OwnedFuture<(), Error>>,
        event_loop: EventLoopProxy<RuffleEvent>,
    ) -> Self {
        Self {
            channel,
            event_loop,
            relative_base_path: PathBuf::new(),
            start_time: Instant::now(),
        }
    }

    /// Construct a navigator backend with fetch and async capability.
    pub fn with_base_path<P: AsRef<Path>>(
        path: P,
        channel: Sender<OwnedFuture<(), Error>>,
        event_loop: EventLoopProxy<RuffleEvent>,
    ) -> Self {
        let mut relative_base_path = PathBuf::new();

        relative_base_path.push(path);

        Self {
            channel,
            event_loop,
            relative_base_path,
            start_time: Instant::now(),
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

    fn time_since_launch(&mut self) -> Duration {
        Instant::now().duration_since(self.start_time)
    }

    fn fetch(&self, url: &str, _options: RequestOptions) -> OwnedFuture<Vec<u8>, Error> {
        // Load from local filesystem.
        // TODO: Support network loads, honor sandbox type (local-with-filesystem, local-with-network, remote, ...)
        let mut path = self.relative_base_path.clone();
        path.push(url);

        Box::pin(async move { fs::read(path).map_err(Error::NetworkError) })
    }

    fn spawn_future(&mut self, future: OwnedFuture<(), Error>) {
        self.channel.send(future).expect("working channel send");

        if self.event_loop.send_event(RuffleEvent::TaskPoll).is_err() {
            log::warn!(
                "A task was queued on an event loop that has already ended. It will not be polled."
            );
        }
    }

    fn resolve_relative_url<'a>(&mut self, url: &'a str) -> Cow<'a, str> {
        let parsed = Url::parse(url);
        if let Err(ParseError::RelativeUrlWithoutBase) = parsed {
            if let Ok(cwd) = std::env::current_dir() {
                let base = Url::from_directory_path(cwd);
                if let Ok(base) = base {
                    let abs = base.join(url);

                    if let Ok(abs) = abs {
                        return abs.into_string().into();
                    }
                }
            }
        }

        url.into()
    }
}
