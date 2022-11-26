//! Bitmap display object

use crate::avm1;
use crate::avm2::{
    Activation as Avm2Activation, ClassObject as Avm2ClassObject, Object as Avm2Object,
    StageObject as Avm2StageObject, Value as Avm2Value,
};
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::{DisplayObjectBase, DisplayObjectPtr, TDisplayObject};
use crate::prelude::*;
use crate::vminterface::Instantiator;
use gc_arena::{Collect, Gc, GcCell, MutationContext};
use ruffle_render::bitmap::BitmapHandle;
use ruffle_render::commands::CommandHandler;
use std::cell::{Ref, RefMut};

/// The AVM2 class for the Bitmap associated with this object.
///
/// Bitmaps may be associated with either a `Bitmap` or a `BitmapData`
/// subclass. Its superclass determines how the Bitmap will be constructed.
#[derive(Clone, Debug, Collect, Copy)]
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
#[derive(Clone, Debug, Collect, Copy)]
#[collect(no_drop)]
pub struct Bitmap<'gc>(GcCell<'gc, BitmapData<'gc>>);

#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct BitmapData<'gc> {
    base: DisplayObjectBase<'gc>,
    static_data: Gc<'gc, BitmapStatic>,

    /// The current bitmap data object.
    bitmap_data: Option<GcCell<'gc, crate::bitmap::bitmap_data::BitmapData<'gc>>>,

    /// The current bitmap handle.
    ///
    /// This needs to be cached separately from the associated bitmap data so
    /// that it can be accessed without a mutation context.
    ///
    /// If this is `None`, then the bitmap does not render anything.
    #[collect(require_static)]
    bitmap_handle: Option<BitmapHandle>,

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
        context: &mut UpdateContext<'_, 'gc, '_>,
        id: CharacterId,
        bitmap_handle: Option<BitmapHandle>,
        width: u16,
        height: u16,
        bitmap_data: Option<GcCell<'gc, crate::bitmap::bitmap_data::BitmapData<'gc>>>,
        smoothing: bool,
    ) -> Self {
        //NOTE: We do *not* solicit a handle from the `bitmap_data` at this
        //time due to mutable borrowing issues.

        Bitmap(GcCell::allocate(
            context.gc_context,
            BitmapData {
                base: Default::default(),
                static_data: Gc::allocate(context.gc_context, BitmapStatic { id, width, height }),
                bitmap_data,
                bitmap_handle,
                smoothing,
                avm2_object: None,
                avm2_bitmap_class: BitmapClass::NoSubclass,
            },
        ))
    }

    /// Create a `Bitmap` with static bitmap data only.
    pub fn new(
        context: &mut UpdateContext<'_, 'gc, '_>,
        id: CharacterId,
        bitmap: ruffle_render::bitmap::Bitmap,
    ) -> Result<Self, ruffle_render::error::Error> {
        let width = bitmap.width() as u16;
        let height = bitmap.height() as u16;
        let bitmap_handle = context.renderer.register_bitmap(bitmap)?;
        let bitmap_data = None;
        let smoothing = true;
        Ok(Self::new_with_bitmap_data(
            context,
            id,
            Some(bitmap_handle),
            width,
            height,
            bitmap_data,
            smoothing,
        ))
    }

    #[allow(dead_code)]
    pub fn bitmap_handle(self) -> Option<BitmapHandle> {
        self.0.read().bitmap_handle
    }

    pub fn width(self) -> u16 {
        let read = self.0.read();

        read.bitmap_data
            .map(|bd| bd.read().width() as u16)
            .unwrap_or_else(|| read.static_data.width)
    }

    pub fn height(self) -> u16 {
        let read = self.0.read();

        read.bitmap_data
            .map(|bd| bd.read().height() as u16)
            .unwrap_or_else(|| read.static_data.height)
    }

    /// Retrieve the bitmap data associated with this `Bitmap`.
    pub fn bitmap_data(self) -> Option<GcCell<'gc, crate::bitmap::bitmap_data::BitmapData<'gc>>> {
        self.0.read().bitmap_data
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
        context: &mut UpdateContext<'_, 'gc, '_>,
        bitmap_data: Option<GcCell<'gc, crate::bitmap::bitmap_data::BitmapData<'gc>>>,
    ) {
        if let Some(bitmap_data) = bitmap_data {
            let bitmap_handle = bitmap_data
                .write(context.gc_context)
                .bitmap_handle(context.renderer);

            let mut write = self.0.write(context.gc_context);

            write.bitmap_data = Some(bitmap_data);
            if let Some(bitmap_handle) = bitmap_handle {
                write.bitmap_handle = Some(bitmap_handle);
            }
        } else {
            let mut write = self.0.write(context.gc_context);

            write.bitmap_data = None;
            write.bitmap_handle = None;
        }
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
        context: &mut UpdateContext<'_, 'gc, '_>,
        class: Avm2ClassObject<'gc>,
    ) {
        let bitmap_class = if class.has_class_in_chain(context.avm2.classes().bitmap) {
            BitmapClass::Bitmap(class)
        } else if class.has_class_in_chain(context.avm2.classes().bitmapdata) {
            BitmapClass::BitmapData(class)
        } else {
            return log::error!("Associated class {:?} for symbol {} must extend flash.display.Bitmap or BitmapData, does neither", class.inner_class_definition().read().name(), self.id());
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
        self.0.read().static_data.id
    }

    fn self_bounds(&self) -> BoundingBox {
        BoundingBox {
            x_min: Twips::ZERO,
            y_min: Twips::ZERO,
            x_max: Twips::from_pixels(Bitmap::width(*self).into()),
            y_max: Twips::from_pixels(Bitmap::height(*self).into()),
            valid: true,
        }
    }

    fn post_instantiation(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
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
                Err(e) => log::error!("Got error when creating AVM2 side of bitmap: {}", e),
            }
        } else {
            context
                .avm1
                .add_to_exec_list(context.gc_context, (*self).into());
        }

        if run_frame {
            self.run_frame(context);
        }
    }

    fn render_self(&self, context: &mut RenderContext<'_, 'gc, '_>) {
        if !context.is_offscreen && !self.world_bounds().intersects(&context.stage.view_bounds()) {
            // Off-screen; culled
            return;
        }

        let bitmap_data = self.0.read();
        if let Some(bitmap_handle) = bitmap_data.bitmap_handle {
            if let Some(inner_bitmap_data) = bitmap_data.bitmap_data {
                if let Ok(mut bd) = inner_bitmap_data.try_write(context.gc_context) {
                    bd.update_dirty_texture(context);
                } else {
                    return; // bail, this is caused by recursive render attempt. TODO: support this.
                };
            }

            context.commands.render_bitmap(
                bitmap_handle,
                context.transform_stack.transform(),
                bitmap_data.smoothing,
            );
        }
    }

    fn object2(&self) -> Avm2Value<'gc> {
        self.0
            .read()
            .avm2_object
            .map(|o| o.into())
            .unwrap_or(Avm2Value::Undefined)
    }

    fn set_object2(&mut self, mc: MutationContext<'gc, '_>, to: Avm2Object<'gc>) {
        self.0.write(mc).avm2_object = Some(to);
    }

    fn as_bitmap(self) -> Option<Bitmap<'gc>> {
        Some(self)
    }
}

/// Static data shared between all instances of a bitmap.
#[derive(Clone, Collect)]
#[collect(no_drop)]
struct BitmapStatic {
    id: CharacterId,
    width: u16,
    height: u16,
}
