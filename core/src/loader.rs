//! Management of async loaders

use crate::avm1::activation::{Activation, ActivationIdentifier};
use crate::avm1::{Avm1, Object, TObject, Value};
use crate::avm2::{Activation as Avm2Activation, Domain as Avm2Domain};
use crate::backend::navigator::{OwnedFuture, RequestOptions};
use crate::context::{ActionQueue, ActionType};
use crate::display_object::{DisplayObject, MorphShape, TDisplayObject};
use crate::player::{Player, NEWEST_PLAYER_VERSION};
use crate::string::AvmString;
use crate::tag_utils::SwfMovie;
use crate::vminterface::Instantiator;
use encoding_rs::UTF_8;
use gc_arena::{Collect, CollectionContext};
use generational_arena::{Arena, Index};
use std::sync::{Arc, Mutex, Weak};
use thiserror::Error;
use url::form_urlencoded;

pub type Handle = Index;

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

    #[error("Could not fetch: {0}")]
    FetchError(String),

    #[error("Invalid SWF")]
    InvalidSwf(#[from] crate::tag_utils::Error),

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
            | Loader::LoadVars { self_handle, .. } => *self_handle = Some(handle),
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
        fetch: OwnedFuture<Vec<u8>, Error>,
    ) -> OwnedFuture<(), Error> {
        let loader = Loader::Form {
            self_handle: None,
            target_object,
        };
        let handle = self.add_loader(loader);
        let loader = self.get_loader_mut(handle).unwrap();

        loader.form_loader(player, fetch)
    }

    /// Kick off a form data load into an AVM1 object.
    ///
    /// Returns the loader's async process, which you will need to spawn.
    pub fn load_form_into_load_vars(
        &mut self,
        player: Weak<Mutex<Player>>,
        target_object: Object<'gc>,
        fetch: OwnedFuture<Vec<u8>, Error>,
    ) -> OwnedFuture<(), Error> {
        let loader = Loader::LoadVars {
            self_handle: None,
            target_object,
        };
        let handle = self.add_loader(loader);
        let loader = self.get_loader_mut(handle).unwrap();

        loader.load_vars_loader(player, fetch)
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
            // clippy reports a false positive for explicitly dropped guards:
            // https://github.com/rust-lang/rust-clippy/issues/6446
            // A workaround for this is to wrap the `.lock()` call in a block instead of explicitly dropping the guard.
            let fetch;
            let url = {
                let player_lock = player.lock().unwrap();
                let url = player_lock.navigator().resolve_relative_url(&url);
                fetch = player_lock.navigator().fetch(&url, options);
                url
            };

            let data = fetch.await.map_err(|error| {
                player
                    .lock()
                    .unwrap()
                    .ui()
                    .display_root_movie_download_failed_message();
                error
            })?;

            let mut movie = SwfMovie::from_data(&data, Some(url.into_owned()), None)?;
            on_metadata(movie.header());
            movie.append_parameters(parameters);
            player.lock().unwrap().set_root_movie(Arc::new(movie));
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
            // clippy reports a false positive for explicitly dropped guards:
            // https://github.com/rust-lang/rust-clippy/issues/6446
            // A workaround for this is to wrap the `.lock()` call in a block instead of explicitly dropping the guard.
            let fetch;
            let url = {
                let player_lock = player.lock().unwrap();
                let url = player_lock.navigator().resolve_relative_url(&url);
                fetch = player_lock.navigator().fetch(&url, options);
                url
            };

            let mut replacing_root_movie = false;
            player.lock().unwrap().update(|uc| -> Result<(), Error> {
                let (clip, broadcaster) = match uc.load_manager.get_loader(handle) {
                    Some(Loader::Movie {
                        target_clip,
                        target_broadcaster,
                        ..
                    }) => (*target_clip, *target_broadcaster),
                    None => return Err(Error::Cancelled),
                    _ => unreachable!(),
                };

                replacing_root_movie = DisplayObject::ptr_eq(clip, uc.stage.root_clip());

                if let Some(mut mc) = clip.as_movie_clip() {
                    mc.unload(uc);
                    mc.replace_with_movie(uc.gc_context, None);
                }

                if let Some(broadcaster) = broadcaster {
                    Avm1::run_stack_frame_for_method(
                        clip,
                        broadcaster,
                        NEWEST_PLAYER_VERSION,
                        uc,
                        "broadcastMessage".into(),
                        &["onLoadStart".into(), clip.object()],
                    );
                }

                Ok(())
            })?;

            if let Ok(data) = fetch.await {
                let movie = Arc::new(SwfMovie::from_data(
                    &data,
                    Some(url.into_owned()),
                    loader_url,
                )?);
                if replacing_root_movie {
                    player.lock().unwrap().set_root_movie(movie);
                    return Ok(());
                }

                player.lock().unwrap().update(|uc| {
                    let (clip, broadcaster) = match uc.load_manager.get_loader(handle) {
                        Some(Loader::Movie {
                            target_clip,
                            target_broadcaster,
                            ..
                        }) => (*target_clip, *target_broadcaster),
                        None => return Err(Error::Cancelled),
                        _ => unreachable!(),
                    };

                    let mut activation = Avm2Activation::from_nothing(uc.reborrow());
                    let parent_domain = activation.avm2().global_domain();
                    let domain = Avm2Domain::movie_domain(&mut activation, parent_domain);
                    uc.library
                        .library_for_movie_mut(movie.clone())
                        .set_avm2_domain(domain);

                    if let Some(broadcaster) = broadcaster {
                        Avm1::run_stack_frame_for_method(
                            clip,
                            broadcaster,
                            NEWEST_PLAYER_VERSION,
                            uc,
                            "broadcastMessage".into(),
                            &[
                                "onLoadProgress".into(),
                                clip.object(),
                                data.len().into(),
                                data.len().into(),
                            ],
                        );
                    }

                    if let Some(mut mc) = clip.as_movie_clip() {
                        mc.replace_with_movie(uc.gc_context, Some(movie.clone()));
                        mc.post_instantiation(uc, None, Instantiator::Movie, false);

                        let mut morph_shapes = fnv::FnvHashMap::default();
                        mc.preload(uc, &mut morph_shapes);

                        // Finalize morph shapes.
                        for (id, static_data) in morph_shapes {
                            let morph_shape = MorphShape::new(uc.gc_context, static_data);
                            uc.library
                                .library_for_movie_mut(movie.clone())
                                .register_character(
                                    id,
                                    crate::character::Character::MorphShape(morph_shape),
                                );
                        }
                    }

                    if let Some(broadcaster) = broadcaster {
                        Avm1::run_stack_frame_for_method(
                            clip,
                            broadcaster,
                            NEWEST_PLAYER_VERSION,
                            uc,
                            "broadcastMessage".into(),
                            // TODO: Pass an actual httpStatus argument instead of 0.
                            &["onLoadComplete".into(), clip.object(), 0.into()],
                        );
                    }

                    if let Some(Loader::Movie { loader_status, .. }) =
                        uc.load_manager.get_loader_mut(handle)
                    {
                        *loader_status = LoaderStatus::Succeeded;
                    };

                    Ok(())
                })
            } else {
                //TODO: Inspect the fetch error.
                //This requires cooperation from the backend to send abstract
                //error types we can actually inspect.
                //This also can get errors from decoding an invalid SWF file,
                //too. We should distinguish those to player code.
                player.lock().unwrap().update(|uc| -> Result<(), Error> {
                    let (clip, broadcaster) = match uc.load_manager.get_loader(handle) {
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
                            NEWEST_PLAYER_VERSION,
                            uc,
                            "broadcastMessage".into(),
                            &[
                                "onLoadError".into(),
                                clip.object(),
                                "LoadNeverCompleted".into(),
                            ],
                        );
                    }

                    if let Some(Loader::Movie { loader_status, .. }) =
                        uc.load_manager.get_loader_mut(handle)
                    {
                        *loader_status = LoaderStatus::Failed;
                    };

                    Ok(())
                })
            }
        })
    }

    pub fn form_loader(
        &mut self,
        player: Weak<Mutex<Player>>,
        fetch: OwnedFuture<Vec<u8>, Error>,
    ) -> OwnedFuture<(), Error> {
        let handle = match self {
            Loader::Form { self_handle, .. } => self_handle.expect("Loader not self-introduced"),
            _ => return Box::pin(async { Err(Error::NotFormLoader) }),
        };

        let player = player
            .upgrade()
            .expect("Could not upgrade weak reference to player");

        Box::pin(async move {
            let data = fetch.await?;

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

                for (k, v) in form_urlencoded::parse(&data) {
                    let k = AvmString::new_utf8(activation.context.gc_context, k);
                    let v = AvmString::new_utf8(activation.context.gc_context, v);
                    that.set(k, v.into(), &mut activation)?;
                }

                Ok(())
            })
        })
    }

    /// Creates a future for a LoadVars load call.
    pub fn load_vars_loader(
        &mut self,
        player: Weak<Mutex<Player>>,
        fetch: OwnedFuture<Vec<u8>, Error>,
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
                    Ok(data) => {
                        let _ =
                            that.call_method("onHTTPStatus".into(), &[200.into()], &mut activation);

                        // Fire the onData method with the loaded string.
                        let string_data = AvmString::new_utf8(
                            activation.context.gc_context,
                            UTF_8.decode(&data).0,
                        );
                        let _ = that.call_method(
                            "onData".into(),
                            &[string_data.into()],
                            &mut activation,
                        );
                    }
                    Err(_) => {
                        // TODO: Log "Error opening URL" trace similar to the Flash Player?
                        // Simulate 404 HTTP status. This should probably be fired elsewhere
                        // because a failed local load doesn't fire a 404.
                        let _ =
                            that.call_method("onHTTPStatus".into(), &[404.into()], &mut activation);

                        // Fire the onData method with no data to indicate an unsuccessful load.
                        let _ =
                            that.call_method("onData".into(), &[Value::Undefined], &mut activation);
                    }
                }

                Ok(())
            })
        })
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
