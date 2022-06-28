//! Loader-info object

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Avm2;
use crate::avm2::Error;
use crate::avm2::{Event, EventData};
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
) -> Result<Object<'gc>, Error> {
    let base = ScriptObjectData::new(class);

    Ok(LoaderInfoObject(GcCell::allocate(
        activation.context.gc_context,
        LoaderInfoObjectData {
            base,
            loaded_stream: None,
            init_fired: false,
        },
    ))
    .into())
}

/// Represents a thing which can be loaded by a loader.
#[derive(Collect, Debug, Clone)]
#[collect(no_drop)]
pub enum LoaderStream<'gc> {
    /// The current stage.
    ///
    /// While it makes no sense to actually retrieve loader info properties off
    /// the stage, it's possible to do so. Some properties yield the
    /// not-yet-loaded error while others are pulled from the root SWF.
    Stage,

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

    /// Whether or not we've fired an 'init' event
    init_fired: bool,
}

impl<'gc> LoaderInfoObject<'gc> {
    /// Box a movie into a loader info object.
    pub fn from_movie(
        activation: &mut Activation<'_, 'gc, '_>,
        movie: Arc<SwfMovie>,
        root: DisplayObject<'gc>,
    ) -> Result<Object<'gc>, Error> {
        let class = activation.avm2().classes().loaderinfo;
        let base = ScriptObjectData::new(class);
        let loaded_stream = Some(LoaderStream::Swf(movie, root));

        let mut this: Object<'gc> = LoaderInfoObject(GcCell::allocate(
            activation.context.gc_context,
            LoaderInfoObjectData {
                base,
                loaded_stream,
                init_fired: false,
            },
        ))
        .into();
        this.install_instance_slots(activation);

        class.call_native_init(Some(this), &[], activation)?;

        Ok(this)
    }

    /// Create a loader info object for the stage.
    pub fn from_stage(activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        let class = activation.avm2().classes().loaderinfo;
        let base = ScriptObjectData::new(class);

        let mut this: Object<'gc> = LoaderInfoObject(GcCell::allocate(
            activation.context.gc_context,
            LoaderInfoObjectData {
                base,
                loaded_stream: Some(LoaderStream::Stage),
                // We never want to fire an "init" event for the special
                // Stagee loaderInfo
                init_fired: true,
            },
        ))
        .into();
        this.install_instance_slots(activation);

        class.call_native_init(Some(this), &[], activation)?;

        Ok(this)
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

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(Value::Object((*self).into()))
    }

    fn loader_stream_init(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        if !self.0.read().init_fired {
            self.0.write(context.gc_context).init_fired = true;
            let mut init_evt = Event::new("init", EventData::Empty);
            init_evt.set_bubbles(false);
            init_evt.set_cancelable(false);

            if let Err(e) = Avm2::dispatch_event(context, init_evt, (*self).into()) {
                log::error!(
                    "Encountered AVM2 error when broadcasting `init` event: {}",
                    e
                );
            }
        }
    }

    /// Unwrap this object's loader stream
    fn as_loader_stream(&self) -> Option<Ref<LoaderStream<'gc>>> {
        if self.0.read().loaded_stream.is_some() {
            Some(Ref::map(self.0.read(), |v| {
                v.loaded_stream.as_ref().unwrap()
            }))
        } else {
            None
        }
    }
}
