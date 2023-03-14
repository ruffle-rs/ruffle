//! Bitmap display object

use crate::avm1;
use crate::avm2::{
    Activation as Avm2Activation, ClassObject as Avm2ClassObject, Object as Avm2Object,
    StageObject as Avm2StageObject, Value as Avm2Value,
};
use crate::bitmap::bitmap_data::BitmapDataWrapper;
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::{DisplayObjectBase, DisplayObjectPtr, TDisplayObject};
use crate::prelude::*;
use crate::tag_utils::SwfMovie;
use crate::vminterface::Instantiator;
use core::fmt;
use gc_arena::{Collect, GcCell, MutationContext};
use ruffle_render::bitmap::BitmapFormat;
use std::cell::{Ref, RefMut};
use std::sync::Arc;

/// The AVM2 class for the Bitmap associated with this object.
///
/// Bitmaps may be associated with either a `Bitmap` or a `BitmapData`
/// subclass. Its superclass determines how the Bitmap will be constructed.
#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub enum BitmapClass<'gc> {
    /// This Bitmap uses the stock Flash Player classes for itself.
    NoSubclass,

    /// This Bitmap overrides its `Bitmap` class and holds a stock `BitmapData`
    /// with its pixel data.
    ///
    /// This is the normal symbol class association for Flex image embeds.
    /// Adobe Animate does not support compiling Bitmaps with `Bitmap`
    /// subclasses (as of version 2022).
    Bitmap(Avm2ClassObject<'gc>),

    /// This Bitmap uses the stock `Bitmap` class with a custom `BitmapData`
    /// subclass to hold its pixel data.
    ///
    /// This is the normal symbol class association for Adobe Animate image
    /// embeds.
    BitmapData(Avm2ClassObject<'gc>),
}

/// A Bitmap display object is a raw bitamp on the stage.
/// This can only be instanitated on the display list in SWFv9 AVM2 files.
/// In AVM1, this is only a library symbol that is referenced by `Graphic`.
/// Normally bitmaps are drawn in Flash as part of a Shape tag (`Graphic`),
/// but starting in AVM2, a raw `Bitmap` display object can be created
/// with the `PlaceObject3` tag.
/// It can also be created in ActionScript using the `Bitmap` class.
#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct Bitmap<'gc>(GcCell<'gc, BitmapData<'gc>>);

impl fmt::Debug for Bitmap<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bitmap")
            .field("ptr", &self.0.as_ptr())
            .finish()
    }
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct BitmapData<'gc> {
    base: DisplayObjectBase<'gc>,
    id: CharacterId,
    movie: Arc<SwfMovie>,

    /// The current bitmap data object.
    bitmap_data: BitmapDataWrapper<'gc>,

    /// The width and height values are cached from the BitmapDataWrapper
    /// when this Bitmap instance is first created,
    /// and continue to be reported even if the BitmapData is disposed.
    width: u32,
    height: u32,

    /// Whether or not bitmap smoothing is enabled.
    smoothing: bool,

    /// The AVM2 side of this object.
    ///
    /// AVM1 code cannot directly reference `Bitmap`s, so this does not support
    /// storing an AVM1 object.
    avm2_object: Option<Avm2Object<'gc>>,

    /// The class associated with this Bitmap.
    avm2_bitmap_class: BitmapClass<'gc>,
}

impl<'gc> Bitmap<'gc> {
    /// Create a `Bitmap` with dynamic bitmap data.
    ///
    /// If `bitmap_data` is provided, the associated `bitmap_handle` must match
    /// the same handle that the data has provided. If it does not match, then
    /// this `Bitmap` will render the wrong data when added to the display
    /// list. If no data is provided then you are free to add whatever handle
    /// you like.
    pub fn new_with_bitmap_data(
        context: &mut UpdateContext<'_, 'gc>,
        id: CharacterId,
        bitmap_data: GcCell<'gc, crate::bitmap::bitmap_data::BitmapData<'gc>>,
        smoothing: bool,
    ) -> Self {
        //NOTE: We do *not* solicit a handle from the `bitmap_data` at this
        //time due to mutable borrowing issues.

        let width = bitmap_data.read().width();
        let height = bitmap_data.read().height();

        Bitmap(GcCell::allocate(
            context.gc_context,
            BitmapData {
                base: Default::default(),
                id,
                bitmap_data: BitmapDataWrapper::new(bitmap_data),
                width,
                height,
                smoothing,
                avm2_object: None,
                avm2_bitmap_class: BitmapClass::NoSubclass,
                movie: context.swf.clone(),
            },
        ))
    }

    /// Create a `Bitmap` with static bitmap data only.
    pub fn new(
        context: &mut UpdateContext<'_, 'gc>,
        id: CharacterId,
        bitmap: ruffle_render::bitmap::Bitmap,
    ) -> Result<Self, ruffle_render::error::Error> {
        let width = bitmap.width();
        let height = bitmap.height();
        let pixels: Vec<_> = bitmap
            .as_colors()
            .map(crate::bitmap::bitmap_data::Color::from)
            .collect();
        let mut bitmap_data = crate::bitmap::bitmap_data::BitmapData::default();
        bitmap_data.set_pixels(
            width,
            height,
            match bitmap.format() {
                BitmapFormat::Rgba => true,
                BitmapFormat::Rgb => false,
            },
            pixels,
        );
        let bitmap_data = GcCell::allocate(context.gc_context, bitmap_data);

        let smoothing = true;
        Ok(Self::new_with_bitmap_data(
            context,
            id,
            bitmap_data,
            smoothing,
        ))
    }

    // Important - we read 'width' and 'height' from the cached
    // values on this object. See the definition of these fields
    // for more information
    pub fn width(self) -> u16 {
        self.0.read().width as u16
    }

    pub fn height(self) -> u16 {
        self.0.read().height as u16
    }

    pub fn bitmap_data_wrapper(self) -> BitmapDataWrapper<'gc> {
        self.0.read().bitmap_data
    }

    /// Retrieve the bitmap data associated with this `Bitmap`.
    pub fn bitmap_data(self) -> GcCell<'gc, crate::bitmap::bitmap_data::BitmapData<'gc>> {
        self.0.read().bitmap_data.sync()
    }

    /// Associate this `Bitmap` with new `BitmapData`.
    ///
    /// Once associated with the new data, the reported width, height, and
    /// bitmap handle of this display object will change to match the given
    /// bitmap data.
    ///
    /// This also forces the `BitmapData` to be sent to the rendering backend,
    /// if that has not already been done.
    pub fn set_bitmap_data(
        self,
        context: &mut UpdateContext<'_, 'gc>,
        bitmap_data: GcCell<'gc, crate::bitmap::bitmap_data::BitmapData<'gc>>,
    ) {
        let mut write = self.0.write(context.gc_context);
        // Refresh our cached values, even if we're writing the same BitmapData
        // that we currently have stored. This will update them to '0' if the
        // BitmapData has been disposed since it was originally set.
        write.width = bitmap_data.read().width();
        write.height = bitmap_data.read().height();
        write.bitmap_data = BitmapDataWrapper::new(bitmap_data);
    }

    pub fn avm2_bitmapdata_class(self) -> Option<Avm2ClassObject<'gc>> {
        match self.0.read().avm2_bitmap_class {
            BitmapClass::BitmapData(c) => Some(c),
            _ => None,
        }
    }

    pub fn avm2_bitmap_class(self) -> Option<Avm2ClassObject<'gc>> {
        match self.0.read().avm2_bitmap_class {
            BitmapClass::Bitmap(c) => Some(c),
            _ => None,
        }
    }

    pub fn set_avm2_bitmapdata_class(
        self,
        context: &mut UpdateContext<'_, 'gc>,
        class: Avm2ClassObject<'gc>,
    ) {
        let bitmap_class = if class.has_class_in_chain(context.avm2.classes().bitmap) {
            BitmapClass::Bitmap(class)
        } else if class.has_class_in_chain(context.avm2.classes().bitmapdata) {
            BitmapClass::BitmapData(class)
        } else {
            return tracing::error!("Associated class {:?} for symbol {} must extend flash.display.Bitmap or BitmapData, does neither", class.inner_class_definition().read().name(), self.id());
        };

        self.0.write(context.gc_context).avm2_bitmap_class = bitmap_class;
    }

    pub fn smoothing(self) -> bool {
        self.0.read().smoothing
    }

    pub fn set_smoothing(self, mc: MutationContext<'gc, '_>, smoothing: bool) {
        self.0.write(mc).smoothing = smoothing;
    }
}

impl<'gc> TDisplayObject<'gc> for Bitmap<'gc> {
    fn base(&self) -> Ref<DisplayObjectBase<'gc>> {
        Ref::map(self.0.read(), |r| &r.base)
    }

    fn base_mut<'a>(&'a self, mc: MutationContext<'gc, '_>) -> RefMut<'a, DisplayObjectBase<'gc>> {
        RefMut::map(self.0.write(mc), |w| &mut w.base)
    }

    fn instantiate(&self, gc_context: MutationContext<'gc, '_>) -> DisplayObject<'gc> {
        Self(GcCell::allocate(gc_context, self.0.read().clone())).into()
    }

    fn as_ptr(&self) -> *const DisplayObjectPtr {
        self.0.as_ptr() as *const DisplayObjectPtr
    }

    fn id(&self) -> CharacterId {
        self.0.read().id
    }

    fn self_bounds(&self) -> Rectangle<Twips> {
        Rectangle {
            x_min: Twips::ZERO,
            y_min: Twips::ZERO,
            x_max: Twips::from_pixels(Bitmap::width(*self).into()),
            y_max: Twips::from_pixels(Bitmap::height(*self).into()),
        }
    }

    fn post_instantiation(
        &self,
        context: &mut UpdateContext<'_, 'gc>,
        _init_object: Option<avm1::Object<'gc>>,
        _instantiated_by: Instantiator,
        run_frame: bool,
    ) {
        if context.is_action_script_3() {
            let mut activation = Avm2Activation::from_nothing(context.reborrow());
            let bitmap = self
                .avm2_bitmap_class()
                .unwrap_or_else(|| activation.context.avm2.classes().bitmap);
            match Avm2StageObject::for_display_object_childless(
                &mut activation,
                (*self).into(),
                bitmap,
            ) {
                Ok(object) => {
                    self.0.write(activation.context.gc_context).avm2_object = Some(object.into())
                }
                Err(e) => tracing::error!("Got error when creating AVM2 side of bitmap: {}", e),
            }

            self.on_construction_complete(context);
        } else {
            context
                .avm1
                .add_to_exec_list(context.gc_context, (*self).into());

            if run_frame {
                self.run_frame_avm1(context);
            }
        }
    }

    fn render_self(&self, context: &mut RenderContext<'_, 'gc>) {
        if !context.is_offscreen && !self.world_bounds().intersects(&context.stage.view_bounds()) {
            // Off-screen; culled
            return;
        }

        let bitmap_data = self.0.read();
        bitmap_data
            .bitmap_data
            .render(bitmap_data.smoothing, context);
    }

    fn object2(&self) -> Avm2Value<'gc> {
        self.0
            .read()
            .avm2_object
            .map(|o| o.into())
            .unwrap_or(Avm2Value::Null)
    }

    fn set_object2(&self, context: &mut UpdateContext<'_, 'gc>, to: Avm2Object<'gc>) {
        self.0.write(context.gc_context).avm2_object = Some(to);
    }

    fn as_bitmap(self) -> Option<Bitmap<'gc>> {
        Some(self)
    }

    fn movie(&self) -> Arc<SwfMovie> {
        self.0.read().movie.clone()
    }
}
