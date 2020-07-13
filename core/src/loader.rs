//! Management of async loaders

use crate::avm1::activation::{Activation, ActivationIdentifier};
use crate::avm1::{AvmString, Object, TObject, Value};
use crate::backend::navigator::OwnedFuture;
use crate::context::{ActionQueue, ActionType};
use crate::display_object::{DisplayObject, MorphShape, TDisplayObject};
use crate::player::{Player, NEWEST_PLAYER_VERSION};
use crate::tag_utils::SwfMovie;
use crate::xml::XMLNode;
use gc_arena::{Collect, CollectionContext, MutationContext};
use generational_arena::{Arena, Index};
use std::string::FromUtf8Error;
use std::sync::{Arc, Mutex, Weak};
use thiserror::Error;
use url::form_urlencoded;

pub type Handle = Index;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Load cancelled")]
    Cancelled,

    #[error("Non-movie loader spawned as movie loader")]
    NotMovieLoader,

    #[error("Non-form loader spawned as form loader")]
    NotFormLoader,

    #[error("Non-XML loader spawned as XML loader")]
    NotXmlLoader,

    #[error("Invalid SWF")]
    InvalidSwf(#[from] crate::tag_utils::Error),

    #[error("Invalid XML encoding")]
    InvalidXmlEncoding(#[from] FromUtf8Error),

    #[error("Network error")]
    NetworkError(#[from] std::io::Error),

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
    /// This function returns the loader handle for later inspection. A loader
    /// handle is valid for as long as the load operation. Once the load
    /// finishes, the handle will be invalidated (and the underlying loader
    /// deleted).
    pub fn add_loader(&mut self, loader: Loader<'gc>) -> Handle {
        let handle = self.0.insert(loader);
        self.0
            .get_mut(handle)
            .unwrap()
            .introduce_loader_handle(handle);

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

    /// Kick off a movie clip load.
    ///
    /// Returns the loader's async process, which you will need to spawn.
    pub fn load_movie_into_clip(
        &mut self,
        player: Weak<Mutex<Player>>,
        target_clip: DisplayObject<'gc>,
        fetch: OwnedFuture<Vec<u8>, Error>,
        target_broadcaster: Option<Object<'gc>>,
    ) -> OwnedFuture<(), Error> {
        let loader = Loader::Movie {
            self_handle: None,
            target_clip,
            target_broadcaster,
            load_complete: false,
        };
        let handle = self.add_loader(loader);

        let loader = self.get_loader_mut(handle).unwrap();
        loader.introduce_loader_handle(handle);

        loader.movie_loader(player, fetch)
    }

    /// Indicates that a movie clip has initialized (ran it's first frame).
    ///
    /// Interested loaders will be invoked from here.
    pub fn movie_clip_on_load(
        &mut self,
        loaded_clip: DisplayObject<'gc>,
        clip_object: Option<Object<'gc>>,
        queue: &mut ActionQueue<'gc>,
        gc_context: MutationContext<'gc, '_>,
    ) {
        let mut invalidated_loaders = vec![];

        for (index, loader) in self.0.iter_mut() {
            if loader.movie_clip_loaded(loaded_clip, clip_object, queue, gc_context) {
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
        loader.introduce_loader_handle(handle);

        loader.form_loader(player, fetch)
    }

    /// Kick off an XML data load into an XML node.
    ///
    /// Returns the loader's async process, which you will need to spawn.
    pub fn load_xml_into_node(
        &mut self,
        player: Weak<Mutex<Player>>,
        target_node: XMLNode<'gc>,
        active_clip: DisplayObject<'gc>,
        fetch: OwnedFuture<Vec<u8>, Error>,
    ) -> OwnedFuture<(), Error> {
        let loader = Loader::XML {
            self_handle: None,
            active_clip,
            target_node,
        };
        let handle = self.add_loader(loader);

        let loader = self.get_loader_mut(handle).unwrap();
        loader.introduce_loader_handle(handle);

        loader.xml_loader(player, fetch)
    }
}

impl<'gc> Default for LoadManager<'gc> {
    fn default() -> Self {
        Self::new()
    }
}

/// A struct that holds garbage-collected pointers for asynchronous code.
pub enum Loader<'gc> {
    /// Loader that is loading a new movie into a movieclip.
    Movie {
        /// The handle to refer to this loader instance.
        self_handle: Option<Handle>,

        /// The target movie clip to load the movie into.
        target_clip: DisplayObject<'gc>,

        /// Event broadcaster (typically a `MovieClipLoader`) to fire events
        /// into.
        target_broadcaster: Option<Object<'gc>>,

        /// Indicates that the load has completed.
        ///
        /// This flag exists to prevent a situation in which loading a movie
        /// into a clip that has not yet fired it's Load event causes the
        /// loader to be prematurely removed. This flag is only set when either
        /// the movie has been replaced (and thus Load events can be trusted)
        /// or an error has occured (in which case we don't care about the
        /// loader anymore).
        load_complete: bool,
    },

    /// Loader that is loading form data into an AVM1 object scope.
    Form {
        /// The handle to refer to this loader instance.
        self_handle: Option<Handle>,

        /// The target AVM1 object to load form data into.
        target_object: Object<'gc>,
    },

    /// Loader that is loading XML data into an XML tree.
    XML {
        /// The handle to refer to this loader instance.
        self_handle: Option<Handle>,

        /// The active movie clip at the time of load invocation.
        ///
        /// This property is a technicality: Under normal circumstances, it's
        /// not supposed to be a load factor, and only exists so that the
        /// runtime can do *something* in really contrived scenarios where we
        /// actually need an active clip.
        active_clip: DisplayObject<'gc>,

        /// The target node whose contents will be replaced with the parsed XML.
        target_node: XMLNode<'gc>,
    },
}

unsafe impl<'gc> Collect for Loader<'gc> {
    fn trace(&self, cc: CollectionContext) {
        match self {
            Loader::Movie {
                target_clip,
                target_broadcaster,
                ..
            } => {
                target_clip.trace(cc);
                target_broadcaster.trace(cc);
            }
            Loader::Form { target_object, .. } => target_object.trace(cc),
            Loader::XML { target_node, .. } => target_node.trace(cc),
        }
    }
}

impl<'gc> Loader<'gc> {
    /// Set the loader handle for this loader.
    ///
    /// An active loader handle is required before asynchronous loader code can
    /// run.
    pub fn introduce_loader_handle(&mut self, handle: Handle) {
        match self {
            Loader::Movie { self_handle, .. } => *self_handle = Some(handle),
            Loader::Form { self_handle, .. } => *self_handle = Some(handle),
            Loader::XML { self_handle, .. } => *self_handle = Some(handle),
        }
    }

    /// Construct a future for the given movie loader.
    ///
    /// The given future should be passed immediately to an executor; it will
    /// take responsibility for running the loader to completion.
    ///
    /// If the loader is not a movie then the returned future will yield an
    /// error immediately once spawned.
    pub fn movie_loader(
        &mut self,
        player: Weak<Mutex<Player>>,
        fetch: OwnedFuture<Vec<u8>, Error>,
    ) -> OwnedFuture<(), Error> {
        let handle = match self {
            Loader::Movie { self_handle, .. } => self_handle.expect("Loader not self-introduced"),
            _ => return Box::pin(async { Err(Error::NotMovieLoader) }),
        };

        let player = player
            .upgrade()
            .expect("Could not upgrade weak reference to player");

        Box::pin(async move {
            player.lock().expect("Could not lock player!!").update(
                |avm, uc| -> Result<(), Error> {
                    let (clip, broadcaster) = match uc.load_manager.get_loader(handle) {
                        Some(Loader::Movie {
                            target_clip,
                            target_broadcaster,
                            ..
                        }) => (*target_clip, *target_broadcaster),
                        None => return Err(Error::Cancelled),
                        _ => unreachable!(),
                    };

                    clip.as_movie_clip().unwrap().unload(uc);

                    clip.as_movie_clip()
                        .unwrap()
                        .replace_with_movie(uc.gc_context, None);

                    if let Some(broadcaster) = broadcaster {
                        avm.run_stack_frame_for_method(
                            clip,
                            broadcaster,
                            NEWEST_PLAYER_VERSION,
                            uc,
                            "broadcastMessage",
                            &["onLoadStart".into(), Value::Object(broadcaster)],
                        );
                    }

                    Ok(())
                },
            )?;

            let data = (fetch.await).and_then(|data| Ok((data.len(), SwfMovie::from_data(&data)?)));
            if let Ok((length, movie)) = data {
                let movie = Arc::new(movie);

                player
                    .lock()
                    .expect("Could not lock player!!")
                    .update(|avm, uc| {
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
                            avm.run_stack_frame_for_method(
                                clip,
                                broadcaster,
                                NEWEST_PLAYER_VERSION,
                                uc,
                                "broadcastMessage",
                                &[
                                    "onLoadProgress".into(),
                                    Value::Object(broadcaster),
                                    length.into(),
                                    length.into(),
                                ],
                            );
                        }

                        let mut mc = clip
                            .as_movie_clip()
                            .expect("Attempted to load movie into not movie clip");

                        mc.replace_with_movie(uc.gc_context, Some(movie.clone()));
                        mc.post_instantiation(avm, uc, clip, None, false);

                        let mut morph_shapes = fnv::FnvHashMap::default();
                        mc.preload(avm, uc, &mut morph_shapes);

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

                        if let Some(broadcaster) = broadcaster {
                            avm.run_stack_frame_for_method(
                                clip,
                                broadcaster,
                                NEWEST_PLAYER_VERSION,
                                uc,
                                "broadcastMessage",
                                &["onLoadComplete".into(), Value::Object(broadcaster)],
                            );
                        }

                        if let Some(Loader::Movie { load_complete, .. }) =
                            uc.load_manager.get_loader_mut(handle)
                        {
                            *load_complete = true;
                        };

                        Ok(())
                    })
            } else {
                //TODO: Inspect the fetch error.
                //This requires cooperation from the backend to send abstract
                //error types we can actually inspect.
                //This also can get errors from decoding an invalid SWF file,
                //too. We should distinguish those to player code.
                player.lock().expect("Could not lock player!!").update(
                    |avm, uc| -> Result<(), Error> {
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
                            avm.run_stack_frame_for_method(
                                clip,
                                broadcaster,
                                NEWEST_PLAYER_VERSION,
                                uc,
                                "broadcastMessage",
                                &[
                                    "onLoadError".into(),
                                    Value::Object(broadcaster),
                                    "LoadNeverCompleted".into(),
                                ],
                            );
                        }

                        if let Some(Loader::Movie { load_complete, .. }) =
                            uc.load_manager.get_loader_mut(handle)
                        {
                            *load_complete = true;
                        };

                        Ok(())
                    },
                )
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

            player.lock().unwrap().update(|avm, uc| {
                let loader = uc.load_manager.get_loader(handle);
                let that = match loader {
                    Some(Loader::Form { target_object, .. }) => *target_object,
                    None => return Err(Error::Cancelled),
                    _ => return Err(Error::NotMovieLoader),
                };

                let mut activation = Activation::from_nothing(
                    avm,
                    ActivationIdentifier::root("[Form Loader]"),
                    uc.swf.version(),
                    avm.global_object_cell(),
                    uc.gc_context,
                    *uc.levels.get(&0).unwrap(),
                );
                for (k, v) in form_urlencoded::parse(&data) {
                    that.set(
                        &k,
                        AvmString::new(uc.gc_context, v.into_owned()).into(),
                        &mut activation,
                        uc,
                    )?;
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
    pub fn movie_clip_loaded(
        &mut self,
        loaded_clip: DisplayObject<'gc>,
        clip_object: Option<Object<'gc>>,
        queue: &mut ActionQueue<'gc>,
        _gc_context: MutationContext<'gc, '_>,
    ) -> bool {
        let (clip, broadcaster, load_complete) = match self {
            Loader::Movie {
                target_clip,
                target_broadcaster,
                load_complete,
                ..
            } => (*target_clip, *target_broadcaster, *load_complete),
            _ => return false,
        };

        if DisplayObject::ptr_eq(loaded_clip, clip) && load_complete {
            if let Some(broadcaster) = broadcaster {
                queue.queue_actions(
                    clip,
                    ActionType::Method {
                        object: broadcaster,
                        name: "broadcastMessage",
                        args: vec![
                            "onLoadInit".into(),
                            clip_object.map(|co| co.into()).unwrap_or(Value::Undefined),
                        ],
                    },
                    false,
                );
            }

            true
        } else {
            false
        }
    }

    pub fn xml_loader(
        &mut self,
        player: Weak<Mutex<Player>>,
        fetch: OwnedFuture<Vec<u8>, Error>,
    ) -> OwnedFuture<(), Error> {
        let handle = match self {
            Loader::XML { self_handle, .. } => self_handle.expect("Loader not self-introduced"),
            _ => return Box::pin(async { Err(Error::NotXmlLoader) }),
        };

        let player = player
            .upgrade()
            .expect("Could not upgrade weak reference to player");

        Box::pin(async move {
            let data = fetch.await;
            if let Ok(data) = data {
                let xmlstring = String::from_utf8(data)?;

                player.lock().expect("Could not lock player!!").update(
                    |avm, uc| -> Result<(), Error> {
                        let (mut node, active_clip) = match uc.load_manager.get_loader(handle) {
                            Some(Loader::XML {
                                target_node,
                                active_clip,
                                ..
                            }) => (*target_node, *active_clip),
                            None => return Err(Error::Cancelled),
                            _ => unreachable!(),
                        };

                        let object =
                            node.script_object(uc.gc_context, Some(avm.prototypes().xml_node));
                        avm.run_stack_frame_for_method(
                            active_clip,
                            object,
                            NEWEST_PLAYER_VERSION,
                            uc,
                            "onHTTPStatus",
                            &[200.into()],
                        );

                        avm.run_stack_frame_for_method(
                            active_clip,
                            object,
                            NEWEST_PLAYER_VERSION,
                            uc,
                            "onData",
                            &[AvmString::new(uc.gc_context, xmlstring).into()],
                        );

                        Ok(())
                    },
                )?;
            } else {
                player.lock().expect("Could not lock player!!").update(
                    |avm, uc| -> Result<(), Error> {
                        let (mut node, active_clip) = match uc.load_manager.get_loader(handle) {
                            Some(Loader::XML {
                                target_node,
                                active_clip,
                                ..
                            }) => (*target_node, *active_clip),
                            None => return Err(Error::Cancelled),
                            _ => unreachable!(),
                        };

                        let object =
                            node.script_object(uc.gc_context, Some(avm.prototypes().xml_node));
                        avm.run_stack_frame_for_method(
                            active_clip,
                            object,
                            NEWEST_PLAYER_VERSION,
                            uc,
                            "onHTTPStatus",
                            &[404.into()],
                        );

                        avm.run_stack_frame_for_method(
                            active_clip,
                            object,
                            NEWEST_PLAYER_VERSION,
                            uc,
                            "onData",
                            &[],
                        );

                        Ok(())
                    },
                )?;
            }

            Ok(())
        })
    }
}
