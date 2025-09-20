use crate::avm1::{
    ActivationIdentifier as Avm1ActivationIdentifier, Object as Avm1Object, Value as Avm1Value,
};
use crate::avm2::{
    Activation as Avm2Activation, Avm2, Error as Avm2Error, EventObject as Avm2EventObject,
    LoaderInfoObject, Multiname as Avm2Multiname, Object as Avm2Object,
    StageObject as Avm2StageObject, TObject as _, Value as Avm2Value,
};
use crate::context::{RenderContext, UpdateContext};
use crate::drawing::Drawing;
use crate::loader::LoadManager;
use crate::prelude::*;
use crate::string::{AvmString, WString};
use crate::tag_utils::SwfMovie;
use crate::types::{Degrees, Percent};
use crate::vminterface::Instantiator;
use bitflags::bitflags;
use gc_arena::barrier::{unlock, Write};
use gc_arena::lock::Lock;
use gc_arena::{Collect, Gc, Mutation};
use ruffle_macros::{enum_trait_object, istr};
use ruffle_render::perspective_projection::PerspectiveProjection;
use ruffle_render::pixel_bender::PixelBenderShaderHandle;
use ruffle_render::transform::{Transform, TransformStack};
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use swf::{ColorTransform, Fixed8};

mod avm1_button;
mod avm2_button;
mod bitmap;
mod container;
mod edit_text;
mod graphic;
mod interactive;
mod loader_display;
mod morph_shape;
mod movie_clip;
mod stage;
mod text;
mod video;

use crate::avm1::Activation;
use crate::display_object::bitmap::BitmapWeak;
pub use crate::display_object::container::{
    dispatch_added_event_only, dispatch_added_to_stage_event_only, DisplayObjectContainer,
    TDisplayObjectContainer,
};
pub use avm1_button::{Avm1Button, ButtonState, ButtonTracking};
pub use avm2_button::Avm2Button;
pub use bitmap::{Bitmap, BitmapClass};
#[allow(unused)]
pub use edit_text::LayoutDebugBoxesFlag;
pub use edit_text::{AutoSizeMode, EditText, TextSelection};
pub use graphic::Graphic;
pub use interactive::{Avm2MousePick, InteractiveObject, TInteractiveObject};
pub use loader_display::LoaderDisplay;
pub use morph_shape::MorphShape;
pub use movie_clip::{MovieClip, MovieClipWeak, Scene};
use ruffle_render::backend::{BitmapCacheEntry, RenderBackend};
use ruffle_render::bitmap::{BitmapHandle, BitmapInfo, PixelSnapping};
use ruffle_render::blend::ExtendedBlendMode;
use ruffle_render::commands::{CommandHandler, CommandList, RenderBlendMode};
use ruffle_render::filters::Filter;
pub use stage::{Stage, StageAlign, StageDisplayState, StageScaleMode, WindowMode};
pub use text::Text;
pub use video::Video;

use self::loader_display::LoaderDisplayWeak;

/// If a `DisplayObject` is marked `cacheAsBitmap` (via tag or AS),
/// this struct keeps the information required to uphold that cache.
/// A cached Display Object must have its bitmap invalidated when
/// any "visual" change happens, which can include:
/// - Changing the rotation
/// - Changing the scale
/// - Changing the alpha
/// - Changing the color transform
/// - Any "visual" change to children, **including** position changes
///
/// Position changes to the cached Display Object does not regenerate the cache,
/// allowing Display Objects to move freely without being regenerated.
///
/// Flash isn't very good at always recognising when it should be invalidated,
/// and there's cases such as changing the blend mode which don't always trigger it.
///
#[derive(Clone, Debug, Default)]
pub struct BitmapCache {
    /// The `Matrix.a` value that was last used with this cache
    matrix_a: f32,
    /// The `Matrix.b` value that was last used with this cache
    matrix_b: f32,
    /// The `Matrix.c` value that was last used with this cache
    matrix_c: f32,
    /// The `Matrix.d` value that was last used with this cache
    matrix_d: f32,

    /// The width of the original bitmap, pre-filters
    source_width: u16,

    /// The height of the original bitmap, pre-filters
    source_height: u16,

    /// The offset used to draw the final bitmap (i.e. if a filter increases the size)
    draw_offset: Point<i32>,

    /// The current contents of the cache, if any. Values are post-filters.
    bitmap: Option<BitmapInfo>,

    /// Whether we warned that this bitmap was too large to be cached
    warned_for_oversize: bool,
}

impl BitmapCache {
    /// Forcefully make this BitmapCache invalid and require regeneration.
    /// This should be used for changes that aren't automatically detected, such as children.
    pub fn make_dirty(&mut self) {
        // Setting the old transform to something invalid is a cheap way of making it invalid,
        // without reserving an extra field for.
        self.matrix_a = f32::NAN;
    }

    fn is_dirty(&self, other: &Matrix, source_width: u16, source_height: u16) -> bool {
        self.matrix_a != other.a
            || self.matrix_b != other.b
            || self.matrix_c != other.c
            || self.matrix_d != other.d
            || self.source_width != source_width
            || self.source_height != source_height
            || self.bitmap.is_none()
    }

    /// Clears any dirtiness and ensure there's an appropriately sized texture allocated
    #[expect(clippy::too_many_arguments)]
    fn update(
        &mut self,
        renderer: &mut dyn RenderBackend,
        matrix: Matrix,
        source_width: u16,
        source_height: u16,
        actual_width: u16,
        actual_height: u16,
        draw_offset: Point<i32>,
        swf_version: u8,
    ) {
        self.matrix_a = matrix.a;
        self.matrix_b = matrix.b;
        self.matrix_c = matrix.c;
        self.matrix_d = matrix.d;
        self.source_width = source_width;
        self.source_height = source_height;
        self.draw_offset = draw_offset;
        if let Some(current) = &mut self.bitmap {
            if current.width == actual_width && current.height == actual_height {
                return; // No need to resize it
            }
        }
        let acceptable_size = if swf_version > 9 {
            let total = actual_width as u32 * actual_height as u32;
            actual_width < 8191 && actual_height < 8191 && total < 16777215
        } else {
            actual_width < 2880 && actual_height < 2880
        };
        if renderer.is_offscreen_supported()
            && actual_width > 0
            && actual_height > 0
            && acceptable_size
        {
            let handle = renderer.create_empty_texture(actual_width as u32, actual_height as u32);
            self.bitmap = handle.ok().map(|handle| BitmapInfo {
                width: actual_width,
                height: actual_height,
                handle,
            });
        } else {
            self.bitmap = None;
        }
    }

    /// Explicitly clears the cached value and drops any resources.
    /// This should only be used in situations where you can't render to the cache and it needs to be
    /// temporarily disabled.
    fn clear(&mut self) {
        self.bitmap = None;
    }

    fn handle(&self) -> Option<BitmapHandle> {
        self.bitmap.as_ref().map(|b| b.handle.clone())
    }
}

#[derive(Clone)]
pub struct RenderOptions {
    /// Whether to skip rendering masks.
    ///
    /// Masks are usually skipped when rendering, but when e.g. rendering
    /// the mask itself, it can't be skipped.
    ///
    /// Masks are skipped by default.
    pub skip_masks: bool,

    /// Whether to apply object's base transform.
    ///
    /// For instance, when calling BitmapData.draw, object's transform is not
    /// applied.
    ///
    /// Transform is applied by default.
    pub apply_transform: bool,

    /// Whether to apply base transform's matrix when rendering.
    ///
    /// Sometimes we need to render an object without applying its matrix, but
    /// with applying other parts of its transform (e.g. color transform).
    /// This happens e.g. when rendering alpha masks.
    ///
    /// Matrix is applied by default.
    pub apply_matrix: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            apply_transform: true,
            skip_masks: true,
            apply_matrix: true,
        }
    }
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
// Ensure this always has the same alignment as its subclasses (needed for `Gc` casts).
#[repr(align(8))]
pub struct DisplayObjectBase<'gc> {
    cell: RefCell<DisplayObjectBaseMut>,
    parent: Lock<Option<DisplayObject<'gc>>>,
    place_frame: Cell<u16>,
    depth: Cell<Depth>,
    name: Lock<Option<AvmString<'gc>>>,
    clip_depth: Cell<Depth>,

    // The transform of this display object.
    // (Split into several fields for easier access)
    matrix: Cell<Matrix>,
    color_transform: Cell<ColorTransform>,
    perspective_projection: Cell<Option<PerspectiveProjection>>,
    tz: Cell<f64>,

    // Cached transform properties `_xscale`, `_yscale`, `_rotation`.
    // These are expensive to calculate, so they will be calculated and cached
    // when AS requests one of these properties.
    rotation: Cell<Degrees>,
    scale_x: Cell<Percent>,
    scale_y: Cell<Percent>,
    skew: Cell<f64>,

    /// The sound transform of sounds playing via this display object.
    sound_transform: Cell<SoundTransform>,

    /// The display object that we are being masked by.
    masker: Lock<Option<DisplayObject<'gc>>>,

    /// The display object we are currently masking.
    maskee: Lock<Option<DisplayObject<'gc>>>,

    meta_data: Lock<Option<Avm2Object<'gc>>>,

    /// The blend mode used when rendering this display object.
    /// Values other than the default `BlendMode::Normal` implicitly cause cache-as-bitmap behavior.
    blend_mode: Cell<ExtendedBlendMode>,

    #[collect(require_static)]

    /// The opaque background color of this display object.
    /// The bounding box of the display object will be filled with the given color. This also
    /// triggers cache-as-bitmap behavior. Only solid backgrounds are supported; the alpha channel
    /// is ignored.
    opaque_background: Cell<Option<Color>>,

    /// Bit flags for various display object properties.
    flags: Cell<DisplayObjectFlags>,

    /// The 'internal' scroll rect used for rendering and methods like 'localToGlobal'.
    /// This is updated from 'pre_render'
    scroll_rect: Cell<Option<Rectangle<Twips>>>,

    /// The 'next' scroll rect, which we will copy to 'scroll_rect' from 'pre_render'.
    /// This is used by the ActionScript 'DisplayObject.scrollRect' getter, which sees
    /// changes immediately (without needing wait for a render)
    next_scroll_rect: Cell<Rectangle<Twips>>,

    /// Rectangle used for 9-slice scaling (`DisplayObject.scale9grid`).
    scaling_grid: Cell<Rectangle<Twips>>,
}

#[derive(Clone)]
struct DisplayObjectBaseMut {
    filters: Box<[Filter]>,

    blend_shader: Option<PixelBenderShaderHandle>,

    /// If this Display Object should cacheAsBitmap - and if so, the cache itself.
    /// None means not cached, Some means cached.
    cache: Option<BitmapCache>,
}

impl Default for DisplayObjectBase<'_> {
    fn default() -> Self {
        Self {
            cell: RefCell::new(DisplayObjectBaseMut {
                filters: Default::default(),
                blend_shader: None,
                cache: None,
            }),
            parent: Default::default(),
            place_frame: Default::default(),
            depth: Default::default(),
            name: Lock::new(None),
            clip_depth: Default::default(),
            matrix: Default::default(),
            color_transform: Default::default(),
            perspective_projection: Default::default(),
            tz: Cell::new(0.0),
            rotation: Cell::new(Degrees::from_radians(0.0)),
            scale_x: Cell::new(Percent::from_unit(1.0)),
            scale_y: Cell::new(Percent::from_unit(1.0)),
            skew: Cell::new(0.0),
            masker: Lock::new(None),
            maskee: Lock::new(None),
            meta_data: Lock::new(None),
            sound_transform: Default::default(),
            blend_mode: Default::default(),
            opaque_background: Default::default(),
            flags: Cell::new(DisplayObjectFlags::VISIBLE),
            scroll_rect: Cell::new(None),
            next_scroll_rect: Default::default(),
            scaling_grid: Default::default(),
        }
    }
}

impl<'gc> DisplayObjectBase<'gc> {
    fn contains_flag(&self, flag: DisplayObjectFlags) -> bool {
        self.flags.get().contains(flag)
    }

    fn set_flag(&self, flag: DisplayObjectFlags, value: bool) {
        let mut flags = self.flags.get();
        flags.set(flag, value);
        self.flags.set(flags);
    }

    /// Reset all properties that would be adjusted by a movie load.
    fn reset_for_movie_load(&self) {
        let flags_to_keep = self.flags.get() & DisplayObjectFlags::LOCK_ROOT;
        self.flags.set(flags_to_keep | DisplayObjectFlags::VISIBLE);
    }

    fn depth(&self) -> Depth {
        self.depth.get()
    }

    fn set_depth(&self, depth: Depth) {
        self.depth.set(depth);
    }

    fn place_frame(&self) -> u16 {
        self.place_frame.get()
    }

    fn set_place_frame(&self, frame: u16) {
        self.place_frame.set(frame);
    }

    fn transform(&self, apply_matrix: bool) -> Transform {
        Transform {
            matrix: if apply_matrix {
                self.matrix.get()
            } else {
                Matrix::IDENTITY
            },
            color_transform: self.color_transform.get(),
            perspective_projection: self.perspective_projection.get(),
            tz: self.tz.get(),
        }
    }

    pub fn matrix(&self) -> Matrix {
        self.matrix.get()
    }

    pub fn set_matrix(&self, matrix: Matrix) {
        self.matrix.set(matrix);
        self.set_scale_rotation_cached(false);
    }

    pub fn color_transform(&self) -> ColorTransform {
        self.color_transform.get()
    }

    pub fn set_color_transform(&self, color_transform: ColorTransform) {
        self.color_transform.set(color_transform);
    }

    pub fn perspective_projection(&self) -> Option<PerspectiveProjection> {
        self.perspective_projection.get()
    }

    pub fn set_perspective_projection(
        &self,
        perspective_projection: Option<PerspectiveProjection>,
    ) -> bool {
        let old = self.perspective_projection.replace(perspective_projection);
        perspective_projection != old
    }

    fn x(&self) -> Twips {
        self.matrix.get().tx
    }

    fn set_x(&self, x: Twips) -> bool {
        let mut matrix = self.matrix.get();
        let changed = matrix.tx != x;
        matrix.tx = x;
        self.matrix.set(matrix);
        self.set_transformed_by_script(true);
        changed
    }

    fn y(&self) -> Twips {
        self.matrix.get().ty
    }

    fn set_y(&self, y: Twips) -> bool {
        let mut matrix = self.matrix.get();
        let changed = matrix.ty != y;
        matrix.ty = y;
        self.matrix.set(matrix);
        self.set_transformed_by_script(true);
        changed
    }

    fn z(&self) -> f64 {
        self.tz.get()
    }

    fn set_z(&self, tz: f64) -> bool {
        let changed = self.tz.get() != tz;
        self.set_transformed_by_script(true);
        self.tz.set(tz);
        changed
    }

    /// Caches the scale and rotation factors for this display object, if necessary.
    /// Calculating these requires heavy trig ops, so we only do it when `_xscale`, `_yscale` or
    /// `_rotation` is accessed.
    fn cache_scale_rotation(&self) {
        if !self.scale_rotation_cached() {
            let Matrix { a, b, c, d, .. } = self.matrix.get();
            let a = f64::from(a);
            let b = f64::from(b);
            let c = f64::from(c);
            let d = f64::from(d);

            // If this object's transform matrix is:
            // [[a c tx]
            //  [b d ty]]
            // After transformation, the X-axis and Y-axis will turn into the column vectors x' = <a, b> and y' = <c, d>.
            // We derive the scale, rotation, and skew values from these transformed axes.
            // The skew value is not exposed by ActionScript, but is remembered internally.
            // xscale = len(x')
            // yscale = len(y')
            // rotation = atan2(b, a)  (the rotation of x' from the normal x-axis).
            // skew = atan2(-c, d) - atan2(b, a)  (the signed difference between y' and x' rotation)

            // This can produce some surprising results due to the overlap between flipping/rotation/skewing.
            // For example, in Flash, using Modify->Transform->Flip Horizontal and then tracing _xscale, _yscale, and _rotation
            // will output 100, 100, and 180. (a horizontal flip could also be a 180 degree skew followed by 180 degree rotation!)
            let rotation_x = f64::atan2(b, a);
            let rotation_y = f64::atan2(-c, d);
            let scale_x = f64::sqrt(a * a + b * b);
            let scale_y = f64::sqrt(c * c + d * d);
            self.rotation.set(Degrees::from_radians(rotation_x));
            self.scale_x.set(Percent::from_unit(scale_x));
            self.scale_y.set(Percent::from_unit(scale_y));
            self.skew.set(rotation_y - rotation_x);
        }
    }

    fn rotation(&self) -> Degrees {
        self.cache_scale_rotation();
        self.rotation.get()
    }

    fn set_rotation(&self, degrees: Degrees) -> bool {
        self.set_transformed_by_script(true);
        self.cache_scale_rotation();
        let changed = self.rotation.get() != degrees;
        self.rotation.set(degrees);

        // FIXME - this isn't quite correct. In Flash player,
        // trying to set rotation to NaN does nothing if the current
        // matrix 'b' and 'd' terms are both zero. However, if one
        // of those terms is non-zero, then the entire matrix gets
        // modified in a way that depends on its starting values.
        // I haven't been able to figure out how to reproduce those
        // values, so for now, we never modify the matrix if the
        // rotation is NaN. Hopefully, there are no SWFs depending
        // on the weird behavior when b or d is non-zero.
        if degrees.into_radians().is_nan() {
            return changed;
        }

        let skew = self.skew.get();
        let cos_x = f64::cos(degrees.into_radians());
        let sin_x = f64::sin(degrees.into_radians());
        let cos_y = f64::cos(degrees.into_radians() + skew);
        let sin_y = f64::sin(degrees.into_radians() + skew);
        let scale_x = self.scale_x.get().unit();
        let scale_y = self.scale_y.get().unit();
        let mut matrix = self.matrix.get();
        matrix.a = (scale_x * cos_x) as f32;
        matrix.b = (scale_x * sin_x) as f32;
        matrix.c = (scale_y * -sin_y) as f32;
        matrix.d = (scale_y * cos_y) as f32;
        self.matrix.set(matrix);

        changed
    }

    fn scale_x(&self) -> Percent {
        self.cache_scale_rotation();
        self.scale_x.get()
    }

    fn set_scale_x(&self, mut value: Percent) -> bool {
        let changed = self.scale_x.get() != value;
        self.set_transformed_by_script(true);
        self.cache_scale_rotation();
        self.scale_x.set(value);

        // Note - in order to match Flash's behavior, the 'scale_x' field is set to NaN
        // (which gets reported back to ActionScript), but we treat it as 0 for
        // the purposes of updating the matrix
        if value.percent().is_nan() {
            value = 0.0.into();
        }

        // Similarly, a rotation of `NaN` can be reported to ActionScript, but we
        // treat it as 0.0 when calculating the matrix
        let mut rot = self.rotation.get().into_radians();
        if rot.is_nan() {
            rot = 0.0;
        }

        let cos = f64::cos(rot);
        let sin = f64::sin(rot);
        let mut matrix = self.matrix.get();
        matrix.a = (cos * value.unit()) as f32;
        matrix.b = (sin * value.unit()) as f32;
        self.matrix.set(matrix);

        changed
    }

    fn scale_y(&self) -> Percent {
        self.cache_scale_rotation();
        self.scale_y.get()
    }

    fn set_scale_y(&self, mut value: Percent) -> bool {
        let changed = self.scale_y.get() != value;
        self.set_transformed_by_script(true);
        self.cache_scale_rotation();
        self.scale_y.set(value);

        // Note - in order to match Flash's behavior, the 'scale_y' field is set to NaN
        // (which gets reported back to ActionScript), but we treat it as 0 for
        // the purposes of updating the matrix
        if value.percent().is_nan() {
            value = 0.0.into();
        }

        // Similarly, a rotation of `NaN` can be reported to ActionScript, but we
        // treat it as 0.0 when calculating the matrix
        let mut rot = self.rotation.get().into_radians();
        if rot.is_nan() {
            rot = 0.0;
        }

        let skew = self.skew.get();
        let cos = f64::cos(rot + skew);
        let sin = f64::sin(rot + skew);
        let mut matrix = self.matrix.get();
        matrix.c = (-sin * value.unit()) as f32;
        matrix.d = (cos * value.unit()) as f32;
        self.matrix.set(matrix);

        changed
    }

    fn name(&self) -> Option<AvmString<'gc>> {
        self.name.get()
    }

    fn set_name(this: &Write<Self>, name: AvmString<'gc>) {
        unlock!(this, Self, name).set(Some(name));
    }

    fn filters(&self) -> Ref<'_, [Filter]> {
        Ref::map(self.cell.borrow(), |c| &*c.filters)
    }

    fn set_filters(&self, filters: Box<[Filter]>) -> bool {
        let mut write = self.cell.borrow_mut();
        let changed = filters != write.filters;
        write.filters = filters;
        drop(write);
        if changed {
            self.recheck_cache_as_bitmap();
        }
        changed
    }

    fn alpha(&self) -> f64 {
        f64::from(self.color_transform().a_multiply)
    }

    fn set_alpha(&self, value: f64) -> bool {
        self.set_transformed_by_script(true);
        let value = Fixed8::from_f64(value);
        let mut tf = self.color_transform.get();
        let changed = tf.a_multiply != value;
        tf.a_multiply = value;
        self.color_transform.set(tf);
        changed
    }

    fn clip_depth(&self) -> Depth {
        self.clip_depth.get()
    }

    fn set_clip_depth(&self, depth: Depth) {
        self.clip_depth.set(depth);
    }

    fn parent(&self) -> Option<DisplayObject<'gc>> {
        self.parent.get()
    }

    /// You should almost always use `DisplayObject.set_parent` instead, which
    /// properly handles 'orphan' movie clips
    fn set_parent_ignoring_orphan_list(this: &Write<Self>, parent: Option<DisplayObject<'gc>>) {
        unlock!(this, Self, parent).set(parent)
    }

    fn avm1_removed(&self) -> bool {
        self.contains_flag(DisplayObjectFlags::AVM1_REMOVED)
    }

    fn avm1_pending_removal(&self) -> bool {
        self.contains_flag(DisplayObjectFlags::AVM1_PENDING_REMOVAL)
    }

    pub fn should_skip_next_enter_frame(&self) -> bool {
        self.contains_flag(DisplayObjectFlags::SKIP_NEXT_ENTER_FRAME)
    }

    pub fn set_skip_next_enter_frame(&self, skip: bool) {
        self.set_flag(DisplayObjectFlags::SKIP_NEXT_ENTER_FRAME, skip);
    }

    fn set_avm1_removed(&self, value: bool) {
        self.set_flag(DisplayObjectFlags::AVM1_REMOVED, value);
    }

    fn set_avm1_pending_removal(&self, value: bool) {
        self.set_flag(DisplayObjectFlags::AVM1_PENDING_REMOVAL, value);
    }

    fn scale_rotation_cached(&self) -> bool {
        self.contains_flag(DisplayObjectFlags::SCALE_ROTATION_CACHED)
    }

    fn set_scale_rotation_cached(&self, set_flag: bool) {
        let flags = if set_flag {
            self.flags.get() | DisplayObjectFlags::SCALE_ROTATION_CACHED
        } else {
            self.flags.get() - DisplayObjectFlags::SCALE_ROTATION_CACHED
        };
        self.flags.set(flags);
    }

    pub fn sound_transform(&self) -> SoundTransform {
        self.sound_transform.get()
    }

    pub fn set_sound_transform(&self, sound_transform: SoundTransform) {
        self.sound_transform.set(sound_transform);
    }

    fn visible(&self) -> bool {
        self.contains_flag(DisplayObjectFlags::VISIBLE)
    }

    fn set_visible(&self, value: bool) -> bool {
        let changed = self.visible() != value;
        self.set_flag(DisplayObjectFlags::VISIBLE, value);
        changed
    }

    fn blend_mode(&self) -> ExtendedBlendMode {
        self.blend_mode.get()
    }

    fn set_blend_mode(&self, value: ExtendedBlendMode) -> bool {
        self.blend_mode.replace(value) != value
    }

    fn blend_shader(&self) -> Option<PixelBenderShaderHandle> {
        self.cell.borrow().blend_shader.clone()
    }

    fn set_blend_shader(&self, value: Option<PixelBenderShaderHandle>) {
        self.cell.borrow_mut().blend_shader = value;
    }

    /// The opaque background color of this display object.
    /// The bounding box of the display object will be filled with this color.
    fn opaque_background(&self) -> Option<Color> {
        self.opaque_background.get()
    }

    /// The opaque background color of this display object.
    /// The bounding box of the display object will be filled with the given color. This also
    /// triggers cache-as-bitmap behavior. Only solid backgrounds are supported; the alpha channel
    /// is ignored.
    fn set_opaque_background(&self, value: Option<Color>) -> bool {
        let value = value.map(|mut color| {
            color.a = 255;
            color
        });
        let changed = self.opaque_background.get() != value;
        self.opaque_background.set(value);
        changed
    }

    fn is_root(&self) -> bool {
        self.contains_flag(DisplayObjectFlags::IS_ROOT)
    }

    fn set_is_root(&self, value: bool) {
        self.set_flag(DisplayObjectFlags::IS_ROOT, value);
    }

    fn lock_root(&self) -> bool {
        self.contains_flag(DisplayObjectFlags::LOCK_ROOT)
    }

    fn set_lock_root(&self, value: bool) {
        self.set_flag(DisplayObjectFlags::LOCK_ROOT, value);
    }

    fn transformed_by_script(&self) -> bool {
        self.contains_flag(DisplayObjectFlags::TRANSFORMED_BY_SCRIPT)
    }

    fn set_transformed_by_script(&self, value: bool) {
        self.set_flag(DisplayObjectFlags::TRANSFORMED_BY_SCRIPT, value);
    }

    fn placed_by_script(&self) -> bool {
        self.contains_flag(DisplayObjectFlags::PLACED_BY_SCRIPT)
    }

    fn set_placed_by_script(&self, value: bool) {
        self.set_flag(DisplayObjectFlags::PLACED_BY_SCRIPT, value);
    }

    fn is_bitmap_cached_preference(&self) -> bool {
        self.contains_flag(DisplayObjectFlags::CACHE_AS_BITMAP)
    }

    fn set_bitmap_cached_preference(&self, value: bool) {
        self.set_flag(DisplayObjectFlags::CACHE_AS_BITMAP, value);
        self.recheck_cache_as_bitmap();
    }

    fn bitmap_cache_mut(&self) -> RefMut<'_, Option<BitmapCache>> {
        RefMut::map(self.cell.borrow_mut(), |c| &mut c.cache)
    }

    /// Invalidates a cached bitmap, if it exists.
    /// This may only be called once per frame - the first call will return true, regardless of
    /// if there was a cache.
    /// Any subsequent calls will return false, indicating that you do not need to invalidate the ancestors.
    /// This is reset during rendering.
    fn invalidate_cached_bitmap(&self) -> bool {
        if self.contains_flag(DisplayObjectFlags::CACHE_INVALIDATED) {
            return false;
        }
        if let Some(cache) = &mut *self.bitmap_cache_mut() {
            cache.make_dirty();
        }
        self.set_flag(DisplayObjectFlags::CACHE_INVALIDATED, true);
        true
    }

    fn clear_invalidate_flag(&self) {
        self.set_flag(DisplayObjectFlags::CACHE_INVALIDATED, false);
    }

    fn recheck_cache_as_bitmap(&self) {
        let mut write = self.cell.borrow_mut();
        let should_cache = self.is_bitmap_cached_preference() || !write.filters.is_empty();
        if should_cache && write.cache.is_none() {
            write.cache = Some(Default::default());
        } else if !should_cache && write.cache.is_some() {
            write.cache = None;
        }
    }

    fn instantiated_by_timeline(&self) -> bool {
        self.contains_flag(DisplayObjectFlags::INSTANTIATED_BY_TIMELINE)
    }

    fn set_instantiated_by_timeline(&self, value: bool) {
        self.set_flag(DisplayObjectFlags::INSTANTIATED_BY_TIMELINE, value);
    }

    fn has_scroll_rect(&self) -> bool {
        self.contains_flag(DisplayObjectFlags::HAS_SCROLL_RECT)
    }

    fn set_has_scroll_rect(&self, value: bool) {
        self.set_flag(DisplayObjectFlags::HAS_SCROLL_RECT, value);
    }

    fn has_explicit_name(&self) -> bool {
        self.contains_flag(DisplayObjectFlags::HAS_EXPLICIT_NAME)
    }

    fn set_has_explicit_name(&self, value: bool) {
        self.set_flag(DisplayObjectFlags::HAS_EXPLICIT_NAME, value);
    }

    fn masker(&self) -> Option<DisplayObject<'gc>> {
        self.masker.get()
    }

    fn set_masker(this: &Write<Self>, node: Option<DisplayObject<'gc>>) {
        unlock!(this, Self, masker).set(node);
    }

    fn maskee(&self) -> Option<DisplayObject<'gc>> {
        self.maskee.get()
    }

    fn set_maskee(this: &Write<Self>, node: Option<DisplayObject<'gc>>) {
        unlock!(this, Self, maskee).set(node);
    }

    fn meta_data(&self) -> Option<Avm2Object<'gc>> {
        self.meta_data.get()
    }

    fn set_meta_data(this: &Write<Self>, value: Avm2Object<'gc>) {
        unlock!(this, Self, meta_data).set(Some(value));
    }

    pub fn has_matrix3d_stub(&self) -> bool {
        self.contains_flag(DisplayObjectFlags::HAS_MATRIX3D_STUB)
    }

    pub fn set_has_matrix3d_stub(&self, value: bool) {
        self.set_flag(DisplayObjectFlags::HAS_MATRIX3D_STUB, value)
    }
}

struct DrawCacheInfo {
    handle: BitmapHandle,
    dirty: bool,
    base_transform: Transform,
    bounds: Rectangle<Twips>,
    draw_offset: Point<i32>,
    filters: Vec<Filter>,
}

pub fn render_base<'gc>(
    this: DisplayObject<'gc>,
    context: &mut RenderContext<'_, 'gc>,
    options: RenderOptions,
) {
    if options.skip_masks && this.maskee().is_some() {
        // Skip rendering masks (unless we are rendering one explicitly).
        return;
    }

    if options.apply_transform {
        let transform = this.base().transform(options.apply_matrix);
        context.transform_stack.push(&transform);
    }

    let blend_mode = this.blend_mode();
    let original_commands = if blend_mode != ExtendedBlendMode::Normal {
        Some(std::mem::take(&mut context.commands))
    } else {
        None
    };

    let cache_info = if context.use_bitmap_cache && this.is_bitmap_cached() {
        let mut cache_info: Option<DrawCacheInfo> = None;
        let base_transform = context.transform_stack.transform();
        let bounds: Rectangle<Twips> = this.render_bounds_with_transform(
            &base_transform.matrix,
            false, // we want to do the filter growth for this object ourselves, to know the offsets
            &context.stage.view_matrix(),
        );
        let name = this.name();
        let mut filters: Vec<Filter> = this.filters().to_owned();
        let swf_version = this.swf_version();
        filters.retain(|f| !f.impotent());

        if let Some(cache) = &mut *this.base().bitmap_cache_mut() {
            let width = bounds.width().to_pixels().ceil().max(0.0);
            let height = bounds.height().to_pixels().ceil().max(0.0);
            if width <= u16::MAX as f64 && height <= u16::MAX as f64 {
                let width = width as u16;
                let height = height as u16;
                let mut filter_rect = Rectangle {
                    x_min: Twips::ZERO,
                    x_max: Twips::from_pixels_i32(width as i32),
                    y_min: Twips::ZERO,
                    y_max: Twips::from_pixels_i32(height as i32),
                };
                let stage_matrix = context.stage.view_matrix();
                for filter in &mut filters {
                    // Scaling is done by *stage view matrix* only, nothing in-between
                    filter.scale(stage_matrix.a, stage_matrix.d);
                    filter_rect = filter.calculate_dest_rect(filter_rect);
                }
                let filter_rect = Rectangle {
                    x_min: filter_rect.x_min.to_pixels().floor() as i32,
                    x_max: filter_rect.x_max.to_pixels().ceil() as i32,
                    y_min: filter_rect.y_min.to_pixels().floor() as i32,
                    y_max: filter_rect.y_max.to_pixels().ceil() as i32,
                };
                let draw_offset = Point::new(filter_rect.x_min, filter_rect.y_min);
                if cache.is_dirty(&base_transform.matrix, width, height) {
                    cache.update(
                        context.renderer,
                        base_transform.matrix,
                        width,
                        height,
                        filter_rect.width() as u16,
                        filter_rect.height() as u16,
                        draw_offset,
                        swf_version,
                    );
                    cache_info = cache.handle().map(|handle| DrawCacheInfo {
                        handle,
                        dirty: true,
                        base_transform,
                        bounds,
                        draw_offset,
                        filters,
                    });
                } else {
                    cache_info = cache.handle().map(|handle| DrawCacheInfo {
                        handle,
                        dirty: false,
                        base_transform,
                        bounds,
                        draw_offset,
                        filters,
                    });
                }
            } else {
                if !cache.warned_for_oversize {
                    tracing::warn!(
                        "Skipping cacheAsBitmap for incredibly large object {:?} ({width} x {height})",
                        name
                    );
                    cache.warned_for_oversize = true;
                }
                cache.clear();
                cache_info = None;
            }
        }
        cache_info
    } else {
        None
    };

    // We can't hold `cache` (which will hold `base`), so this is split up
    if let Some(cache_info) = cache_info {
        // In order to render an object to a texture, we need to draw its entire bounds.
        // Calculate the offset from tx/ty in order to accommodate any drawings that extend the bounds
        // negatively
        let offset_x = cache_info.bounds.x_min - cache_info.base_transform.matrix.tx
            + Twips::from_pixels_i32(cache_info.draw_offset.x);
        let offset_y = cache_info.bounds.y_min - cache_info.base_transform.matrix.ty
            + Twips::from_pixels_i32(cache_info.draw_offset.y);

        if cache_info.dirty {
            let mut transform_stack = TransformStack::new();
            transform_stack.push(&Transform {
                color_transform: Default::default(),
                matrix: Matrix {
                    tx: -offset_x,
                    ty: -offset_y,
                    ..cache_info.base_transform.matrix
                },
                perspective_projection: cache_info.base_transform.perspective_projection,
                tz: Default::default(),
            });
            let mut offscreen_context = RenderContext {
                renderer: context.renderer,
                commands: CommandList::new(),
                cache_draws: context.cache_draws,
                gc_context: context.gc_context,
                library: context.library,
                transform_stack: &mut transform_stack,
                is_offscreen: true,
                use_bitmap_cache: true,
                stage: context.stage,
            };
            this.render_self(&mut offscreen_context);
            offscreen_context.cache_draws.push(BitmapCacheEntry {
                handle: cache_info.handle.clone(),
                commands: offscreen_context.commands,
                clear: this.opaque_background().unwrap_or_default(),
                filters: cache_info.filters,
            });
        }

        // When rendering it back, ensure we're only keeping the translation - scale/rotation is within the image already
        apply_standard_mask_and_scroll(
            this,
            context,
            |context| {
                context.commands.render_bitmap(
                    cache_info.handle,
                    Transform {
                        matrix: Matrix {
                            tx: context.transform_stack.transform().matrix.tx + offset_x,
                            ty: context.transform_stack.transform().matrix.ty + offset_y,
                            ..Default::default()
                        },
                        color_transform: cache_info.base_transform.color_transform,
                        perspective_projection: cache_info.base_transform.perspective_projection,
                        tz: cache_info.base_transform.tz,
                    },
                    true,
                    PixelSnapping::Always, // cacheAsBitmap forces pixel snapping
                )
            },
            &options,
        );
    } else {
        if let Some(background) = this.opaque_background() {
            // This is intended for use with cacheAsBitmap, but can be set for non-cached objects too
            // It wants the entire bounding box to be cleared before any draws happen
            let bounds: Rectangle<Twips> = this.render_bounds_with_transform(
                &context.transform_stack.transform().matrix,
                true,
                &context.stage.view_matrix(),
            );
            context
                .commands
                .draw_rect(background, Matrix::create_box_from_rectangle(&bounds));
        }
        apply_standard_mask_and_scroll(
            this,
            context,
            |context| this.render_self(context),
            &options,
        );
    }

    if let Some(original_commands) = original_commands {
        let sub_commands = std::mem::replace(&mut context.commands, original_commands);
        // If there's nothing to draw, throw away the blend entirely.
        if !sub_commands.is_empty() {
            let render_blend_mode = if let ExtendedBlendMode::Shader = blend_mode {
                // Note - Flash appears to let you set `dobj.blendMode = BlendMode.SHADER` without
                // having `dobj.blendShader` result, but the resulting rendered displayobject
                // seems to be corrupted. For now, let's panic, and see if any swfs actually
                // rely on this behavior.
                RenderBlendMode::Shader(this.blend_shader().expect("Missing blend shader"))
            } else {
                RenderBlendMode::Builtin(blend_mode.try_into().unwrap())
            };
            context.commands.blend(sub_commands, render_blend_mode);
        }
    }

    if options.apply_transform {
        context.transform_stack.pop();
    }
}

/// This applies the **standard** method of `mask` and `scrollRect`.
///
/// It uses the stencil buffer so that any pixel drawn in the mask will allow the inner contents to show.
/// This is what is used for most cases, except for cacheAsBitmap-on-cacheAsBitmap.
pub fn apply_standard_mask_and_scroll<'gc, F>(
    this: DisplayObject<'gc>,
    context: &mut RenderContext<'_, 'gc>,
    draw: F,
    options: &RenderOptions,
) where
    F: FnOnce(&mut RenderContext<'_, 'gc>),
{
    let scroll_rect_matrix = if let Some(rect) = this.scroll_rect() {
        let cur_transform = context.transform_stack.transform();
        // The matrix we use for actually drawing a rectangle for cropping purposes
        // Note that we do *not* apply the translation yet
        Some(
            cur_transform.matrix
                * Matrix::scale(
                    rect.width().to_pixels() as f32,
                    rect.height().to_pixels() as f32,
                ),
        )
    } else {
        None
    };

    if let Some(rect) = this.scroll_rect() {
        // Translate everything that we render (including DisplayObject.mask)
        context.transform_stack.push(&Transform {
            matrix: Matrix::translate(-rect.x_min, -rect.y_min),
            color_transform: Default::default(),
            perspective_projection: None,
            tz: 0.0,
        });
    }

    enum Mask<'gc> {
        None,
        Stencil(DisplayObject<'gc>),
        Alpha(DisplayObject<'gc>),
    }

    let mask = match this.masker() {
        None => Mask::None,
        Some(mask) if this.is_bitmap_cached() && mask.is_bitmap_cached() => Mask::Alpha(mask),
        Some(mask) => Mask::Stencil(mask),
    };

    let mut mask_transform = ruffle_render::transform::Transform::default();
    if let Mask::Stencil(m) | Mask::Alpha(m) = mask {
        if options.apply_transform {
            mask_transform.matrix = this.global_to_local_matrix().unwrap_or_default();
        }
        mask_transform.matrix *= m.local_to_global_matrix();
    }
    if let Mask::Stencil(m) = mask {
        context.commands.push_mask();
        context.transform_stack.push(&mask_transform);
        m.render_self(context);
        context.transform_stack.pop();
        context.commands.activate_mask();
    }

    // There are two parts to 'DisplayObject.scrollRect':
    // a scroll effect (translation), and a crop effect.
    // This scroll is implementing by applying a translation matrix
    // when we defined 'scroll_rect_matrix'.
    // The crop is implemented as a rectangular mask using the height
    // and width provided by 'scrollRect'.

    // Note that this mask is applied *in addition to* a mask defined
    // with 'DisplayObject.mask'. We will end up rendering content that
    // lies in the intersection of the scroll rect and DisplayObject.mask,
    // which is exactly the behavior that we want.
    if let Some(rect_mat) = scroll_rect_matrix {
        context.commands.push_mask();
        // The color doesn't matter, as this is a mask.
        context.commands.draw_rect(Color::WHITE, rect_mat);
        context.commands.activate_mask();
    }

    if let Mask::Alpha(m) = mask {
        let original_commands = std::mem::take(&mut context.commands);

        draw(context);

        let maskee_commands = std::mem::take(&mut context.commands);

        context.transform_stack.push(&mask_transform);
        let options = RenderOptions {
            skip_masks: false,
            apply_matrix: false,
            ..Default::default()
        };
        m.render_with_options(context, options);
        context.transform_stack.pop();

        let mask_commands = std::mem::replace(&mut context.commands, original_commands);

        context
            .commands
            .render_alpha_mask(maskee_commands, mask_commands);
    } else {
        draw(context);
    }

    if let Some(rect_mat) = scroll_rect_matrix {
        // Draw the rectangle again after deactivating the mask,
        // to reset the stencil buffer.
        context.commands.deactivate_mask();
        context.commands.draw_rect(Color::WHITE, rect_mat);
        context.commands.pop_mask();
    }

    if let Mask::Stencil(m) = mask {
        context.commands.deactivate_mask();
        context.transform_stack.push(&mask_transform);
        m.render_self(context);
        context.transform_stack.pop();
        context.commands.pop_mask();
    }

    if scroll_rect_matrix.is_some() {
        // Remove the translation that we pushed
        context.transform_stack.pop();
    }
}

#[enum_trait_object(
    #[derive(Clone, Collect, Debug, Copy)]
    #[collect(no_drop)]
    pub enum DisplayObject<'gc> {
        Stage(Stage<'gc>),
        Bitmap(Bitmap<'gc>),
        Avm1Button(Avm1Button<'gc>),
        Avm2Button(Avm2Button<'gc>),
        EditText(EditText<'gc>),
        Graphic(Graphic<'gc>),
        MorphShape(MorphShape<'gc>),
        MovieClip(MovieClip<'gc>),
        Text(Text<'gc>),
        Video(Video<'gc>),
        LoaderDisplay(LoaderDisplay<'gc>)
    }
)]
pub trait TDisplayObject<'gc>:
    'gc + Clone + Copy + Collect<'gc> + Debug + Into<DisplayObject<'gc>>
{
    fn base(self) -> Gc<'gc, DisplayObjectBase<'gc>>;

    #[no_dynamic]
    fn as_ptr(self) -> *const DisplayObjectPtr {
        Gc::as_ptr(self.base()).cast()
    }

    /// The `SCALE_ROTATION_CACHED` flag should only be set in SWFv5+.
    /// So scaling/rotation values always have to get recalculated from the matrix in SWFv4.
    #[no_dynamic]
    fn set_scale_rotation_cached(self) {
        if self.swf_version() >= 5 {
            self.base().set_scale_rotation_cached(true);
        }
    }

    fn id(self) -> CharacterId;

    #[no_dynamic]
    fn depth(self) -> Depth {
        self.base().depth()
    }

    #[no_dynamic]
    fn set_depth(self, depth: Depth) {
        self.base().set_depth(depth)
    }

    /// The untransformed inherent bounding box of this object.
    /// These bounds do **not** include child DisplayObjects.
    /// To get the bounds including children, use `bounds`, `local_bounds`, or `world_bounds`.
    ///
    /// Implementors must override this method.
    /// Leaf DisplayObjects should return their bounds.
    /// Composite DisplayObjects that only contain children should return `&Default::default()`
    fn self_bounds(self) -> Rectangle<Twips>;

    /// The untransformed bounding box of this object including children.
    #[no_dynamic]
    fn bounds(self) -> Rectangle<Twips> {
        self.bounds_with_transform(&Matrix::default())
    }

    /// The local bounding box of this object including children, in its parent's coordinate system.
    #[no_dynamic]
    fn local_bounds(self) -> Rectangle<Twips> {
        self.bounds_with_transform(&self.base().matrix())
    }

    /// The world bounding box of this object including children, relative to the stage.
    #[no_dynamic]
    fn world_bounds(self) -> Rectangle<Twips> {
        self.bounds_with_transform(&self.local_to_global_matrix())
    }

    /// The world bounding box of this object, as reported by `Transform.pixelBounds`.
    fn pixel_bounds(self) -> Rectangle<Twips> {
        self.world_bounds()
    }

    /// Bounds used for drawing debug rects and picking objects.
    #[no_dynamic]
    fn debug_rect_bounds(self) -> Rectangle<Twips> {
        // Make the rect at least as big as highlight bounds to ensure that anything
        // interactive is also highlighted even if not included in world bounds.
        let highlight_bounds = self
            .as_interactive()
            .map(|int| int.highlight_bounds())
            .unwrap_or_default();
        self.world_bounds().union(&highlight_bounds)
    }

    /// Gets the bounds of this object and all children, transformed by a given matrix.
    /// This function recurses down and transforms the AABB each child before adding
    /// it to the bounding box. This gives a tighter AABB then if we simply transformed
    /// the overall AABB.
    fn bounds_with_transform(self, matrix: &Matrix) -> Rectangle<Twips> {
        // A scroll rect completely overrides an object's bounds,
        // and can even grow the bounding box to be larger than the actual content
        if let Some(scroll_rect) = self.scroll_rect() {
            return *matrix
                * Rectangle {
                    x_min: Twips::ZERO,
                    y_min: Twips::ZERO,
                    x_max: scroll_rect.width(),
                    y_max: scroll_rect.height(),
                };
        }

        let mut bounds = *matrix * self.self_bounds();

        if let Some(ctr) = self.as_container() {
            for child in ctr.iter_render_list() {
                let matrix = *matrix * child.base().matrix();
                bounds = bounds.union(&child.bounds_with_transform(&matrix));
            }
        }

        bounds
    }

    /// Gets the **render bounds** of this object and all its children.
    /// This differs from the bounds that are exposed to Flash, in two main ways:
    /// - It may be larger if filters are applied which will increase the size of what's shown
    /// - It does not respect scroll rects
    fn render_bounds_with_transform(
        self,
        matrix: &Matrix,
        include_own_filters: bool,
        view_matrix: &Matrix,
    ) -> Rectangle<Twips> {
        let mut bounds = *matrix * self.self_bounds();

        if let Some(ctr) = self.as_container() {
            for child in ctr.iter_render_list() {
                let matrix = *matrix * child.base().matrix();
                bounds =
                    bounds.union(&child.render_bounds_with_transform(&matrix, true, view_matrix));
            }
        }

        if include_own_filters {
            for mut filter in self.filters().iter().cloned() {
                filter.scale(view_matrix.a, view_matrix.d);
                bounds = filter.calculate_dest_rect(bounds);
            }
        }

        bounds
    }

    #[no_dynamic]
    fn place_frame(self) -> u16 {
        self.base().place_frame()
    }

    #[no_dynamic]
    fn set_place_frame(self, frame: u16) {
        self.base().set_place_frame(frame)
    }

    /// Sets the matrix of this object.
    /// This does NOT invalidate the cache, as it's often used with other operations.
    /// It is the callers responsibility to do so.
    fn set_matrix(self, matrix: Matrix) {
        self.base().set_matrix(matrix);
    }

    /// Sets the color transform of this object.
    /// This does NOT invalidate the cache, as it's often used with other operations.
    /// It is the callers responsibility to do so.
    #[no_dynamic]
    fn set_color_transform(self, color_transform: ColorTransform) {
        self.base().set_color_transform(color_transform)
    }

    /// Sets the perspective projection of this object.
    /// This invalidates any ancestors cacheAsBitmap automatically.
    fn set_perspective_projection(self, perspective_projection: Option<PerspectiveProjection>) {
        if self
            .base()
            .set_perspective_projection(perspective_projection)
        {
            if let Some(parent) = self.parent() {
                // Self-transform changes are automatically handled,
                // we only want to inform ancestors to avoid unnecessary invalidations for tx/ty
                parent.invalidate_cached_bitmap();
            }
        }
    }

    /// Should only be used to implement 'Transform.concatenatedMatrix'
    #[no_dynamic]
    fn local_to_global_matrix_without_own_scroll_rect(self) -> Matrix {
        let mut node = self.parent();
        let mut matrix = self.base().matrix();
        while let Some(display_object) = node {
            // We want to transform to Stage-local coordinates,
            // so do *not* apply the Stage's matrix
            if display_object.as_stage().is_some() {
                break;
            }
            if let Some(rect) = display_object.scroll_rect() {
                matrix = Matrix::translate(-rect.x_min, -rect.y_min) * matrix;
            }
            matrix = display_object.base().matrix() * matrix;
            node = display_object.parent();
        }
        matrix
    }

    /// Returns the matrix for transforming from this object's local space to global stage space.
    fn local_to_global_matrix(self) -> Matrix {
        let mut matrix = Matrix::IDENTITY;
        if let Some(rect) = self.scroll_rect() {
            matrix = Matrix::translate(-rect.x_min, -rect.y_min) * matrix;
        }
        self.local_to_global_matrix_without_own_scroll_rect() * matrix
    }

    /// Returns the matrix for transforming from global stage to this object's local space.
    /// `None` is returned if the object has zero scale.
    #[no_dynamic]
    fn global_to_local_matrix(self) -> Option<Matrix> {
        self.local_to_global_matrix().inverse()
    }

    /// Converts a local position to a global stage position
    #[no_dynamic]
    fn local_to_global(self, local: Point<Twips>) -> Point<Twips> {
        self.local_to_global_matrix() * local
    }

    /// Converts a local position on the stage to a local position on this display object
    /// Returns `None` if the object has zero scale.
    #[no_dynamic]
    fn global_to_local(self, global: Point<Twips>) -> Option<Point<Twips>> {
        self.global_to_local_matrix().map(|matrix| matrix * global)
    }

    /// Converts the mouse position on the stage to a local position on this display object.
    /// If the object has zero scale, then the stage `TWIPS_TO_PIXELS` matrix will be used.
    /// This matches Flash's behavior for `mouseX`/`mouseY` on an object with zero scale.
    #[no_dynamic]
    fn local_mouse_position(self, context: &UpdateContext<'gc>) -> Point<Twips> {
        let stage = context.stage;
        let pixel_ratio = stage.view_matrix().a;
        let virtual_to_device = Matrix::scale(pixel_ratio, pixel_ratio);

        // Get mouse pos in global device pixels
        let global_twips = *context.mouse_position;
        let global_device_twips = virtual_to_device * global_twips;
        let global_device_pixels = Matrix::TWIPS_TO_PIXELS * global_device_twips;

        // Make transformation matrix
        let local_twips_to_global_twips = self.local_to_global_matrix();
        let twips_to_device_pixels = virtual_to_device * Matrix::TWIPS_TO_PIXELS;
        let local_twips_to_global_device_pixels =
            twips_to_device_pixels * local_twips_to_global_twips;
        let global_device_pixels_to_local_twips = local_twips_to_global_device_pixels
            .inverse()
            .unwrap_or(Matrix::IDENTITY);

        // Get local mouse position in twips
        global_device_pixels_to_local_twips * global_device_pixels
    }

    /// The `x` position in pixels of this display object in local space.
    /// Returned by the `_x`/`x` ActionScript properties.
    fn x(self) -> Twips {
        self.base().x()
    }

    /// Sets the `x` position in pixels of this display object in local space.
    /// Set by the `_x`/`x` ActionScript properties.
    /// This invalidates any ancestors cacheAsBitmap automatically.
    fn set_x(self, x: Twips) {
        if self.base().set_x(x) {
            if let Some(parent) = self.parent() {
                // Self-transform changes are automatically handled,
                // we only want to inform ancestors to avoid unnecessary invalidations for tx/ty
                parent.invalidate_cached_bitmap();
            }
        }
    }

    /// The `y` position in pixels of this display object in local space.
    /// Returned by the `_y`/`y` ActionScript properties.
    fn y(self) -> Twips {
        self.base().y()
    }

    /// Sets the `y` position in pixels of this display object in local space.
    /// Set by the `_y`/`y` ActionScript properties.
    /// This invalidates any ancestors cacheAsBitmap automatically.
    fn set_y(self, y: Twips) {
        if self.base().set_y(y) {
            if let Some(parent) = self.parent() {
                // Self-transform changes are automatically handled,
                // we only want to inform ancestors to avoid unnecessary invalidations for tx/ty
                parent.invalidate_cached_bitmap();
            }
        }
    }

    /// The `z` position in local space.
    /// Returned by the `z` ActionScript properties.
    fn z(self) -> f64 {
        self.base().z()
    }

    /// Sets the `z` position of this display object in local space.
    /// Set by the `z` ActionScript properties.
    /// This invalidates any ancestors cacheAsBitmap automatically.
    fn set_z(self, z: f64) {
        if self.base().set_z(z) {
            if let Some(parent) = self.parent() {
                // Self-transform changes are automatically handled,
                // we only want to inform ancestors to avoid unnecessary invalidations for tx/ty
                parent.invalidate_cached_bitmap();
            }
        }
    }

    /// The rotation in degrees this display object in local space.
    /// Returned by the `_rotation`/`rotation` ActionScript properties.
    #[no_dynamic]
    fn rotation(self) -> Degrees {
        let degrees = self.base().rotation();
        self.set_scale_rotation_cached();
        degrees
    }

    /// Sets the rotation in degrees this display object in local space.
    /// Set by the `_rotation`/`rotation` ActionScript properties.
    /// This invalidates any ancestors cacheAsBitmap automatically.
    #[no_dynamic]
    fn set_rotation(self, radians: Degrees) {
        if self.base().set_rotation(radians) {
            self.set_scale_rotation_cached();
            if let Some(parent) = self.parent() {
                // Self-transform changes are automatically handled,
                // we only want to inform ancestors to avoid unnecessary invalidations for tx/ty
                parent.invalidate_cached_bitmap();
            }
        }
    }

    /// The X axis scale for this display object in local space.
    /// Returned by the `_xscale`/`scaleX` ActionScript properties.
    #[no_dynamic]
    fn scale_x(self) -> Percent {
        let percent = self.base().scale_x();
        self.set_scale_rotation_cached();
        percent
    }

    /// Sets the X axis scale for this display object in local space.
    /// Set by the `_xscale`/`scaleX` ActionScript properties.
    /// This invalidates any ancestors cacheAsBitmap automatically.
    #[no_dynamic]
    fn set_scale_x(self, value: Percent) {
        if self.base().set_scale_x(value) {
            self.set_scale_rotation_cached();
            if let Some(parent) = self.parent() {
                // Self-transform changes are automatically handled,
                // we only want to inform ancestors to avoid unnecessary invalidations for tx/ty
                parent.invalidate_cached_bitmap();
            }
        }
    }

    /// The Y axis scale for this display object in local space.
    /// Returned by the `_yscale`/`scaleY` ActionScript properties.
    #[no_dynamic]
    fn scale_y(self) -> Percent {
        let percent = self.base().scale_y();
        self.set_scale_rotation_cached();
        percent
    }

    /// Sets the Y axis scale for this display object in local space.
    /// Returned by the `_yscale`/`scaleY` ActionScript properties.
    /// This invalidates any ancestors cacheAsBitmap automatically.
    #[no_dynamic]
    fn set_scale_y(self, value: Percent) {
        if self.base().set_scale_y(value) {
            self.set_scale_rotation_cached();
            if let Some(parent) = self.parent() {
                // Self-transform changes are automatically handled,
                // we only want to inform ancestors to avoid unnecessary invalidations for tx/ty
                parent.invalidate_cached_bitmap();
            }
        }
    }

    /// Gets the pixel width of the AABB containing this display object in local space.
    /// Returned by the ActionScript `_width`/`width` properties.
    fn width(self) -> f64 {
        self.local_bounds().width().to_pixels()
    }

    /// Sets the pixel width of this display object in local space.
    /// The width is based on the AABB of the object.
    /// Set by the ActionScript `_width`/`width` properties.
    /// This does odd things on rotated clips to match the behavior of Flash.
    fn set_width(self, _context: &mut UpdateContext<'gc>, value: f64) {
        let object_bounds = self.bounds();
        let object_width = object_bounds.width().to_pixels();
        let object_height = object_bounds.height().to_pixels();
        let aspect_ratio = object_height / object_width;

        let (target_scale_x, target_scale_y) = if object_width != 0.0 {
            (value / object_width, value / object_height)
        } else {
            (0.0, 0.0)
        };

        // No idea about the derivation of this -- figured it out via lots of trial and error.
        // It has to do with the length of the sides A, B of an AABB enclosing the object's OBB with sides a, b:
        // A = sin(t) * a + cos(t) * b
        // B = cos(t) * a + sin(t) * b
        let prev_scale_x = self.scale_x().unit();
        let prev_scale_y = self.scale_y().unit();
        let rotation = self.rotation();
        let cos = f64::abs(f64::cos(rotation.into_radians()));
        let sin = f64::abs(f64::sin(rotation.into_radians()));
        let mut new_scale_x = aspect_ratio * (cos * target_scale_x + sin * target_scale_y)
            / ((cos + aspect_ratio * sin) * (aspect_ratio * cos + sin));
        let mut new_scale_y =
            (sin * prev_scale_x + aspect_ratio * cos * prev_scale_y) / (aspect_ratio * cos + sin);

        if !new_scale_x.is_finite() {
            new_scale_x = 0.0;
        }

        if !new_scale_y.is_finite() {
            new_scale_y = 0.0;
        }

        self.set_scale_x(Percent::from_unit(new_scale_x));
        self.set_scale_y(Percent::from_unit(new_scale_y));
    }

    /// Gets the pixel height of the AABB containing this display object in local space.
    /// Returned by the ActionScript `_height`/`height` properties.
    fn height(self) -> f64 {
        self.local_bounds().height().to_pixels()
    }

    /// Sets the pixel height of this display object in local space.
    /// Set by the ActionScript `_height`/`height` properties.
    /// This does odd things on rotated clips to match the behavior of Flash.
    fn set_height(self, _context: &mut UpdateContext<'gc>, value: f64) {
        let object_bounds = self.bounds();
        let object_width = object_bounds.width().to_pixels();
        let object_height = object_bounds.height().to_pixels();
        let aspect_ratio = object_width / object_height;

        let (target_scale_x, target_scale_y) = if object_height != 0.0 {
            (value / object_width, value / object_height)
        } else {
            (0.0, 0.0)
        };

        // No idea about the derivation of this -- figured it out via lots of trial and error.
        // It has to do with the length of the sides A, B of an AABB enclosing the object's OBB with sides a, b:
        // A = sin(t) * a + cos(t) * b
        // B = cos(t) * a + sin(t) * b
        let prev_scale_x = self.scale_x().unit();
        let prev_scale_y = self.scale_y().unit();
        let rotation = self.rotation();
        let cos = f64::abs(f64::cos(rotation.into_radians()));
        let sin = f64::abs(f64::sin(rotation.into_radians()));
        let mut new_scale_x =
            (aspect_ratio * cos * prev_scale_x + sin * prev_scale_y) / (aspect_ratio * cos + sin);
        let mut new_scale_y = aspect_ratio * (sin * target_scale_x + cos * target_scale_y)
            / ((cos + aspect_ratio * sin) * (aspect_ratio * cos + sin));

        if !new_scale_x.is_finite() {
            new_scale_x = 0.0;
        }

        if !new_scale_y.is_finite() {
            new_scale_y = 0.0;
        }

        self.set_scale_x(Percent::from_unit(new_scale_x));
        self.set_scale_y(Percent::from_unit(new_scale_y));
    }

    /// The opacity of this display object.
    /// 1 is fully opaque.
    /// Returned by the `_alpha`/`alpha` ActionScript properties.
    #[no_dynamic]
    fn alpha(self) -> f64 {
        self.base().alpha()
    }

    /// Sets the opacity of this display object.
    /// 1 is fully opaque.
    /// Set by the `_alpha`/`alpha` ActionScript properties.
    /// This invalidates any cacheAsBitmap automatically.
    #[no_dynamic]
    fn set_alpha(self, value: f64) {
        if self.base().set_alpha(value) {
            if let Some(parent) = self.parent() {
                // Self-transform changes are automatically handled
                parent.invalidate_cached_bitmap();
            }
        }
    }

    #[no_dynamic]
    fn name(self) -> Option<AvmString<'gc>> {
        self.base().name()
    }

    #[no_dynamic]
    fn set_name(self, mc: &Mutation<'gc>, name: AvmString<'gc>) {
        DisplayObjectBase::set_name(Gc::write(mc, self.base()), name)
    }

    fn filters(self) -> Ref<'gc, [Filter]> {
        Gc::as_ref(self.base()).filters()
    }

    fn set_filters(self, filters: Box<[Filter]>) {
        if self.base().set_filters(filters) {
            self.invalidate_cached_bitmap();
        }
    }

    /// Returns the dot-syntax path to this display object, e.g. `_level0.foo.clip`
    #[no_dynamic]
    fn path(self) -> WString {
        if let Some(parent) = self.avm1_parent() {
            let mut path = parent.path();
            path.push_byte(b'.');
            if let Some(name) = self.name() {
                path.push_str(&name);
            }
            path
        } else {
            WString::from_utf8_owned(format!("_level{}", self.depth()))
        }
    }

    /// Returns the Flash 4 slash-syntax path to this display object, e.g. `/foo/clip`.
    /// Returned by the `_target` property in AVM1.
    #[no_dynamic]
    fn slash_path(self) -> WString {
        fn build_slash_path(object: DisplayObject<'_>) -> WString {
            if let Some(parent) = object.avm1_parent() {
                let mut path = build_slash_path(parent);
                path.push_byte(b'/');
                if let Some(name) = object.name() {
                    path.push_str(&name);
                }
                path
            } else {
                let level = object.depth();
                if level == 0 {
                    // _level0 does not append its name in slash syntax.
                    WString::new()
                } else {
                    // Other levels do append their name.
                    WString::from_utf8_owned(format!("_level{level}"))
                }
            }
        }

        if self.avm1_parent().is_some() {
            build_slash_path(self)
        } else {
            // _target of _level0 should just be '/'.
            WString::from_unit(b'/'.into())
        }
    }

    #[no_dynamic]
    fn clip_depth(self) -> Depth {
        self.base().clip_depth()
    }

    #[no_dynamic]
    fn set_clip_depth(self, depth: Depth) {
        self.base().set_clip_depth(depth);
    }

    /// Retrieve the parent of this display object.
    ///
    /// This version of the function merely exposes the display object parent,
    /// without any further filtering.
    #[no_dynamic]
    fn parent(self) -> Option<DisplayObject<'gc>> {
        self.base().parent()
    }

    /// Set the parent of this display object.
    #[no_dynamic]
    fn set_parent(self, context: &mut UpdateContext<'gc>, parent: Option<DisplayObject<'gc>>) {
        let had_parent = self.parent().is_some();
        let write = Gc::write(context.gc(), self.base());
        DisplayObjectBase::set_parent_ignoring_orphan_list(write, parent);
        let parent_removed = had_parent && parent.is_none();

        if parent_removed {
            if let Some(int) = self.as_interactive() {
                int.drop_focus(context);
            }

            self.on_parent_removed(context);
        }
    }

    /// This method is called when the parent is removed.
    /// It may be overwritten to inject some implementation-specific behavior.
    fn on_parent_removed(self, _context: &mut UpdateContext<'gc>) {}

    /// Retrieve the parent of this display object.
    ///
    /// This version of the function implements the concept of parenthood as
    /// seen in AVM1. Notably, it disallows access to the `Stage` and to
    /// non-AVM1 DisplayObjects; for an unfiltered concept of parent,
    /// use the `parent` method.
    #[no_dynamic]
    fn avm1_parent(self) -> Option<DisplayObject<'gc>> {
        self.parent()
            .filter(|p| p.as_stage().is_none())
            .filter(|p| !p.movie().is_action_script_3())
    }

    /// Retrieve the parent of this display object.
    ///
    /// This version of the function implements the concept of parenthood as
    /// seen in AVM2. Notably, it disallows access to non-container parents.
    #[no_dynamic]
    fn avm2_parent(self) -> Option<DisplayObject<'gc>> {
        self.parent().filter(|p| p.as_container().is_some())
    }

    #[no_dynamic]
    fn masker(self) -> Option<DisplayObject<'gc>> {
        self.base().masker()
    }

    #[no_dynamic]
    fn set_masker(
        self,
        mc: &Mutation<'gc>,
        node: Option<DisplayObject<'gc>>,
        remove_old_link: bool,
    ) {
        if remove_old_link {
            let old_masker = self.base().masker();
            if let Some(old_masker) = old_masker {
                old_masker.set_maskee(mc, None, false);
            }
            if let Some(parent) = self.parent() {
                // Masks are natively handled by cacheAsBitmap - don't invalidate self, only parents
                parent.invalidate_cached_bitmap();
            }
        }
        DisplayObjectBase::set_masker(Gc::write(mc, self.base()), node);
    }

    #[no_dynamic]
    fn maskee(self) -> Option<DisplayObject<'gc>> {
        self.base().maskee()
    }

    #[no_dynamic]
    fn set_maskee(
        self,
        mc: &Mutation<'gc>,
        node: Option<DisplayObject<'gc>>,
        remove_old_link: bool,
    ) {
        if remove_old_link {
            let old_maskee = self.base().maskee();
            if let Some(old_maskee) = old_maskee {
                old_maskee.set_masker(mc, None, false);
            }
            self.invalidate_cached_bitmap();
        }
        DisplayObjectBase::set_maskee(Gc::write(mc, self.base()), node);
    }

    /// High level method for setting the mask. Sets both masker and maskee.
    ///
    /// Equivalent to setting the mask from AVM.
    #[no_dynamic]
    fn set_mask(self, mask: Option<DisplayObject<'gc>>, mc: &Mutation<'gc>) {
        self.set_clip_depth(0);
        self.set_masker(mc, mask, true);
        if let Some(mask) = mask {
            mask.set_clip_depth(0);
            mask.set_maskee(mc, Some(self), true);
        }
    }

    #[no_dynamic]
    fn scroll_rect(self) -> Option<Rectangle<Twips>> {
        self.base().scroll_rect.get()
    }

    #[no_dynamic]
    fn next_scroll_rect(self) -> Rectangle<Twips> {
        self.base().next_scroll_rect.get()
    }

    #[no_dynamic]
    fn set_next_scroll_rect(self, rectangle: Rectangle<Twips>) {
        self.base().next_scroll_rect.set(rectangle);

        // Scroll rect is natively handled by cacheAsBitmap - don't invalidate self, only parents
        if let Some(parent) = self.parent() {
            parent.invalidate_cached_bitmap();
        }
    }

    #[no_dynamic]
    fn scaling_grid(self) -> Rectangle<Twips> {
        self.base().scaling_grid.get()
    }

    #[no_dynamic]
    fn set_scaling_grid(self, rect: Rectangle<Twips>) {
        self.base().scaling_grid.set(rect);
    }

    #[no_dynamic]
    /// Whether this object has been removed. Only applies to AVM1.
    fn avm1_removed(self) -> bool {
        self.base().avm1_removed()
    }

    #[no_dynamic]
    // Sets whether this object has been removed. Only applies to AVM1
    fn set_avm1_removed(self, value: bool) {
        self.base().set_avm1_removed(value)
    }

    #[no_dynamic]
    /// Is this object waiting to be removed on the start of the next frame
    fn avm1_pending_removal(self) -> bool {
        self.base().avm1_pending_removal()
    }

    #[no_dynamic]
    fn set_avm1_pending_removal(self, value: bool) {
        self.base().set_avm1_pending_removal(value)
    }

    /// Whether this display object is visible.
    /// Invisible objects are not rendered, but otherwise continue to exist normally.
    /// Returned by the `_visible`/`visible` ActionScript properties.
    #[no_dynamic]
    fn visible(self) -> bool {
        self.base().visible()
    }

    /// Sets whether this display object will be visible.
    /// Invisible objects are not rendered, but otherwise continue to exist normally.
    /// Returned by the `_visible`/`visible` ActionScript properties.
    #[no_dynamic]
    fn set_visible(self, context: &mut UpdateContext<'gc>, value: bool) {
        if self.base().set_visible(value) {
            if let Some(parent) = self.parent() {
                // We don't need to invalidate ourselves, we're just toggling if the bitmap is rendered.
                parent.invalidate_cached_bitmap();
            }
        }

        if !value {
            if let Some(int) = self.as_interactive() {
                // The focus is dropped when it's made invisible.
                int.drop_focus(context);
            }
        }
    }

    #[no_dynamic]
    fn meta_data(self) -> Option<Avm2Object<'gc>> {
        self.base().meta_data()
    }

    #[no_dynamic]
    fn set_meta_data(self, mc: &Mutation<'gc>, value: Avm2Object<'gc>) {
        DisplayObjectBase::set_meta_data(Gc::write(mc, self.base()), value);
    }

    /// The blend mode used when rendering this display object.
    /// Values other than the default `BlendMode::Normal` implicitly cause cache-as-bitmap behavior.
    #[no_dynamic]
    fn blend_mode(self) -> ExtendedBlendMode {
        self.base().blend_mode()
    }

    /// Sets the blend mode used when rendering this display object.
    /// Values other than the default `BlendMode::Normal` implicitly cause cache-as-bitmap behavior.
    #[no_dynamic]
    fn set_blend_mode(self, value: ExtendedBlendMode) {
        if self.base().set_blend_mode(value) {
            if let Some(parent) = self.parent() {
                // We don't need to invalidate ourselves, we're just toggling how the bitmap is rendered.

                // Note that Flash does not always invalidate on changing the blend mode;
                // but that's a bug we don't need to copy :)
                parent.invalidate_cached_bitmap();
            }
        }
    }

    #[no_dynamic]
    fn blend_shader(self) -> Option<PixelBenderShaderHandle> {
        self.base().blend_shader()
    }

    #[no_dynamic]
    fn set_blend_shader(self, value: Option<PixelBenderShaderHandle>) {
        self.base().set_blend_shader(value);
        self.set_blend_mode(ExtendedBlendMode::Shader);
    }

    #[no_dynamic]
    /// The opaque background color of this display object.
    fn opaque_background(self) -> Option<Color> {
        self.base().opaque_background()
    }

    /// Sets the opaque background color of this display object.
    /// The bounding box of the display object will be filled with the given color. This also
    /// triggers cache-as-bitmap behavior. Only solid backgrounds are supported; the alpha channel
    /// is ignored.
    #[no_dynamic]
    fn set_opaque_background(self, value: Option<Color>) {
        if self.base().set_opaque_background(value) {
            self.invalidate_cached_bitmap();
        }
    }

    /// Whether this display object represents the root of loaded content.
    #[no_dynamic]
    fn is_root(self) -> bool {
        self.base().is_root()
    }

    /// Sets whether this display object represents the root of loaded content.
    #[no_dynamic]
    fn set_is_root(self, value: bool) {
        self.base().set_is_root(value);
    }

    /// The sound transform for sounds played inside this display object.
    #[no_dynamic]
    fn set_sound_transform(
        self,
        context: &mut UpdateContext<'gc>,
        sound_transform: SoundTransform,
    ) {
        self.base().set_sound_transform(sound_transform);
        context.set_sound_transforms_dirty();
    }

    /// Whether this display object is used as the _root of itself and its children.
    /// Returned by the `_lockroot` ActionScript property.
    #[no_dynamic]
    fn lock_root(self) -> bool {
        self.base().lock_root()
    }

    /// Sets whether this display object is used as the _root of itself and its children.
    /// Returned by the `_lockroot` ActionScript property.
    #[no_dynamic]
    fn set_lock_root(self, value: bool) {
        self.base().set_lock_root(value);
    }

    /// Whether this display object has been transformed by ActionScript.
    /// When this flag is set, changes from SWF `PlaceObject` tags are ignored.
    #[no_dynamic]
    fn transformed_by_script(self) -> bool {
        self.base().transformed_by_script()
    }

    /// Sets whether this display object has been transformed by ActionScript.
    /// When this flag is set, changes from SWF `PlaceObject` tags are ignored.
    #[no_dynamic]
    fn set_transformed_by_script(self, value: bool) {
        self.base().set_transformed_by_script(value)
    }

    /// Whether this display object prefers to be cached into a bitmap rendering.
    /// This is the PlaceObject `cacheAsBitmap` flag - and may be overridden if filters are applied.
    /// Consider `is_bitmap_cached` for if a bitmap cache is actually in use.
    #[no_dynamic]
    fn is_bitmap_cached_preference(self) -> bool {
        self.base().is_bitmap_cached_preference()
    }

    /// Whether this display object is using a bitmap cache, whether by preference or necessity.
    #[no_dynamic]
    fn is_bitmap_cached(self) -> bool {
        self.base().cell.borrow().cache.is_some()
    }

    /// Explicitly sets the preference of this display object to be cached into a bitmap rendering.
    /// Note that the object will still be bitmap cached if a filter is active.
    #[no_dynamic]
    fn set_bitmap_cached_preference(self, value: bool) {
        self.base().set_bitmap_cached_preference(value)
    }

    /// Whether this display object has a scroll rectangle applied.
    #[no_dynamic]
    fn has_scroll_rect(self) -> bool {
        self.base().has_scroll_rect()
    }

    /// Sets whether this display object has a scroll rectangle applied.
    #[no_dynamic]
    fn set_has_scroll_rect(self, value: bool) {
        self.base().set_has_scroll_rect(value)
    }

    /// Whether this display object has been created by ActionScript 3.
    /// When this flag is set, changes from SWF `RemoveObject` tags are
    /// ignored.
    #[no_dynamic]
    fn placed_by_script(self) -> bool {
        self.base().placed_by_script()
    }

    /// Sets whether this display object has been created by ActionScript 3.
    /// When this flag is set, changes from SWF `RemoveObject` tags are
    /// ignored.
    #[no_dynamic]
    fn set_placed_by_script(self, value: bool) {
        self.base().set_placed_by_script(value)
    }

    /// Whether this display object has been instantiated by the timeline.
    /// When this flag is set, attempts to change the object's name from AVM2
    /// throw an exception.
    #[no_dynamic]
    fn instantiated_by_timeline(self) -> bool {
        self.base().instantiated_by_timeline()
    }

    /// Sets whether this display object has been instantiated by the timeline.
    /// When this flag is set, attempts to change the object's name from AVM2
    /// throw an exception.
    #[no_dynamic]
    fn set_instantiated_by_timeline(self, value: bool) {
        self.base().set_instantiated_by_timeline(value);
    }

    /// Whether this display object was placed by a SWF tag with an explicit
    /// name.
    ///
    /// When this flag is set, the object will attempt to set a dynamic property
    /// on the parent with the same name as itself.
    #[no_dynamic]
    fn has_explicit_name(self) -> bool {
        self.base().has_explicit_name()
    }

    /// Sets whether this display object was placed by a SWF tag with an
    /// explicit name.
    ///
    /// When this flag is set, the object will attempt to set a dynamic property
    /// on the parent with the same name as itself.
    #[no_dynamic]
    fn set_has_explicit_name(self, value: bool) {
        self.base().set_has_explicit_name(value);
    }
    fn state(&self) -> Option<ButtonState> {
        None
    }

    fn set_state(self, _context: &mut UpdateContext<'gc>, _state: ButtonState) {}

    /// Run any start-of-frame actions for this display object.
    ///
    /// When fired on `Stage`, this also emits the AVM2 `enterFrame` broadcast.
    fn enter_frame(self, _context: &mut UpdateContext<'gc>) {}

    /// Construct all display objects that the timeline indicates should exist
    /// this frame, and their children.
    ///
    /// This function should ensure the following, from the point of view of
    /// downstream VMs:
    ///
    /// 1. That the object itself has been allocated, if not constructed
    /// 2. That newly created children have been instantiated and are present
    ///    as properties on the class
    fn construct_frame(self, _context: &mut UpdateContext<'gc>) {}

    /// To be called when an AVM2 display object has finished being constructed.
    ///
    /// This function must be called once and ONLY once, after the object's
    /// AVM2 side has been constructed. Typically, this is in construct_frame,
    /// unless your object needs to construct itself earlier or later. When
    /// this function is called on the child, it will fire its add events and,
    /// if possible, set a named property on the parent matching the name of
    /// the object.
    ///
    /// This still needs to be called for objects placed by AVM2, since we
    /// need to stop the underlying MovieClip if the constructed class
    /// does not extend MovieClip.
    ///
    /// Since we construct AVM2 display objects after they are allocated and
    /// placed on the render list, these steps have to be done by the child
    /// object to signal to its parent that it was added.
    #[no_dynamic]
    #[inline(never)]
    fn on_construction_complete(self, context: &mut UpdateContext<'gc>) {
        let placed_by_script = self.placed_by_script();
        self.fire_added_events(context);
        // Check `self.placed_by_script()` before we fire events, since those
        // events might `placed_by_script`
        if !placed_by_script {
            self.set_on_parent_field(context);
        }

        if let Some(movie) = self.as_movie_clip() {
            let obj = movie
                .object2()
                .expect("MovieClip object should have been constructed");
            let movieclip_class = context.avm2.classes().movieclip.inner_class_definition();
            // It's possible to have a DefineSprite tag with multiple frames, but have
            // the corresponding `SymbolClass` *not* extend `MovieClip` (e.g. extending `Sprite` directly.)
            // When this occurs, Flash Player will run the first frame, and immediately stop.
            // However, Flash Player runs frames for the root movie clip, even if it doesn't extend `MovieClip`.
            if !obj.is_of_type(movieclip_class) && !movie.is_root() {
                movie.stop(context);
            }
            movie.set_initialized();
        }
    }

    #[no_dynamic]
    fn fire_added_events(self, context: &mut UpdateContext<'gc>) {
        if !self.placed_by_script() {
            // Since we construct AVM2 display objects after they are
            // allocated and placed on the render list, we have to emit all
            // events after this point.
            //
            // Children added to buttons by the timeline do not emit events.
            if self.parent().and_then(|p| p.as_avm2_button()).is_none() {
                dispatch_added_event_only(self, context);
                if self.avm2_stage(context).is_some() {
                    dispatch_added_to_stage_event_only(self, context);
                }
            }
        }
    }

    #[no_dynamic]
    fn set_on_parent_field(self, context: &mut UpdateContext<'gc>) {
        //TODO: Don't report missing property errors.
        //TODO: Don't attempt to set properties if object was placed without a name.
        if self.has_explicit_name() {
            if let Some(parent) = self.parent().and_then(|p| p.object2()) {
                let parent = Avm2Value::from(parent);

                if let Some(child) = self.object2() {
                    if let Some(name) = self.name() {
                        let domain = context
                            .library
                            .library_for_movie(self.movie())
                            .unwrap()
                            .avm2_domain();
                        let mut activation = Avm2Activation::from_domain(context, domain);
                        let multiname =
                            Avm2Multiname::new(activation.avm2().find_public_namespace(), name);
                        if let Err(e) =
                            parent.init_property(&multiname, child.into(), &mut activation)
                        {
                            tracing::error!(
                                "Got error when setting AVM2 child named \"{}\": {}",
                                &name,
                                e
                            );
                        }
                    }
                }
            }
        }
    }

    /// Emit a `frameConstructed` event on this DisplayObject and any children it
    /// may have.
    #[no_dynamic]
    fn frame_constructed(self, context: &mut UpdateContext<'gc>) {
        let frame_constructed_evt =
            Avm2EventObject::bare_default_event(context, "frameConstructed");
        let dobject_constr = context.avm2.classes().display_object;
        Avm2::broadcast_event(context, frame_constructed_evt, dobject_constr);
    }

    /// Run any frame scripts (if they exist and this object needs to run them).
    fn run_frame_scripts(self, context: &mut UpdateContext<'gc>) {
        if let Some(container) = self.as_container() {
            for child in container.iter_render_list() {
                child.run_frame_scripts(context);
            }
        }
    }

    /// Emit an `exitFrame` broadcast event.
    #[no_dynamic]
    fn exit_frame(self, context: &mut UpdateContext<'gc>) {
        let exit_frame_evt = Avm2EventObject::bare_default_event(context, "exitFrame");
        let dobject_constr = context.avm2.classes().display_object;
        Avm2::broadcast_event(context, exit_frame_evt, dobject_constr);

        LoadManager::run_exit_frame(context);
    }

    /// Called before the child is about to be rendered.
    /// Note that this happens even if the child is invisible
    /// (as long as the child is still on a render list)
    #[no_dynamic]
    fn pre_render(self, _context: &mut RenderContext<'_, 'gc>) {
        let this = self.base();
        this.clear_invalidate_flag();
        this.scroll_rect
            .set(this.has_scroll_rect().then(|| this.next_scroll_rect.get()));
    }

    fn render_self(self, _context: &mut RenderContext<'_, 'gc>) {}

    #[no_dynamic]
    fn render(self, context: &mut RenderContext<'_, 'gc>) {
        self.render_with_options(context, Default::default())
    }

    fn render_with_options(self, context: &mut RenderContext<'_, 'gc>, options: RenderOptions) {
        render_base(self.into(), context, options)
    }

    #[cfg(not(feature = "avm_debug"))]
    #[no_dynamic]
    fn display_render_tree(self, _depth: usize) {}

    #[cfg(feature = "avm_debug")]
    #[no_dynamic]
    fn display_render_tree(self, depth: usize) {
        let mut self_str = &*format!("{self:?}");
        if let Some(end_char) = self_str.find(|c: char| !c.is_ascii_alphanumeric()) {
            self_str = &self_str[..end_char];
        }

        let bounds = self.world_bounds();

        let mut classname = "".to_string();
        if let Some(o) = self.object2() {
            classname = format!("{:?}", o.base().class_name());
        }

        println!(
            "{} rel({},{}) abs({},{}) {} {} {} id={} depth={}",
            " ".repeat(depth),
            self.x(),
            self.y(),
            bounds.x_min.to_pixels(),
            bounds.y_min.to_pixels(),
            classname,
            self.name().map(|s| s.to_string()).unwrap_or_default(),
            self_str,
            self.id(),
            depth
        );

        if let Some(ctr) = self.as_container() {
            ctr.recurse_render_tree(depth + 1);
        }
    }

    fn avm1_unload(self, context: &mut UpdateContext<'gc>) {
        // Unload children.
        if let Some(ctr) = self.as_container() {
            for child in ctr.iter_render_list() {
                child.avm1_unload(context);
            }
        }

        if let Some(node) = self.maskee() {
            node.set_masker(context.gc(), None, true);
        } else if let Some(node) = self.masker() {
            node.set_maskee(context.gc(), None, true);
        }

        // Unregister any text field variable bindings, and replace them on the unbound list.
        Avm1TextFieldBinding::unregister_bindings(self.into(), context);

        self.set_avm1_removed(true);
    }

    fn avm1_text_field_bindings(&self) -> Option<Ref<'_, [Avm1TextFieldBinding<'gc>]>> {
        None
    }

    fn avm1_text_field_bindings_mut(
        &self,
        _mc: &Mutation<'gc>,
    ) -> Option<RefMut<'_, Vec<Avm1TextFieldBinding<'gc>>>> {
        None
    }

    #[no_dynamic]
    fn apply_place_object(self, context: &mut UpdateContext<'gc>, place_object: &swf::PlaceObject) {
        // PlaceObject tags only apply if this object has not been dynamically moved by AS code.
        if !self.transformed_by_script() {
            if let Some(matrix) = place_object.matrix {
                self.set_matrix(matrix.into());
                if let Some(parent) = self.parent() {
                    // Self-transform changes are automatically handled,
                    // we only want to inform ancestors to avoid unnecessary invalidations for tx/ty
                    parent.invalidate_cached_bitmap();
                }
            }
            if let Some(color_transform) = &place_object.color_transform {
                self.set_color_transform(*color_transform);
                if let Some(parent) = self.parent() {
                    parent.invalidate_cached_bitmap();
                }
            }
            if let Some(ratio) = place_object.ratio {
                if let Some(morph_shape) = self.as_morph_shape() {
                    morph_shape.set_ratio(ratio);
                } else if let Some(video) = self.as_video() {
                    video.seek(context, ratio.into());
                }
            }
            if let Some(is_bitmap_cached) = place_object.is_bitmap_cached {
                self.set_bitmap_cached_preference(is_bitmap_cached);
            }
            if let Some(blend_mode) = place_object.blend_mode {
                self.set_blend_mode(blend_mode.into());
            }
            if self.swf_version() >= 11 {
                if let Some(visible) = place_object.is_visible {
                    self.set_visible(context, visible);
                }
                if let Some(mut color) = place_object.background_color {
                    let color = if color.a > 0 {
                        // Force opaque background to have no transpranecy.
                        color.a = 255;
                        Some(color)
                    } else {
                        None
                    };
                    self.set_opaque_background(color);
                }
            }
            if let Some(filters) = &place_object.filters {
                self.set_filters(filters.iter().map(Filter::from).collect());
            }
            // Purposely omitted properties:
            // name, clip_depth, clip_actions
            // These properties are only set on initial placement in `MovieClip::instantiate_child`
            // and can not be modified by subsequent PlaceObject tags.
        }
    }

    /// Called when this object should be replaced by a PlaceObject tag.
    fn replace_with(self, _context: &mut UpdateContext<'gc>, _id: CharacterId) {
        // Noop for most symbols; only shapes can replace their innards with another Graphic.
    }

    fn object(self) -> Avm1Value<'gc> {
        Avm1Value::Undefined // TODO: Implement for every type and delete this fallback.
    }

    fn object2(self) -> Option<Avm2StageObject<'gc>>;

    fn set_object2(self, _context: &mut UpdateContext<'gc>, _to: Avm2StageObject<'gc>) {}

    #[no_dynamic]
    fn object2_or_null(self) -> Avm2Value<'gc> {
        self.object2().map(|o| o.into()).unwrap_or(Avm2Value::Null)
    }

    /// Tests if a given stage position point intersects with the world bounds of this object.
    #[no_dynamic]
    fn hit_test_bounds(self, point: Point<Twips>) -> bool {
        self.world_bounds().contains(point)
    }

    /// Tests if a given object's world bounds intersects with the world bounds
    /// of this object.
    #[no_dynamic]
    fn hit_test_object(self, other: DisplayObject<'gc>) -> bool {
        self.world_bounds().intersects(&other.world_bounds())
    }

    /// Tests if a given stage position point intersects within this object, considering the art.
    fn hit_test_shape(
        self,
        _context: &mut UpdateContext<'gc>,
        point: Point<Twips>,
        options: HitTestOptions,
    ) -> bool {
        // Default to using bounding box.
        (!options.contains(HitTestOptions::SKIP_INVISIBLE) || self.visible())
            && self.hit_test_bounds(point)
    }

    fn post_instantiation(
        self,
        _context: &mut UpdateContext<'gc>,
        _init_object: Option<Avm1Object<'gc>>,
        _instantiated_by: Instantiator,
        _run_frame: bool,
    ) {
        // Noop.
    }

    /// Return the version of the SWF that created this movie clip.
    fn swf_version(self) -> u8 {
        self.movie().version()
    }

    /// Return the SWF that defines this display object.
    fn movie(self) -> Arc<SwfMovie>;

    fn loader_info(self) -> Option<LoaderInfoObject<'gc>> {
        None
    }

    fn instantiate(self, gc_context: &Mutation<'gc>) -> DisplayObject<'gc>;

    /// Whether this object can be used as a mask.
    /// If this returns false and this object is used as a mask, the mask will not be applied.
    /// This is used by movie clips to disable the mask when there are no children, for example.
    fn allow_as_mask(self) -> bool {
        true
    }

    /// Obtain the top-most non-Stage parent of the display tree hierarchy.
    ///
    /// This function implements the AVM1 concept of root clips. For the AVM2
    /// version, see `avm2_root`.
    #[no_dynamic]
    fn avm1_root(self) -> DisplayObject<'gc> {
        let mut root = self;
        loop {
            if root.lock_root() {
                break;
            }
            if let Some(parent) = root.avm1_parent() {
                if !parent.movie().is_action_script_3() {
                    root = parent;
                } else {
                    // We've traversed upwards into a loader AVM2 movie, so break.
                    break;
                }
            } else {
                break;
            }
        }
        root
    }

    /// `avm1_root`, but disregards _lockroot
    #[no_dynamic]
    fn avm1_root_no_lock(self) -> DisplayObject<'gc> {
        let mut root = self;
        while let Some(parent) = root.avm1_parent() {
            if !parent.movie().is_action_script_3() {
                root = parent;
            } else {
                // We've traversed upwards into a loader AVM2 movie, so break.
                break;
            }
        }
        root
    }

    /// Obtain the top-most Stage or LoaderDisplay object of the display tree hierarchy, for use in mixed AVM.
    #[no_dynamic]
    fn avm1_stage(self) -> DisplayObject<'gc> {
        let mut root = self;
        loop {
            if let Some(parent) = root.parent() {
                if matches!(
                    parent,
                    DisplayObject::LoaderDisplay(_) | DisplayObject::Stage(_)
                ) {
                    return parent;
                }
                root = parent;
            } else {
                return root;
            }
        }
    }

    /// Obtain the top-most non-Stage parent of the display tree hierarchy, if
    /// a suitable object exists.
    ///
    /// This function implements the AVM2 concept of root clips. For the AVM1
    /// version, see `avm1_root`.
    #[no_dynamic]
    fn avm2_root(self) -> Option<DisplayObject<'gc>> {
        let mut parent = Some(self);
        while let Some(p) = parent {
            if p.is_root() {
                return parent;
            }
            if let Some(p_parent) = p.parent() {
                if !p_parent.movie().is_action_script_3() {
                    // We've traversed upwards into a loader AVM1 movie, so return the current parent.
                    return parent;
                }
            }
            parent = p.parent();
        }
        None
    }

    /// Obtain the root of the display tree hierarchy, if a suitable object
    /// exists.
    ///
    /// This implements the AVM2 concept of `stage`. Notably, it deliberately
    /// will fail to locate the current player's stage for objects that are not
    /// rooted to the DisplayObject hierarchy correctly. If you just want to
    /// access the current player's stage, grab it from the context.
    #[no_dynamic]
    fn avm2_stage(self, _context: &UpdateContext<'gc>) -> Option<DisplayObject<'gc>> {
        let mut parent = Some(self);
        while let Some(p) = parent {
            if p.as_stage().is_some() {
                return parent;
            }
            parent = p.parent();
        }
        None
    }

    /// Determine if this display object is currently on the stage.
    #[no_dynamic]
    fn is_on_stage(self, context: &UpdateContext<'gc>) -> bool {
        let mut ancestor = self.avm2_parent();
        while let Some(parent) = ancestor {
            if parent.avm2_parent().is_some() {
                ancestor = parent.avm2_parent();
            } else {
                break;
            }
        }

        let ancestor = ancestor.unwrap_or(self);
        DisplayObject::ptr_eq(ancestor, context.stage.into())
    }

    /// Assigns a default instance name `instanceN` to this object.
    #[no_dynamic]
    fn set_default_instance_name(self, context: &mut UpdateContext<'gc>) {
        if self.base().name().is_none() {
            let name = format!("instance{}", *context.instance_counter);
            self.set_name(context.gc(), AvmString::new_utf8(context.gc(), name));
            *context.instance_counter = context.instance_counter.wrapping_add(1);
        }
    }

    /// Assigns a default root name to this object.
    ///
    /// The default root names change based on the AVM configuration of the
    /// clip; AVM2 clips get `rootN` while AVM1 clips get blank strings.
    #[no_dynamic]
    fn set_default_root_name(self, context: &mut UpdateContext<'gc>) {
        if self.movie().is_action_script_3() {
            let name = AvmString::new_utf8(context.gc(), format!("root{}", self.depth() + 1));
            self.set_name(context.gc(), name);
        } else {
            self.set_name(context.gc(), istr!(context, ""));
        }
    }

    /// Inform this object and its ancestors that it has visually changed and must be redrawn.
    /// If this object or any ancestor is marked as cacheAsBitmap, it will invalidate that cache.
    #[no_dynamic]
    fn invalidate_cached_bitmap(self) {
        if self.base().invalidate_cached_bitmap() {
            // Don't inform ancestors if we've already done so this frame
            if let Some(parent) = self.parent() {
                parent.invalidate_cached_bitmap();
            }
        }
    }

    /// Retrieve a named property from the AVM1 object.
    ///
    /// This is required as some boolean properties in AVM1 can in fact hold any value.
    #[no_dynamic]
    fn get_avm1_boolean_property<F>(
        self,
        name: AvmString<'gc>,
        context: &mut UpdateContext<'gc>,
        default: F,
    ) -> bool
    where
        F: FnOnce(&mut UpdateContext<'gc>) -> bool,
    {
        if let Avm1Value::Object(object) = self.object() {
            let mut activation = Activation::from_nothing(
                context,
                Avm1ActivationIdentifier::root("[AVM1 Boolean Property]"),
                self.avm1_root(),
            );
            if let Ok(value) = object.get(name, &mut activation) {
                match value {
                    Avm1Value::Undefined => default(activation.context),
                    _ => value.as_bool(activation.swf_version()),
                }
            } else {
                default(activation.context)
            }
        } else {
            false
        }
    }

    #[no_dynamic]
    fn set_avm1_property(
        self,
        name: AvmString<'gc>,
        value: Avm1Value<'gc>,
        context: &mut UpdateContext<'gc>,
    ) {
        if let Avm1Value::Object(object) = self.object() {
            let mut activation = Activation::from_nothing(
                context,
                Avm1ActivationIdentifier::root("[AVM1 Property Set]"),
                self.avm1_root(),
            );
            let _ = object.set(name, value, &mut activation);
        }
    }

    fn as_drawing(&self) -> Option<RefMut<'_, Drawing>> {
        None
    }

    #[no_dynamic]
    fn as_container(self) -> Option<DisplayObjectContainer<'gc>> {
        match self {
            Self::Avm1Button(dobj) => Some(DisplayObjectContainer::Avm1Button(dobj)),
            Self::LoaderDisplay(dobj) => Some(DisplayObjectContainer::LoaderDisplay(dobj)),
            Self::MovieClip(dobj) => Some(DisplayObjectContainer::MovieClip(dobj)),
            Self::Stage(dobj) => Some(DisplayObjectContainer::Stage(dobj)),
            _ => None,
        }
    }
}

pub enum DisplayObjectPtr {}

macro_rules! impl_downcast_methods {
    ($(
        $vis:vis fn $fn_name:ident for $variant:ident;
    )*) => { $(
        #[doc = concat!("Downcast this display object as a `", stringify!($variant), "`.")]
        #[inline(always)]
        $vis fn $fn_name(self) -> Option<$variant<'gc>> {
            if let Self::$variant(obj) = self {
                Some(obj)
            } else {
                None
            }
        }
    )* }
}

impl<'gc> DisplayObject<'gc> {
    pub fn ptr_eq(a: DisplayObject<'gc>, b: DisplayObject<'gc>) -> bool {
        std::ptr::eq(a.as_ptr(), b.as_ptr())
    }

    pub fn option_ptr_eq(a: Option<DisplayObject<'gc>>, b: Option<DisplayObject<'gc>>) -> bool {
        a.map(|o| o.as_ptr()) == b.map(|o| o.as_ptr())
    }

    impl_downcast_methods! {
        pub fn as_stage for Stage;
        pub fn as_avm1_button for Avm1Button;
        pub fn as_avm2_button for Avm2Button;
        pub fn as_movie_clip for MovieClip;
        pub fn as_edit_text for EditText;
        pub fn as_text for Text;
        pub fn as_morph_shape for MorphShape;
        pub fn as_video for Video;
        pub fn as_bitmap for Bitmap;
    }

    pub fn as_interactive(self) -> Option<InteractiveObject<'gc>> {
        match self {
            Self::Avm1Button(dobj) => Some(InteractiveObject::Avm1Button(dobj)),
            Self::Avm2Button(dobj) => Some(InteractiveObject::Avm2Button(dobj)),
            Self::EditText(dobj) => Some(InteractiveObject::EditText(dobj)),
            Self::LoaderDisplay(dobj) => Some(InteractiveObject::LoaderDisplay(dobj)),
            Self::MovieClip(dobj) => Some(InteractiveObject::MovieClip(dobj)),
            Self::Stage(dobj) => Some(InteractiveObject::Stage(dobj)),
            _ => None,
        }
    }

    pub fn downgrade(self) -> DisplayObjectWeak<'gc> {
        match self {
            DisplayObject::MovieClip(mc) => DisplayObjectWeak::MovieClip(mc.downgrade()),
            DisplayObject::LoaderDisplay(l) => DisplayObjectWeak::LoaderDisplay(l.downgrade()),
            DisplayObject::Bitmap(b) => DisplayObjectWeak::Bitmap(b.downgrade()),
            _ => panic!("Downgrade not yet implemented for {self:?}"),
        }
    }
}

bitflags! {
    /// Bit flags used by `DisplayObject`.
    #[derive(Clone, Copy)]
    struct DisplayObjectFlags: u16 {
        /// Whether this object has been removed from the display list.
        /// Necessary in AVM1 to throw away queued actions from removed movie clips.
        const AVM1_REMOVED             = 1 << 0;

        /// If this object is visible (`_visible` property).
        const VISIBLE                  = 1 << 1;

        /// Whether the `_xscale`, `_yscale` and `_rotation` of the object have been calculated and cached.
        const SCALE_ROTATION_CACHED    = 1 << 2;

        /// Whether this object has been transformed by ActionScript.
        /// When this flag is set, changes from SWF `PlaceObject` tags are ignored.
        const TRANSFORMED_BY_SCRIPT    = 1 << 3;

        /// Whether this object has been placed in a container by ActionScript 3.
        /// When this flag is set, changes from SWF `RemoveObject` tags are ignored.
        const PLACED_BY_SCRIPT         = 1 << 4;

        /// Whether this object has been instantiated by a SWF tag.
        /// When this flag is set, attempts to change the object's name from AVM2 throw an exception.
        const INSTANTIATED_BY_TIMELINE = 1 << 5;

        /// Whether this object is a "root", the top-most display object of a loaded SWF or Bitmap.
        /// Used by `MovieClip.getBytesLoaded` in AVM1 and `DisplayObject.root` in AVM2.
        const IS_ROOT                  = 1 << 6;

        /// Whether this object has `_lockroot` set to true, in which case
        /// it becomes the _root of itself and of any children
        const LOCK_ROOT                = 1 << 7;

        /// Whether this object will be cached to bitmap.
        const CACHE_AS_BITMAP          = 1 << 8;

        /// Whether this object has a scroll rectangle applied.
        const HAS_SCROLL_RECT          = 1 << 9;

        /// Whether this object has an explicit name.
        const HAS_EXPLICIT_NAME        = 1 << 10;

        /// Flag set when we should skip running our next 'enterFrame'
        /// for ourself and our children.
        /// This is set for objects constructed from ActionScript,
        /// which are observed to lag behind objects placed by the timeline
        /// (even if they are both placed in the same frame)
        const SKIP_NEXT_ENTER_FRAME    = 1 << 11;

        /// If this object has already had `invalidate_cached_bitmap` called this frame
        const CACHE_INVALIDATED        = 1 << 12;

        /// If this AVM1 object is pending removal (will be removed on the next frame).
        const AVM1_PENDING_REMOVAL     = 1 << 13;

        /// Whether this object has matrix3D (used for stubbing).
        const HAS_MATRIX3D_STUB        = 1 << 14;
    }
}

bitflags! {
    /// Defines how hit testing should be performed.
    /// Used for mouse picking and ActionScript's hitTestClip functions.
    #[derive(Clone, Copy)]
    pub struct HitTestOptions: u8 {
        /// Ignore objects used as masks (setMask / clipDepth).
        const SKIP_MASK = 1 << 0;

        /// Ignore objects with the ActionScript's visibility flag turned off.
        const SKIP_INVISIBLE = 1 << 1;

        /// Check only the specified object. Ignore any children of that object.
        const SKIP_CHILDREN = 1 << 2;

        /// The options used for `hitTest` calls in ActionScript.
        const AVM_HIT_TEST = Self::SKIP_MASK.bits();

        /// The options used for mouse picking, such as clicking on buttons.
        const MOUSE_PICK = Self::SKIP_MASK.bits() | Self::SKIP_INVISIBLE.bits();
    }
}

/// A binding from a property of an AVM1 StageObject to an EditText text field.
#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct Avm1TextFieldBinding<'gc> {
    pub text_field: EditText<'gc>,
    pub variable_name: AvmString<'gc>,
}

impl<'gc> Avm1TextFieldBinding<'gc> {
    pub fn bind_variables(activation: &mut Activation<'_, 'gc>) {
        // Check all unbound text fields to see if they apply to this object.
        // TODO: Replace with `Vec::drain_filter` when stable.
        let mut i = 0;
        let mut len = activation.context.unbound_text_fields.len();
        while i < len {
            if activation.context.unbound_text_fields[i]
                .try_bind_text_field_variable(activation, false)
            {
                activation.context.unbound_text_fields.swap_remove(i);
                len -= 1;
            } else {
                i += 1;
            }
        }
    }

    /// Registers a text field variable binding for this stage object.
    /// Whenever a property with the given name is changed, we should change the text in the text field.
    pub fn register_binding(self, dobj: DisplayObject<'gc>, mc: &Mutation<'gc>) {
        if let Some(mut bindings) = dobj.avm1_text_field_bindings_mut(mc) {
            bindings.push(self);
        }
    }

    /// Removes a text field binding for the given text field.
    /// Does not place the text field on the unbound list.
    /// Caller is responsible for placing the text field on the unbound list, if necessary.
    pub fn clear_binding(dobj: DisplayObject<'gc>, text_field: EditText<'gc>, mc: &Mutation<'gc>) {
        if let Some(mut bindings) = dobj.avm1_text_field_bindings_mut(mc) {
            bindings.retain(|b| !DisplayObject::ptr_eq(text_field.into(), b.text_field.into()));
        }
    }

    /// Clears all text field bindings from this stage object, and places the textfields on the unbound list.
    /// This is called when the object is removed from the stage.
    pub fn unregister_bindings(dobj: DisplayObject<'gc>, context: &mut UpdateContext<'gc>) {
        let mc = context.gc();
        if let Some(mut bindings) = dobj.avm1_text_field_bindings_mut(mc) {
            for binding in bindings.drain(..) {
                binding.text_field.clear_bound_display_object(context);
                context.unbound_text_fields.push(binding.text_field);
            }
        }
    }
}

/// Represents the sound transform of sounds played inside a Flash MovieClip.
/// Every value is a percentage (0-100), but out of range values are allowed.
/// In AVM1, this is returned by `Sound.getTransform`.
/// In AVM2, this is returned by `Sprite.soundTransform`.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct SoundTransform {
    pub volume: i32,
    pub left_to_left: i32,
    pub left_to_right: i32,
    pub right_to_left: i32,
    pub right_to_right: i32,
}

impl SoundTransform {
    pub const MAX_VOLUME: i32 = 100;

    /// Applies another SoundTransform on top of this SoundTransform.
    #[must_use]
    pub fn concat(mut self, other: SoundTransform) -> SoundTransform {
        const MAX_VOLUME: i64 = SoundTransform::MAX_VOLUME as i64;

        // It seems like Flash masks the results below to 30-bit integers:
        // * Negative values are equivalent to their absolute value (their sign bit is unset).
        // * Specifically, 0x40000000, -0x40000000 and -0x80000000 are equivalent to zero.
        const MASK: i32 = (1 << 30) - 1;

        self.volume = (i64::from(self.volume) * i64::from(other.volume) / MAX_VOLUME) as i32 & MASK;

        // This is a 2x2 matrix multiply between the transforms.
        // Done with integer math to match Flash behavior.
        let ll0: i64 = self.left_to_left.into();
        let lr0: i64 = self.left_to_right.into();
        let rl0: i64 = self.right_to_left.into();
        let rr0: i64 = self.right_to_right.into();
        let ll1: i64 = other.left_to_left.into();
        let lr1: i64 = other.left_to_right.into();
        let rl1: i64 = other.right_to_left.into();
        let rr1: i64 = other.right_to_right.into();
        self.left_to_left = ((ll0 * ll1 + rl0 * lr1) / MAX_VOLUME) as i32 & MASK;
        self.left_to_right = ((lr0 * ll1 + rr0 * lr1) / MAX_VOLUME) as i32 & MASK;
        self.right_to_left = ((ll0 * rl1 + rl0 * rr1) / MAX_VOLUME) as i32 & MASK;
        self.right_to_right = ((lr0 * rl1 + rr0 * rr1) / MAX_VOLUME) as i32 & MASK;

        self
    }

    /// Returns the pan of this transform.
    /// -100 is full left and 100 is full right.
    /// This matches the behavior of AVM1 `Sound.getPan()`
    pub fn pan(&self) -> i32 {
        // It's not clear why Flash has the weird `abs` behavior, but this
        // matches the values that Flash returns (see `sound` regression test).
        if self.left_to_left != Self::MAX_VOLUME {
            Self::MAX_VOLUME - self.left_to_left.abs()
        } else {
            self.right_to_right.abs() - Self::MAX_VOLUME
        }
    }

    /// Modifies the pan of this transform.
    /// -100 is full left and 100 is full right.
    /// This matches the behavior of AVM1 `Sound.setPan()`.
    #[must_use]
    pub fn with_pan(mut self, pan: i32) -> SoundTransform {
        if pan >= 0 {
            self.left_to_left = Self::MAX_VOLUME - pan;
            self.right_to_right = Self::MAX_VOLUME;
        } else {
            self.left_to_left = Self::MAX_VOLUME;
            self.right_to_right = Self::MAX_VOLUME + pan;
        }
        self.left_to_right = 0;
        self.right_to_left = 0;
        self
    }

    pub fn from_avm2_object(as3_st: Avm2Object<'_>) -> Self {
        let sound_transform = as3_st
            .as_sound_transform()
            .expect("Should pass SoundTransform");

        SoundTransform {
            left_to_left: (sound_transform.left_to_left() * 100.0) as i32,
            left_to_right: (sound_transform.left_to_right() * 100.0) as i32,
            right_to_left: (sound_transform.right_to_left() * 100.0) as i32,
            right_to_right: (sound_transform.right_to_right() * 100.0) as i32,
            volume: (sound_transform.volume() * 100.0) as i32,
        }
    }

    pub fn into_avm2_object<'gc>(
        self,
        activation: &mut Avm2Activation<'_, 'gc>,
    ) -> Result<Avm2Object<'gc>, Avm2Error<'gc>> {
        let as3_st = activation
            .avm2()
            .classes()
            .soundtransform
            .construct(activation, &[])?
            .as_object()
            .unwrap()
            .as_sound_transform()
            .unwrap();

        as3_st.set_left_to_left(self.left_to_left as f64 / 100.0);
        as3_st.set_left_to_right(self.left_to_right as f64 / 100.0);
        as3_st.set_right_to_left(self.right_to_left as f64 / 100.0);
        as3_st.set_right_to_right(self.right_to_right as f64 / 100.0);
        as3_st.set_volume(self.volume as f64 / 100.0);

        Ok(as3_st.into())
    }
}

impl Default for SoundTransform {
    fn default() -> Self {
        Self {
            volume: 100,
            left_to_left: 100,
            left_to_right: 0,
            right_to_left: 0,
            right_to_right: 100,
        }
    }
}

/// A version of `DisplayObject` that holds weak pointers.
/// Currently, this is only used by orphan handling, so we only
/// need two variants. If other use cases arise, feel free
/// to add more variants.
#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub enum DisplayObjectWeak<'gc> {
    MovieClip(MovieClipWeak<'gc>),
    LoaderDisplay(LoaderDisplayWeak<'gc>),
    Bitmap(BitmapWeak<'gc>),
}

impl<'gc> DisplayObjectWeak<'gc> {
    pub fn as_ptr(&self) -> *const DisplayObjectPtr {
        match self {
            DisplayObjectWeak::MovieClip(mc) => mc.as_ptr(),
            DisplayObjectWeak::LoaderDisplay(ld) => ld.as_ptr(),
            DisplayObjectWeak::Bitmap(b) => b.as_ptr(),
        }
    }

    pub fn upgrade(&self, mc: &Mutation<'gc>) -> Option<DisplayObject<'gc>> {
        match self {
            DisplayObjectWeak::MovieClip(movie) => movie.upgrade(mc).map(|m| m.into()),
            DisplayObjectWeak::LoaderDisplay(ld) => ld.upgrade(mc).map(|ld| ld.into()),
            DisplayObjectWeak::Bitmap(b) => b.upgrade(mc).map(|ld| ld.into()),
        }
    }
}
