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
use gc_arena::barrier::unlock;
use gc_arena::{
    lock::{Lock, RefLock},
    Collect, Gc, GcWeak, Mutation,
};
use std::cell::{Cell, Ref};
use std::sync::Arc;

/// ActionScript cannot construct a LoaderInfo. Note that LoaderInfo isn't a final class.
pub fn loader_info_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let class_name = class.inner_class_definition().name().local_name();

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

impl<'gc> LoaderStream<'gc> {
    pub fn movie(&self) -> &Arc<SwfMovie> {
        match self {
            LoaderStream::NotYetLoaded(movie, _, _) => movie,
            LoaderStream::Swf(movie, _) => movie,
        }
    }
}

/// An Object which represents a loadable object, such as a SWF movie or image
/// resource.
#[derive(Collect, Clone, Copy)]
#[collect(no_drop)]
pub struct LoaderInfoObject<'gc>(pub Gc<'gc, LoaderInfoObjectData<'gc>>);

#[derive(Collect, Clone, Copy, Debug)]
#[collect(no_drop)]
pub struct LoaderInfoObjectWeak<'gc>(pub GcWeak<'gc, LoaderInfoObjectData<'gc>>);

impl fmt::Debug for LoaderInfoObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoaderInfoObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Collect, Clone)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct LoaderInfoObjectData<'gc> {
    /// All normal script data.
    base: ScriptObjectData<'gc>,

    /// The loaded stream that this gets its info from.
    loaded_stream: RefLock<LoaderStream<'gc>>,

    loader: Option<Object<'gc>>,

    /// Whether or not we've fired our 'init' event
    init_event_fired: Cell<bool>,

    /// Whether or not we've fired our 'complete' event
    complete_event_fired: Cell<bool>,

    /// The `EventDispatcher` used for `LoaderInfo.sharedEvents`.
    // FIXME: If we ever implement sandboxing, then ensure that we allow
    // events to be fired across security boundaries using this object.
    shared_events: Object<'gc>,

    uncaught_error_events: Object<'gc>,

    cached_avm1movie: Lock<Option<Object<'gc>>>,

    content_type: Cell<ContentType>,

    expose_content: Cell<bool>,

    errored: Cell<bool>,
}

const _: () = assert!(std::mem::offset_of!(LoaderInfoObjectData, base) == 0);
const _: () = assert!(
    std::mem::align_of::<LoaderInfoObjectData>() == std::mem::align_of::<ScriptObjectData>()
);

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
        let loaded_stream = LoaderStream::Swf(movie, root);

        let this: Object<'gc> = LoaderInfoObject(Gc::new(
            activation.context.gc_context,
            LoaderInfoObjectData {
                base,
                loaded_stream: RefLock::new(loaded_stream),
                loader,
                init_event_fired: Cell::new(false),
                complete_event_fired: Cell::new(false),
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
                cached_avm1movie: Lock::new(None),
                content_type: Cell::new(ContentType::Swf),
                expose_content: Cell::new(false),
                errored: Cell::new(false),
            },
        ))
        .into();

        class.call_super_init(this.into(), &[], activation)?;

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

        let this: Object<'gc> = LoaderInfoObject(Gc::new(
            activation.context.gc_context,
            LoaderInfoObjectData {
                base,
                loaded_stream: RefLock::new(LoaderStream::NotYetLoaded(movie, root_clip, is_stage)),
                loader,
                init_event_fired: Cell::new(false),
                complete_event_fired: Cell::new(false),
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
                cached_avm1movie: Lock::new(None),
                content_type: Cell::new(ContentType::Unknown),
                expose_content: Cell::new(false),
                errored: Cell::new(false),
            },
        ))
        .into();

        class.call_super_init(this.into(), &[], activation)?;

        Ok(this)
    }

    pub fn loader(&self) -> Option<Object<'gc>> {
        self.0.loader
    }

    pub fn shared_events(&self) -> Object<'gc> {
        self.0.shared_events
    }

    pub fn uncaught_error_events(&self) -> Object<'gc> {
        self.0.uncaught_error_events
    }

    /// Gets the `ContentType`, 'hiding' it by returning `ContentType::Unknown`
    /// if we haven't yet fired the 'init' event. The real ContentType first becomes
    /// visible to ActionScript in the 'init' event.
    pub fn content_type_hide_before_init(&self) -> ContentType {
        if self.0.init_event_fired.get() {
            self.0.content_type.get()
        } else {
            ContentType::Unknown
        }
    }

    pub fn set_errored(&self, val: bool) {
        self.0.errored.set(val);
    }

    pub fn errored(&self) -> bool {
        self.0.errored.get()
    }

    pub fn init_event_fired(&self) -> bool {
        self.0.init_event_fired.get()
    }

    pub fn reset_init_and_complete_events(&self) {
        self.0.init_event_fired.set(false);
        self.0.complete_event_fired.set(false);
    }

    /// Fires the 'init' and 'complete' events if they haven't been fired yet.
    /// Returns `true` if both events have been fired (either as a result of
    /// this call, or due to a previous call).
    pub fn fire_init_and_complete_events(
        &self,
        context: &mut UpdateContext<'gc>,
        status: u16,
        redirected: bool,
    ) -> bool {
        self.0.expose_content.set(true);
        if !self.0.init_event_fired.get() {
            self.0.init_event_fired.set(true);

            // TODO - 'init' should be fired earlier during the download.
            // Right now, we fire it when downloading is fully completed.
            let init_evt = EventObject::bare_default_event(context, "init");
            Avm2::dispatch_event(context, init_evt, (*self).into());
        }

        if !self.0.complete_event_fired.get() {
            // NOTE: We have to check load progress here because this function
            // is called unconditionally at the end of every frame.
            let (should_complete, from_url) = match &*self.0.loaded_stream.borrow() {
                LoaderStream::Swf(ref movie, root) => (
                    root.as_movie_clip()
                        .map(|mc| mc.loaded_bytes() as i32 >= mc.total_bytes())
                        .unwrap_or(true),
                    movie.loader_url().is_some(),
                ),
                _ => (false, false),
            };

            if should_complete {
                let mut activation = Activation::from_nothing(context);
                if from_url {
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
                }

                self.0.complete_event_fired.set(true);
                let complete_evt = EventObject::bare_default_event(context, "complete");
                Avm2::dispatch_event(context, complete_evt, (*self).into());
                return true;
            }
            return false;
        }
        true
    }

    /// Unwrap this object's loader stream
    pub fn as_loader_stream(&self) -> Option<Ref<LoaderStream<'gc>>> {
        Some(self.0.loaded_stream.borrow())
    }

    pub fn expose_content(&self) -> bool {
        self.0.expose_content.get()
    }

    /// Makes the 'content' visible to ActionScript.
    /// This is used by certain special loaders (the stage and root movie),
    /// which expose the loaded content before the 'init' event is fired.
    pub fn set_expose_content(&self) {
        self.0.expose_content.set(true);
    }

    pub fn set_loader_stream(&self, stream: LoaderStream<'gc>, mc: &Mutation<'gc>) {
        *unlock!(Gc::write(mc, self.0), LoaderInfoObjectData, loaded_stream).borrow_mut() = stream;
    }

    pub fn set_content_type(&self, content_type: ContentType) {
        self.0.content_type.set(content_type);
    }

    /// Returns the AVM1Movie corresponding to the loaded movie- if
    /// it doesn't exist yet, creates it.
    pub fn get_or_init_avm1movie(
        &self,
        activation: &mut Activation<'_, 'gc>,
        obj: DisplayObject<'gc>,
    ) -> Object<'gc> {
        let cached_avm1movie = self.0.cached_avm1movie.get();
        if cached_avm1movie.is_none() {
            let class_object = activation.avm2().classes().avm1movie;
            let object = StageObject::for_display_object(activation, obj, class_object)
                .expect("for_display_object cannot return Err");

            class_object
                .call_super_init(object.into(), &[], activation)
                .expect("Native init should succeed");

            unlock!(
                Gc::write(activation.context.gc_context, self.0),
                LoaderInfoObjectData,
                cached_avm1movie
            )
            .set(Some(object.into()));
        }

        self.0.cached_avm1movie.get().unwrap()
    }

    pub fn unload(&self, activation: &mut Activation<'_, 'gc>) {
        // Reset properties
        let empty_swf = Arc::new(SwfMovie::empty(activation.context.swf.version()));
        let loader_stream = LoaderStream::NotYetLoaded(empty_swf, None, false);
        self.set_loader_stream(loader_stream, activation.context.gc_context);
        self.set_errored(false);
        self.reset_init_and_complete_events();

        let loader = self
            .0
            .loader
            .expect("LoaderInfo must have been created by Loader");

        // Remove the Loader's content element, and ignore the resulting
        // error if the loader hadn't loaded it.
        let _ = crate::avm2::globals::flash::display::display_object_container::remove_child_at(
            activation,
            loader,
            &[0.into()],
        );
    }
}

impl<'gc> TObject<'gc> for LoaderInfoObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        // SAFETY: Object data is repr(C), and a compile-time assert ensures
        // that the ScriptObjectData stays at offset 0 of the struct- so the
        // layouts are compatible

        unsafe { Gc::cast(self.0) }
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object((*self).into()))
    }

    fn as_loader_info_object(&self) -> Option<&LoaderInfoObject<'gc>> {
        Some(self)
    }
}
