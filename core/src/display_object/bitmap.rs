//! Bitmap display object

use crate::avm1::Object as Avm1Object;
use crate::avm2::{
    Activation as Avm2Activation, Avm2, BitmapDataObject as Avm2BitmapDataObject,
    ClassObject as Avm2ClassObject, FunctionArgs as Avm2FunctionArgs,
    StageObject as Avm2StageObject,
};
use crate::bitmap::bitmap_data::BitmapData;
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::{DisplayObjectBase, DisplayObjectPtr, DisplayObjectWeak};
use crate::prelude::*;
use crate::tag_utils::SwfMovie;
use crate::vminterface::Instantiator;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::lock::Lock;
use gc_arena::{Collect, Gc, GcWeak, Mutation};
use ruffle_common::utils::HasPrefixField;
use ruffle_render::bitmap::{BitmapFormat, PixelSnapping};
use std::cell::Cell;
use std::sync::Arc;

#[derive(Clone, Debug, Collect, Copy)]
#[collect(no_drop)]
pub struct BitmapWeak<'gc>(GcWeak<'gc, BitmapGraphicData<'gc>>);

impl<'gc> BitmapWeak<'gc> {
    pub fn upgrade(self, mc: &Mutation<'gc>) -> Option<Bitmap<'gc>> {
        self.0.upgrade(mc).map(Bitmap)
    }

    pub fn as_ptr(self) -> *const DisplayObjectPtr {
        self.0.as_ptr() as *const DisplayObjectPtr
    }
}

/// The AVM2 class for the Bitmap associated with this object.
///
/// Bitmaps may be associated with either a `Bitmap` or a `BitmapData`
/// subclass. Its superclass determines how the Bitmap will be constructed.
#[derive(Clone, Collect, Copy, Debug)]
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

impl<'gc> BitmapClass<'gc> {
    pub fn from_class_object(
        class: Avm2ClassObject<'gc>,
        context: &mut UpdateContext<'gc>,
    ) -> Option<Self> {
        let class_definition = class.inner_class_definition();
        if class_definition.has_class_in_chain(context.avm2.class_defs().bitmap) {
            Some(BitmapClass::Bitmap(class))
        } else if class_definition.has_class_in_chain(context.avm2.class_defs().bitmapdata) {
            Some(BitmapClass::BitmapData(class))
        } else {
            None
        }
    }
}

/// A Bitmap display object is a raw bitmap on the stage.
/// This can only be instanitated on the display list in SWFv9 AVM2 files.
/// In AVM1, this is only a library symbol that is referenced by `Graphic`.
/// Normally bitmaps are drawn in Flash as part of a Shape tag (`Graphic`),
/// but starting in AVM2, a raw `Bitmap` display object can be created
/// with the `PlaceObject3` tag.
/// It can also be created in ActionScript using the `Bitmap` class.
#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct Bitmap<'gc>(Gc<'gc, BitmapGraphicData<'gc>>);

impl fmt::Debug for Bitmap<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bitmap")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Clone, Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct BitmapGraphicData<'gc> {
    base: DisplayObjectBase<'gc>,
    movie: Arc<SwfMovie>,

    /// The AVM2 side of this object.
    ///
    /// AVM1 code cannot directly reference `Bitmap`s, so this does not support
    /// storing an AVM1 object.
    avm2_object: Lock<Option<Avm2StageObject<'gc>>>,

    /// The class associated with this Bitmap.
    avm2_bitmap_class: Lock<BitmapClass<'gc>>,

    /// The current bitmap data object.
    bitmap_data: Lock<BitmapData<'gc>>,

    /// The width and height values are cached from the BitmapData
    /// when this Bitmap instance is first created,
    /// and continue to be reported even if the BitmapData is disposed.
    width: Cell<u32>,
    height: Cell<u32>,

    id: CharacterId,

    /// Whether or not bitmap smoothing is enabled.
    smoothing: Cell<bool>,

    /// How to snap this bitmap to the pixel grid
    pixel_snapping: Cell<PixelSnapping>,
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
        mc: &Mutation<'gc>,
        id: CharacterId,
        bitmap_data: BitmapData<'gc>,
        smoothing: bool,
        movie: &Arc<SwfMovie>,
    ) -> Self {
        // NOTE: We do *not* solicit a handle from the `bitmap_data` at this
        // time due to mutable borrowing issues.

        let width = bitmap_data.width();
        let height = bitmap_data.height();

        let bitmap = Bitmap(Gc::new(
            mc,
            BitmapGraphicData {
                base: Default::default(),
                id,
                bitmap_data: Lock::new(bitmap_data),
                width: Cell::new(width),
                height: Cell::new(height),
                smoothing: Cell::new(smoothing),
                pixel_snapping: Cell::new(PixelSnapping::Auto),
                avm2_object: Lock::new(None),
                avm2_bitmap_class: Lock::new(BitmapClass::NoSubclass),
                movie: movie.clone(),
            },
        ));

        bitmap_data.add_display_object(mc, DisplayObjectWeak::Bitmap(bitmap.downgrade()));

        bitmap
    }

    /// Create a `Bitmap` with static bitmap data only.
    pub fn new(
        mc: &Mutation<'gc>,
        id: CharacterId,
        bitmap: ruffle_render::bitmap::Bitmap,
        movie: Arc<SwfMovie>,
    ) -> Self {
        let width = bitmap.width();
        let height = bitmap.height();
        let transparency = match bitmap.format() {
            BitmapFormat::Rgba => true,
            BitmapFormat::Rgb => false,
            _ => unreachable!(
                "Bitmap objects can only be constructed from RGB or RGBA source bitmaps"
            ),
        };
        let pixels: Vec<_> = bitmap
            .as_colors()
            .map(crate::bitmap::bitmap_data::Color::from)
            .collect();
        let bitmap_data = BitmapData::new_with_pixels(mc, width, height, transparency, pixels);

        let smoothing = true;
        Self::new_with_bitmap_data(mc, id, bitmap_data, smoothing, &movie)
    }

    // Important - we read 'width' and 'height' from the cached
    // values on this object. See the definition of these fields
    // for more information
    pub fn bitmap_width(self) -> u16 {
        self.0.width.get() as u16
    }

    pub fn bitmap_height(self) -> u16 {
        self.0.height.get() as u16
    }

    pub fn pixel_snapping(self) -> PixelSnapping {
        self.0.pixel_snapping.get()
    }

    pub fn set_pixel_snapping(self, value: PixelSnapping) {
        self.0.pixel_snapping.set(value);
    }

    pub fn bitmap_data(self) -> BitmapData<'gc> {
        self.0.bitmap_data.get()
    }

    /// Associate this `Bitmap` with new `BitmapData`.
    ///
    /// Once associated with the new data, the reported width, height, and
    /// bitmap handle of this display object will change to match the given
    /// bitmap data.
    ///
    /// This also forces the `BitmapData` to be sent to the rendering backend,
    /// if that has not already been done.
    pub fn set_bitmap_data(self, context: &mut UpdateContext<'gc>, bitmap_data: BitmapData<'gc>) {
        let weak_self = DisplayObjectWeak::Bitmap(self.downgrade());

        self.0
            .bitmap_data
            .get()
            .remove_display_object(context.gc(), weak_self);

        // Refresh our cached values, even if we're writing the same BitmapData
        // that we currently have stored. This will update them to '0' if the
        // BitmapData has been disposed since it was originally set.
        self.0.width.set(bitmap_data.width());
        self.0.height.set(bitmap_data.height());
        unlock!(
            Gc::write(context.gc(), self.0),
            BitmapGraphicData,
            bitmap_data
        )
        .set(bitmap_data);

        bitmap_data.add_display_object(context.gc(), weak_self);
    }

    pub fn avm2_bitmapdata_class(self) -> Option<Avm2ClassObject<'gc>> {
        match self.0.avm2_bitmap_class.get() {
            BitmapClass::BitmapData(c) => Some(c),
            _ => None,
        }
    }

    pub fn avm2_bitmap_class(self) -> Option<Avm2ClassObject<'gc>> {
        match self.0.avm2_bitmap_class.get() {
            BitmapClass::Bitmap(c) => Some(c),
            _ => None,
        }
    }

    pub fn set_avm2_bitmapdata_class(self, mc: &Mutation<'gc>, class: BitmapClass<'gc>) {
        unlock!(Gc::write(mc, self.0), BitmapGraphicData, avm2_bitmap_class).set(class);
    }

    fn set_avm2_object(self, mc: &Mutation<'gc>, object: Option<Avm2StageObject<'gc>>) {
        unlock!(Gc::write(mc, self.0), BitmapGraphicData, avm2_object).set(object);
    }

    pub fn smoothing(self) -> bool {
        self.0.smoothing.get()
    }

    pub fn set_smoothing(self, smoothing: bool) {
        self.0.smoothing.set(smoothing);
    }

    pub fn downgrade(self) -> BitmapWeak<'gc> {
        BitmapWeak(Gc::downgrade(self.0))
    }
}

impl<'gc> TDisplayObject<'gc> for Bitmap<'gc> {
    fn base(self) -> Gc<'gc, DisplayObjectBase<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }

    fn instantiate(self, gc_context: &Mutation<'gc>) -> DisplayObject<'gc> {
        Self(Gc::new(gc_context, self.0.as_ref().clone())).into()
    }

    fn id(self) -> CharacterId {
        self.0.id
    }

    fn self_bounds(self) -> Rectangle<Twips> {
        Rectangle {
            x_min: Twips::ZERO,
            y_min: Twips::ZERO,
            x_max: Twips::from_pixels(self.bitmap_width().into()),
            y_max: Twips::from_pixels(self.bitmap_height().into()),
        }
    }

    fn post_instantiation(
        self,
        context: &mut UpdateContext<'gc>,
        _init_object: Option<Avm1Object<'gc>>,
        instantiated_by: Instantiator,
        _run_frame: bool,
    ) {
        let mc = context.gc();

        if self.movie().is_action_script_3() {
            self.set_default_instance_name(context);

            if !instantiated_by.is_avm() {
                let bitmap_cls = self
                    .avm2_bitmap_class()
                    .unwrap_or_else(|| context.avm2.classes().bitmap);
                let bitmapdata_cls = self
                    .avm2_bitmapdata_class()
                    .unwrap_or_else(|| context.avm2.classes().bitmapdata);

                let mut activation = Avm2Activation::from_nothing(context);

                let bitmap_obj =
                    Avm2StageObject::for_display_object(activation.gc(), self.into(), bitmap_cls);

                let call_result = bitmap_cls.call_init(
                    bitmap_obj.into(),
                    Avm2FunctionArgs::empty(),
                    &mut activation,
                );
                if let Err(err) = call_result {
                    Avm2::uncaught_error(
                        &mut activation,
                        Some(self.into()),
                        err,
                        "Error running AVM2 construction for bitmap",
                    );
                };

                self.set_avm2_object(activation.gc(), Some(bitmap_obj));

                // Use a dummy BitmapData when calling the constructor on the user subclass
                // - the constructor should see an invalid BitmapData before calling 'super',
                // even if it's linked to an image.

                let bitmap_data_obj = Avm2BitmapDataObject::from_bitmap_data_and_class(
                    activation.gc(),
                    BitmapData::dummy(mc),
                    bitmapdata_cls,
                );

                // We call the custom BitmapData class with width and height...
                // but, it always seems to be 1 in Flash Player when constructed
                // from timeline? This will not actually cause us to create a
                // BitmapData with dimensions (1, 1) - when the custom class
                // makes a super() call, the BitmapData constructor will load
                // in the real data from the linked SymbolClass.
                let args = &[1.into(), 1.into()];
                let call_result = bitmapdata_cls.call_init(
                    bitmap_data_obj.into(),
                    Avm2FunctionArgs::from_slice(args),
                    &mut activation,
                );
                if let Err(err) = call_result {
                    Avm2::uncaught_error(
                        &mut activation,
                        Some(self.into()),
                        err,
                        "Error running AVM2 construction for bitmap data",
                    );
                }

                self.set_bitmap_data(activation.context, bitmap_data_obj.get_bitmap_data());
            }

            self.on_construction_complete(context);
        }
    }

    fn render_self(self, context: &mut RenderContext<'_, 'gc>) {
        if !context.is_offscreen && !self.world_bounds().intersects(&context.stage.view_bounds()) {
            // Off-screen; culled
            return;
        }

        self.0.bitmap_data.get().render(
            self.0.smoothing.get(),
            context,
            self.0.pixel_snapping.get(),
        );
    }

    fn object1(self) -> Option<crate::avm1::Object<'gc>> {
        None
    }

    fn object2(self) -> Option<Avm2StageObject<'gc>> {
        self.0.avm2_object.get()
    }

    fn set_object2(self, context: &mut UpdateContext<'gc>, to: Avm2StageObject<'gc>) {
        self.set_avm2_object(context.gc(), Some(to));
    }

    fn movie(self) -> Arc<SwfMovie> {
        self.0.movie.clone()
    }
}
