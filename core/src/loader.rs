//! Management of async loaders

use crate::avm1::activation::{Activation, ActivationIdentifier};
use crate::avm1::function::ExecutionReason;
use crate::avm1::{Avm1, Object, TObject, Value};
use crate::avm2::bytearray::ByteArrayStorage;
use crate::avm2::names::Namespace;
use crate::avm2::object::ByteArrayObject;
use crate::avm2::object::TObject as _;
use crate::avm2::{
    Activation as Avm2Activation, Avm2, Domain as Avm2Domain, Event as Avm2Event,
    EventData as Avm2EventData, Object as Avm2Object, QName, Value as Avm2Value,
};
use crate::backend::navigator::{OwnedFuture, RequestOptions};
use crate::backend::render::{determine_jpeg_tag_format, JpegTagFormat};
use crate::context::{ActionQueue, ActionType, UpdateContext};
use crate::display_object::{Bitmap, DisplayObject, TDisplayObject, TDisplayObjectContainer};
use crate::player::Player;
use crate::string::AvmString;
use crate::tag_utils::SwfMovie;
use crate::vminterface::Instantiator;
use encoding_rs::UTF_8;
use gc_arena::{Collect, CollectionContext};
use generational_arena::{Arena, Index};
use std::fmt;
use std::sync::{Arc, Mutex, Weak};
use swf::read::read_compression_type;
use thiserror::Error;
use url::form_urlencoded;

pub type Handle = Index;

/// Enumeration of all content types that `Loader` can handle.
///
/// This is a superset of `JpegTagFormat`.
#[derive(PartialEq, Debug)]
pub enum ContentType {
    Swf,
    Jpeg,
    Png,
    Gif,
    Unknown,
}

impl From<JpegTagFormat> for ContentType {
    fn from(jtf: JpegTagFormat) -> Self {
        match jtf {
            JpegTagFormat::Jpeg => Self::Jpeg,
            JpegTagFormat::Png => Self::Png,
            JpegTagFormat::Gif => Self::Gif,
            JpegTagFormat::Unknown => Self::Unknown,
        }
    }
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Swf => write!(f, "SWF"),
            Self::Jpeg => write!(f, "JPEG"),
            Self::Png => write!(f, "PNG"),
            Self::Gif => write!(f, "GIF"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

impl ContentType {
    fn sniff(data: &[u8]) -> ContentType {
        if read_compression_type(data).is_ok() {
            ContentType::Swf
        } else {
            determine_jpeg_tag_format(data).into()
        }
    }

    /// Assert that content is of a given type, and error otherwise.
    fn expect(self, expected: Self) -> Result<Self, Error> {
        if self == expected {
            Ok(self)
        } else {
            Err(Error::UnexpectedData(expected, self))
        }
    }
}

#[derive(Collect, Copy, Clone)]
#[collect(no_drop)]
pub enum DataFormat {
    Binary,
    Text,
    Variables,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Load cancelled")]
    Cancelled,

    #[error("Non-root-movie loader spawned as root movie loader")]
    NotRootMovieLoader,

    #[error("Non-movie loader spawned as movie loader")]
    NotMovieLoader,

    #[error("Non-form loader spawned as form loader")]
    NotFormLoader,

    #[error("Non-load vars loader spawned as load vars loader")]
    NotLoadVarsLoader,

    #[error("Non-data loader spawned as data loader")]
    NotLoadDataLoader,

    #[error("Could not fetch: {0}")]
    FetchError(String),

    #[error("Invalid SWF")]
    InvalidSwf(#[from] crate::tag_utils::Error),

    #[error("Unexpected content of type {1}, expected {0}")]
    UnexpectedData(ContentType, ContentType),

    // TODO: We can't support lifetimes on this error object yet (or we'll need some backends inside
    // the GC arena). We're losing info here. How do we fix that?
    #[error("Error running avm1 script: {0}")]
    Avm1Error(String),
}

impl From<crate::avm1::error::Error<'_>> for Error {
    fn from(error: crate::avm1::error::Error<'_>) -> Self {
        Error::Avm1Error(error.to_string())
    }
}

/// Holds all in-progress loads for the player.
pub struct LoadManager<'gc>(Arena<Loader<'gc>>);

unsafe impl<'gc> Collect for LoadManager<'gc> {
    fn trace(&self, cc: CollectionContext) {
        for (_, loader) in self.0.iter() {
            loader.trace(cc)
        }
    }
}

impl<'gc> LoadManager<'gc> {
    /// Construct a new `LoadManager`.
    pub fn new() -> Self {
        Self(Arena::new())
    }

    /// Add a new loader to the `LoadManager`.
    ///
    /// Returns the loader handle for later inspection. A loader handle is
    /// valid for as long as the load operation. Once the load finishes,
    /// the handle will be invalidated (and the underlying loader deleted).
    pub fn add_loader(&mut self, loader: Loader<'gc>) -> Handle {
        let handle = self.0.insert(loader);
        match self.get_loader_mut(handle).unwrap() {
            Loader::RootMovie { self_handle, .. }
            | Loader::Movie { self_handle, .. }
            | Loader::Form { self_handle, .. }
            | Loader::LoadVars { self_handle, .. }
            | Loader::LoadURLLoader { self_handle, .. } => *self_handle = Some(handle),
        }
        handle
    }

    /// Retrieve a loader by handle.
    pub fn get_loader(&self, handle: Handle) -> Option<&Loader<'gc>> {
        self.0.get(handle)
    }

    /// Retrieve a loader by handle for mutation.
    pub fn get_loader_mut(&mut self, handle: Handle) -> Option<&mut Loader<'gc>> {
        self.0.get_mut(handle)
    }

    /// Kick off the root movie load.
    ///
    /// The root movie is special because it determines a few bits of player
    /// state, such as the size of the stage and the current frame rate. Ergo,
    /// this method should only be called once, by the player that is trying to
    /// kick off its root movie load.
    pub fn load_root_movie(
        &mut self,
        player: Weak<Mutex<Player>>,
        url: &str,
        options: RequestOptions,
        parameters: Vec<(String, String)>,
        on_metadata: Box<dyn FnOnce(&swf::HeaderExt)>,
    ) -> OwnedFuture<(), Error> {
        let loader = Loader::RootMovie { self_handle: None };
        let handle = self.add_loader(loader);
        let loader = self.get_loader_mut(handle).unwrap();
        loader.root_movie_loader(player, url.to_owned(), options, parameters, on_metadata)
    }

    /// Kick off a movie clip load.
    ///
    /// Returns the loader's async process, which you will need to spawn.
    pub fn load_movie_into_clip(
        &mut self,
        player: Weak<Mutex<Player>>,
        target_clip: DisplayObject<'gc>,
        url: &str,
        options: RequestOptions,
        loader_url: Option<String>,
        target_broadcaster: Option<Object<'gc>>,
    ) -> OwnedFuture<(), Error> {
        let loader = Loader::Movie {
            self_handle: None,
            target_clip,
            target_broadcaster,
            loader_status: LoaderStatus::Pending,
        };
        let handle = self.add_loader(loader);
        let loader = self.get_loader_mut(handle).unwrap();
        loader.movie_loader(player, url.to_owned(), options, loader_url)
    }

    /// Indicates that a movie clip has initialized (ran its first frame).
    ///
    /// Interested loaders will be invoked from here.
    pub fn movie_clip_on_load(&mut self, queue: &mut ActionQueue<'gc>) {
        let mut invalidated_loaders = vec![];

        for (index, loader) in self.0.iter_mut().rev() {
            if loader.movie_clip_loaded(queue) {
                invalidated_loaders.push(index);
            }
        }

        for index in invalidated_loaders {
            self.0.remove(index);
        }
    }

    /// Kick off a form data load into an AVM1 object.
    ///
    /// Returns the loader's async process, which you will need to spawn.
    pub fn load_form_into_object(
        &mut self,
        player: Weak<Mutex<Player>>,
        target_object: Object<'gc>,
        url: &str,
        options: RequestOptions,
    ) -> OwnedFuture<(), Error> {
        let loader = Loader::Form {
            self_handle: None,
            target_object,
        };
        let handle = self.add_loader(loader);
        let loader = self.get_loader_mut(handle).unwrap();
        loader.form_loader(player, url.to_owned(), options)
    }

    /// Kick off a form data load into an AVM1 object.
    ///
    /// Returns the loader's async process, which you will need to spawn.
    pub fn load_form_into_load_vars(
        &mut self,
        player: Weak<Mutex<Player>>,
        target_object: Object<'gc>,
        url: &str,
        options: RequestOptions,
    ) -> OwnedFuture<(), Error> {
        let loader = Loader::LoadVars {
            self_handle: None,
            target_object,
        };
        let handle = self.add_loader(loader);
        let loader = self.get_loader_mut(handle).unwrap();
        loader.load_vars_loader(player, url.to_owned(), options)
    }

    /// Kick off a data load into a `URLLoader`, updating
    /// its `data` property when the load completes.
    ///
    /// Returns the loader's async process, which you will need to spawn.
    pub fn load_data_into_url_loader(
        &mut self,
        player: Weak<Mutex<Player>>,
        target_object: Avm2Object<'gc>,
        url: &str,
        options: RequestOptions,
        data_format: DataFormat,
    ) -> OwnedFuture<(), Error> {
        let loader = Loader::LoadURLLoader {
            self_handle: None,
            target_object,
        };
        let handle = self.add_loader(loader);
        let loader = self.get_loader_mut(handle).unwrap();
        loader.load_url_loader(player, url.to_owned(), options, data_format)
    }
}

impl<'gc> Default for LoadManager<'gc> {
    fn default() -> Self {
        Self::new()
    }
}

/// The completion status of a `Loader` loading a movie.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Collect)]
#[collect(require_static)]
pub enum LoaderStatus {
    /// The movie hasn't been loaded yet.
    Pending,
    /// The movie loaded successfully.
    Succeeded,
    /// An error occurred while loading the movie.
    Failed,
}

/// A struct that holds garbage-collected pointers for asynchronous code.
#[derive(Collect)]
#[collect(no_drop)]
pub enum Loader<'gc> {
    /// Loader that is loading the root movie of a player.
    RootMovie {
        /// The handle to refer to this loader instance.
        #[collect(require_static)]
        self_handle: Option<Handle>,
    },

    /// Loader that is loading a new movie into a MovieClip.
    Movie {
        /// The handle to refer to this loader instance.
        #[collect(require_static)]
        self_handle: Option<Handle>,

        /// The target movie clip to load the movie into.
        target_clip: DisplayObject<'gc>,

        /// Event broadcaster (typically a `MovieClipLoader`) to fire events
        /// into.
        target_broadcaster: Option<Object<'gc>>,

        /// Indicates the completion status of this loader.
        ///
        /// This flag exists to prevent a situation in which loading a movie
        /// into a clip that has not yet fired its Load event causes the
        /// loader to be prematurely removed. This flag is only set when either
        /// the movie has been replaced (and thus Load events can be trusted)
        /// or an error has occurred (in which case we don't care about the
        /// loader anymore).
        loader_status: LoaderStatus,
    },

    /// Loader that is loading form data into an AVM1 object scope.
    Form {
        /// The handle to refer to this loader instance.
        #[collect(require_static)]
        self_handle: Option<Handle>,

        /// The target AVM1 object to load form data into.
        target_object: Object<'gc>,
    },

    /// Loader that is loading form data into an AVM1 LoadVars object.
    LoadVars {
        /// The handle to refer to this loader instance.
        #[collect(require_static)]
        self_handle: Option<Handle>,

        /// The target AVM1 object to load form data into.
        target_object: Object<'gc>,
    },

    /// Loader that is loading data into a `URLLoader`'s `data` property
    /// The `data` property is only updated after the data is loaded completely
    LoadURLLoader {
        /// The handle to refer to this loader instance.
        #[collect(require_static)]
        self_handle: Option<Handle>,

        /// The target `URLLoader` to load data into.
        target_object: Avm2Object<'gc>,
    },
}

impl<'gc> Loader<'gc> {
    /// Construct a future for the root movie loader.
    fn root_movie_loader(
        &mut self,
        player: Weak<Mutex<Player>>,
        url: String,
        options: RequestOptions,
        parameters: Vec<(String, String)>,
        on_metadata: Box<dyn FnOnce(&swf::HeaderExt)>,
    ) -> OwnedFuture<(), Error> {
        let _handle = match self {
            Loader::RootMovie { self_handle, .. } => {
                self_handle.expect("Loader not self-introduced")
            }
            _ => return Box::pin(async { Err(Error::NotMovieLoader) }),
        };

        let player = player
            .upgrade()
            .expect("Could not upgrade weak reference to player");

        Box::pin(async move {
            let fetch = player.lock().unwrap().navigator().fetch(&url, options);

            let response = fetch.await.map_err(|error| {
                player
                    .lock()
                    .unwrap()
                    .ui()
                    .display_root_movie_download_failed_message();
                error
            })?;

            let mut movie = SwfMovie::from_data(&response.body, Some(response.url), None)?;
            on_metadata(movie.header());
            movie.append_parameters(parameters);
            player.lock().unwrap().set_root_movie(movie);
            Ok(())
        })
    }

    /// Construct a future for the given movie loader.
    ///
    /// The given future should be passed immediately to an executor; it will
    /// take responsibility for running the loader to completion.
    ///
    /// If the loader is not a movie then the returned future will yield an
    /// error immediately once spawned.
    fn movie_loader(
        &mut self,
        player: Weak<Mutex<Player>>,
        url: String,
        options: RequestOptions,
        loader_url: Option<String>,
    ) -> OwnedFuture<(), Error> {
        let handle = match self {
            Loader::Movie { self_handle, .. } => self_handle.expect("Loader not self-introduced"),
            _ => return Box::pin(async { Err(Error::NotMovieLoader) }),
        };

        let player = player
            .upgrade()
            .expect("Could not upgrade weak reference to player");

        Box::pin(async move {
            let fetch = player.lock().unwrap().navigator().fetch(&url, options);

            let mut replacing_root_movie = false;
            player.lock().unwrap().update(|uc| -> Result<(), Error> {
                let clip = match uc.load_manager.get_loader(handle) {
                    Some(Loader::Movie { target_clip, .. }) => *target_clip,
                    None => return Err(Error::Cancelled),
                    _ => unreachable!(),
                };

                replacing_root_movie = DisplayObject::ptr_eq(clip, uc.stage.root_clip());

                if let Some(mut mc) = clip.as_movie_clip() {
                    mc.unload(uc);
                    mc.replace_with_movie(uc.gc_context, None);
                }

                Loader::movie_loader_start(handle, uc)
            })?;

            if let Ok(response) = fetch.await {
                let sniffed_type = ContentType::sniff(&response.body);
                let mut length = response.body.len();

                if replacing_root_movie {
                    sniffed_type.expect(ContentType::Swf)?;

                    let movie =
                        SwfMovie::from_data(&response.body, Some(response.url), loader_url)?;
                    player.lock().unwrap().set_root_movie(movie);
                    return Ok(());
                }

                player.lock().unwrap().update(|uc| {
                    let clip = match uc.load_manager.get_loader(handle) {
                        Some(Loader::Movie { target_clip, .. }) => *target_clip,
                        None => return Err(Error::Cancelled),
                        _ => unreachable!(),
                    };

                    match sniffed_type {
                        ContentType::Swf => {
                            let movie = Arc::new(SwfMovie::from_data(
                                &response.body,
                                Some(response.url),
                                loader_url,
                            )?);

                            let mut activation = Avm2Activation::from_nothing(uc.reborrow());
                            let parent_domain = activation.avm2().global_domain();
                            let domain = Avm2Domain::movie_domain(&mut activation, parent_domain);
                            uc.library
                                .library_for_movie_mut(movie.clone())
                                .set_avm2_domain(domain);

                            if let Some(mut mc) = clip.as_movie_clip() {
                                mc.replace_with_movie(uc.gc_context, Some(movie));
                                mc.post_instantiation(uc, None, Instantiator::Movie, false);
                                mc.preload(uc);
                            }
                        }
                        ContentType::Gif | ContentType::Jpeg | ContentType::Png => {
                            let bitmap = uc.renderer.register_bitmap_jpeg_2(&response.body)?;
                            let bitmap_obj =
                                Bitmap::new(uc, 0, bitmap.handle, bitmap.width, bitmap.height);

                            if let Some(mc) = clip.as_movie_clip() {
                                mc.replace_at_depth(uc, bitmap_obj.into(), 1);
                            }
                        }
                        ContentType::Unknown => {
                            length = 0;
                        }
                    }

                    Loader::movie_loader_progress(handle, uc, length, length)?;

                    Loader::movie_loader_complete(handle, uc)?;

                    Ok(())
                })?; //TODO: content sniffing errors need to be reported somehow
            } else {
                player
                    .lock()
                    .unwrap()
                    .update(|uc| -> Result<(), Error> { Loader::movie_loader_error(handle, uc) })?;
            }

            Ok(())
        })
    }

    fn form_loader(
        &mut self,
        player: Weak<Mutex<Player>>,
        url: String,
        options: RequestOptions,
    ) -> OwnedFuture<(), Error> {
        let handle = match self {
            Loader::Form { self_handle, .. } => self_handle.expect("Loader not self-introduced"),
            _ => return Box::pin(async { Err(Error::NotFormLoader) }),
        };

        let player = player
            .upgrade()
            .expect("Could not upgrade weak reference to player");

        Box::pin(async move {
            let fetch = player.lock().unwrap().navigator().fetch(&url, options);

            let response = fetch.await?;

            // Fire the load handler.
            player.lock().unwrap().update(|uc| {
                let loader = uc.load_manager.get_loader(handle);
                let that = match loader {
                    Some(&Loader::Form { target_object, .. }) => target_object,
                    None => return Err(Error::Cancelled),
                    _ => return Err(Error::NotFormLoader),
                };

                let mut activation = Activation::from_stub(
                    uc.reborrow(),
                    ActivationIdentifier::root("[Form Loader]"),
                );

                for (k, v) in form_urlencoded::parse(&response.body) {
                    let k = AvmString::new_utf8(activation.context.gc_context, k);
                    let v = AvmString::new_utf8(activation.context.gc_context, v);
                    that.set(k, v.into(), &mut activation)?;
                }

                Ok(())
            })
        })
    }

    /// Creates a future for a LoadVars load call.
    fn load_vars_loader(
        &mut self,
        player: Weak<Mutex<Player>>,
        url: String,
        options: RequestOptions,
    ) -> OwnedFuture<(), Error> {
        let handle = match self {
            Loader::LoadVars { self_handle, .. } => {
                self_handle.expect("Loader not self-introduced")
            }
            _ => return Box::pin(async { Err(Error::NotLoadVarsLoader) }),
        };

        let player = player
            .upgrade()
            .expect("Could not upgrade weak reference to player");

        Box::pin(async move {
            let fetch = player.lock().unwrap().navigator().fetch(&url, options);

            let data = fetch.await;

            // Fire the load handler.
            player.lock().unwrap().update(|uc| {
                let loader = uc.load_manager.get_loader(handle);
                let that = match loader {
                    Some(&Loader::LoadVars { target_object, .. }) => target_object,
                    None => return Err(Error::Cancelled),
                    _ => return Err(Error::NotLoadVarsLoader),
                };

                let mut activation =
                    Activation::from_stub(uc.reborrow(), ActivationIdentifier::root("[Loader]"));

                match data {
                    Ok(response) => {
                        let _ = that.call_method(
                            "onHTTPStatus".into(),
                            &[200.into()],
                            &mut activation,
                            ExecutionReason::Special,
                        );

                        // Fire the onData method with the loaded string.
                        let string_data = AvmString::new_utf8(
                            activation.context.gc_context,
                            UTF_8.decode(&response.body).0,
                        );
                        let _ = that.call_method(
                            "onData".into(),
                            &[string_data.into()],
                            &mut activation,
                            ExecutionReason::Special,
                        );
                    }
                    Err(_) => {
                        // TODO: Log "Error opening URL" trace similar to the Flash Player?
                        // Simulate 404 HTTP status. This should probably be fired elsewhere
                        // because a failed local load doesn't fire a 404.
                        let _ = that.call_method(
                            "onHTTPStatus".into(),
                            &[404.into()],
                            &mut activation,
                            ExecutionReason::Special,
                        );

                        // Fire the onData method with no data to indicate an unsuccessful load.
                        let _ = that.call_method(
                            "onData".into(),
                            &[Value::Undefined],
                            &mut activation,
                            ExecutionReason::Special,
                        );
                    }
                }

                Ok(())
            })
        })
    }

    /// Creates a future for a LoadURLLoader load call.
    fn load_url_loader(
        &mut self,
        player: Weak<Mutex<Player>>,
        url: String,
        options: RequestOptions,
        data_format: DataFormat,
    ) -> OwnedFuture<(), Error> {
        let handle = match self {
            Loader::LoadURLLoader { self_handle, .. } => {
                self_handle.expect("Loader not self-introduced")
            }
            _ => return Box::pin(async { Err(Error::NotLoadDataLoader) }),
        };

        let player = player
            .upgrade()
            .expect("Could not upgrade weak reference to player");

        Box::pin(async move {
            let fetch = player.lock().unwrap().navigator().fetch(&url, options);
            let response = fetch.await;

            player.lock().unwrap().update(|uc| {
                let loader = uc.load_manager.get_loader(handle);
                let target = match loader {
                    Some(&Loader::LoadURLLoader { target_object, .. }) => target_object,
                    // We would have already returned after the previous 'update' call
                    _ => unreachable!(),
                };

                let mut activation = Avm2Activation::from_nothing(uc.reborrow());

                fn set_data<'a, 'gc: 'a, 'gc_context: 'a>(
                    body: Vec<u8>,
                    activation: &mut Avm2Activation<'a, 'gc, 'gc_context>,
                    mut target: Avm2Object<'gc>,
                    data_format: DataFormat,
                ) {
                    let data_object = match data_format {
                        DataFormat::Binary => {
                            let storage = ByteArrayStorage::from_vec(body);
                            let bytearray =
                                ByteArrayObject::from_storage(activation, storage).unwrap();
                            bytearray.into()
                        }
                        DataFormat::Text => {
                            // FIXME - what do we do if the data is not UTF-8?
                            Avm2Value::String(
                                AvmString::new_utf8_bytes(activation.context.gc_context, body)
                                    .unwrap(),
                            )
                        }
                        DataFormat::Variables => {
                            log::warn!(
                                "Support for URLLoaderDataFormat.VARIABLES not yet implemented"
                            );
                            Avm2Value::Undefined
                        }
                    };

                    target
                        .set_property(
                            &QName::new(Namespace::public(), "data").into(),
                            data_object,
                            activation,
                        )
                        .unwrap();
                }

                match response {
                    Ok(response) => {
                        // FIXME - the "open" event should be fired earlier, just before
                        // we start to fetch the data.
                        // However, the "open" event should not be fired if an IO error
                        // occurs opening the connection (e.g. if a file does not exist on disk).
                        // We currently have no way of detecting this, so we settle for firing
                        // the event after the entire fetch is complete. This causes there
                        // to a longer delay between the initial load triggered by the script
                        // and the "load" event firing, but it ensures that we match
                        // the Flash behavior w.r.t when an event is fired vs not fired.
                        let mut open_evt = Avm2Event::new("open", Avm2EventData::Empty);
                        open_evt.set_bubbles(false);
                        open_evt.set_cancelable(false);

                        if let Err(e) =
                            Avm2::dispatch_event(&mut activation.context, open_evt, target)
                        {
                            log::error!(
                                "Encountered AVM2 error when broadcasting `open` event: {}",
                                e
                            );
                        }

                        set_data(response.body, &mut activation, target, data_format);

                        let mut complete_evt = Avm2Event::new("complete", Avm2EventData::Empty);
                        complete_evt.set_bubbles(false);
                        complete_evt.set_cancelable(false);

                        if let Err(e) = Avm2::dispatch_event(uc, complete_evt, target) {
                            log::error!(
                                "Encountered AVM2 error when broadcasting `complete` event: {}",
                                e
                            );
                        }
                    }
                    Err(_err) => {
                        // Testing with Flash shoes that the 'data' property is cleared
                        // when an error occurs

                        set_data(Vec::new(), &mut activation, target, data_format);

                        // FIXME - Match the exact error message generated by Flash
                        let mut io_error_evt = Avm2Event::new(
                            "ioError",
                            Avm2EventData::IOError {
                                text: AvmString::new_utf8(
                                    activation.context.gc_context,
                                    "Error #2032: Stream Error",
                                ),
                                error_id: 2032,
                            },
                        );
                        io_error_evt.set_bubbles(false);
                        io_error_evt.set_cancelable(false);

                        if let Err(e) = Avm2::dispatch_event(uc, io_error_evt, target) {
                            log::error!(
                                "Encountered AVM2 error when broadcasting `ioError` event: {}",
                                e
                            );
                        }
                    }
                }

                Ok(())
            })
        })
    }

    /// Report a movie loader start event to script code.
    fn movie_loader_start(handle: Index, uc: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let me = uc.load_manager.get_loader_mut(handle);
        if me.is_none() {
            return Err(Error::Cancelled);
        }

        let me = me.unwrap();

        let (clip, broadcaster) = match me {
            Loader::Movie {
                target_clip,
                target_broadcaster,
                ..
            } => (*target_clip, *target_broadcaster),
            _ => unreachable!(),
        };

        if let Some(broadcaster) = broadcaster {
            Avm1::run_stack_frame_for_method(
                clip,
                broadcaster,
                uc,
                "broadcastMessage".into(),
                &["onLoadStart".into(), clip.object()],
            );
        }

        Ok(())
    }

    /// Report a movie loader progress event to script code.
    fn movie_loader_progress(
        handle: Index,
        uc: &mut UpdateContext<'_, 'gc, '_>,
        cur_len: usize,
        total_len: usize,
    ) -> Result<(), Error> {
        let me = uc.load_manager.get_loader_mut(handle);
        if me.is_none() {
            return Err(Error::Cancelled);
        }

        let me = me.unwrap();

        let (clip, broadcaster) = match me {
            Loader::Movie {
                target_clip,
                target_broadcaster,
                ..
            } => (*target_clip, *target_broadcaster),
            _ => unreachable!(),
        };

        if let Some(broadcaster) = broadcaster {
            Avm1::run_stack_frame_for_method(
                clip,
                broadcaster,
                uc,
                "broadcastMessage".into(),
                &[
                    "onLoadProgress".into(),
                    clip.object(),
                    cur_len.into(),
                    total_len.into(),
                ],
            );
        }

        Ok(())
    }

    /// Report a movie loader completion to script code.
    fn movie_loader_complete(
        handle: Index,
        uc: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let (clip, broadcaster) = match uc.load_manager.get_loader_mut(handle) {
            Some(Loader::Movie {
                target_clip,
                target_broadcaster,
                ..
            }) => (*target_clip, *target_broadcaster),
            None => return Err(Error::Cancelled),
            _ => unreachable!(),
        };

        if let Some(broadcaster) = broadcaster {
            Avm1::run_stack_frame_for_method(
                clip,
                broadcaster,
                uc,
                "broadcastMessage".into(),
                // TODO: Pass an actual httpStatus argument instead of 0.
                &["onLoadComplete".into(), clip.object(), 0.into()],
            );
        }

        if let Loader::Movie { loader_status, .. } = uc.load_manager.get_loader_mut(handle).unwrap()
        {
            *loader_status = LoaderStatus::Succeeded;
        };

        Ok(())
    }

    /// Report a movie loader error to script code.
    ///
    /// This is an associated function because we cannot borrow both the update
    /// context and one of it's loaders.
    fn movie_loader_error(handle: Index, uc: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        //TODO: Inspect the fetch error.
        //This requires cooperation from the backend to send abstract
        //error types we can actually inspect.
        //This also can get errors from decoding an invalid SWF file,
        //too. We should distinguish those to player code.
        let (clip, broadcaster) = match uc.load_manager.get_loader_mut(handle) {
            Some(Loader::Movie {
                target_clip,
                target_broadcaster,
                ..
            }) => (*target_clip, *target_broadcaster),
            None => return Err(Error::Cancelled),
            _ => unreachable!(),
        };

        if let Some(broadcaster) = broadcaster {
            Avm1::run_stack_frame_for_method(
                clip,
                broadcaster,
                uc,
                "broadcastMessage".into(),
                &[
                    "onLoadError".into(),
                    clip.object(),
                    "LoadNeverCompleted".into(),
                ],
            );
        }

        if let Loader::Movie { loader_status, .. } = uc.load_manager.get_loader_mut(handle).unwrap()
        {
            *loader_status = LoaderStatus::Failed;
        };

        Ok(())
    }

    /// Event handler morally equivalent to `onLoad` on a movie clip.
    ///
    /// Returns `true` if the loader has completed and should be removed.
    ///
    /// Used to fire listener events on clips and terminate completed loaders.
    fn movie_clip_loaded(&mut self, queue: &mut ActionQueue<'gc>) -> bool {
        let (clip, broadcaster, loader_status) = match self {
            Loader::Movie {
                target_clip,
                target_broadcaster,
                loader_status,
                ..
            } => (*target_clip, *target_broadcaster, *loader_status),
            _ => return false,
        };

        match loader_status {
            LoaderStatus::Pending => false,
            LoaderStatus::Failed => true,
            LoaderStatus::Succeeded => {
                if let Some(broadcaster) = broadcaster {
                    queue.queue_actions(
                        clip,
                        ActionType::Method {
                            object: broadcaster,
                            name: "broadcastMessage",
                            args: vec!["onLoadInit".into(), clip.object()],
                        },
                        false,
                    );
                }
                true
            }
        }
    }
}
