//! Management of async loaders

use crate::avm1::{Activation, ActivationIdentifier};
use crate::avm1::{Attribute, Avm1};
use crate::avm1::{ExecutionReason, NativeObject};
use crate::avm1::{Object, ObjectHandle, Value};
use crate::avm2::bytearray::ByteArrayStorage;
use crate::avm2::globals::flash::utils::byte_array::strip_bom;
use crate::avm2::object::{
    ByteArrayObject, EventObject as Avm2EventObject, FileReferenceObject,
    FileReferenceObjectHandle, LoaderInfoObject, LoaderStream, ScriptObject as Avm2ScriptObject,
    ScriptObjectHandle as Avm2ScriptObjectHandle, SoundLoadingState, SoundObject,
    SoundObjectHandle, TObject as _,
};
use crate::avm2::{
    Activation as Avm2Activation, Avm2, BitmapDataObject, Domain as Avm2Domain,
    Object as Avm2Object,
};
use crate::avm2_stub_method_context;
use crate::backend::navigator::{
    ErrorResponse, FetchReason, OwnedFuture, Request, SuccessResponse,
};
use crate::backend::ui::DialogResultFuture;
use crate::bitmap::bitmap_data::BitmapData;
use crate::bitmap::bitmap_data::Color;
use crate::context::{ActionQueue, ActionType, UpdateContext};
use crate::display_object::{
    DisplayObject, MovieClip, MovieClipHandle, TDisplayObject, TDisplayObjectContainer,
    TInteractiveObject,
};
use crate::events::ClipEvent;
use crate::frame_lifecycle::catchup_display_object_to_frame;
use crate::limits::ExecutionLimit;
use crate::player::{Player, PostFrameCallback};
use crate::streams::{NetStream, NetStreamHandle};
use crate::string::{AvmString, StringContext};
use crate::tag_utils::SwfMovie;
use crate::vminterface::Instantiator;
use chardetng::EncodingDetector;
use encoding_rs::{UTF_8, WINDOWS_1252};
use gc_arena::Collect;
use gc_arena::collect::Trace;
use indexmap::IndexMap;
use ruffle_macros::istr;
use ruffle_render::utils::{JpegTagFormat, determine_jpeg_tag_format};
use slotmap::{SlotMap, new_key_type};
use std::borrow::Borrow;
use std::fmt;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use swf::read::{extract_swz, read_compression_type};
use thiserror::Error;
use url::{ParseError, Url, form_urlencoded};

new_key_type! {
    pub struct LoaderHandle;
}

/// The depth of AVM1 movies that AVM2 loads.
const LOADER_INSERTED_AVM1_DEPTH: i32 = -0xF000;

/// How Ruffle should load movies.
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoadBehavior {
    /// Allow movies to execute before they have finished loading.
    ///
    /// Frames/bytes loaded values will tick up normally and progress events
    /// will be fired at regular intervals. Movie preload animations will play
    /// normally.
    Streaming,

    /// Delay execution of loaded movies until they have finished loading.
    ///
    /// Movies will see themselves load immediately. Preload animations will be
    /// skipped. This may break movies that depend on loading during execution.
    Delayed,

    /// Block Ruffle until movies have finished loading.
    ///
    /// This has the same implications as `Delay`, but tag processing will be
    /// done synchronously. Complex movies will visibly block the player from
    /// accepting user input and the application will appear to freeze.
    Blocking,
}

impl fmt::Display for LoadBehavior {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            LoadBehavior::Streaming => "streaming",
            LoadBehavior::Delayed => "delayed",
            LoadBehavior::Blocking => "blocking",
        })
    }
}

pub struct ParseEnumError;

impl FromStr for LoadBehavior {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let behavior = match s {
            "streaming" => LoadBehavior::Streaming,
            "delayed" => LoadBehavior::Delayed,
            "blocking" => LoadBehavior::Blocking,
            _ => return Err(ParseEnumError),
        };
        Ok(behavior)
    }
}

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

#[derive(Debug, Error)]
pub enum Error {
    #[error("Load cancelled")]
    Cancelled,

    #[error("HTTP Status is not OK: {0} status: {1} redirected: {2} length: {3}")]
    HttpNotOk(String, u16, bool, u64),

    /// The domain could not be resolved, either because it is invalid or a DNS error occurred
    #[error("Domain resolution failure: {0}")]
    InvalidDomain(String),

    #[error("Blocked host: {0}")]
    BlockedHost(String),

    #[error("Invalid SWF: {0}")]
    InvalidSwf(#[from] swf::error::Error),

    #[error("Invalid bitmap")]
    InvalidBitmap(#[from] ruffle_render::error::Error),

    #[error("Invalid sound: {0}")]
    InvalidSound(#[from] crate::backend::audio::DecodeError),

    #[error("Unexpected content of type {1}, expected {0}")]
    UnexpectedData(ContentType, ContentType),

    #[error("Could not fetch: {0:?}")]
    FetchError(String),

    // TODO: We can't support lifetimes on this error object yet (or we'll need some backends inside
    // the GC arena). We're losing info here. How do we fix that?
    #[error("Error running avm1 script: {0}")]
    Avm1Error(String),

    #[error("System I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Cannot parse integer value: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
}

impl From<crate::avm1::Error<'_>> for Error {
    fn from(error: crate::avm1::Error<'_>) -> Self {
        Error::Avm1Error(error.to_string())
    }
}

/// Holds all in-progress loads for the player.
pub struct LoadManager<'gc>(SlotMap<LoaderHandle, MovieLoader<'gc>>);

unsafe impl<'gc> Collect<'gc> for LoadManager<'gc> {
    fn trace<C: Trace<'gc>>(&self, cc: &mut C) {
        for (_, loader) in self.0.iter() {
            cc.trace(loader);
        }
    }
}

impl<'gc> LoadManager<'gc> {
    /// Construct a new `LoadManager`.
    pub fn new() -> Self {
        Self(SlotMap::with_key())
    }

    /// Add a new loader to the `LoadManager`.
    ///
    /// Returns the loader handle for later inspection. A loader handle is
    /// valid for as long as the load operation.
    ///
    /// After the load finishes, the loader should be removed (and the handle
    /// invalidated). This can be done with remove_loader.
    /// Movie loaders are removed automatically after the loader status is set
    /// accordingly.
    pub fn add_loader(&mut self, loader: MovieLoader<'gc>) -> LoaderHandle {
        let handle = self.0.insert(loader);
        let loader = self.get_loader_mut(handle).unwrap();
        loader.self_handle = Some(handle);
        handle
    }

    /// Remove a completed loader.
    /// This is used to remove a loader after the loading or unloading process has completed.
    pub fn remove_loader(&mut self, handle: LoaderHandle) {
        self.0.remove(handle);
    }

    /// Retrieve a loader by handle.
    pub fn get_loader(&self, handle: LoaderHandle) -> Option<&MovieLoader<'gc>> {
        self.0.get(handle)
    }

    /// Retrieve a loader by handle for mutation.
    pub fn get_loader_mut(&mut self, handle: LoaderHandle) -> Option<&mut MovieLoader<'gc>> {
        self.0.get_mut(handle)
    }

    /// Kick off a movie clip load.
    ///
    /// Returns the loader's async process, which you will need to spawn.
    pub fn load_movie_into_clip(
        &mut self,
        player: Arc<Mutex<Player>>,
        target_clip: DisplayObject<'gc>,
        request: Request,
        loader_url: Option<String>,
        vm_data: MovieLoaderVMData<'gc>,
    ) -> OwnedFuture<(), Error> {
        // When an AVM2 movie loads an AVM1 movie, that AVM1 movie cannot load
        // another movie over itself, as in loadMovie(..., _root). Attempts to
        // do so will be silently ignored.
        //
        // However, if the AVM1 movie uses MovieClipLoader.loadClip to load into
        // its _root, FP32 will segfault. We don't reproduce that behavior.
        if matches!(vm_data, MovieLoaderVMData::Avm1 { .. }) {
            // This check works because the only time AVM1 can access an MC with
            // `loader_info` set is when that MC is the root MC of an AVM1 movie
            // that was loaded by AVM2
            if target_clip
                .as_movie_clip()
                .and_then(|mc| mc.loader_info())
                .is_some()
            {
                // Return a future that does nothing
                return Box::pin(async move { Ok(()) });
            }
        }

        let loader = MovieLoader {
            self_handle: None,
            target_clip,
            vm_data,
            loader_status: LoaderStatus::Pending,
            from_bytes: false,
            movie: None,
        };
        let handle = self.add_loader(loader);
        let loader = self.get_loader_mut(handle).unwrap();
        loader.movie_loader(player, request, loader_url)
    }

    pub fn load_asset_movie(
        uc: &UpdateContext<'gc>,
        request: Request,
        importer_movie: MovieClip<'gc>,
    ) -> OwnedFuture<(), Error> {
        let player = uc.player_handle();
        let importer_movie = MovieClipHandle::stash(uc, importer_movie);

        Box::pin(async move {
            let fetch = player.lock().unwrap().fetch(request, FetchReason::LoadSwf);

            match wait_for_full_response(fetch).await {
                Ok((body, url, _status, _redirected)) => {
                    let content_type = ContentType::sniff(&body);
                    tracing::info!("Loading imported movie: {:?}", url);
                    match content_type {
                        ContentType::Swf => {
                            let movie = SwfMovie::from_data(&body, url.clone(), Some(url.clone()))
                                .expect("Could not load movie");

                            let movie = Arc::new(movie);

                            player.lock().unwrap().mutate_with_update_context(|uc| {
                                let importer_movie = importer_movie.fetch(uc);
                                let clip = MovieClip::new_import_assets(uc, movie, importer_movie);

                                clip.set_cur_preload_frame(0);
                                let mut execution_limit = ExecutionLimit::none();

                                tracing::debug!("Preloading swf to run exports {:?}", url);

                                // Create library for exports before preloading
                                uc.library.library_for_movie_mut(clip.movie());
                                let res = clip.preload(uc, &mut execution_limit);
                                tracing::debug!(
                                    "Preloaded swf to run exports result {:?} {}",
                                    url,
                                    res
                                );
                                importer_movie.finish_importing();
                            });
                            Ok(())
                        }
                        _ => {
                            tracing::warn!(
                                "Unsupported content type for ImportAssets: {:?}",
                                content_type
                            );
                            Ok(())
                        }
                    }
                }
                Err(e) => Err(Error::FetchError(format!(
                    "Could not fetch: {:?} because {:?}",
                    e.url, e.error
                ))),
            }
        })
    }

    /// Kick off a movie clip load.
    ///
    /// Returns the loader's async process, which you will need to spawn.
    pub fn load_movie_into_clip_bytes(
        context: &mut UpdateContext<'gc>,
        target_clip: DisplayObject<'gc>,
        bytes: Vec<u8>,
        vm_data: MovieLoaderVMData<'gc>,
    ) -> Result<(), Error> {
        let loader = MovieLoader {
            self_handle: None,
            target_clip,
            vm_data,
            loader_status: LoaderStatus::Pending,
            movie: None,
            from_bytes: true,
        };
        let handle = context.load_manager.add_loader(loader);
        MovieLoader::movie_loader_bytes(handle, context, bytes)
    }

    /// Fires the `onLoad` listener event for every MovieClip that has been
    /// initialized (ran its first frame).
    ///
    /// This also removes all movie loaders that have completed.
    pub fn movie_clip_on_load(
        &mut self,
        queue: &mut ActionQueue<'gc>,
        strings: &StringContext<'gc>,
    ) {
        // FIXME: This relies on the iteration order of the slotmap, which
        // is not defined. The container should be replaced with something
        // that preserves insertion order, such as `LinkedHashMap` -
        // unfortunately that doesn't provide automatic key generation.
        let mut loaders: Vec<_> = self.0.keys().collect();
        // `SlotMap` doesn't provide reverse iteration, so reversing afterwards.
        loaders.reverse();

        // Removing the keys from `loaders` whose movie hasn't loaded yet.
        loaders.retain(|handle| {
            self.0
                .get_mut(*handle)
                .expect("valid key")
                .movie_clip_loaded(queue, strings)
        });

        // Cleaning up the loaders that are done.
        for index in loaders {
            self.0.remove(index);
        }
    }

    /// Process tags on all loaders in the Parsing phase.
    ///
    /// Returns true if *all* loaders finished preloading.
    pub fn preload_tick(context: &mut UpdateContext<'gc>, limit: &mut ExecutionLimit) -> bool {
        let mut did_finish = true;
        let handles: Vec<_> = context.load_manager.0.iter().map(|(h, _)| h).collect();

        for handle in handles {
            let status = match context.load_manager.get_loader(handle) {
                Some(MovieLoader { loader_status, .. }) => Some(loader_status),
                _ => None,
            };

            if matches!(status, Some(LoaderStatus::Parsing)) {
                match MovieLoader::preload_tick(handle, context, limit, 0, false) {
                    Ok(f) => did_finish = did_finish && f,
                    Err(e) => tracing::error!("Error encountered while preloading movie: {}", e),
                }
            }
        }

        did_finish
    }

    pub fn run_exit_frame(context: &mut UpdateContext<'gc>) {
        // The root movie might not have come from a loader, so check it separately.
        // `fire_init_and_complete_events` is idempotent, so we unconditionally call it here
        if let Some(movie) = context
            .stage
            .child_by_index(0)
            .and_then(|o| o.as_movie_clip())
        {
            movie.try_fire_loaderinfo_events(context);
        }
        let handles: Vec<_> = context.load_manager.0.iter().map(|(h, _)| h).collect();
        for handle in handles {
            if let Some(MovieLoader { target_clip, .. }) = context.load_manager.get_loader(handle)
                && let Some(movie) = target_clip.as_movie_clip()
                && movie.try_fire_loaderinfo_events(context)
            {
                context.load_manager.remove_loader(handle)
            }
        }
    }
}

impl Default for LoadManager<'_> {
    fn default() -> Self {
        Self::new()
    }
}

async fn wait_for_full_response(
    response: OwnedFuture<Box<dyn SuccessResponse>, ErrorResponse>,
) -> Result<(Vec<u8>, String, u16, bool), ErrorResponse> {
    let response = response.await?;
    let url = response.url().to_string();
    let status = response.status();
    let redirected = response.redirected();
    let body = response.body().await;

    match body {
        Ok(body) => Ok((body, url, status, redirected)),
        Err(error) => Err(ErrorResponse { url, error }),
    }
}

/// The completion status of a `Loader` loading a movie.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

#[derive(Collect, Clone, Copy)]
#[collect(no_drop)]
pub enum MovieLoaderVMData<'gc> {
    Avm1 {
        broadcaster: Option<Object<'gc>>,
    },
    Avm2 {
        loader_info: LoaderInfoObject<'gc>,

        /// The context of the SWF being loaded.
        context: Option<Avm2Object<'gc>>,

        /// The default domain this SWF will use.
        default_domain: Avm2Domain<'gc>,
    },
}

/// A struct that holds the state for asynchronous movie loads.
#[derive(Collect)]
#[collect(no_drop)]
pub struct MovieLoader<'gc> {
    /// The handle to refer to this loader instance.
    #[collect(require_static)]
    self_handle: Option<LoaderHandle>,

    /// The target movie clip to load the movie into.
    target_clip: DisplayObject<'gc>,

    // Virtual-machine specific data (AVM1 or AVM2)
    vm_data: MovieLoaderVMData<'gc>,

    /// Indicates the completion status of this loader.
    ///
    /// This flag exists to prevent a situation in which loading a movie
    /// into a clip that has not yet fired its Load event causes the
    /// loader to be prematurely removed. This flag is only set when either
    /// the movie has been replaced (and thus Load events can be trusted)
    /// or an error has occurred (in which case we don't care about the
    /// loader anymore).
    #[collect(require_static)]
    loader_status: LoaderStatus,

    /// The SWF being loaded.
    ///
    /// This is only available if the asynchronous loader path has
    /// completed and we expect the Player to periodically tick preload
    /// until loading completes.
    movie: Option<Arc<SwfMovie>>,

    /// Whether or not this was loaded as a result of a `Loader.loadBytes` call
    from_bytes: bool,
}

impl<'gc> MovieLoader<'gc> {
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
        handle: LoaderHandle,
        context: &mut UpdateContext<'gc>,
        limit: &mut ExecutionLimit,
        status: u16,
        redirected: bool,
    ) -> Result<bool, Error> {
        let mc = match context.load_manager.get_loader(handle) {
            Some(Self {
                target_clip,
                movie,
                from_bytes,
                ..
            }) => {
                if movie.is_none() {
                    //Non-SWF load or file not loaded yet
                    return Ok(false);
                }

                // Loader.loadBytes movies never participate in preloading
                if *from_bytes {
                    return Ok(true);
                }

                if target_clip.as_movie_clip().is_none() {
                    // Non-movie-clip loads should not be handled in preload_tick
                    tracing::error!("Cannot preload non-movie-clip loader");
                    return Ok(false);
                }

                *target_clip
            }
            None => return Err(Error::Cancelled),
        };

        let mc = mc.as_movie_clip().unwrap();

        let did_finish = mc.preload(context, limit);

        MovieLoader::movie_loader_progress(
            handle,
            context,
            mc.compressed_loaded_bytes() as usize,
            mc.compressed_total_bytes() as usize,
        )?;

        if did_finish {
            MovieLoader::movie_loader_complete(
                handle,
                context,
                Some(mc.into()),
                status,
                redirected,
            )?;
        }

        Ok(did_finish)
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
        player: Arc<Mutex<Player>>,
        request: Request,
        loader_url: Option<String>,
    ) -> OwnedFuture<(), Error> {
        let handle = self.self_handle.expect("Loader not self-introduced");

        Box::pin(async move {
            let request_url = request.url().to_string();
            let resolved_url = player.lock().unwrap().navigator().resolve_url(&request_url);

            let fetch = player.lock().unwrap().fetch(request, FetchReason::LoadSwf);

            let mut replacing_root_movie = false;
            player.lock().unwrap().update(|uc| -> Result<(), Error> {
                let clip = match uc.load_manager.get_loader(handle) {
                    Some(MovieLoader { target_clip, .. }) => *target_clip,
                    None => return Err(Error::Cancelled),
                };

                replacing_root_movie = uc
                    .stage
                    .root_clip()
                    .map(|root| DisplayObject::ptr_eq(clip, root))
                    .unwrap_or(false);

                if let Some(mut mc) = clip.as_movie_clip() {
                    if !mc.movie().is_action_script_3() {
                        mc.avm1_unload(uc);

                        // Clear deletable properties on the target before loading
                        // Properties written during the subsequent onLoad events will persist
                        if let Some(clip_object) = mc.object1() {
                            let mut activation = Activation::from_nothing(
                                uc,
                                ActivationIdentifier::root("unknown"),
                                clip,
                            );

                            for key in clip_object.get_keys(&mut activation, true) {
                                clip_object.delete(&mut activation, key);
                            }
                        }
                    }

                    // Before the actual SWF is loaded, an initial loading state is entered.
                    MovieLoader::load_initial_loading_swf(&mut mc, uc, &request_url, resolved_url);
                }

                MovieLoader::movie_loader_start(handle, uc)
            })?;

            match wait_for_full_response(fetch).await {
                Ok((body, url, _status, _redirected)) if replacing_root_movie => {
                    ContentType::sniff(&body).expect(ContentType::Swf)?;

                    let movie = SwfMovie::from_data(&body, url, loader_url)?;
                    player.lock().unwrap().mutate_with_update_context(|uc| {
                        // Make a copy of the properties on the root, so we can put them back after replacing it
                        let mut root_properties: IndexMap<AvmString, Value> = IndexMap::new();
                        if let Some(root) = uc.stage.root_clip()
                            && let Some(root_object) = root.object1()
                        {
                            let mut activation = Activation::from_nothing(
                                uc,
                                ActivationIdentifier::root("unknown"),
                                root,
                            );
                            for key in root_object.get_keys(&mut activation, true) {
                                let val = root_object
                                    .get_stored(key, &mut activation)
                                    .unwrap_or(Value::Undefined);
                                root_properties.insert(key, val);
                            }
                        }

                        uc.replace_root_movie(movie);

                        // Add the copied properties back onto the new root
                        if !root_properties.is_empty()
                            && let Some(root) = uc.stage.root_clip()
                            && let Some(clip_object) = root.object1()
                        {
                            let mut activation = Activation::from_nothing(
                                uc,
                                ActivationIdentifier::root("unknown"),
                                root,
                            );
                            for (key, val) in root_properties {
                                let _ = clip_object.set(key, val, &mut activation);
                            }
                        }
                    });
                    return Ok(());
                }
                Ok((body, url, status, redirected)) => {
                    player.lock().unwrap().mutate_with_update_context(|uc| {
                        MovieLoader::movie_loader_data(
                            handle,
                            uc,
                            &body,
                            url.to_string(),
                            status,
                            redirected,
                            loader_url,
                        )
                    })?;
                }
                Err(response) => {
                    tracing::error!(
                        "Error during movie loading of {:?}: {:?}",
                        response.url,
                        response.error
                    );
                    player.lock().unwrap().update(|uc| -> Result<(), Error> {
                        // FIXME - match Flash's error message

                        let (status_code, redirected) =
                            if let Error::HttpNotOk(_, status_code, redirected, _) = response.error
                            {
                                (status_code, redirected)
                            } else {
                                (0, false)
                            };
                        MovieLoader::movie_loader_error(
                            handle,
                            uc,
                            "Movie loader error",
                            status_code,
                            redirected,
                            response.url,
                        )
                    })?;
                }
            }

            Ok(())
        })
    }

    pub fn movie_loader_bytes(
        handle: LoaderHandle,
        uc: &mut UpdateContext<'gc>,
        bytes: Vec<u8>,
    ) -> Result<(), Error> {
        let clip = match uc.load_manager.get_loader(handle) {
            Some(Self { target_clip, .. }) => *target_clip,
            None => return Err(Error::Cancelled),
        };

        let replacing_root_movie = uc
            .stage
            .root_clip()
            .map(|root| DisplayObject::ptr_eq(clip, root))
            .unwrap_or(false);

        if let Some(mc) = clip.as_movie_clip() {
            if !mc.movie().is_action_script_3() {
                mc.avm1_unload(uc);
            }
            mc.replace_with_movie(uc, None, false, None);
        }

        let loader_url = Some(uc.root_swf.url().to_string());

        if replacing_root_movie {
            ContentType::sniff(&bytes).expect(ContentType::Swf)?;

            let movie = SwfMovie::from_data(&bytes, "file:///".into(), loader_url)?;
            avm2_stub_method_context!(
                uc,
                "flash.display.Loader",
                "loadBytes",
                "replacing root movie"
            );
            uc.replace_root_movie(movie);
            return Ok(());
        }

        MovieLoader::movie_loader_data(handle, uc, &bytes, "file:///".into(), 0, false, loader_url)
    }
}

/// Kick off the root movie load.
///
/// The root movie is special because it determines a few bits of player
/// state, such as the size of the stage and the current frame rate. Ergo,
/// this method should only be called once, by the player that is trying to
/// kick off its root movie load.
#[must_use]
pub fn load_root_movie<'gc>(
    uc: &UpdateContext<'gc>,
    request: Request,
    parameters: Vec<(String, String)>,
    on_metadata: Box<dyn FnOnce(&swf::HeaderExt)>,
) -> OwnedFuture<(), Error> {
    let player = uc.player_handle();

    Box::pin(async move {
        let fetch = player.lock().unwrap().fetch(request, FetchReason::LoadSwf);
        let response = fetch.await.map_err(|error| {
            player
                .lock()
                .unwrap()
                .ui()
                .display_root_movie_download_failed_message(false, error.error.to_string());
            error.error
        })?;
        let swf_url = response.url().into_owned();
        let body = response.body().await.inspect_err(|error| {
            player
                .lock()
                .unwrap()
                .ui()
                .display_root_movie_download_failed_message(true, error.to_string());
        })?;

        // The spoofed root movie URL takes precedence over the actual URL.
        let spoofed_or_swf_url = player
            .lock()
            .unwrap()
            .spoofed_url()
            .map(|u| u.to_string())
            .unwrap_or(swf_url);

        let mut movie =
            SwfMovie::from_data(&body, spoofed_or_swf_url, None).inspect_err(|error| {
                player
                    .lock()
                    .unwrap()
                    .ui()
                    .display_root_movie_download_failed_message(true, error.to_string());
            })?;
        on_metadata(movie.header());
        movie.append_parameters(parameters);
        player.lock().unwrap().mutate_with_update_context(|uc| {
            uc.set_root_movie(movie);
        });
        Ok(())
    })
}

/// Kick off a form data load into an AVM1 object.
///
/// Returns the loader's async process, which you will need to spawn.
#[must_use]
pub fn load_form_into_object<'gc>(
    uc: &UpdateContext<'gc>,
    target_object: Object<'gc>,
    request: Request,
) -> OwnedFuture<(), Error> {
    let player = uc.player_handle();
    let target_object = ObjectHandle::stash(uc, target_object);

    Box::pin(async move {
        let fetch = player.lock().unwrap().fetch(request, FetchReason::Other);

        let response = fetch.await.map_err(|e| e.error)?;
        let response_encoding = response.text_encoding();
        let body = response.body().await?;

        // Fire the load handler.
        player.lock().unwrap().update(|uc| {
            let that = target_object.fetch(uc);

            let mut activation =
                Activation::from_stub(uc, ActivationIdentifier::root("[Form Loader]"));

            let utf8_string;
            let utf8_body = if activation.context.system.use_codepage {
                // Determine the encoding
                let encoding = if let Some(encoding) = response_encoding {
                    encoding
                } else {
                    let mut encoding_detector = EncodingDetector::new();
                    encoding_detector.feed(&body, true);
                    encoding_detector.guess(None, true)
                };

                // Convert the text into UTF-8
                utf8_string = encoding.decode(&body).0;
                utf8_string.as_bytes()
            } else if activation.context.root_swf.version() <= 5 {
                utf8_string = WINDOWS_1252.decode(&body).0;
                utf8_string.as_bytes()
            } else {
                &body
            };

            for (k, v) in form_urlencoded::parse(utf8_body) {
                let k = AvmString::new_utf8(activation.gc(), k);
                let v = AvmString::new_utf8(activation.gc(), v);
                that.set(k, v.into(), &mut activation)?;
            }

            // Fire the onData method and event.
            if let Some(display_object) = that.as_display_object()
                && let Some(movie_clip) = display_object.as_movie_clip()
            {
                activation.context.action_queue.queue_action(
                    movie_clip.into(),
                    ActionType::Method {
                        object: that,
                        name: istr!("onData"),
                        args: vec![],
                    },
                    false,
                );
                movie_clip.event_dispatch(activation.context, ClipEvent::Data);
            }

            Ok(())
        })
    })
}

/// Kick off a form data load into an `LoadVars` AVM1 object.
///
/// Returns the loader's async process, which you will need to spawn.
#[must_use]
pub fn load_form_into_load_vars<'gc>(
    uc: &UpdateContext<'gc>,
    target_object: Object<'gc>,
    request: Request,
) -> OwnedFuture<(), Error> {
    let player = uc.player_handle();
    let target_object = ObjectHandle::stash(uc, target_object);

    Box::pin(async move {
        let fetch = player.lock().unwrap().fetch(request, FetchReason::Other);
        let response = wait_for_full_response(fetch).await;

        // Fire the load handler.
        player.lock().unwrap().update(|uc| {
            let that = target_object.fetch(uc);

            let mut activation = Activation::from_stub(uc, ActivationIdentifier::root("[Loader]"));

            match response {
                Ok((body, _, status, _)) => {
                    let length = body.len();

                    // Set the properties used by the getBytesTotal and getBytesLoaded methods.
                    that.set(istr!("_bytesTotal"), length.into(), &mut activation)?;
                    if length > 0 {
                        that.set(istr!("_bytesLoaded"), length.into(), &mut activation)?;
                    }

                    let _ = that.call_method(
                        istr!("onHTTPStatus"),
                        &[status.into()],
                        &mut activation,
                        ExecutionReason::Special,
                    );

                    // Fire the onData method with the loaded string.
                    // If the loaded data is an empty string, the load is considered unsuccessful.
                    let value_data = if length == 0 {
                        Value::Undefined
                    } else {
                        AvmString::new_utf8(activation.gc(), UTF_8.decode(&body).0).into()
                    };
                    let _ = that.call_method(
                        istr!("onData"),
                        &[value_data],
                        &mut activation,
                        ExecutionReason::Special,
                    );
                }
                Err(response) => {
                    // TODO: Log "Error opening URL" trace similar to the Flash Player?

                    let status_code = if let Error::HttpNotOk(_, status_code, _, _) = response.error
                    {
                        status_code
                    } else {
                        0
                    };

                    let _ = that.call_method(
                        istr!("onHTTPStatus"),
                        &[status_code.into()],
                        &mut activation,
                        ExecutionReason::Special,
                    );

                    // Fire the onData method with no data to indicate an unsuccessful load.
                    let _ = that.call_method(
                        istr!("onData"),
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

/// Kick off an AVM1 StyleSheet load.
///
/// Returns the loader's async process, which you will need to spawn.
pub fn load_stylesheet<'gc>(
    uc: &UpdateContext<'gc>,
    target_object: Object<'gc>,
    request: Request,
) -> OwnedFuture<(), Error> {
    let player = uc.player_handle();
    let target_object = ObjectHandle::stash(uc, target_object);

    Box::pin(async move {
        let fetch = player.lock().unwrap().fetch(request, FetchReason::Other);
        let response = wait_for_full_response(fetch).await;

        // Fire the load handler.
        player.lock().unwrap().update(|uc| {
            let that = target_object.fetch(uc);

            let mut activation = Activation::from_stub(uc, ActivationIdentifier::root("[Loader]"));

            match response {
                Ok((body, _, _, _)) => {
                    // Fire the parse & onLoad methods with the loaded string.
                    let css = AvmString::new_utf8(activation.gc(), UTF_8.decode(&body).0);
                    let success = that
                        .call_method(
                            istr!("parse"),
                            &[css.into()],
                            &mut activation,
                            ExecutionReason::Special,
                        )
                        .unwrap_or(Value::Bool(false));
                    let _ = that.call_method(
                        istr!("onLoad"),
                        &[success],
                        &mut activation,
                        ExecutionReason::Special,
                    );
                }
                Err(_) => {
                    // TODO: Log "Error opening URL" trace similar to the Flash Player?

                    let _ = that.call_method(
                        istr!("onLoad"),
                        &[Value::Bool(false)],
                        &mut activation,
                        ExecutionReason::Special,
                    );
                }
            }

            Ok(())
        })
    })
}

/// Kick off a data load into a `URLLoader`, updating
/// its `data` property when the load completes.
///
/// Returns the loader's async process, which you will need to spawn.
#[must_use]
pub fn load_data_into_url_loader<'gc>(
    uc: &UpdateContext<'gc>,
    target: Avm2ScriptObject<'gc>,
    request: Request,
) -> OwnedFuture<(), Error> {
    let player = uc.player_handle();
    let target = Avm2ScriptObjectHandle::stash(uc, target);

    Box::pin(async move {
        let fetch = player
            .lock()
            .unwrap()
            .fetch(request, FetchReason::UrlLoader);
        let response = wait_for_full_response(fetch).await;

        player.lock().unwrap().update(|uc| {
            let target = Avm2Object::from(target.fetch(uc));

            let mut activation = Avm2Activation::from_nothing(uc);

            fn set_data<'a, 'gc: 'a>(
                body: Vec<u8>,
                activation: &mut Avm2Activation<'a, 'gc>,
                target: Avm2Object<'gc>,
            ) {
                use crate::avm2::globals::slots::flash_net_url_loader as url_loader_slots;

                let body_len = body.len().into();

                // TODO - update these as the download progresses
                target
                    .set_slot(url_loader_slots::BYTES_LOADED, body_len, activation)
                    .unwrap();
                target
                    .set_slot(url_loader_slots::BYTES_TOTAL, body_len, activation)
                    .unwrap();

                let data_format = target
                    .get_slot(url_loader_slots::DATA_FORMAT)
                    .coerce_to_string(activation)
                    .expect("The dataFormat field is typed String");

                let data_object = if &data_format == b"binary" {
                    let storage = ByteArrayStorage::from_vec(activation.context, body);
                    let bytearray = ByteArrayObject::from_storage(activation.context, storage);

                    Some(bytearray.into())
                } else if &data_format == b"variables" {
                    if body.is_empty() {
                        None
                    } else {
                        let string_value = strip_bom(activation, &body);

                        activation
                            .avm2()
                            .classes()
                            .urlvariables
                            .construct(activation, &[string_value.into()])
                            .ok()
                    }
                } else {
                    if &data_format != b"text" {
                        tracing::warn!("Invalid URLLoaderDataFormat: {}", data_format);
                    }

                    Some(strip_bom(activation, &body).into())
                };

                if let Some(data_object) = data_object {
                    target
                        .set_slot(url_loader_slots::DATA, data_object, activation)
                        .unwrap();
                }
            }

            match response {
                Ok((body, _, status, redirected)) => {
                    let total_len = body.len();

                    // FIXME - the "open" event should be fired earlier, just before
                    // we start to fetch the data.
                    // However, the "open" event should not be fired if an IO error
                    // occurs opening the connection (e.g. if a file does not exist on disk).
                    // We currently have no way of detecting this, so we settle for firing
                    // the event after the entire fetch is complete. This causes there
                    // to a longer delay between the initial load triggered by the script
                    // and the "load" event firing, but it ensures that we match
                    // the Flash behavior w.r.t when an event is fired vs not fired.
                    let open_evt = Avm2EventObject::bare_default_event(activation.context, "open");
                    Avm2::dispatch_event(activation.context, open_evt, target);
                    set_data(body, &mut activation, target);

                    // FIXME - we should fire "progress" events as we receive data, not
                    // just at the end
                    let progress_evt = Avm2EventObject::progress_event(
                        &mut activation,
                        "progress",
                        total_len,
                        total_len,
                    );

                    Avm2::dispatch_event(activation.context, progress_evt, target);

                    let http_status_evt =
                        Avm2EventObject::http_status_event(&mut activation, status, redirected);

                    Avm2::dispatch_event(activation.context, http_status_evt, target);

                    let complete_evt =
                        Avm2EventObject::bare_default_event(activation.context, "complete");
                    Avm2::dispatch_event(uc, complete_evt, target);
                }
                Err(response) => {
                    tracing::error!(
                        "Error during URLLoader load of {:?}: {:?}",
                        response.url,
                        response.error
                    );

                    // Testing with Flash shoes that the 'data' property is cleared
                    // when an error occurs

                    set_data(Vec::new(), &mut activation, target);

                    let (status_code, redirected) =
                        if let Error::HttpNotOk(_, status_code, redirected, _) = response.error {
                            (status_code, redirected)
                        } else {
                            (0, false)
                        };
                    let http_status_evt = Avm2EventObject::http_status_event(
                        &mut activation,
                        status_code,
                        redirected,
                    );

                    Avm2::dispatch_event(activation.context, http_status_evt, target);

                    // FIXME - Match the exact error message generated by Flash
                    let io_error_evt = Avm2EventObject::io_error_event(
                        &mut activation,
                        "Error #2032: Stream Error",
                        2032,
                    );

                    Avm2::dispatch_event(uc, io_error_evt, target);
                }
            }

            Ok(())
        })
    })
}

/// Kick off an AVM1 audio load.
///
/// Returns the loader's async process, which you will need to spawn.
#[must_use]
pub fn load_sound_avm1<'gc>(
    uc: &UpdateContext<'gc>,
    sound_object: Object<'gc>,
    request: Request,
    is_streaming: bool,
) -> OwnedFuture<(), Error> {
    let player = uc.player_handle();
    let sound_object = ObjectHandle::stash(uc, sound_object);

    Box::pin(async move {
        let fetch = player.lock().unwrap().fetch(request, FetchReason::Other);
        let response = wait_for_full_response(fetch).await;

        // Fire the load handler.
        player.lock().unwrap().update(|uc| {
            let sound_object = sound_object.fetch(uc);

            let NativeObject::Sound(sound) = sound_object.native() else {
                panic!("NativeObject must be Sound");
            };

            let mut activation = Activation::from_stub(uc, ActivationIdentifier::root("[Loader]"));

            let success = response
                .map_err(|e| e.error)
                .and_then(|(body, _, _, _)| {
                    let handle = activation.context.audio.register_mp3(&body)?;
                    sound.load_sound(&mut activation, sound_object, handle);
                    sound.set_duration(Some(0));
                    sound.load_id3(&mut activation, sound_object, &body)?;
                    let duration = activation
                        .context
                        .audio
                        .get_sound_duration(handle)
                        .map(|d| d.round() as u32);
                    sound.set_duration(duration);
                    Ok(())
                })
                .is_ok();

            let _ = sound_object.call_method(
                istr!("onLoad"),
                &[success.into()],
                &mut activation,
                ExecutionReason::Special,
            );

            // Streaming sounds should auto-play.
            if is_streaming {
                crate::avm1::start_sound(&mut activation, sound_object, &[])?;
            }

            Ok(())
        })
    })
}

/// Kick off an AVM2 audio load.
///
/// Returns the loader's async process, which you will need to spawn.
#[must_use]
pub fn load_sound_avm2<'gc>(
    uc: &UpdateContext<'gc>,
    sound: SoundObject<'gc>,
    request: Request,
) -> OwnedFuture<(), Error> {
    let player = uc.player_handle();
    let sound = SoundObjectHandle::stash(uc, sound);

    Box::pin(async move {
        let fetch = player.lock().unwrap().fetch(request, FetchReason::Other);
        let response = wait_for_full_response(fetch).await;

        player.lock().unwrap().update(|uc| {
            let sound = sound.fetch(uc);
            let sound_object = Avm2Object::from(sound);

            if sound.loading_state() == SoundLoadingState::Loaded {
                // Sound has already been loaded.
                return Ok(());
            }

            match response {
                Ok((body, _, _, _)) => {
                    let handle = uc.audio.register_mp3(&body)?;
                    sound.set_sound(uc, handle);

                    let total_len = body.len();

                    // FIXME - the "open" event should be fired earlier, and not fired in case of ioerror.
                    let mut activation = Avm2Activation::from_nothing(uc);
                    let open_evt = Avm2EventObject::bare_default_event(activation.context, "open");
                    Avm2::dispatch_event(activation.context, open_evt, sound_object);

                    // FIXME - As in load_url_loader, we should fire "progress" events as we receive data,
                    // not just at the end
                    let progress_evt = Avm2EventObject::progress_event(
                        &mut activation,
                        "progress",
                        total_len,
                        total_len,
                    );

                    Avm2::dispatch_event(activation.context, progress_evt, sound_object);

                    sound.read_and_call_id3_event(&mut activation, body.as_slice());

                    let complete_evt =
                        Avm2EventObject::bare_default_event(activation.context, "complete");
                    Avm2::dispatch_event(activation.context, complete_evt, sound_object);
                }
                Err(_err) => {
                    let mut activation = Avm2Activation::from_nothing(uc);

                    // FIXME: Match the exact error message generated by Flash.
                    let io_error_evt = Avm2EventObject::io_error_event(
                        &mut activation,
                        "Error #2032: Stream Error",
                        2032,
                    );

                    Avm2::dispatch_event(uc, io_error_evt, sound_object);
                }
            }

            Ok(())
        })
    })
}

/// Buffer video or audio into a NetStream.
#[must_use]
pub fn load_netstream<'gc>(
    uc: &UpdateContext<'gc>,
    stream: NetStream<'gc>,
    request: Request,
) -> OwnedFuture<(), Error> {
    let player = uc.player_handle();
    let stream = NetStreamHandle::stash(uc, stream);

    Box::pin(async move {
        let fetch = player.lock().unwrap().fetch(request, FetchReason::Other);
        match fetch.await {
            Ok(mut response) => {
                let expected_length = response.expected_length();

                player.lock().unwrap().update(|uc| -> Result<(), Error> {
                    let stream = stream.fetch(uc);

                    stream.reset_buffer(uc);
                    if let Ok(Some(len)) = expected_length {
                        stream.set_expected_length(len as usize);
                    }

                    Ok(())
                })?;

                loop {
                    let chunk = response.next_chunk().await;
                    let is_end = matches!(chunk, Ok(None));
                    player.lock().unwrap().update(|uc| -> Result<(), Error> {
                        let stream = stream.fetch(uc);

                        match chunk {
                            Ok(Some(mut data)) => stream.load_buffer(uc, &mut data),
                            Ok(None) => stream.finish_buffer(),
                            Err(err) => stream.report_error(err),
                        }
                        Ok(())
                    })?;

                    if is_end {
                        break;
                    }
                }

                Ok(())
            }
            Err(response) => player.lock().unwrap().update(|uc| {
                let stream = stream.fetch(uc);

                stream.report_error(response.error);
                Ok(())
            }),
        }
    })
}

impl<'gc> MovieLoader<'gc> {
    /// Report a movie loader start event to script code.
    fn movie_loader_start(handle: LoaderHandle, uc: &mut UpdateContext<'gc>) -> Result<(), Error> {
        let (clip, vm_data) = match uc.load_manager.get_loader(handle) {
            Some(Self {
                target_clip,
                vm_data,
                ..
            }) => (*target_clip, *vm_data),
            None => return Err(Error::Cancelled),
        };

        match vm_data {
            MovieLoaderVMData::Avm1 { broadcaster } => {
                if let Some(broadcaster) = broadcaster {
                    Avm1::run_stack_frame_for_method(
                        clip,
                        broadcaster,
                        istr!(uc, "broadcastMessage"),
                        &[istr!(uc, "onLoadStart").into(), clip.object1_or_undef()],
                        uc,
                    );
                }
            }
            MovieLoaderVMData::Avm2 { loader_info, .. } => {
                let activation = Avm2Activation::from_nothing(uc);

                let open_evt = Avm2EventObject::bare_default_event(activation.context, "open");
                Avm2::dispatch_event(uc, open_evt, loader_info.into());
            }
        }

        Ok(())
    }

    /// Load data into a movie loader.
    fn movie_loader_data(
        handle: LoaderHandle,
        uc: &mut UpdateContext<'gc>,
        data: &[u8],
        url: String,
        status: u16,
        redirected: bool,
        loader_url: Option<String>,
    ) -> Result<(), Error> {
        let sniffed_type = ContentType::sniff(data);
        let length = data.len();

        if sniffed_type == ContentType::Unknown
            && let Ok(data) = extract_swz(data)
        {
            return Self::movie_loader_data(handle, uc, &data, url, status, redirected, loader_url);
        }
        let (clip, vm_data, from_bytes) = match uc.load_manager.get_loader(handle) {
            Some(Self {
                target_clip,
                vm_data,
                from_bytes,
                ..
            }) => (*target_clip, *vm_data, *from_bytes),
            None => return Err(Error::Cancelled),
        };

        let domain = if let MovieLoaderVMData::Avm2 {
            context,
            default_domain,
            ..
        } = vm_data
        {
            use crate::avm2::globals::slots::flash_system_loader_context as loader_context_slots;

            context
                .map(|o| o.get_slot(loader_context_slots::APPLICATION_DOMAIN))
                .and_then(|v| v.as_object())
                .and_then(|o| o.as_application_domain())
                .unwrap_or_else(|| {
                    let parent_domain = default_domain;
                    Avm2Domain::movie_domain(uc, parent_domain)
                })
        } else {
            // This is necessary when the MovieLoaderData is AVM1,
            // but loaded an AVM2 SWF (mixed AVM).
            uc.avm2.stage_domain()
        };

        let movie = match sniffed_type {
            ContentType::Swf => {
                Arc::new(SwfMovie::from_data(data, url.clone(), loader_url.clone())?)
            }
            ContentType::Gif | ContentType::Jpeg | ContentType::Png => {
                Arc::new(SwfMovie::from_loaded_image(url.clone(), length))
            }
            ContentType::Unknown => Arc::new(SwfMovie::error_movie(url.clone())),
        };

        match uc.load_manager.get_loader_mut(handle) {
            Some(Self {
                movie: old,
                loader_status,
                ..
            }) => {
                *loader_status = LoaderStatus::Parsing;
                *old = Some(movie.clone())
            }
            _ => unreachable!(),
        };

        if let MovieLoaderVMData::Avm2 { loader_info, .. } = vm_data {
            loader_info.set_content_type(sniffed_type);
            let fake_movie = Arc::new(SwfMovie::fake_with_compressed_len(
                uc.root_swf.version(),
                loader_url.clone(),
                data.len(),
            ));

            // Expose 'bytesTotal' (via the fake movie) during the first 'progress' event,
            // but nothing else (in particular, the `parameters` and `url` properties are not set
            // to their real values)
            loader_info.set_loader_stream(
                LoaderStream::NotYetLoaded(fake_movie, Some(clip), false),
                uc.gc(),
            );

            // Flash always fires an initial 'progress' event with
            // bytesLoaded=0 and bytesTotal set to the proper value.
            // This only seems to happen for an AVM2 event handler
            MovieLoader::movie_loader_progress(handle, uc, 0, length)?;

            // Update the LoaderStream - we now have a real SWF movie and a real target clip
            // This is intentionally set *after* the first 'progress' event, to match Flash's behavior
            // (`LoaderInfo.parameters` is always empty during the first 'progress' event)
            loader_info.set_loader_stream(
                LoaderStream::NotYetLoaded(movie.clone(), Some(clip), false),
                uc.gc(),
            );
        }

        match sniffed_type {
            ContentType::Swf => {
                let library = uc.library.library_for_movie_mut(movie.clone());

                library.set_avm2_domain(domain);

                if let Some(mc) = clip.as_movie_clip() {
                    let loader_info = if let MovieLoaderVMData::Avm2 { loader_info, .. } = vm_data {
                        Some(loader_info)
                    } else {
                        None
                    };

                    // Store our downloaded `SwfMovie` into our target `MovieClip`,
                    // and initialize it.

                    mc.replace_with_movie(uc, Some(movie.clone()), true, loader_info);

                    // Update the MovieClip's script object prototype to match the new movie's version.
                    // This is needed because the level clip may have been created by a loader with
                    // a different SWF version, and its script object was set up with the wrong prototype.
                    if !movie.is_action_script_3()
                        && let Some(object) = mc.object1()
                    {
                        let new_proto = uc.avm1.prototypes(mc.swf_version()).movie_clip;

                        object.define_value(
                            uc.gc(),
                            istr!(uc, "__proto__"),
                            new_proto.into(),
                            Attribute::DONT_ENUM | Attribute::DONT_DELETE,
                        );
                    }

                    // Loaded movies are considered to be timeline-instantiated
                    mc.set_instantiated_by_timeline(true);

                    if matches!(vm_data, MovieLoaderVMData::Avm2 { .. })
                        && !movie.is_action_script_3()
                    {
                        // When an AVM2 movie loads an AVM1 movie, we need to call `post_instantiation` here.
                        mc.post_instantiation(uc, None, Instantiator::Movie, false);

                        mc.set_depth(LOADER_INSERTED_AVM1_DEPTH);

                        // We also need to create its AVM2-side object, the `AVM1Movie`.
                        mc.set_avm1movie(uc);
                    }

                    if from_bytes {
                        mc.preload(uc, &mut ExecutionLimit::none());
                        MovieLoader::movie_loader_progress(
                            handle,
                            uc,
                            mc.compressed_loaded_bytes() as usize,
                            mc.compressed_total_bytes() as usize,
                        )?;
                        uc.post_frame_callbacks.push(PostFrameCallback {
                            callback: Box::new(move |uc, dobj: DisplayObject<'_>| {
                                if let Err(e) =
                                    MovieLoader::movie_loader_complete(handle, uc, Some(dobj), 0, false)
                                {
                                    tracing::error!("Error finishing loading of Loader.loadBytes movie {dobj:?}: {e:?}");
                                }
                            }),
                            data: clip,
                        });
                    }
                }

                // NOTE: Certain tests specifically expect small files to preload immediately
                if !from_bytes {
                    MovieLoader::preload_tick(
                        handle,
                        uc,
                        &mut ExecutionLimit::with_max_ops_and_time(10000, Duration::from_millis(1)),
                        status,
                        redirected,
                    )?;
                };

                return Ok(());
            }
            ContentType::Gif | ContentType::Jpeg | ContentType::Png => {
                let mut activation = Avm2Activation::from_nothing(uc);

                let library = activation.context.library.library_for_movie_mut(movie);

                library.set_avm2_domain(domain);

                // This will construct AVM2-side objects even under AVM1, but it doesn't matter,
                // since Bitmap and BitmapData never have AVM1-side objects.
                let bitmap = ruffle_render::utils::decode_define_bits_jpeg(data, None)?;

                let transparency = true;
                let bitmapdata = BitmapData::new_with_pixels(
                    activation.gc(),
                    bitmap.width(),
                    bitmap.height(),
                    transparency,
                    bitmap.as_colors().map(Color::from).collect(),
                );

                let bitmapdata_avm2 =
                    BitmapDataObject::from_bitmap_data(activation.context, bitmapdata);

                let bitmap_avm2 = activation
                    .avm2()
                    .classes()
                    .bitmap
                    .construct(&mut activation, &[bitmapdata_avm2.into()])
                    .unwrap()
                    .as_object()
                    .unwrap();

                let bitmap_dobj = bitmap_avm2.as_display_object().unwrap();

                if let MovieLoaderVMData::Avm2 { loader_info, .. } = vm_data {
                    let fake_movie = Arc::new(SwfMovie::fake_with_compressed_len(
                        activation.context.root_swf.version(),
                        loader_url.clone(),
                        data.len(),
                    ));

                    loader_info.set_loader_stream(
                        LoaderStream::NotYetLoaded(fake_movie, Some(bitmap_dobj), false),
                        activation.gc(),
                    );
                }

                MovieLoader::movie_loader_progress(handle, activation.context, length, length)?;

                if let MovieLoaderVMData::Avm2 { loader_info, .. } = vm_data {
                    let fake_movie = Arc::new(SwfMovie::fake_with_compressed_data(
                        activation.context.root_swf.version(),
                        loader_url,
                        data.to_vec(),
                    ));

                    loader_info.set_loader_stream(
                        LoaderStream::NotYetLoaded(fake_movie, Some(bitmap_dobj), false),
                        activation.gc(),
                    );
                }

                if from_bytes {
                    // Note - flash player seems to delay this for *two* frames for some reason
                    uc.post_frame_callbacks.push(PostFrameCallback {
                        callback: Box::new(move |uc, bitmap_obj| {
                            uc.post_frame_callbacks.push(PostFrameCallback {
                                callback: Box::new(move |uc, bitmap_obj| {
                                    if let Err(e) = MovieLoader::movie_loader_complete(
                                        handle,
                                        uc,
                                        Some(bitmap_obj),
                                        status,
                                        redirected,
                                    ) {
                                        tracing::error!("Error finishing loading of Loader.loadBytes image {bitmap_obj:?}: {e:?}");
                                    }
                                }),
                                data: bitmap_obj,
                            })
                        }),
                        data: bitmap_dobj,
                    });
                } else {
                    MovieLoader::movie_loader_complete(
                        handle,
                        activation.context,
                        Some(bitmap_dobj),
                        status,
                        redirected,
                    )?;
                }
            }
            ContentType::Unknown => {
                match vm_data {
                    MovieLoaderVMData::Avm1 { .. } => {
                        // If the file is no valid supported file, the MovieClip enters the error state
                        if let Some(mut mc) = clip.as_movie_clip() {
                            MovieLoader::load_error_swf(&mut mc, uc, url);
                        }

                        // AVM1 fires the event with the current and total length as 0
                        MovieLoader::movie_loader_progress(handle, uc, 0, 0)?;
                        MovieLoader::movie_loader_complete(handle, uc, None, status, redirected)?;
                    }
                    MovieLoaderVMData::Avm2 { loader_info, .. } => {
                        let fake_movie = Arc::new(SwfMovie::fake_with_compressed_len(
                            uc.root_swf.version(),
                            loader_url,
                            data.len(),
                        ));

                        loader_info.set_errored(true);

                        loader_info.set_loader_stream(
                            LoaderStream::NotYetLoaded(fake_movie, None, false),
                            uc.gc(),
                        );

                        MovieLoader::movie_loader_progress(handle, uc, length, length)?;
                        let mut error = "Error #2124: Loaded file is an unknown type.".to_string();
                        if !from_bytes {
                            error += &format!(" URL: {url}");
                        }

                        MovieLoader::movie_loader_error(
                            handle, uc, &error, status, redirected, url,
                        )?;
                    }
                }
            }
        }

        //TODO: content sniffing errors need to be reported somehow
        Ok(())
    }

    /// Report a movie loader progress event to script code.
    ///
    /// The current and total length are always reported as compressed lengths.
    fn movie_loader_progress(
        handle: LoaderHandle,
        uc: &mut UpdateContext<'gc>,
        cur_len: usize,
        total_len: usize,
    ) -> Result<(), Error> {
        let (target_clip, vm_data) = match uc.load_manager.get_loader(handle) {
            Some(Self {
                target_clip,
                vm_data,
                ..
            }) => (*target_clip, *vm_data),
            None => return Err(Error::Cancelled),
        };

        match vm_data {
            MovieLoaderVMData::Avm1 { broadcaster } => {
                if let Some(broadcaster) = broadcaster {
                    Avm1::run_stack_frame_for_method(
                        target_clip,
                        broadcaster,
                        istr!(uc, "broadcastMessage"),
                        &[
                            istr!(uc, "onLoadProgress").into(),
                            target_clip.object1_or_undef(),
                            cur_len.into(),
                            total_len.into(),
                        ],
                        uc,
                    );
                }
            }
            MovieLoaderVMData::Avm2 { loader_info, .. } => {
                let mut activation = Avm2Activation::from_nothing(uc);

                let progress_evt = Avm2EventObject::progress_event(
                    &mut activation,
                    "progress",
                    cur_len,
                    total_len,
                );

                Avm2::dispatch_event(uc, progress_evt, loader_info.into());
            }
        }

        Ok(())
    }

    /// Report a movie loader completion to script code.
    fn movie_loader_complete(
        handle: LoaderHandle,
        uc: &mut UpdateContext<'gc>,
        dobj: Option<DisplayObject<'gc>>,
        status: u16,
        redirected: bool,
    ) -> Result<(), Error> {
        let (target_clip, vm_data, movie) = match uc.load_manager.get_loader(handle) {
            Some(Self {
                target_clip,
                movie,
                vm_data,
                ..
            }) => (*target_clip, *vm_data, movie.clone()),
            None => return Err(Error::Cancelled),
        };

        let loader_info = if let MovieLoaderVMData::Avm2 { loader_info, .. } = vm_data {
            Some(loader_info)
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
                LoaderStream::NotYetLoaded(movie.clone().unwrap(), Some(dobj.unwrap()), false),
                uc.gc(),
            );
        }

        if let Some(mc) = dobj.and_then(|dobj| dobj.as_movie_clip()) {
            // We call these methods after we initialize the `LoaderInfo`, but before
            // we add the loaded clip as a child. The frame constructor should see
            // 'this.parent == null' and 'this.stage == null'
            mc.post_instantiation(uc, None, Instantiator::Movie, false);
            catchup_display_object_to_frame(uc, mc.into());
            // Movie clips created from ActionScript (including from a Loader) skip the next enterFrame,
            // and consequently are observed to have their currentFrame lag one
            // frame behind objects placed by the timeline (even if they were
            // both placed in the same frame to begin with).
            mc.base().set_skip_next_enter_frame(true);

            let flashvars = movie.as_ref().unwrap().parameters();
            if let Some(object) = mc.object1() {
                for (key, value) in flashvars {
                    object.define_value(
                        uc.gc(),
                        AvmString::new_utf8(uc.gc(), key),
                        AvmString::new_utf8(uc.gc(), value).into(),
                        Attribute::empty(),
                    );
                }
            }
        }

        if let MovieLoaderVMData::Avm2 { loader_info, .. } = vm_data {
            let mut loader = loader_info
                .loader()
                .expect("Loader should be Some")
                .as_display_object()
                .unwrap()
                .as_container()
                .unwrap();

            // This isn't completely correct - the 'large_preload' test observes the child
            // being set after an 'enterFrame' call. However, our current logic should
            // hopefully be good enough.
            avm2_stub_method_context!(
                uc,
                "flash.display.Loader",
                "load",
                "addChild at the correct time"
            );

            loader_info.set_expose_content();

            // Note that we do *not* use the 'addChild' method here:
            // Per the flash docs, our implementation always throws
            // an 'unsupported' error. Also, the AVM2 side of our movie
            // clip does not yet exist. Any children added inside the movie
            // frame constructor will see an 'added' event immediately, and
            // an 'addedToStage' event *after* the constructor finishes
            // when we add the movie as a child of the loader.
            loader.insert_at_index(uc, dobj.unwrap(), 0);

            if !movie.unwrap().is_action_script_3() {
                loader.insert_child_into_depth_list(uc, LOADER_INSERTED_AVM1_DEPTH, dobj.unwrap());
            }
        } else if let Some(dobj) = dobj {
            // This is a load of an image into AVM1 - add it as a child of the target clip.
            if dobj.as_movie_clip().is_none() {
                let mc = target_clip.as_movie_clip().unwrap();
                mc.replace_with_movie(uc, Some(movie.unwrap()), true, None);
                mc.replace_at_depth(uc, dobj, 1);

                // This sets the MovieClip image state correctly.
                mc.set_current_frame(1);
                mc.set_cur_preload_frame(2);
            }
        }

        match vm_data {
            MovieLoaderVMData::Avm1 { broadcaster } => {
                if let Some(broadcaster) = broadcaster {
                    Avm1::run_stack_frame_for_method(
                        target_clip,
                        broadcaster,
                        istr!(uc, "broadcastMessage"),
                        // TODO: Pass an actual httpStatus argument instead of 0.
                        &[
                            istr!(uc, "onLoadComplete").into(),
                            target_clip.object1_or_undef(),
                            status.into(),
                        ],
                        uc,
                    );
                }
            }
            // This is fired after we process the movie's first frame,
            // in `MovieClip.on_exit_frame`
            MovieLoaderVMData::Avm2 { loader_info, .. } => {
                let current_movie = { loader_info.loader_stream().movie().clone() };
                loader_info
                    .set_loader_stream(LoaderStream::Swf(current_movie, dobj.unwrap()), uc.gc());

                if let Some(dobj) = dobj
                    && dobj.as_movie_clip().is_none()
                {
                    loader_info.fire_init_and_complete_events(uc, status, redirected);
                }
            }
        }

        let loader = uc.load_manager.get_loader_mut(handle).unwrap();
        loader.loader_status = LoaderStatus::Succeeded;

        Ok(())
    }

    /// Report a movie loader error to script code.
    ///
    /// This is an associated function because we cannot borrow both the update
    /// context and one of it's loaders.
    fn movie_loader_error(
        handle: LoaderHandle,
        uc: &mut UpdateContext<'gc>,
        msg: &str,
        status: u16,
        redirected: bool,
        swf_url: String,
    ) -> Result<(), Error> {
        //TODO: Inspect the fetch error.
        //This requires cooperation from the backend to send abstract
        //error types we can actually inspect.
        //This also can get errors from decoding an invalid SWF file,
        //too. We should distinguish those to player code.
        let (clip, vm_data) = match uc.load_manager.get_loader(handle) {
            Some(Self {
                target_clip,
                vm_data,
                ..
            }) => (*target_clip, *vm_data),
            None => return Err(Error::Cancelled),
        };

        // If the SWF can't be loaded, the MovieClip enters the error state
        if let Some(mut mc) = clip.as_movie_clip() {
            MovieLoader::load_error_swf(&mut mc, uc, swf_url);
        }

        match vm_data {
            MovieLoaderVMData::Avm1 { broadcaster } => {
                if let Some(broadcaster) = broadcaster {
                    let error_message = AvmString::new_ascii_static(uc.gc(), b"LoadNeverCompleted");

                    Avm1::run_stack_frame_for_method(
                        clip,
                        broadcaster,
                        istr!(uc, "broadcastMessage"),
                        &[
                            istr!(uc, "onLoadError").into(),
                            clip.object1_or_undef(),
                            error_message.into(),
                        ],
                        uc,
                    );
                }
            }
            MovieLoaderVMData::Avm2 { loader_info, .. } => {
                let mut activation = Avm2Activation::from_nothing(uc);

                let http_status_evt =
                    Avm2EventObject::http_status_event(&mut activation, status, redirected);

                Avm2::dispatch_event(activation.context, http_status_evt, loader_info.into());

                // FIXME - Match the exact error message and code generated by Flash
                let io_error_evt = Avm2EventObject::io_error_event(&mut activation, msg, 0);

                Avm2::dispatch_event(uc, io_error_evt, loader_info.into());
            }
        }

        let loader = uc.load_manager.get_loader_mut(handle).unwrap();
        loader.loader_status = LoaderStatus::Failed;

        Ok(())
    }

    /// This makes the MovieClip enter the initial loading state in which some
    /// attributes have certain initial loading values to signal that the file is
    /// currently being loaded and neither an error has occurred nor the first frame
    /// has been successfully loaded yet.
    fn load_initial_loading_swf(
        mc: &mut MovieClip<'gc>,
        uc: &mut UpdateContext<'gc>,
        request_url: &str,
        resolved_url: Result<Url, ParseError>,
    ) {
        match resolved_url {
            Err(_) => {
                MovieLoader::load_error_swf(mc, uc, request_url.to_string());
            }
            Ok(url) => {
                // If the loaded SWF is a local file, the initial loading state equals the error state.
                if url.scheme() == "file" {
                    MovieLoader::load_error_swf(mc, uc, url.to_string());
                } else {
                    // Replacing the movie sets total_frames and frames_loaded correctly.
                    // The movie just needs to be the default empty movie with the correct URL.
                    // In this loading state, the URL is the URL of the parent movie / doesn't change.

                    let current_movie = mc.movie();
                    let current_version = current_movie.version();
                    let current_url = current_movie.url();
                    let mut initial_loading_movie = SwfMovie::empty(current_version, None);
                    initial_loading_movie.set_url(current_url.to_string());

                    mc.replace_with_movie(uc, Some(Arc::new(initial_loading_movie)), true, None);

                    // Maybe this (keeping the current URL) should be the default behaviour
                    // of replace_with_movie?
                    // TODO: See where it gets invoked without a movie as well and what the
                    // correct URL result is in these cases.
                }
            }
        }
    }

    /// This makes the MovieClip enter the error state in which some attributes have
    /// certain error values to signal that no valid file could be loaded.
    /// An error state movie stub which provides the correct values is created and
    /// loaded.
    ///
    /// This happens if no file could be loaded or if the loaded content is no valid
    /// supported content.
    ///
    /// swf_url is always the final URL obtained after any redirects.
    fn load_error_swf(mc: &mut MovieClip<'gc>, uc: &mut UpdateContext<'gc>, mut swf_url: String) {
        // If a local URL is fetched using the flash plugin, the _url property
        // won't be changed => It keeps being the parent SWF URL.
        if cfg!(target_family = "wasm")
            && let Ok(url) = Url::parse(&swf_url)
            && url.scheme() == "file"
        {
            swf_url = mc.movie().url().to_string();
        };

        let error_movie = SwfMovie::error_movie(swf_url);
        // This also sets total_frames correctly
        mc.replace_with_movie(uc, Some(Arc::new(error_movie)), true, None);
        mc.set_cur_preload_frame(0);
    }

    /// Event handler morally equivalent to `onLoad` on a movie clip.
    ///
    /// Returns `true` if the loader has completed and should be removed.
    ///
    /// Used to fire listener events on clips and terminate completed loaders.
    fn movie_clip_loaded(
        &mut self,
        queue: &mut ActionQueue<'gc>,
        strings: &StringContext<'gc>,
    ) -> bool {
        match self.loader_status {
            LoaderStatus::Pending => false,
            LoaderStatus::Parsing => false,
            LoaderStatus::Failed => true,
            LoaderStatus::Succeeded => {
                // AVM2 is handled separately
                if let MovieLoaderVMData::Avm1 {
                    broadcaster: Some(broadcaster),
                } = self.vm_data
                {
                    queue.queue_action(
                        self.target_clip,
                        ActionType::Method {
                            object: broadcaster,
                            name: istr!(strings, "broadcastMessage"),
                            args: vec![
                                istr!(strings, "onLoadInit").into(),
                                self.target_clip.object1_or_undef(),
                            ],
                        },
                        false,
                    );
                }
                // If the movie was loaded from avm1, clean it up now. If a movie (including an AVM1 movie)
                // was loaded from avm2, clean it up in `run_exit_frame`, after we have a chance to fire
                // the AVM2-side events
                matches!(self.vm_data, MovieLoaderVMData::Avm1 { .. })
            }
        }
    }
}

/// Display a dialog allowing a user to select a file from an AVM1 scope.
///
/// Returns a future that will be resolved when a file is selected.
#[must_use]
pub fn select_file_dialog_avm1<'gc>(
    uc: &UpdateContext<'gc>,
    target_object: Object<'gc>,
    dialog: DialogResultFuture,
) -> OwnedFuture<(), Error> {
    let handle = ObjectHandle::stash(uc, target_object);
    select_file_dialog(uc, FileReferenceHandle::Avm1(handle), dialog)
}

/// Display a dialog allowing a user to select a file from an AVM2 scope.
///
/// Returns a future that will be resolved when a file is selected.
#[must_use]
pub fn select_file_dialog_avm2<'gc>(
    uc: &UpdateContext<'gc>,
    target_object: FileReferenceObject<'gc>,
    dialog: DialogResultFuture,
) -> OwnedFuture<(), Error> {
    let handle = FileReferenceObjectHandle::stash(uc, target_object);
    select_file_dialog(uc, FileReferenceHandle::Avm2(handle), dialog)
}

enum FileReferenceHandle {
    Avm1(ObjectHandle),
    Avm2(FileReferenceObjectHandle),
}

fn select_file_dialog<'gc>(
    uc: &UpdateContext<'gc>,
    target_object: FileReferenceHandle,
    dialog: DialogResultFuture,
) -> OwnedFuture<(), Error> {
    let player = uc.player_handle();

    Box::pin(async move {
        let dialog_result = dialog.await;

        // Dialog is done, allow opening new dialogs
        player.lock().unwrap().ui_mut().close_file_dialog();

        // Fire the load handler.
        player.lock().unwrap().update(|uc| -> Result<(), Error> {
            match target_object {
                FileReferenceHandle::Avm1(target_object) => {
                    let target_object = target_object.fetch(uc);

                    let file_ref = match target_object.native() {
                        NativeObject::FileReference(fr) => fr,
                        _ => panic!("NativeObject must be FileReference"),
                    };

                    let mut activation =
                        Activation::from_stub(uc, ActivationIdentifier::root("[File Dialog]"));

                    match dialog_result {
                        Ok(dialog_result) => {
                            use crate::avm1::globals::as_broadcaster;

                            if !dialog_result.is_cancelled() {
                                file_ref.init_from_dialog_result(
                                    &mut activation,
                                    dialog_result.borrow(),
                                );
                                as_broadcaster::broadcast_internal(
                                    target_object,
                                    &[target_object.into()],
                                    istr!("onSelect"),
                                    &mut activation,
                                )?;
                            } else {
                                as_broadcaster::broadcast_internal(
                                    target_object,
                                    &[target_object.into()],
                                    istr!("onCancel"),
                                    &mut activation,
                                )?;
                            }
                        }
                        Err(err) => {
                            tracing::warn!("Error on file dialog: {:?}", err);
                        }
                    }
                    Ok(())
                }
                FileReferenceHandle::Avm2(target_object) => {
                    let target_object = target_object.fetch(uc);
                    match dialog_result {
                        Ok(dialog_result) => {
                            if !dialog_result.is_cancelled() {
                                target_object.init_from_dialog_result(dialog_result);

                                let select_event =
                                    Avm2EventObject::bare_default_event(uc, "select");
                                Avm2::dispatch_event(uc, select_event, target_object.into());
                            } else {
                                let cancel_event =
                                    Avm2EventObject::bare_default_event(uc, "cancel");
                                Avm2::dispatch_event(uc, cancel_event, target_object.into());
                            }
                        }
                        Err(err) => {
                            tracing::warn!("Error on file dialog: {:?}", err);
                        }
                    }

                    Ok(())
                }
            }
        })
    })
}

/// Display a dialog allowing a user to save a file to disk from an AVM2 scope.
#[must_use]
pub fn save_file_dialog<'gc>(
    uc: &UpdateContext<'gc>,
    target_object: FileReferenceObject<'gc>,
    dialog: DialogResultFuture,
    data: Vec<u8>,
) -> OwnedFuture<(), Error> {
    let player = uc.player_handle();
    let target_object = FileReferenceObjectHandle::stash(uc, target_object);
    Box::pin(async move {
        let dialog_result = dialog.await;

        // Dialog is done, allow opening new dialogs
        player.lock().unwrap().ui_mut().close_file_dialog();

        // Fire the load handler.
        player.lock().unwrap().update(|uc| -> Result<(), Error> {
            let target_object = target_object.fetch(uc);

            match dialog_result {
                Ok(mut dialog_result) => {
                    if !dialog_result.is_cancelled() {
                        dialog_result.write_and_refresh(&data);
                        target_object.init_from_dialog_result(dialog_result);

                        let mut activation = Avm2Activation::from_nothing(uc);

                        let select_event =
                            Avm2EventObject::bare_default_event(activation.context, "select");
                        Avm2::dispatch_event(
                            activation.context,
                            select_event,
                            target_object.into(),
                        );

                        let open_event =
                            Avm2EventObject::bare_default_event(activation.context, "open");
                        Avm2::dispatch_event(activation.context, open_event, target_object.into());

                        let progress_evt = Avm2EventObject::progress_event(
                            &mut activation,
                            "progress",
                            data.len(),
                            data.len(),
                        );
                        Avm2::dispatch_event(
                            activation.context,
                            progress_evt,
                            target_object.into(),
                        );

                        let complete_event =
                            Avm2EventObject::bare_default_event(activation.context, "complete");
                        Avm2::dispatch_event(
                            activation.context,
                            complete_event,
                            target_object.into(),
                        );
                    } else {
                        let cancel_event = Avm2EventObject::bare_default_event(uc, "cancel");
                        Avm2::dispatch_event(uc, cancel_event, target_object.into());
                    }
                }
                Err(err) => {
                    tracing::warn!("Save dialog had an error {:?}", err);
                }
            }

            Ok(())
        })
    })
}

/// Display a dialog allowing a user to download a file.
///
/// Fetches the data from `url`, saves the data to the selected destination and processes callbacks
/// by calling methods on the provided AVM1 `FileReference` object.
///
/// Returns a future that will be resolved when a file is selected and the download has completed.
#[must_use]
pub fn download_file_dialog<'gc>(
    uc: &UpdateContext<'gc>,
    target_object: Object<'gc>,
    dialog: DialogResultFuture,
    url: String,
) -> OwnedFuture<(), Error> {
    let player = uc.player_handle();
    let target_object = ObjectHandle::stash(uc, target_object);

    Box::pin(async move {
        let dialog_result = dialog.await;

        // Dialog is done, allow opening new dialogs
        player.lock().unwrap().ui_mut().close_file_dialog();

        // Download the data
        let req = Request::get(url.clone());
        // Doing this in two steps to prevent holding the player lock during fetch
        let future = player.lock().unwrap().fetch(req, FetchReason::Other);
        let download_res = wait_for_full_response(future).await;

        // Fire the load handler.
        player.lock().unwrap().update(|uc| -> Result<(), Error> {
            let target_object = target_object.fetch(uc);

            let file_ref = match target_object.native() {
                NativeObject::FileReference(fr) => fr,
                _ => panic!("NativeObject must be FileReference"),
            };

            let mut activation =
                Activation::from_stub(uc, ActivationIdentifier::root("[File Dialog]"));
            use crate::avm1::globals::as_broadcaster;

            match dialog_result {
                Ok(mut dialog_result) => {
                    if !dialog_result.is_cancelled() {
                        // onSelect and onOpen should be called before the download begins
                        // We simulate this by using the initial dialog result
                        file_ref.init_from_dialog_result(&mut activation, dialog_result.borrow());

                        as_broadcaster::broadcast_internal(
                            target_object,
                            &[target_object.into()],
                            istr!("onSelect"),
                            &mut activation,
                        )?;

                        match download_res {
                            Ok((body, _, _, _)) => {
                                as_broadcaster::broadcast_internal(
                                    target_object,
                                    &[target_object.into()],
                                    istr!("onOpen"),
                                    &mut activation,
                                )?;

                                // onProgress and onComplete expect to receive the current state
                                // of the file, as we simulate an instant 100% download from the
                                // perspective of AS, we want to refresh the file_ref internal data
                                // before invoking the callbacks

                                dialog_result.write_and_refresh(&body);
                                file_ref.init_from_dialog_result(
                                    &mut activation,
                                    dialog_result.borrow(),
                                );

                                let total_bytes = body.len();

                                as_broadcaster::broadcast_internal(
                                    target_object,
                                    &[target_object.into(), total_bytes.into(), total_bytes.into()],
                                    istr!("onProgress"),
                                    &mut activation,
                                )?;

                                as_broadcaster::broadcast_internal(
                                    target_object,
                                    &[target_object.into()],
                                    istr!("onComplete"),
                                    &mut activation,
                                )?;
                            }
                            Err(err) => {
                                match err.error {
                                    Error::InvalidDomain(_) => {
                                        activation
                                            .context
                                            .avm_trace(&format!("Error opening URL '{url}'"));

                                        as_broadcaster::broadcast_internal(
                                            target_object,
                                            &[target_object.into()],
                                            istr!("onIOError"),
                                            &mut activation,
                                        )?;
                                    }
                                    Error::HttpNotOk(_, _, _, body_len) => {
                                        // If the error happens before the connection is
                                        // established, then don't invoke onOpen
                                        as_broadcaster::broadcast_internal(
                                            target_object,
                                            &[target_object.into()],
                                            istr!("onOpen"),
                                            &mut activation,
                                        )?;

                                        activation
                                            .context
                                            .avm_trace(&format!("Error opening URL '{url}'"));

                                        as_broadcaster::broadcast_internal(
                                            target_object,
                                            &[target_object.into()],
                                            istr!("onIOError"),
                                            &mut activation,
                                        )?;

                                        // Flash still executes the onProgress callback, even after an error
                                        // However it should only be called if the error occurred after the connection was established
                                        as_broadcaster::broadcast_internal(
                                            target_object,
                                            &[
                                                target_object.into(),
                                                body_len.into(),
                                                body_len.into(),
                                            ],
                                            istr!("onProgress"),
                                            &mut activation,
                                        )?;
                                    }
                                    Error::FetchError(_) => {
                                        // If the error happens before the connection is
                                        // established, then don't invoke onOpen
                                        as_broadcaster::broadcast_internal(
                                            target_object,
                                            &[target_object.into()],
                                            istr!("onOpen"),
                                            &mut activation,
                                        )?;

                                        activation
                                            .context
                                            .avm_trace(&format!("Error opening URL '{url}'"));

                                        as_broadcaster::broadcast_internal(
                                            target_object,
                                            &[target_object.into()],
                                            istr!("onIOError"),
                                            &mut activation,
                                        )?;
                                    }
                                    _ => {
                                        tracing::warn!(
                                            "Unhandled non-fetch error on download: {:?}",
                                            err.error
                                        );
                                    }
                                }
                            }
                        }
                    } else {
                        as_broadcaster::broadcast_internal(
                            target_object,
                            &[target_object.into()],
                            istr!("onCancel"),
                            &mut activation,
                        )?;
                    }
                }
                Err(err) => {
                    tracing::warn!("Download dialog had an error {:?}", err);
                }
            }

            Ok(())
        })
    })
}

/// Uploads the given `data` to the provided `url`.
/// `file_name` is sent along with the data, as part of the multipart/form-data body.
///
/// `target_object` is the AVM1 `FileReference` object which initialized the upload.
///
/// Returns a future that will be resolved when the file upload has completed.
#[must_use]
pub fn upload_file<'gc>(
    uc: &UpdateContext<'gc>,
    target_object: Object<'gc>,
    url: String,
    data: Vec<u8>,
    file_name: String,
) -> OwnedFuture<(), Error> {
    let player = uc.player_handle();
    let target_object = ObjectHandle::stash(uc, target_object);

    Box::pin(async move {
        let total_size_bytes = data.len();

        //FIXME: The code below won't work if the payload contains the boundary separator
        if file_name.contains("------------BOUNDARY")
            || data.windows(20).any(|b| b == b"------------BOUNDARY")
        {
            tracing::error!("File upload data contains boundary separator, request cannot be sent");
            return Err(Error::Cancelled);
        }

        // Format the data into multipart/form-data
        let mut out_data = Vec::new();
        out_data.extend_from_slice(b"------------BOUNDARY\n");
        out_data.extend_from_slice(b"Content-Disposition: form-data; name=\"Filename\"\n\n");
        out_data.extend_from_slice(file_name.as_bytes());
        out_data.extend_from_slice(b"\n------------BOUNDARY\n");
        out_data
            .extend_from_slice(b"Content-Disposition: form-data; name=\"Filedata\"; filename=\"");
        out_data.extend_from_slice(file_name.as_bytes());
        out_data.extend_from_slice(b"\"\n");
        out_data.extend_from_slice(b"Content-Type: application/octet-stream\n\n");
        out_data.extend_from_slice(&data);
        out_data.extend_from_slice(b"\n------------BOUNDARY\n");
        out_data.extend_from_slice(b"Content-Disposition: form-data; name=\"Upload\"\n\n");
        out_data.extend_from_slice(b"Submit Query");
        out_data.extend_from_slice(b"\n------------BOUNDARY\n");

        // Upload the data
        let req = Request::post(
            url,
            Some((
                out_data,
                "multipart/form-data; boundary=------------BOUNDARY".to_string(),
            )),
        );
        // Doing this in two steps to prevent holding the player lock during fetch
        let future = player.lock().unwrap().fetch(req, FetchReason::Other);
        let result = future.await;

        // Fire the load handler.
        player.lock().unwrap().update(|uc| -> Result<(), Error> {
            let target_object = target_object.fetch(uc);

            let mut activation =
                Activation::from_stub(uc, ActivationIdentifier::root("[File Dialog]"));

            use crate::avm1::globals::as_broadcaster;
            as_broadcaster::broadcast_internal(
                target_object,
                &[target_object.into()],
                istr!("onOpen"),
                &mut activation,
            )?;

            match result {
                Ok(_) => {
                    as_broadcaster::broadcast_internal(
                        target_object,
                        &[
                            target_object.into(),
                            total_size_bytes.into(),
                            total_size_bytes.into(),
                        ],
                        istr!("onProgress"),
                        &mut activation,
                    )?;

                    as_broadcaster::broadcast_internal(
                        target_object,
                        &[target_object.into()],
                        istr!("onComplete"),
                        &mut activation,
                    )?;
                }
                Err(err) => {
                    // If the error was due to the domain not existing, then this should call
                    // onIoError only
                    // If the error was instead due to the server returning a non successful response code,
                    // this should call onProgress with the size of the error response body,
                    // then should call onHTTPError

                    match err.error {
                        Error::InvalidDomain(_) => {
                            as_broadcaster::broadcast_internal(
                                target_object,
                                &[target_object.into()],
                                istr!("onIOError"),
                                &mut activation,
                            )?;
                        }
                        Error::HttpNotOk(_, _, _, _) => {
                            as_broadcaster::broadcast_internal(
                                target_object,
                                &[
                                    target_object.into(),
                                    total_size_bytes.into(),
                                    total_size_bytes.into(),
                                ],
                                istr!("onProgress"),
                                &mut activation,
                            )?;

                            as_broadcaster::broadcast_internal(
                                target_object,
                                &[target_object.into()],
                                istr!("onHTTPError"),
                                &mut activation,
                            )?;
                        }
                        Error::FetchError(msg) => {
                            tracing::warn!("Unhandled fetch error: {:?}", msg);
                            // For now we will just handle this like a dns error
                            as_broadcaster::broadcast_internal(
                                target_object,
                                &[target_object.into()],
                                istr!("onIOError"),
                                &mut activation,
                            )?;
                        }
                        _ => {
                            // We got something other than a FetchError from calling fetch, this should be unlikely
                            tracing::warn!("Unhandled non-fetch error on upload: {:?}", err.error);
                        }
                    }
                }
            }

            Ok(())
        })
    })
}
