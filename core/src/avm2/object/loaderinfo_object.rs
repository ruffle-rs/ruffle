//! Loader-info object

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Avm2;
use crate::avm2::Error;
use crate::avm2::EventObject;
use crate::context::UpdateContext;
use crate::display_object::DisplayObject;
use crate::tag_utils::SwfMovie;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};
use std::sync::Arc;

/// A class instance allocator that allocates LoaderInfo objects.
pub fn loaderinfo_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(LoaderInfoObject(GcCell::allocate(
        activation.context.gc_context,
        LoaderInfoObjectData {
            base,
            loaded_stream: None,
            loader: None,
            events_fired: false,
            shared_events: activation
                .context
                .avm2
                .classes()
                .eventdispatcher
                .construct(activation, &[])?,
        },
    ))
    .into())
}

/// Represents a thing which can be loaded by a loader.
#[derive(Collect, Debug, Clone)]
#[collect(no_drop)]
pub enum LoaderStream<'gc> {
    /// While it makes no sense to actually retrieve loader info properties off
    /// the stage, it's possible to do so. Some properties yield the
    /// not-yet-loaded error while others are pulled from the root SWF.
    ///
    /// For loader infos that point to an actual loaded movie rather than the
    /// stage, the DisplayObject parameter is provided. It is always `None` for
    /// the stage.
    NotYetLoaded(Arc<SwfMovie>, Option<DisplayObject<'gc>>),

    /// A loaded SWF movie.
    ///
    /// The associated `DisplayObject` is the root movieclip.
    Swf(Arc<SwfMovie>, DisplayObject<'gc>),
}

/// An Object which represents a loadable object, such as a SWF movie or image
/// resource.
#[derive(Collect, Debug, Clone, Copy)]
#[collect(no_drop)]
pub struct LoaderInfoObject<'gc>(GcCell<'gc, LoaderInfoObjectData<'gc>>);

#[derive(Collect, Debug, Clone)]
#[collect(no_drop)]
pub struct LoaderInfoObjectData<'gc> {
    /// All normal script data.
    base: ScriptObjectData<'gc>,

    /// The loaded stream that this gets it's info from.
    loaded_stream: Option<LoaderStream<'gc>>,

    loader: Option<Object<'gc>>,

    /// Whether or not we've fired our 'init' and 'complete' events
    events_fired: bool,

    /// The `EventDispatcher` used for `LoaderInfo.sharedEvents`.
    // FIXME: If we ever implement sandboxing, then ensure that we allow
    // events to be fired across security boundaries using this object.
    shared_events: Object<'gc>,
}

impl<'gc> LoaderInfoObject<'gc> {
    /// Box a movie into a loader info object.
    pub fn from_movie(
        activation: &mut Activation<'_, 'gc, '_>,
        movie: Arc<SwfMovie>,
        root: DisplayObject<'gc>,
        loader: Option<Object<'gc>>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let class = activation.avm2().classes().loaderinfo;
        let base = ScriptObjectData::new(class);
        let loaded_stream = Some(LoaderStream::Swf(movie, root));

        let mut this: Object<'gc> = LoaderInfoObject(GcCell::allocate(
            activation.context.gc_context,
            LoaderInfoObjectData {
                base,
                loaded_stream,
                loader,
                events_fired: false,
                shared_events: activation
                    .context
                    .avm2
                    .classes()
                    .eventdispatcher
                    .construct(activation, &[])?,
            },
        ))
        .into();
        this.install_instance_slots(activation);

        class.call_native_init(Some(this), &[], activation)?;

        Ok(this)
    }

    /// Create a loader info object that has not yet been loaded.
    ///
    /// Use `None` as the root clip to indicate that this is the stage's loader
    /// info.
    pub fn not_yet_loaded(
        activation: &mut Activation<'_, 'gc, '_>,
        movie: Arc<SwfMovie>,
        loader: Option<Object<'gc>>,
        root_clip: Option<DisplayObject<'gc>>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let class = activation.avm2().classes().loaderinfo;
        let base = ScriptObjectData::new(class);

        let mut this: Object<'gc> = LoaderInfoObject(GcCell::allocate(
            activation.context.gc_context,
            LoaderInfoObjectData {
                base,
                loaded_stream: Some(LoaderStream::NotYetLoaded(movie, root_clip)),
                loader,
                events_fired: false,
                shared_events: activation
                    .context
                    .avm2
                    .classes()
                    .eventdispatcher
                    .construct(activation, &[])?,
            },
        ))
        .into();
        this.install_instance_slots(activation);

        class.call_native_init(Some(this), &[], activation)?;

        Ok(this)
    }

    pub fn loader(&self) -> Option<Object<'gc>> {
        return self.0.read().loader;
    }

    pub fn shared_events(&self) -> Object<'gc> {
        return self.0.read().shared_events;
    }

    pub fn fire_init_and_complete_events(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        if !self.0.read().events_fired {
            self.0.write(context.gc_context).events_fired = true;

            // TODO - 'init' should be fired earlier during the download.
            // Right now, we fire it when downloading is fully completed.
            let init_evt = EventObject::bare_default_event(context, "init");

            if let Err(e) = Avm2::dispatch_event(context, init_evt, (*self).into()) {
                log::error!(
                    "Encountered AVM2 error when broadcasting `init` event: {}",
                    e
                );
            }

            let complete_evt = EventObject::bare_default_event(context, "complete");

            if let Err(e) = Avm2::dispatch_event(context, complete_evt, (*self).into()) {
                log::error!(
                    "Encountered AVM2 error when broadcasting `complete` event: {}",
                    e
                );
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

    pub fn set_loader_stream(&self, stream: LoaderStream<'gc>, mc: MutationContext<'gc, '_>) {
        self.0.write(mc).loaded_stream = Some(stream);
    }
}

impl<'gc> TObject<'gc> for LoaderInfoObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object((*self).into()))
    }

    fn as_loader_info_object(&self) -> Option<&LoaderInfoObject<'gc>> {
        Some(self)
    }
}
