//! Navigator backend for web
use js_sys::{Array, ArrayBuffer, Uint8Array};
use ruffle_core::backend::navigator::{
    url_from_relative_url, NavigationMethod, NavigatorBackend, OwnedFuture, RequestOptions,
};
use ruffle_core::indexmap::IndexMap;
use ruffle_core::loader::Error;
use std::borrow::Cow;
use std::time::Duration;
use url::Url;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{
    window, Blob, BlobPropertyBag, Document, Performance, Request, RequestInit, Response,
};

pub struct WebNavigatorBackend {
    performance: Performance,
    start_time: f64,
    allow_script_access: bool,
    upgrade_to_https: bool,
    base_url: Option<String>,
}

impl WebNavigatorBackend {
    pub fn new(
        allow_script_access: bool,
        upgrade_to_https: bool,
        base_url: Option<String>,
    ) -> Self {
        let window = web_sys::window().expect("window()");
        let performance = window.performance().expect("window.performance()");

        // Upgarde to HTTPS takes effect if the current page is hosted on HTTPS.
        let upgrade_to_https =
            upgrade_to_https && window.location().protocol().unwrap_or_default() == "https:";

        WebNavigatorBackend {
            start_time: performance.now(),
            performance,
            allow_script_access,
            upgrade_to_https,
            base_url,
        }
    }

    fn base_uri(&self, document: &Document) -> Option<String> {
        if let Some(base_url) = self.base_url.clone() {
            Some(base_url)
        } else if let Ok(Some(base_uri)) = document.base_uri() {
            Some(base_uri)
        } else {
            None
        }
    }
}

impl NavigatorBackend for WebNavigatorBackend {
    fn navigate_to_url(
        &self,
        url: String,
        window_spec: Option<String>,
        vars_method: Option<(NavigationMethod, IndexMap<String, String>)>,
    ) {
        // If the URL is empty, we ignore the request
        if url.is_empty() {
            return;
        }

        if let Some(window) = window() {
            let document = window.document().expect("Could not get document");

            let base_uri = match self.base_uri(&document) {
                Some(base_uri) => base_uri,
                _ => return,
            };

            let url = if let Ok(new_url) = url_from_relative_url(&base_uri, &url) {
                new_url
            } else {
                return;
            };

            // If allowScriptAccess is disabled, we should reject the javascript scheme
            if !self.allow_script_access && url.scheme() == "javascript" {
                log::warn!("SWF tried to run a script, but script access is not allowed");
                return;
            }

            //TODO: Should we return a result for failed opens? Does Flash care?
            match (vars_method, window_spec) {
                (Some((navmethod, formvars)), window_spec) => {
                    let form_url = self.pre_process_url(url).to_string();

                    let body = match document.body() {
                        Some(body) => body,
                        None => return,
                    };

                    let form = document
                        .create_element("form")
                        .unwrap()
                        .dyn_into::<web_sys::HtmlFormElement>()
                        .unwrap();

                    let _ = form.set_attribute(
                        "method",
                        match navmethod {
                            NavigationMethod::Get => "get",
                            NavigationMethod::Post => "post",
                        },
                    );

                    let _ = form.set_attribute("action", &form_url);

                    if let Some(target) = window_spec {
                        let _ = form.set_attribute("target", &target);
                    }

                    for (k, v) in formvars.iter() {
                        let hidden = document.create_element("input").unwrap();

                        let _ = hidden.set_attribute("type", "hidden");
                        let _ = hidden.set_attribute("name", k);
                        let _ = hidden.set_attribute("value", v);

                        let _ = form.append_child(&hidden);
                    }

                    let _ = body.append_child(&form);
                    let _ = form.submit();
                }
                (_, Some(ref window_name)) if !window_name.is_empty() => {
                    let _ = window.open_with_url_and_target(url.as_str(), window_name);
                }
                _ => {
                    let _ = window.location().assign(url.as_str());
                }
            };
        }
    }

    fn time_since_launch(&mut self) -> Duration {
        let dt = self.performance.now() - self.start_time;
        Duration::from_millis(dt as u64)
    }

    fn fetch(&self, url: &str, options: RequestOptions) -> OwnedFuture<Vec<u8>, Error> {
        let url = if let Ok(parsed_url) = Url::parse(url) {
            self.pre_process_url(parsed_url).to_string()
        } else {
            url.to_string()
        };

        Box::pin(async move {
            let mut init = RequestInit::new();

            init.method(match options.method() {
                NavigationMethod::Get => "GET",
                NavigationMethod::Post => "POST",
            });

            if let Some((data, mime)) = options.body() {
                let arraydata = ArrayBuffer::new(data.len() as u32);
                let u8data = Uint8Array::new(&arraydata);

                for (i, byte) in data.iter().enumerate() {
                    u8data.fill(*byte, i as u32, i as u32 + 1);
                }

                let blobparts = Array::new();
                blobparts.push(&arraydata);

                let mut blobprops = BlobPropertyBag::new();
                blobprops.type_(mime);

                let datablob =
                    Blob::new_with_buffer_source_sequence_and_options(&blobparts, &blobprops)
                        .unwrap()
                        .dyn_into()
                        .unwrap();

                init.body(Some(&datablob));
            }

            let request = Request::new_with_str_and_init(&url, &init)
                .map_err(|_| Error::FetchError(format!("Unable to create request for {}", url)))?;

            let window = web_sys::window().unwrap();
            let fetchval = JsFuture::from(window.fetch_with_request(&request)).await;
            if fetchval.is_err() {
                return Err(Error::NetworkError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Could not fetch, got JS Error",
                )));
            }

            let resp: Response = fetchval.unwrap().dyn_into().unwrap();

            if !resp.ok() {
                return Err(Error::FetchError(format!(
                    "HTTP status is not ok, got {}",
                    resp.status_text()
                )));
            }

            let data: ArrayBuffer = JsFuture::from(resp.array_buffer().unwrap())
                .await
                .unwrap()
                .dyn_into()
                .unwrap();
            let jsarray = Uint8Array::new(&data);
            let mut rust_array = vec![0; jsarray.length() as usize];
            jsarray.copy_to(&mut rust_array);

            Ok(rust_array)
        })
    }

    fn spawn_future(&mut self, future: OwnedFuture<(), Error>) {
        spawn_local(async move {
            if let Err(e) = future.await {
                log::error!("Asynchronous error occurred: {}", e);
            }
        })
    }

    fn resolve_relative_url<'a>(&mut self, url: &'a str) -> Cow<'a, str> {
        let window = web_sys::window().expect("window()");
        let document = window.document().expect("document()");

        if let Some(base_uri) = self.base_uri(&document) {
            if let Ok(new_url) = url_from_relative_url(&base_uri, url) {
                return String::from(new_url).into();
            }
        }

        url.into()
    }

    fn pre_process_url(&self, mut url: Url) -> Url {
        if self.upgrade_to_https && url.scheme() == "http" && url.set_scheme("https").is_err() {
            log::error!("Url::set_scheme failed on: {}", url);
        }
        url
    }
}
