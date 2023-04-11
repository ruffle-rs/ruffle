//! Navigator backend for web
use js_sys::{Array, ArrayBuffer, Uint8Array};
use ruffle_core::backend::navigator::{
    NavigateWebsiteHandlingMode, NavigationMethod, NavigatorBackend, OwnedFuture, Request, Response,
};
use ruffle_core::config::NetworkingRestrictionMode;
use ruffle_core::indexmap::IndexMap;
use ruffle_core::loader::Error;
use std::borrow::Cow;
use std::sync::Arc;
use tracing_subscriber::layer::Layered;
use tracing_subscriber::Registry;
use tracing_wasm::WASMLayer;
use url::Url;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{
    window, Blob, BlobPropertyBag, HtmlFormElement, HtmlInputElement, Request as WebRequest,
    RequestInit, Response as WebResponse,
};

pub struct WebNavigatorBackend {
    log_subscriber: Arc<Layered<WASMLayer, Registry>>,
    allow_script_access: bool,
    allow_networking: NetworkingRestrictionMode,
    upgrade_to_https: bool,
    base_url: Option<Url>,
    navigate_website_handling_mode: NavigateWebsiteHandlingMode,
}

impl WebNavigatorBackend {
    pub fn new(
        allow_script_access: bool,
        allow_networking: NetworkingRestrictionMode,
        upgrade_to_https: bool,
        base_url: Option<String>,
        log_subscriber: Arc<Layered<WASMLayer, Registry>>,
        navigate_website_handling_mode: NavigateWebsiteHandlingMode,
    ) -> Self {
        let window = web_sys::window().expect("window()");

        // Upgrade to HTTPS takes effect if the current page is hosted on HTTPS.
        let upgrade_to_https =
            upgrade_to_https && window.location().protocol().expect("protocol()") == "https:";

        // Retrieve and parse `document.baseURI`.
        let document_base_uri = || {
            let document = window.document().expect("document()");
            if let Ok(Some(base_uri)) = document.base_uri() {
                return Url::parse(&base_uri).ok();
            }

            None
        };

        let base_url = if let Some(mut base_url) = base_url {
            // Adding trailing slash so `Url::parse` will not drop the last part.
            if !base_url.ends_with('/') {
                base_url.push('/');
            }

            Url::parse(&base_url)
                .ok()
                .or_else(|| document_base_uri().and_then(|base_uri| base_uri.join(&base_url).ok()))
        } else {
            document_base_uri()
        };

        if base_url.is_none() {
            tracing::error!("Could not get base URL for base directory inference.");
        }

        Self {
            allow_script_access,
            allow_networking,
            upgrade_to_https,
            base_url,
            log_subscriber,
            navigate_website_handling_mode,
        }
    }

    fn resolve_url<'a>(&self, url: &'a str) -> Cow<'a, str> {
        if let Some(base_url) = &self.base_url {
            if let Ok(url) = base_url.join(url) {
                return self.pre_process_url(url).to_string().into();
            }
        }

        url.into()
    }
}

impl NavigatorBackend for WebNavigatorBackend {
    fn navigate_to_url(
        &self,
        url: &str,
        target: &str,
        vars_method: Option<(NavigationMethod, IndexMap<String, String>)>,
    ) {
        // If the URL is empty, ignore the request.
        if url.is_empty() {
            return;
        }

        let url = self.resolve_url(url);

        // If allowNetworking is set to internal or none, block all navigate_to_url calls.
        if self.allow_networking != NetworkingRestrictionMode::All {
            tracing::warn!("SWF tried to open a URL, but opening URLs is not allowed");
            return;
        }

        // If `allowScriptAccess` is disabled, reject the `javascript:` scheme.
        let js_call = if let Ok(url) = Url::parse(&url) {
            if !self.allow_script_access && url.scheme() == "javascript" {
                tracing::warn!("SWF tried to run a script, but script access is not allowed");
                return;
            }
            url.scheme() == "javascript"
        } else {
            false
        };

        let window = window().expect("window()");

        if !js_call {
            if self.navigate_website_handling_mode == NavigateWebsiteHandlingMode::Confirm {
                let message = "The SWF file wants to open the website ".to_owned() + &url;
                let confirm = window
                    .confirm_with_message(&message)
                    .expect("confirm_with_message()");
                if !confirm {
                    tracing::info!(
                        "SWF tried to open a website, but the user declined the request"
                    );
                    return;
                }
            } else if self.navigate_website_handling_mode == NavigateWebsiteHandlingMode::Deny {
                tracing::warn!("SWF tried to open a website, but opening a website is not allowed");
                return;
            }
            // If the user confirmed or if in Allow mode, open the website
        }

        // TODO: Should we return a result for failed opens? Does Flash care?
        match vars_method {
            Some((navmethod, formvars)) => {
                let document = window.document().expect("document()");
                let body = match document.body() {
                    Some(body) => body,
                    None => return,
                };

                let form: HtmlFormElement = document
                    .create_element("form")
                    .expect("create_element() must succeed")
                    .dyn_into()
                    .expect("create_element(\"form\") didn't give us a form");

                form.set_method(&navmethod.to_string());
                form.set_action(&url);

                if !target.is_empty() {
                    form.set_target(target);
                }

                for (key, value) in formvars {
                    let hidden: HtmlInputElement = document
                        .create_element("input")
                        .expect("create_element() must succeed")
                        .dyn_into()
                        .expect("create_element(\"input\") didn't give us an input");

                    hidden.set_type("hidden");
                    hidden.set_name(&key);
                    hidden.set_value(&value);

                    let _ = form.append_child(&hidden);
                }

                let _ = body.append_child(&form);
                let _ = form.submit();
            }
            None => {
                if target.is_empty() {
                    let _ = window.location().assign(&url);
                } else {
                    let _ = window.open_with_url_and_target(&url, target);
                }
            }
        };
    }

    fn fetch(&self, request: Request) -> OwnedFuture<Response, Error> {
        let url = self.resolve_url(request.url()).into_owned();

        Box::pin(async move {
            let mut init = RequestInit::new();

            init.method(&request.method().to_string());

            if let Some((data, mime)) = request.body() {
                let blob = Blob::new_with_buffer_source_sequence_and_options(
                    &Array::from_iter([Uint8Array::from(data.as_slice()).buffer()]),
                    BlobPropertyBag::new().type_(mime),
                )
                .map_err(|_| Error::FetchError("Got JS error".to_string()))?
                .dyn_into()
                .map_err(|_| Error::FetchError("Got JS error".to_string()))?;

                init.body(Some(&blob));
            }

            let request = WebRequest::new_with_str_and_init(&url, &init)
                .map_err(|_| Error::FetchError(format!("Unable to create request for {url}")))?;

            let window = web_sys::window().expect("window()");
            let fetchval = JsFuture::from(window.fetch_with_request(&request))
                .await
                .map_err(|_| Error::FetchError("Got JS error".to_string()))?;

            let response: WebResponse = fetchval
                .dyn_into()
                .map_err(|_| Error::FetchError("Fetch result wasn't a WebResponse".to_string()))?;
            if !response.ok() {
                return Err(Error::FetchError(format!(
                    "HTTP status is not ok, got {}",
                    response.status_text()
                )));
            }

            let url = response.url();

            let body: ArrayBuffer = JsFuture::from(
                response
                    .array_buffer()
                    .map_err(|_| Error::FetchError("Got JS error".to_string()))?,
            )
            .await
            .map_err(|_| {
                Error::FetchError("Could not allocate array buffer for response".to_string())
            })?
            .dyn_into()
            .map_err(|_| {
                Error::FetchError("array_buffer result wasn't an ArrayBuffer".to_string())
            })?;
            let body = Uint8Array::new(&body).to_vec();

            Ok(Response { url, body })
        })
    }

    fn spawn_future(&mut self, future: OwnedFuture<(), Error>) {
        let subscriber = self.log_subscriber.clone();
        spawn_local(async move {
            let _subscriber = tracing::subscriber::set_default(subscriber);
            if let Err(e) = future.await {
                tracing::error!("Asynchronous error occurred: {}", e);
            }
        })
    }

    fn pre_process_url(&self, mut url: Url) -> Url {
        if self.upgrade_to_https && url.scheme() == "http" && url.set_scheme("https").is_err() {
            tracing::error!("Url::set_scheme failed on: {}", url);
        }
        url
    }
}
