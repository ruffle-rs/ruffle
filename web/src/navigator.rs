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
use web_sys::{window, Blob, BlobPropertyBag, Performance, Request, RequestInit, Response};

pub struct WebNavigatorBackend {
    performance: Performance,
    start_time: f64,
    upgrade_to_https: bool,
}

impl WebNavigatorBackend {
    pub fn new(upgrade_to_https: bool) -> Self {
        let window = web_sys::window().expect("window()");
        let performance = window.performance().expect("window.performance()");

        WebNavigatorBackend {
            start_time: performance.now(),
            performance,
            upgrade_to_https,
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
        if let Some(window) = window() {
            let url = if let Ok(parsed_url) = Url::parse(&url) {
                self.pre_process_url(parsed_url).to_string()
            } else {
                url.to_string()
            };

            //TODO: Should we return a result for failed opens? Does Flash care?
            #[allow(unused_must_use)]
            match (vars_method, window_spec) {
                (Some((navmethod, formvars)), window_spec) => {
                    let document = match window.document() {
                        Some(document) => document,
                        None => return,
                    };

                    let form = document
                        .create_element("form")
                        .unwrap()
                        .dyn_into::<web_sys::HtmlFormElement>()
                        .unwrap();

                    form.set_attribute(
                        "method",
                        match navmethod {
                            NavigationMethod::GET => "get",
                            NavigationMethod::POST => "post",
                        },
                    );

                    form.set_attribute("action", &url);

                    if let Some(target) = window_spec {
                        form.set_attribute("target", &target);
                    }

                    for (k, v) in formvars.iter() {
                        let hidden = document.create_element("hidden").unwrap();

                        hidden.set_attribute("type", "hidden");
                        hidden.set_attribute("name", k);
                        hidden.set_attribute("value", v);

                        form.append_child(&hidden);
                    }

                    document.body().unwrap().append_child(&form);
                    form.submit();
                }
                (_, Some(ref window_name)) if !window_name.is_empty() => {
                    window.open_with_url_and_target(&url, window_name);
                }
                _ => {
                    window.location().assign(&url);
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
                NavigationMethod::GET => "GET",
                NavigationMethod::POST => "POST",
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

            let request = Request::new_with_str_and_init(&url, &init).unwrap();

            let window = web_sys::window().unwrap();
            let fetchval = JsFuture::from(window.fetch_with_request(&request)).await;
            if fetchval.is_err() {
                return Err(Error::NetworkError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Could not fetch, got JS Error",
                )));
            }

            let resp: Response = fetchval.unwrap().dyn_into().unwrap();
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

        if let Ok(Some(base_uri)) = document.base_uri() {
            if let Ok(new_url) = url_from_relative_url(&base_uri, url) {
                return new_url.into_string().into();
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
