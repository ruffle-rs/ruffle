//! Loader-info object

use crate::avm2::activation::Activation;
use crate::avm2::error::argument_error;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, StageObject, TObject};
use crate::avm2::value::Value;
use crate::avm2::Avm2;
use crate::avm2::Error;
use crate::avm2::EventObject;
use crate::context::UpdateContext;
use crate::display_object::{DisplayObject, TDisplayObject};
use crate::loader::ContentType;
use crate::tag_utils::SwfMovie;
use core::fmt;
use gc_arena::{Collect, GcCell, GcWeakCell, Mutation};
use std::cell::{Ref, RefMut};
use std::sync::Arc;

/// ActionScript cannot construct a LoaderInfo. Note that LoaderInfo isn't a final class.
pub fn loader_info_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let class_name = class.inner_class_definition().read().name().local_name();

    Err(Error::AvmError(argument_error(
        activation,
        &format!("Error #2012: {class_name}$ class cannot be instantiated."),
        2012,
    )?))
}

/// Represents a thing which can be loaded by a loader.
#[derive(Collect, Clone)]
#[collect(no_drop)]
pub enum LoaderStream<'gc> {
    /// An SWF movie that has not yet loaded.
    ///
    /// The main differences from `Swf` loader streams is that certain loader
    /// info properties are `null` until the SWF is fully loaded. Furthermore,
    /// the `DisplayObject` parameter is optional, to represent movies that do
    /// not yet have a root clip.
    ///
    /// While the `Stage` is not a loadable object, it has `loaderInfo`, with
    /// properties that roughly mirror an unloaded movie clip. Properties that
    /// are valid on `Stage.loaderInfo` will be pulled from the root SWF.
    ///
    /// The `bool` parameter indicates if this is the `Stage`'s loader info;
    /// this is because certain `Stage` properties are accessible even when the
    /// associated movie is not yet loaded.
    NotYetLoaded(Arc<SwfMovie>, Option<DisplayObject<'gc>>, bool),

    /// A loaded SWF movie.
    ///
    /// The associated `DisplayObject` is the root movieclip.
    Swf(Arc<SwfMovie>, DisplayObject<'gc>),
}

/// An Object which represents a loadable object, such as a SWF movie or image
/// resource.
#[derive(Collect, Clone, Copy)]
#[collect(no_drop)]
pub struct LoaderInfoObject<'gc>(pub GcCell<'gc, LoaderInfoObjectData<'gc>>);

#[derive(Collect, Clone, Copy, Debug)]
#[collect(no_drop)]
pub struct LoaderInfoObjectWeak<'gc>(pub GcWeakCell<'gc, LoaderInfoObjectData<'gc>>);

impl fmt::Debug for LoaderInfoObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoaderInfoObject")
            .field("ptr", &self.0.as_ptr())
            .finish()
    }
}

#[derive(Collect, Clone)]
#[collect(no_drop)]
pub struct LoaderInfoObjectData<'gc> {
    /// All normal script data.
    base: ScriptObjectData<'gc>,

    /// The loaded stream that this gets its info from.
    loaded_stream: Option<LoaderStream<'gc>>,

    loader: Option<Object<'gc>>,

    /// Whether or not we've fired our 'init' event
    init_event_fired: bool,

    /// Whether or not we've fired our 'complete' event
    complete_event_fired: bool,

    /// The `EventDispatcher` used for `LoaderInfo.sharedEvents`.
    // FIXME: If we ever implement sandboxing, then ensure that we allow
    // events to be fired across security boundaries using this object.
    shared_events: Object<'gc>,

    uncaught_error_events: Object<'gc>,

    cached_avm1movie: Option<Object<'gc>>,

    #[collect(require_static)]
    content_type: ContentType,
}

impl<'gc> LoaderInfoObject<'gc> {
    /// Box a movie into a loader info object.
    pub fn from_movie(
        activation: &mut Activation<'_, 'gc>,
        movie: Arc<SwfMovie>,
        root: DisplayObject<'gc>,
        loader: Option<Object<'gc>>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let class = activation.avm2().classes().loaderinfo;
        let base = ScriptObjectData::new(class);
        let loaded_stream = Some(LoaderStream::Swf(movie, root));

        let this: Object<'gc> = LoaderInfoObject(GcCell::new(
            activation.context.gc_context,
            LoaderInfoObjectData {
                base,
                loaded_stream,
                loader,
                init_event_fired: false,
                complete_event_fired: false,
                shared_events: activation
                    .context
                    .avm2
                    .classes()
                    .eventdispatcher
                    .construct(activation, &[])?,
                uncaught_error_events: activation
                    .context
                    .avm2
                    .classes()
                    .uncaughterrorevents
                    .construct(activation, &[])?,
                cached_avm1movie: None,
                content_type: ContentType::Swf,
            },
        ))
        .into();
        this.install_instance_slots(activation.context.gc_context);

        class.call_native_init(this.into(), &[], activation)?;

        Ok(this)
    }

    /// Create a loader info object that has not yet been loaded.
    ///
    /// Use `None` as the root clip to indicate that this is the stage's loader
    /// info.
    pub fn not_yet_loaded(
        activation: &mut Activation<'_, 'gc>,
        movie: Arc<SwfMovie>,
        loader: Option<Object<'gc>>,
        root_clip: Option<DisplayObject<'gc>>,
        is_stage: bool,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let class = activation.avm2().classes().loaderinfo;
        let base = ScriptObjectData::new(class);

        let this: Object<'gc> = LoaderInfoObject(GcCell::new(
            activation.context.gc_context,
            LoaderInfoObjectData {
                base,
                loaded_stream: Some(LoaderStream::NotYetLoaded(movie, root_clip, is_stage)),
                loader,
                init_event_fired: false,
                complete_event_fired: false,
                shared_events: activation
                    .context
                    .avm2
                    .classes()
                    .eventdispatcher
                    .construct(activation, &[])?,
                uncaught_error_events: activation
                    .context
                    .avm2
                    .classes()
                    .uncaughterrorevents
                    .construct(activation, &[])?,
                cached_avm1movie: None,
                content_type: ContentType::Unknown,
            },
        ))
        .into();
        this.install_instance_slots(activation.context.gc_context);

        class.call_native_init(this.into(), &[], activation)?;

        Ok(this)
    }

    pub fn loader(&self) -> Option<Object<'gc>> {
        return self.0.read().loader;
    }

    pub fn shared_events(&self) -> Object<'gc> {
        return self.0.read().shared_events;
    }

    pub fn uncaught_error_events(&self) -> Object<'gc> {
        return self.0.read().uncaught_error_events;
    }

    /// Gets the `ContentType`, 'hiding' it by returning `ContentType::Unknown`
    /// if we haven't yet fired the 'init' event. The real ContentType first becomes
    /// visible to ActionScript in the 'init' event.
    pub fn content_type_hide_before_init(&self) -> ContentType {
        if self.0.read().init_event_fired {
            self.0.read().content_type
        } else {
            ContentType::Unknown
        }
    }

    pub fn fire_init_and_complete_events(
        &self,
        context: &mut UpdateContext<'_, 'gc>,
        status: u16,
        redirected: bool,
    ) {
        if !self.0.read().init_event_fired {
            self.0.write(context.gc_context).init_event_fired = true;

            // TODO - 'init' should be fired earlier during the download.
            // Right now, we fire it when downloading is fully completed.
            let init_evt = EventObject::bare_default_event(context, "init");
            Avm2::dispatch_event(context, init_evt, (*self).into());
        }

        if !self.0.read().complete_event_fired {
            // NOTE: We have to check load progress here because this function
            // is called unconditionally at the end of every frame.
            let should_complete = match self.0.read().loaded_stream {
                Some(LoaderStream::Swf(_, root)) => root
                    .as_movie_clip()
                    .map(|mc| mc.loaded_bytes() as i32 >= mc.total_bytes())
                    .unwrap_or(true),
                _ => false,
            };

            if should_complete {
                let mut activation = Activation::from_nothing(context.reborrow());
                let http_status_evt = activation
                    .avm2()
                    .classes()
                    .httpstatusevent
                    .construct(
                        &mut activation,
                        &[
                            "httpStatus".into(),
                            false.into(),
                            false.into(),
                            status.into(),
                            redirected.into(),
                        ],
                    )
                    .unwrap();

                Avm2::dispatch_event(context, http_status_evt, (*self).into());

                self.0.write(context.gc_context).complete_event_fired = true;
                let complete_evt = EventObject::bare_default_event(context, "complete");
                Avm2::dispatch_event(context, complete_evt, (*self).into());
            }
        }
    }

    /// Unwrap this object's loader stream
    pub fn as_loader_stream(&self) -> Option<Ref<LoaderStream<'gc>>> {
        if self.0.read().loaded_stream.is_some() {
            Some(Ref::map(self.0.read(), |v: &LoaderInfoObjectData<'gc>| {
                v.loaded_stream.as_ref().unwrap()
            }))
        } else {
            None
        }
    }

    pub fn set_loader_stream(&self, stream: LoaderStream<'gc>, mc: &Mutation<'gc>) {
        self.0.write(mc).loaded_stream = Some(stream);
    }

    pub fn set_content_type(&self, content_type: ContentType, mc: &Mutation<'gc>) {
        self.0.write(mc).content_type = content_type;
    }

    /// Returns the AVM1Movie corresponding to the loaded movie- if
    /// it doesn't exist yet, creates it.
    pub fn get_or_init_avm1movie(
        &self,
        activation: &mut Activation<'_, 'gc>,
        obj: DisplayObject<'gc>,
    ) -> Object<'gc> {
        let cached_avm1movie = self.0.read().cached_avm1movie;
        if cached_avm1movie.is_none() {
            let class_object = activation.avm2().classes().avm1movie;
            let object = StageObject::for_display_object(activation, obj, class_object)
                .expect("for_display_object cannot return Err");

            class_object
                .call_native_init(object.into(), &[], activation)
                .expect("Native init should succeed");

            self.0.write(activation.context.gc_context).cached_avm1movie = Some(object.into());
        }

        return self.0.read().cached_avm1movie.unwrap();
    }

    pub fn unload(&self, activation: &mut Activation<'_, 'gc>) {
        let empty_swf = Arc::new(SwfMovie::empty(activation.context.swf.version()));
        let loader_stream = LoaderStream::NotYetLoaded(empty_swf, None, false);
        self.set_loader_stream(loader_stream, activation.context.gc_context);
    }
}

impl<'gc> TObject<'gc> for LoaderInfoObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: &Mutation<'gc>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object((*self).into()))
    }

    fn as_loader_info_object(&self) -> Option<&LoaderInfoObject<'gc>> {
        Some(self)
    }
}
