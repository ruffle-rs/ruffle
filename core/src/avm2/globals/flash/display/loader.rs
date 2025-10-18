//! `flash.display.Loader` builtin/prototype

use indexmap::IndexMap;

use crate::avm2::activation::Activation;
use crate::avm2::error::make_error_2007;
use crate::avm2::globals::flash::display::display_object::initialize_for_allocator;
use crate::avm2::globals::slots::flash_display_loader as loader_slots;
use crate::avm2::globals::slots::flash_net_url_request as url_request_slots;
use crate::avm2::globals::slots::flash_net_url_request_header as url_request_header_slots;
use crate::avm2::object::LoaderInfoObject;
use crate::avm2::object::LoaderStream;
use crate::avm2::object::TObject as _;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::ClassObject;
use crate::avm2::{Error, Object};
use crate::avm2_stub_method;
use crate::backend::navigator::{NavigationMethod, Request};
use crate::display_object::LoaderDisplay;
use crate::display_object::MovieClip;
use crate::loader::LoadManager;
use crate::loader::MovieLoaderVMData;
use crate::tag_utils::SwfMovie;
use std::sync::Arc;

pub fn loader_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    // Loader does not have an associated `Character` variant, and can never be
    // instantiated from the timeline.
    let display_object =
        LoaderDisplay::empty(activation, activation.context.root_swf.clone()).into();
    let loader = initialize_for_allocator(activation.context, display_object, class);

    // Note that the initialization of `_contentLoaderInfo` is intentionally done here,
    // and not in the Loader constructor - subclasess of Loader can observe 'contentLoaderInfo'
    // being set before super() is called.

    // Some LoaderInfo properties (such as 'bytesLoaded' and 'bytesTotal') are always
    // accessible, even before the 'init' event has fired. Using an empty movie gives
    // us the correct value (0) for them.
    let movie = &activation.context.root_swf;
    let loader_info = LoaderInfoObject::not_yet_loaded(
        activation,
        Arc::new(SwfMovie::empty(movie.version(), Some(movie.url().into()))),
        Some(loader),
        None,
        false,
    )?;
    loader.set_slot(
        loader_slots::_CONTENT_LOADER_INFO,
        loader_info.into(),
        activation,
    )?;
    Ok(loader)
}

pub fn load<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let url_request = args.get_object(activation, 0, "request")?;
    let context = args.try_get_object(1);

    let loader_info = this
        .get_slot(loader_slots::_CONTENT_LOADER_INFO)
        .as_object()
        .unwrap();

    let loader_info = loader_info.as_loader_info_object().unwrap();

    if loader_info.init_event_fired() {
        // FIXME: When calling load/loadBytes, then calling load/loadBytes again
        // before the `init` event is fired, the first load is cancelled.
        avm2_stub_method!(
            activation,
            "flash.display.Loader",
            "load",
            "reusing a Loader"
        );
    }

    // Unload the loader, in case something was already loaded.
    loader_info.unload(activation);

    // This is a dummy MovieClip, which will get overwritten in `Loader`
    let movie = &activation.context.root_swf;
    let content = MovieClip::new(
        Arc::new(SwfMovie::empty(movie.version(), Some(movie.url().into()))),
        activation.gc(),
    );

    // Update the LoaderStream - we still have a fake SwfMovie, but we now have the real target clip.
    loader_info.set_loader_stream(
        LoaderStream::NotYetLoaded(
            Arc::new(SwfMovie::empty(movie.version(), Some(movie.url().into()))),
            Some(content.into()),
            false,
        ),
        activation.gc(),
    );

    let request = request_from_url_request(activation, url_request)?;

    let url = request.url().to_string();
    let future = activation.context.load_manager.load_movie_into_clip(
        activation.context.player_handle(),
        content.into(),
        request,
        Some(url),
        MovieLoaderVMData::Avm2 {
            loader_info,
            context,
            default_domain: activation
                .caller_domain()
                .expect("Missing caller domain in Loader.load"),
        },
    );
    activation.context.navigator.spawn_future(future);

    Ok(Value::Undefined)
}

pub fn request_from_url_request<'gc>(
    activation: &mut Activation<'_, 'gc>,
    url_request: Object<'gc>,
) -> Result<Request, Error<'gc>> {
    // FIXME: set `followRedirects`  and `userAgent`
    // from the `URLRequest`

    let mut url = match url_request.get_slot(url_request_slots::_URL) {
        Value::Null => return Err(make_error_2007(activation, "url")),
        url => url.coerce_to_string(activation)?.to_string(),
    };

    let method = url_request
        .get_slot(url_request_slots::_METHOD)
        .coerce_to_string(activation)?;

    let headers = url_request
        .get_slot(url_request_slots::_REQUEST_HEADERS)
        .as_object();

    let mut string_headers = IndexMap::default();
    if let Some(headers) = headers {
        let headers = headers.as_array_storage().unwrap();

        for i in 0..headers.length() {
            let Some(header) = headers.get(i).and_then(|h| h.as_object()) else {
                continue;
            };

            // Non-URLRequestHeader objects are skipped
            if header.is_of_type(activation.avm2().class_defs().urlrequestheader) {
                let name = header
                    .get_slot(url_request_header_slots::NAME)
                    .coerce_to_string(activation)?
                    .to_string();
                let value = header
                    .get_slot(url_request_header_slots::VALUE)
                    .coerce_to_string(activation)?
                    .to_string();

                // Note - testing with Flash Player shows that later entries in the array
                // overwrite earlier ones with the same name. Flash Player never sends an HTTP
                // request with duplicate headers
                string_headers.insert(name, value);
            }
        }
    }

    let mut method =
        NavigationMethod::from_method_str(&method).expect("URLRequest should have a valid method");
    let data = url_request.get_slot(url_request_slots::_DATA);
    let body = match (method, data) {
        (_, Value::Null | Value::Undefined) => None,
        (NavigationMethod::Get, data) => {
            // This looks "wrong" but it's Flash-correct.
            // It simply appends the data to the URL if there's already a query,
            // otherwise it adds ?data.
            // This does mean that if there's a #fragment in the URL after the query,
            // the new data gets appended to *that* - which is totally wrong but whatcha gonna do?
            if !url.contains('?') {
                url.push('?');
            }
            url.push_str(&data.coerce_to_string(activation)?.to_string());
            None
        }
        (NavigationMethod::Post, data) => {
            let content_type = url_request
                .get_slot(url_request_slots::_CONTENT_TYPE)
                .coerce_to_string(activation)?
                .to_string();

            let payload = if let Some(ba) = data.as_object().and_then(|o| o.as_bytearray_object()) {
                // Note that this does *not* respect or modify the position.
                ba.storage().bytes().to_vec()
            } else {
                data.coerce_to_string(activation)?
                    .to_utf8_lossy()
                    .as_bytes()
                    .to_vec()
            };

            if payload.is_empty() {
                None
            } else {
                Some((payload, content_type))
            }
        }
    };

    // Flash behaviour:
    // When payload is null or empty, flash will ignore the method and do a GET request instead.
    if body.is_none() {
        method = NavigationMethod::Get;
    }

    let mut request = Request::request(method, url.to_string(), body);
    request.set_headers(string_headers);

    Ok(request)
}

pub fn load_bytes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let arg0 = args.get_object(activation, 0, "data")?;
    let bytes = arg0.as_bytearray().unwrap().bytes().to_vec();
    let context = args.try_get_object(1);

    let loader_info = this
        .get_slot(loader_slots::_CONTENT_LOADER_INFO)
        .as_object()
        .unwrap();

    let loader_info = loader_info.as_loader_info_object().unwrap();

    if loader_info.init_event_fired() {
        // FIXME: When calling load/loadBytes, then calling load/loadBytes again
        // before the `init` event is fired, the first load is cancelled.
        avm2_stub_method!(
            activation,
            "flash.display.Loader",
            "loadBytes",
            "reusing a Loader"
        );
    }

    // Unload the loader, in case something was already loaded.
    loader_info.unload(activation);

    // This is a dummy MovieClip, which will get overwritten in `Loader`
    let movie = &activation.context.root_swf;
    let content = MovieClip::new(
        Arc::new(SwfMovie::empty(movie.version(), Some(movie.url().into()))),
        activation.gc(),
    );

    let default_domain = activation
        .caller_domain()
        .expect("Missing caller domain in Loader.loadBytes");

    if let Err(e) = LoadManager::load_movie_into_clip_bytes(
        activation.context,
        content.into(),
        bytes,
        MovieLoaderVMData::Avm2 {
            loader_info,
            context,
            default_domain,
        },
    ) {
        return Err(Error::rust_error(
            format!("Error in Loader.loadBytes: {e:?}").into(),
        ));
    }

    Ok(Value::Undefined)
}

pub fn unload<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    // TODO: Broadcast an "unload" event on the LoaderInfo
    avm2_stub_method!(activation, "flash.display.Loader", "unload");

    let loader_info = this
        .get_slot(loader_slots::_CONTENT_LOADER_INFO)
        .as_object()
        .unwrap();

    let loader_info = loader_info.as_loader_info_object().unwrap();

    loader_info.unload(activation);

    Ok(Value::Undefined)
}
