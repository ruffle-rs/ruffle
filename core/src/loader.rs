//! Management of async loaders

use crate::avm1::Avm1;
use crate::avm1::ExecutionReason;
use crate::avm1::{Activation, ActivationIdentifier};
use crate::avm1::{Object, SoundObject, TObject, Value};
use crate::avm2::bytearray::ByteArrayStorage;
use crate::avm2::object::ByteArrayObject;
use crate::avm2::object::EventObject as Avm2EventObject;
use crate::avm2::object::LoaderStream;
use crate::avm2::object::TObject as _;
use crate::avm2::{
    Activation as Avm2Activation, Avm2, Domain as Avm2Domain, Multiname as Avm2Multiname,
    Object as Avm2Object, Value as Avm2Value,
};
use crate::backend::navigator::{OwnedFuture, Request};
use crate::context::{ActionQueue, ActionType, UpdateContext};
use crate::display_object::{
    Bitmap, DisplayObject, TDisplayObject, TDisplayObjectContainer, TInteractiveObject,
};
use crate::events::ClipEvent;
use crate::frame_lifecycle::catchup_display_object_to_frame;
use crate::limits::ExecutionLimit;
use crate::player::Player;
use crate::string::AvmString;
use crate::tag_utils::SwfMovie;
use crate::vminterface::Instantiator;
use encoding_rs::UTF_8;
use gc_arena::{Collect, CollectionContext};
use generational_arena::{Arena, Index};
use ruffle_render::utils::{determine_jpeg_tag_format, JpegTagFormat};
use std::fmt;
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;
use swf::read::read_compression_type;
use thiserror::Error;
use url::form_urlencoded;

pub type Handle = Index;

/// Enumeration of all content types that `Loader` can handle.
///
/// This is a superset of `JpegTagFormat`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub enum DataFormat {
    Binary,
    Text,
    Variables,
}

#[derive(Debug, Error)]
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

    #[error("Non-sound loader spawned as sound loader")]
    NotSoundLoader,

    #[error("Could not fetch: {0}")]
    FetchError(String),

    #[error("Invalid SWF: {0}")]
    InvalidSwf(#[from] crate::tag_utils::Error),

    #[error("Invalid bitmap")]
    InvalidBitmap(#[from] ruffle_render::error::Error),

    #[error("Invalid sound: {0}")]
    InvalidSound(#[from] crate::backend::audio::DecodeError),

    #[error("Unexpected content of type {1}, expected {0}")]
    UnexpectedData(ContentType, ContentType),

    // TODO: We can't support lifetimes on this error object yet (or we'll need some backends inside
    // the GC arena). We're losing info here. How do we fix that?
    #[error("Error running avm1 script: {0}")]
    Avm1Error(String),

    // TODO: We can't support lifetimes on this error object yet (or we'll need some backends inside
    // the GC arena). We're losing info here. How do we fix that?
    #[error("Error running avm2 script: {0}")]
    Avm2Error(String),
}

impl From<crate::avm1::Error<'_>> for Error {
    fn from(error: crate::avm1::Error<'_>) -> Self {
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
            | Loader::LoadURLLoader { self_handle, .. }
            | Loader::SoundAvm1 { self_handle, .. }
            | Loader::SoundAvm2 { self_handle, .. } => *self_handle = Some(handle),
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
        request: Request,
        parameters: Vec<(String, String)>,
        on_metadata: Box<dyn FnOnce(&swf::HeaderExt)>,
    ) -> OwnedFuture<(), Error> {
        let loader = Loader::RootMovie { self_handle: None };
        let handle = self.add_loader(loader);
        let loader = self.get_loader_mut(handle).unwrap();
        loader.root_movie_loader(player, request, parameters, on_metadata)
    }

    /// Kick off a movie clip load.
    ///
    /// Returns the loader's async process, which you will need to spawn.
    pub fn load_movie_into_clip(
        &mut self,
        player: Weak<Mutex<Player>>,
        target_clip: DisplayObject<'gc>,
        request: Request,
        loader_url: Option<String>,
        event_handler: Option<MovieLoaderEventHandler<'gc>>,
    ) -> OwnedFuture<(), Error> {
        let loader = Loader::Movie {
            self_handle: None,
            target_clip,
            event_handler,
            loader_status: LoaderStatus::Pending,
            movie: None,
        };
        let handle = self.add_loader(loader);
        let loader = self.get_loader_mut(handle).unwrap();
        loader.movie_loader(player, request, loader_url)
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
        request: Request,
    ) -> OwnedFuture<(), Error> {
        let loader = Loader::Form {
            self_handle: None,
            target_object,
        };
        let handle = self.add_loader(loader);
        let loader = self.get_loader_mut(handle).unwrap();
        loader.form_loader(player, request)
    }

    /// Kick off a form data load into an AVM1 object.
    ///
    /// Returns the loader's async process, which you will need to spawn.
    pub fn load_form_into_load_vars(
        &mut self,
        player: Weak<Mutex<Player>>,
        target_object: Object<'gc>,
        request: Request,
    ) -> OwnedFuture<(), Error> {
        let loader = Loader::LoadVars {
            self_handle: None,
            target_object,
        };
        let handle = self.add_loader(loader);
        let loader = self.get_loader_mut(handle).unwrap();
        loader.load_vars_loader(player, request)
    }

    /// Kick off a data load into a `URLLoader`, updating
    /// its `data` property when the load completes.
    ///
    /// Returns the loader's async process, which you will need to spawn.
    pub fn load_data_into_url_loader(
        &mut self,
        player: Weak<Mutex<Player>>,
        target_object: Avm2Object<'gc>,
        request: Request,
        data_format: DataFormat,
    ) -> OwnedFuture<(), Error> {
        let loader = Loader::LoadURLLoader {
            self_handle: None,
            target_object,
        };
        let handle = self.add_loader(loader);
        let loader = self.get_loader_mut(handle).unwrap();
        loader.load_url_loader(player, request, data_format)
    }

    /// Kick off an AVM1 audio load.
    ///
    /// Returns the loader's async process, which you will need to spawn.
    pub fn load_sound_avm1(
        &mut self,
        player: Weak<Mutex<Player>>,
        target_object: SoundObject<'gc>,
        request: Request,
        is_streaming: bool,
    ) -> OwnedFuture<(), Error> {
        let loader = Loader::SoundAvm1 {
            self_handle: None,
            target_object,
        };
        let handle = self.add_loader(loader);
        let loader = self.get_loader_mut(handle).unwrap();
        loader.sound_loader_avm1(player, request, is_streaming)
    }

    /// Kick off an AVM2 audio load.
    ///
    /// Returns the loader's async process, which you will need to spawn.
    pub fn load_sound_avm2(
        &mut self,
        player: Weak<Mutex<Player>>,
        target_object: Avm2Object<'gc>,
        request: Request,
    ) -> OwnedFuture<(), Error> {
        let loader = Loader::SoundAvm2 {
            self_handle: None,
            target_object,
        };
        let handle = self.add_loader(loader);
        let loader = self.get_loader_mut(handle).unwrap();
        loader.sound_loader_avm2(player, request)
    }

    /// Process tags on all loaders in the Parsing phase.
    ///
    /// Returns true if *all* loaders finished preloading.
    pub fn preload_tick(
        context: &mut UpdateContext<'_, 'gc, '_>,
        limit: &mut ExecutionLimit,
    ) -> bool {
        let mut did_finish = true;
        let handles: Vec<_> = context.load_manager.0.iter().map(|(h, _)| h).collect();

        for handle in handles {
            let status = match context.load_manager.get_loader(handle) {
                Some(Loader::Movie { loader_status, .. }) => Some(loader_status),
                _ => None,
            };

            if matches!(status, Some(LoaderStatus::Parsing)) {
                match Loader::preload_tick(handle, context, limit) {
                    Ok(f) => did_finish = did_finish && f,
                    Err(e) => log::error!("Error encountered while preloading movie: {}", e),
                }
            }
        }

        did_finish
    }
}

impl<'gc> Default for LoadManager<'gc> {
    fn default() -> Self {
        Self::new()
    }
}

/// The completion status of a `Loader` loading a movie.
#[derive(Clone, Collect, Copy, Debug, Eq, PartialEq)]
#[collect(require_static)]
pub enum LoaderStatus {
    /// The movie hasn't been loaded yet.
    Pending,
    /// The movie is currently being parsed (e.g. mc.preload)
    Parsing,
    /// The movie loaded successfully.
    Succeeded,
    /// An error occurred while loading the movie.
    Failed,
}

#[derive(Collect, Clone, Copy, Debug)]
#[collect(no_drop)]
pub enum MovieLoaderEventHandler<'gc> {
    Avm1Broadcast(Object<'gc>),
    Avm2LoaderInfo(Avm2Object<'gc>),
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
        event_handler: Option<MovieLoaderEventHandler<'gc>>,

        /// Indicates the completion status of this loader.
        ///
        /// This flag exists to prevent a situation in which loading a movie
        /// into a clip that has not yet fired its Load event causes the
        /// loader to be prematurely removed. This flag is only set when either
        /// the movie has been replaced (and thus Load events can be trusted)
        /// or an error has occurred (in which case we don't care about the
        /// loader anymore).
        loader_status: LoaderStatus,

        /// The SWF being loaded.
        ///
        /// This is only available if the asynchronous loader path has
        /// completed and we expect the Player to periodically tick preload
        /// until loading completes.
        movie: Option<Arc<SwfMovie>>,
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

    /// Loader that is loading an MP3 into an AVM1 Sound object.
    SoundAvm1 {
        /// The handle to refer to this loader instance.
        #[collect(require_static)]
        self_handle: Option<Handle>,

        /// The target AVM1 object to load the audio into.
        target_object: SoundObject<'gc>,
    },

    /// Loader that is loading an MP3 into an AVM2 Sound object.
    SoundAvm2 {
        /// The handle to refer to this loader instance.
        #[collect(require_static)]
        self_handle: Option<Handle>,

        /// The target AVM1 object to load the audio into.
        target_object: Avm2Object<'gc>,
    },
}

impl<'gc> Loader<'gc> {
    /// Process tags on a loaded movie.
    ///
    /// Is only callable on Movie loaders, panics otherwise. Will
    /// do nothing unless the movie is ready to be preloaded. Movies which
    /// complete their preload will fire all events and be removed from the
    /// load manager queue.
    ///
    /// Returns true if the movie finished preloading.
    ///
    /// Returns any AVM errors encountered while sending events to user code.
    fn preload_tick(
        handle: Handle,
        context: &mut UpdateContext<'_, 'gc, '_>,
        limit: &mut ExecutionLimit,
    ) -> Result<bool, Error> {
        let (mc, event_handler, movie) = match context.load_manager.get_loader_mut(handle) {
            Some(Self::Movie {
                target_clip,
                event_handler,
                movie,
                ..
            }) => {
                if movie.is_none() {
                    //Non-SWF load or file not loaded yet
                    return Ok(false);
                }

                if target_clip.as_movie_clip().is_none() {
                    // Non-movie-clip loads should not be handled in preload_tick
                    log::error!("Cannot preload non-movie-clip loader");
                    return Ok(false);
                }

                (*target_clip, *event_handler, movie.clone().unwrap())
            }
            None => return Err(Error::Cancelled),
            Some(_) => panic!("Attempted to preload a non-SWF loader"),
        };

        let mc = mc.as_movie_clip().unwrap();

        let did_finish = mc.preload(context, limit);
        if did_finish {
            mc.post_instantiation(context, None, Instantiator::Movie, false);
            catchup_display_object_to_frame(context, mc.into());
        }

        Loader::movie_loader_progress(
            handle,
            context,
            mc.compressed_loaded_bytes() as usize,
            mc.compressed_total_bytes() as usize,
        )?;

        if did_finish {
            let loader_info =
                if let Some(MovieLoaderEventHandler::Avm2LoaderInfo(loader_info)) = event_handler {
                    Some(*loader_info.as_loader_info_object().unwrap())
                } else {
                    None
                };

            if let Some(loader_info) = loader_info {
                // Store the real movie into the `LoaderStream`, so that
                // 'bytesTotal' starts returning the correct value
                // (we previously had a fake empty SwfMovie).
                // However, we still use `LoaderStream::NotYetLoaded`, since
                // the actual MovieClip display object has not run its first
                // frame yet.
                loader_info.set_loader_stream(
                    LoaderStream::NotYetLoaded(movie, Some(mc.into())),
                    context.gc_context,
                );
            }

            if let Some(MovieLoaderEventHandler::Avm2LoaderInfo(loader_info)) = event_handler {
                let mut activation = Avm2Activation::from_nothing(context.reborrow());
                let mut loader = loader_info
                    .get_property(&Avm2Multiname::public("loader"), &mut activation)
                    .map_err(|e| Error::Avm2Error(e.to_string()))?
                    .as_object()
                    .unwrap()
                    .as_display_object()
                    .unwrap()
                    .as_container()
                    .unwrap();

                // Note that we do *not* use the 'addChild' method here:
                // Per the flash docs, our implementation always throws
                // an 'unsupported' error. Also, the AVM2 side of our movie
                // clip does not yet exist.
                loader.insert_at_index(&mut activation.context, mc.into(), 0);
            }

            Loader::movie_loader_complete(handle, context)?;
        }

        Ok(did_finish)
    }

    /// Construct a future for the root movie loader.
    fn root_movie_loader(
        &mut self,
        player: Weak<Mutex<Player>>,
        request: Request,
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
            let fetch = player.lock().unwrap().navigator().fetch(request);

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
        request: Request,
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
            let fetch = player.lock().unwrap().navigator().fetch(request);

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
                    mc.replace_with_movie(uc, None, None);
                }

                Loader::movie_loader_start(handle, uc)
            })?;

            match fetch.await {
                Ok(response) => {
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
                        let (clip, event_handler) = match uc.load_manager.get_loader(handle) {
                            Some(Loader::Movie {
                                target_clip,
                                event_handler,
                                ..
                            }) => (*target_clip, *event_handler),
                            None => return Err(Error::Cancelled),
                            _ => unreachable!(),
                        };

                        if let ContentType::Unknown = sniffed_type {
                            length = 0;
                        }

                        if let Some(MovieLoaderEventHandler::Avm2LoaderInfo(_)) = event_handler {
                            // Flash always fires an initial 'progress' event with
                            // bytesLoaded=0 and bytesTotal set to the proper value.
                            // This only seems to happen for an AVM2 event handler
                            Loader::movie_loader_progress(handle, uc, 0, length)?;
                        }

                        match sniffed_type {
                            ContentType::Swf => {
                                let movie = Arc::new(SwfMovie::from_data(
                                    &response.body,
                                    Some(response.url),
                                    loader_url,
                                )?);

                                match uc.load_manager.get_loader_mut(handle) {
                                    Some(Loader::Movie {
                                        movie: old,
                                        loader_status,
                                        ..
                                    }) => {
                                        *loader_status = LoaderStatus::Parsing;
                                        *old = Some(movie.clone())
                                    }
                                    _ => unreachable!(),
                                };

                                let mut activation = Avm2Activation::from_nothing(uc.reborrow());
                                let parent_domain = activation.avm2().global_domain();
                                let domain =
                                    Avm2Domain::movie_domain(&mut activation, parent_domain);
                                activation
                                    .context
                                    .library
                                    .library_for_movie_mut(movie.clone())
                                    .set_avm2_domain(domain);

                                if let Some(mut mc) = clip.as_movie_clip() {
                                    let loader_info = if let Some(
                                        MovieLoaderEventHandler::Avm2LoaderInfo(loader_info),
                                    ) = event_handler
                                    {
                                        Some(*loader_info.as_loader_info_object().unwrap())
                                    } else {
                                        None
                                    };

                                    // Store our downloaded `SwfMovie` into our target `MovieClip`,
                                    // and initialize it.

                                    mc.replace_with_movie(
                                        &mut activation.context,
                                        Some(movie),
                                        loader_info,
                                    );
                                }

                                // NOTE: Certain tests specifically expect small files to preload immediately
                                Loader::preload_tick(
                                    handle,
                                    uc,
                                    &mut ExecutionLimit::with_max_actions_and_time(
                                        10000,
                                        Duration::from_millis(1),
                                    ),
                                )?;

                                return Ok(());
                            }
                            ContentType::Gif | ContentType::Jpeg | ContentType::Png => {
                                let bitmap = uc.renderer.register_bitmap_jpeg_2(&response.body)?;
                                let bitmap_obj =
                                    Bitmap::new(uc, 0, bitmap.handle, bitmap.width, bitmap.height);

                                if let Some(mc) = clip.as_movie_clip() {
                                    mc.replace_at_depth(uc, bitmap_obj.into(), 1);
                                }
                            }
                            ContentType::Unknown => {}
                        }

                        Loader::movie_loader_progress(handle, uc, length, length)?;
                        Loader::movie_loader_complete(handle, uc)?;

                        Ok(())
                    })?; //TODO: content sniffing errors need to be reported somehow
                }
                Err(e) => {
                    log::error!("Error during movie loading: {:?}", e);
                    player.lock().unwrap().update(|uc| -> Result<(), Error> {
                        Loader::movie_loader_error(handle, uc)
                    })?;
                }
            }

            Ok(())
        })
    }

    fn form_loader(
        &mut self,
        player: Weak<Mutex<Player>>,
        request: Request,
    ) -> OwnedFuture<(), Error> {
        let handle = match self {
            Loader::Form { self_handle, .. } => self_handle.expect("Loader not self-introduced"),
            _ => return Box::pin(async { Err(Error::NotFormLoader) }),
        };

        let player = player
            .upgrade()
            .expect("Could not upgrade weak reference to player");

        Box::pin(async move {
            let fetch = player.lock().unwrap().navigator().fetch(request);

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

                // Fire the onData method and event.
                if let Some(display_object) = that.as_display_object() {
                    if let Some(movie_clip) = display_object.as_movie_clip() {
                        activation.context.action_queue.queue_actions(
                            movie_clip.into(),
                            ActionType::Method {
                                object: that,
                                name: "onData",
                                args: vec![],
                            },
                            false,
                        );
                        movie_clip.event_dispatch(&mut activation.context, ClipEvent::Data);
                    }
                }

                Ok(())
            })
        })
    }

    /// Creates a future for a LoadVars load call.
    fn load_vars_loader(
        &mut self,
        player: Weak<Mutex<Player>>,
        request: Request,
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
            let fetch = player.lock().unwrap().navigator().fetch(request);

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
        request: Request,
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
            let fetch = player.lock().unwrap().navigator().fetch(request);
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
                        DataFormat::Text => Avm2Value::String(AvmString::new_utf8_bytes(
                            activation.context.gc_context,
                            &body,
                        )),
                        DataFormat::Variables => {
                            log::warn!(
                                "Support for URLLoaderDataFormat.VARIABLES not yet implemented"
                            );
                            Avm2Value::Undefined
                        }
                    };

                    target
                        .set_property(&Avm2Multiname::public("data"), data_object, activation)
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
                        let open_evt =
                            Avm2EventObject::bare_default_event(&mut activation.context, "open");

                        if let Err(e) =
                            Avm2::dispatch_event(&mut activation.context, open_evt, target)
                        {
                            log::error!(
                                "Encountered AVM2 error when broadcasting `open` event: {}",
                                e
                            );
                        }

                        set_data(response.body, &mut activation, target, data_format);

                        let complete_evt = Avm2EventObject::bare_default_event(
                            &mut activation.context,
                            "complete",
                        );

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

                        let io_error_evt_cls = activation.avm2().classes().ioerrorevent;
                        let io_error_evt = io_error_evt_cls
                            .construct(
                                &mut activation,
                                &[
                                    "ioError".into(),
                                    false.into(),
                                    false.into(),
                                    "Error #2032: Stream Error".into(),
                                    2032.into(),
                                ],
                            )
                            .map_err(|e| Error::Avm2Error(e.to_string()))?;

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

    /// Creates a future for a Sound load call.
    fn sound_loader_avm1(
        &mut self,
        player: Weak<Mutex<Player>>,
        request: Request,
        is_streaming: bool,
    ) -> OwnedFuture<(), Error> {
        let handle = match self {
            Loader::SoundAvm1 { self_handle, .. } => {
                self_handle.expect("Loader not self-introduced")
            }
            _ => return Box::pin(async { Err(Error::NotLoadVarsLoader) }),
        };

        let player = player
            .upgrade()
            .expect("Could not upgrade weak reference to player");

        Box::pin(async move {
            let fetch = player.lock().unwrap().navigator().fetch(request);
            let data = fetch.await;

            // Fire the load handler.
            player.lock().unwrap().update(|uc| {
                let loader = uc.load_manager.get_loader(handle);
                let sound_object = match loader {
                    Some(&Loader::SoundAvm1 { target_object, .. }) => target_object,
                    None => return Err(Error::Cancelled),
                    _ => return Err(Error::NotSoundLoader),
                };

                let success = data
                    .and_then(|data| {
                        let handle = uc.audio.register_mp3(&data.body)?;
                        sound_object.set_sound(uc.gc_context, Some(handle));
                        let duration = uc
                            .audio
                            .get_sound_duration(handle)
                            .map(|d| d.round() as u32);
                        sound_object.set_duration(uc.gc_context, duration);
                        Ok(())
                    })
                    .is_ok();

                let mut activation =
                    Activation::from_stub(uc.reborrow(), ActivationIdentifier::root("[Loader]"));
                let _ = sound_object.call_method(
                    "onLoad".into(),
                    &[success.into()],
                    &mut activation,
                    ExecutionReason::Special,
                );

                // Streaming sounds should auto-play.
                if is_streaming {
                    crate::avm1::start_sound(&mut activation, sound_object.into(), &[])?;
                }

                Ok(())
            })
        })
    }

    /// Creates a future for a LoadURLLoader load call.
    fn sound_loader_avm2(
        &mut self,
        player: Weak<Mutex<Player>>,
        request: Request,
    ) -> OwnedFuture<(), Error> {
        let handle = match self {
            Loader::SoundAvm2 { self_handle, .. } => {
                self_handle.expect("Loader not self-introduced")
            }
            _ => return Box::pin(async { Err(Error::NotLoadDataLoader) }),
        };

        let player = player
            .upgrade()
            .expect("Could not upgrade weak reference to player");

        Box::pin(async move {
            let fetch = player.lock().unwrap().navigator().fetch(request);
            let response = fetch.await;

            player.lock().unwrap().update(|uc| {
                let loader = uc.load_manager.get_loader(handle);
                let sound_object = match loader {
                    Some(&Loader::SoundAvm2 { target_object, .. }) => target_object,
                    None => return Err(Error::Cancelled),
                    _ => return Err(Error::NotSoundLoader),
                };

                match response {
                    Ok(response) => {
                        let handle = uc.audio.register_mp3(&response.body)?;
                        sound_object.set_sound(uc.gc_context, handle);

                        // FIXME - the "open" event should be fired earlier, and not fired in case of ioerror.
                        let mut activation = Avm2Activation::from_nothing(uc.reborrow());
                        let open_evt =
                            Avm2EventObject::bare_default_event(&mut activation.context, "open");
                        if let Err(e) =
                            Avm2::dispatch_event(&mut activation.context, open_evt, sound_object)
                        {
                            log::error!(
                                "Encountered AVM2 error when broadcasting `open` event: {}",
                                e
                            );
                        }

                        let complete_evt = Avm2EventObject::bare_default_event(
                            &mut activation.context,
                            "complete",
                        );
                        if let Err(e) = Avm2::dispatch_event(uc, complete_evt, sound_object) {
                            log::error!(
                                "Encountered AVM2 error when broadcasting `complete` event: {}",
                                e
                            );
                        }
                    }
                    Err(_err) => {
                        // FIXME: Match the exact error message generated by Flash.
                        let mut activation = Avm2Activation::from_nothing(uc.reborrow());
                        let io_error_evt_cls = activation.avm2().classes().ioerrorevent;
                        let io_error_evt = io_error_evt_cls
                            .construct(
                                &mut activation,
                                &[
                                    "ioError".into(),
                                    false.into(),
                                    false.into(),
                                    "Error #2032: Stream Error".into(),
                                    2032.into(),
                                ],
                            )
                            .map_err(|e| Error::Avm2Error(e.to_string()))?;

                        if let Err(e) = Avm2::dispatch_event(uc, io_error_evt, sound_object) {
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

        let (clip, event_handler) = match me {
            Loader::Movie {
                target_clip,
                event_handler,
                ..
            } => (*target_clip, *event_handler),
            _ => unreachable!(),
        };

        match event_handler {
            Some(MovieLoaderEventHandler::Avm1Broadcast(broadcaster)) => {
                Avm1::run_stack_frame_for_method(
                    clip,
                    broadcaster,
                    uc,
                    "broadcastMessage".into(),
                    &["onLoadStart".into(), clip.object()],
                );
            }
            Some(MovieLoaderEventHandler::Avm2LoaderInfo(loader_info)) => {
                let mut activation = Avm2Activation::from_nothing(uc.reborrow());
                let open_evt = Avm2EventObject::bare_default_event(&mut activation.context, "open");

                if let Err(e) = Avm2::dispatch_event(uc, open_evt, loader_info) {
                    log::error!(
                        "Encountered AVM2 error when broadcasting `open` event: {}",
                        e
                    );
                }
            }
            None => {}
        }

        Ok(())
    }

    /// Report a movie loader progress event to script code.
    ///
    /// The current and total length are always reported as compressed lengths.
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

        let (clip, event_handler) = match me {
            Loader::Movie {
                target_clip,
                event_handler,
                ..
            } => (*target_clip, *event_handler),
            _ => unreachable!(),
        };

        match event_handler {
            Some(MovieLoaderEventHandler::Avm1Broadcast(broadcaster)) => {
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
            Some(MovieLoaderEventHandler::Avm2LoaderInfo(loader_info)) => {
                let mut activation = Avm2Activation::from_nothing(uc.reborrow());

                let progress_evt = activation
                    .avm2()
                    .classes()
                    .progressevent
                    .construct(
                        &mut activation,
                        &[
                            "progress".into(),
                            false.into(),
                            false.into(),
                            cur_len.into(),
                            total_len.into(),
                        ],
                    )
                    .map_err(|e| Error::Avm2Error(e.to_string()))?;

                if let Err(e) = Avm2::dispatch_event(uc, progress_evt, loader_info) {
                    log::error!(
                        "Encountered AVM2 error when broadcasting `progress` event: {}",
                        e
                    );
                }
            }
            None => {}
        }

        Ok(())
    }

    /// Report a movie loader completion to script code.
    fn movie_loader_complete(
        handle: Index,
        uc: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let (clip, event_handler) = match uc.load_manager.get_loader_mut(handle) {
            Some(Loader::Movie {
                target_clip,
                event_handler,
                ..
            }) => (*target_clip, *event_handler),
            None => return Err(Error::Cancelled),
            _ => unreachable!(),
        };

        match event_handler {
            Some(MovieLoaderEventHandler::Avm1Broadcast(broadcaster)) => {
                Avm1::run_stack_frame_for_method(
                    clip,
                    broadcaster,
                    uc,
                    "broadcastMessage".into(),
                    // TODO: Pass an actual httpStatus argument instead of 0.
                    &["onLoadComplete".into(), clip.object(), 0.into()],
                );
            }
            // This is fired after we process the movie's first frame,
            // in `MovieClip.on_exit_frame`
            Some(MovieLoaderEventHandler::Avm2LoaderInfo(loader_info)) => {
                loader_info
                    .as_loader_info_object()
                    .unwrap()
                    .set_loader_stream(
                        LoaderStream::Swf(clip.as_movie_clip().unwrap().movie().unwrap(), clip),
                        uc.gc_context,
                    );
            }
            None => {}
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
        let (clip, event_handler) = match uc.load_manager.get_loader_mut(handle) {
            Some(Loader::Movie {
                target_clip,
                event_handler,
                ..
            }) => (*target_clip, *event_handler),
            None => return Err(Error::Cancelled),
            _ => unreachable!(),
        };

        match event_handler {
            Some(MovieLoaderEventHandler::Avm1Broadcast(broadcaster)) => {
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
            Some(MovieLoaderEventHandler::Avm2LoaderInfo(loader_info)) => {
                let mut activation = Avm2Activation::from_nothing(uc.reborrow());
                // FIXME - Match the exact error message generated by Flash

                let io_error_evt_cls = activation.avm2().classes().ioerrorevent;
                let io_error_evt = io_error_evt_cls
                    .construct(
                        &mut activation,
                        &[
                            "ioError".into(),
                            false.into(),
                            false.into(),
                            "Movie loader error".into(),
                            0.into(),
                        ],
                    )
                    .map_err(|e| Error::Avm2Error(e.to_string()))?;

                if let Err(e) = Avm2::dispatch_event(uc, io_error_evt, loader_info) {
                    log::error!(
                        "Encountered AVM2 error when broadcasting `ioError` event: {}",
                        e
                    );
                }
            }
            None => {}
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
        let (clip, event_handler, loader_status) = match self {
            Loader::Movie {
                target_clip,
                event_handler,
                loader_status,
                ..
            } => (*target_clip, *event_handler, *loader_status),
            _ => return false,
        };

        match loader_status {
            LoaderStatus::Pending => false,
            LoaderStatus::Parsing => false,
            LoaderStatus::Failed => true,
            LoaderStatus::Succeeded => {
                // AVM2 is handled separately
                if let Some(MovieLoaderEventHandler::Avm1Broadcast(broadcaster)) = event_handler {
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
