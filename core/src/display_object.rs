use crate::avm1::{
    ActivationIdentifier as Avm1ActivationIdentifier, Object as Avm1Object, TObject as Avm1TObject,
    Value as Avm1Value,
};
use crate::avm2::{
    Activation as Avm2Activation, Avm2, Error as Avm2Error, EventObject as Avm2EventObject,
    Multiname as Avm2Multiname, Object as Avm2Object, TObject as Avm2TObject, Value as Avm2Value,
};
use crate::context::{RenderContext, UpdateContext};
use crate::drawing::Drawing;
use crate::prelude::*;
use crate::string::{AvmString, WString};
use crate::tag_utils::SwfMovie;
use crate::types::{Degrees, Percent};
use crate::vminterface::Instantiator;
use bitflags::bitflags;
use gc_arena::{Collect, Mutation};
use ruffle_macros::enum_trait_object;
use ruffle_render::pixel_bender::PixelBenderShaderHandle;
use ruffle_render::transform::{Transform, TransformStack};
use std::cell::{Ref, RefMut};
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
    #[allow(clippy::too_many_arguments)]
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

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct DisplayObjectBase<'gc> {
    parent: Option<DisplayObject<'gc>>,
    place_frame: u16,
    depth: Depth,
    #[collect(require_static)]
    transform: Transform,
    name: Option<AvmString<'gc>>,
    #[collect(require_static)]
    filters: Vec<Filter>,
    clip_depth: Depth,

    // Cached transform properties `_xscale`, `_yscale`, `_rotation`.
    // These are expensive to calculate, so they will be calculated and cached
    // when AS requests one of these properties.
    #[collect(require_static)]
    rotation: Degrees,
    #[collect(require_static)]
    scale_x: Percent,
    #[collect(require_static)]
    scale_y: Percent,

    skew: f64,

    /// The next display object in order of execution.
    ///
    /// `None` in an AVM2 movie.
    next_avm1_clip: Option<DisplayObject<'gc>>,

    /// The sound transform of sounds playing via this display object.
    #[collect(require_static)]
    sound_transform: SoundTransform,

    /// The display object that we are being masked by.
    masker: Option<DisplayObject<'gc>>,

    /// The display object we are currently masking.
    maskee: Option<DisplayObject<'gc>>,

    meta_data: Option<Avm2Object<'gc>>,

    /// The blend mode used when rendering this display object.
    /// Values other than the default `BlendMode::Normal` implicitly cause cache-as-bitmap behavior.
    #[collect(require_static)]
    blend_mode: ExtendedBlendMode,

    #[collect(require_static)]
    blend_shader: Option<PixelBenderShaderHandle>,

    /// The opaque background color of this display object.
    /// The bounding box of the display object will be filled with the given color. This also
    /// triggers cache-as-bitmap behavior. Only solid backgrounds are supported; the alpha channel
    /// is ignored.
    #[collect(require_static)]
    opaque_background: Option<Color>,

    /// Bit flags for various display object properties.
    #[collect(require_static)]
    flags: DisplayObjectFlags,

    /// The 'internal' scroll rect used for rendering and methods like 'localToGlobal'.
    /// This is updated from 'pre_render'
    #[collect(require_static)]
    scroll_rect: Option<Rectangle<Twips>>,

    /// The 'next' scroll rect, which we will copy to 'scroll_rect' from 'pre_render'.
    /// This is used by the ActionScript 'DisplayObject.scrollRect' getter, which sees
    /// changes immediately (without needing wait for a render)
    #[collect(require_static)]
    next_scroll_rect: Rectangle<Twips>,

    /// Rectangle used for 9-slice scaling (`DisplayObject.scale9grid`).
    #[collect(require_static)]
    scaling_grid: Rectangle<Twips>,

    /// If this Display Object should cacheAsBitmap - and if so, the cache itself.
    /// None means not cached, Some means cached.
    #[collect(require_static)]
    cache: Option<BitmapCache>,
}

impl<'gc> Default for DisplayObjectBase<'gc> {
    fn default() -> Self {
        Self {
            parent: Default::default(),
            place_frame: Default::default(),
            depth: Default::default(),
            transform: Default::default(),
            name: None,
            filters: Default::default(),
            clip_depth: Default::default(),
            rotation: Degrees::from_radians(0.0),
            scale_x: Percent::from_unit(1.0),
            scale_y: Percent::from_unit(1.0),
            skew: 0.0,
            next_avm1_clip: None,
            masker: None,
            maskee: None,
            meta_data: None,
            sound_transform: Default::default(),
            blend_mode: Default::default(),
            blend_shader: None,
            opaque_background: Default::default(),
            flags: DisplayObjectFlags::VISIBLE,
            scroll_rect: None,
            next_scroll_rect: Default::default(),
            scaling_grid: Default::default(),
            cache: None,
        }
    }
}

impl<'gc> DisplayObjectBase<'gc> {
    /// Reset all properties that would be adjusted by a movie load.
    fn reset_for_movie_load(&mut self) {
        let flags_to_keep = self.flags & DisplayObjectFlags::LOCK_ROOT;
        self.flags = flags_to_keep | DisplayObjectFlags::VISIBLE;
    }

    fn depth(&self) -> Depth {
        self.depth
    }

    fn set_depth(&mut self, depth: Depth) {
        self.depth = depth;
    }

    fn place_frame(&self) -> u16 {
        self.place_frame
    }

    fn set_place_frame(&mut self, frame: u16) {
        self.place_frame = frame;
    }

    fn transform(&self) -> &Transform {
        &self.transform
    }

    pub fn matrix(&self) -> &Matrix {
        &self.transform.matrix
    }

    pub fn matrix_mut(&mut self) -> &mut Matrix {
        &mut self.transform.matrix
    }

    pub fn set_matrix(&mut self, matrix: Matrix) {
        self.transform.matrix = matrix;
        self.set_scale_rotation_cached(false);
    }

    pub fn color_transform(&self) -> &ColorTransform {
        &self.transform.color_transform
    }

    pub fn color_transform_mut(&mut self) -> &mut ColorTransform {
        &mut self.transform.color_transform
    }

    pub fn set_color_transform(&mut self, color_transform: ColorTransform) {
        self.transform.color_transform = color_transform;
    }

    fn x(&self) -> Twips {
        self.transform.matrix.tx
    }

    fn set_x(&mut self, x: Twips) -> bool {
        let changed = self.transform.matrix.tx != x;
        self.set_transformed_by_script(true);
        self.transform.matrix.tx = x;
        changed
    }

    fn y(&self) -> Twips {
        self.transform.matrix.ty
    }

    fn set_y(&mut self, y: Twips) -> bool {
        let changed = self.transform.matrix.ty != y;
        self.set_transformed_by_script(true);
        self.transform.matrix.ty = y;
        changed
    }

    /// Caches the scale and rotation factors for this display object, if necessary.
    /// Calculating these requires heavy trig ops, so we only do it when `_xscale`, `_yscale` or
    /// `_rotation` is accessed.
    fn cache_scale_rotation(&mut self) {
        if !self.scale_rotation_cached() {
            let (a, b, c, d) = (
                f64::from(self.transform.matrix.a),
                f64::from(self.transform.matrix.b),
                f64::from(self.transform.matrix.c),
                f64::from(self.transform.matrix.d),
            );
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
            self.rotation = Degrees::from_radians(rotation_x);
            self.scale_x = Percent::from_unit(scale_x);
            self.scale_y = Percent::from_unit(scale_y);
            self.skew = rotation_y - rotation_x;
        }
    }

    fn rotation(&mut self) -> Degrees {
        self.cache_scale_rotation();
        self.rotation
    }

    fn set_rotation(&mut self, degrees: Degrees) -> bool {
        self.set_transformed_by_script(true);
        self.cache_scale_rotation();
        let changed = self.rotation != degrees;
        self.rotation = degrees;

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

        let cos_x = f64::cos(degrees.into_radians());
        let sin_x = f64::sin(degrees.into_radians());
        let cos_y = f64::cos(degrees.into_radians() + self.skew);
        let sin_y = f64::sin(degrees.into_radians() + self.skew);
        let matrix = &mut self.transform.matrix;
        matrix.a = (self.scale_x.unit() * cos_x) as f32;
        matrix.b = (self.scale_x.unit() * sin_x) as f32;
        matrix.c = (self.scale_y.unit() * -sin_y) as f32;
        matrix.d = (self.scale_y.unit() * cos_y) as f32;

        changed
    }

    fn scale_x(&mut self) -> Percent {
        self.cache_scale_rotation();
        self.scale_x
    }

    fn set_scale_x(&mut self, mut value: Percent) -> bool {
        let changed = self.scale_x != value;
        self.set_transformed_by_script(true);
        self.cache_scale_rotation();
        self.scale_x = value;

        // Note - in order to match Flash's behavior, the 'scale_x' field is set to NaN
        // (which gets reported back to ActionScript), but we treat it as 0 for
        // the purposes of updating the matrix
        if value.percent().is_nan() {
            value = 0.0.into();
        }

        // Similarly, a rotation of `NaN` can be reported to ActionScript, but we
        // treat it as 0.0 when calculating the matrix
        let mut rot = self.rotation.into_radians();
        if rot.is_nan() {
            rot = 0.0;
        }

        let cos = f64::cos(rot);
        let sin = f64::sin(rot);
        let matrix = &mut self.transform.matrix;
        matrix.a = (cos * value.unit()) as f32;
        matrix.b = (sin * value.unit()) as f32;

        changed
    }

    fn scale_y(&mut self) -> Percent {
        self.cache_scale_rotation();
        self.scale_y
    }

    fn set_scale_y(&mut self, mut value: Percent) -> bool {
        let changed = self.scale_y != value;
        self.set_transformed_by_script(true);
        self.cache_scale_rotation();
        self.scale_y = value;

        // Note - in order to match Flash's behavior, the 'scale_y' field is set to NaN
        // (which gets reported back to ActionScript), but we treat it as 0 for
        // the purposes of updating the matrix
        if value.percent().is_nan() {
            value = 0.0.into();
        }

        // Similarly, a rotation of `NaN` can be reported to ActionScript, but we
        // treat it as 0.0 when calculating the matrix
        let mut rot = self.rotation.into_radians();
        if rot.is_nan() {
            rot = 0.0;
        }

        let cos = f64::cos(rot + self.skew);
        let sin = f64::sin(rot + self.skew);
        let matrix = &mut self.transform.matrix;
        matrix.c = (-sin * value.unit()) as f32;
        matrix.d = (cos * value.unit()) as f32;

        changed
    }

    fn name(&self) -> Option<AvmString<'gc>> {
        self.name
    }

    fn set_name(&mut self, name: AvmString<'gc>) {
        self.name = Some(name);
    }

    fn filters(&self) -> Vec<Filter> {
        self.filters.clone()
    }

    fn set_filters(&mut self, filters: Vec<Filter>) -> bool {
        if filters != self.filters {
            self.filters = filters;
            self.recheck_cache_as_bitmap();
            true
        } else {
            false
        }
    }

    fn alpha(&self) -> f64 {
        f64::from(self.color_transform().a_multiply)
    }

    fn set_alpha(&mut self, value: f64) -> bool {
        let changed = self.alpha() != value;
        self.set_transformed_by_script(true);
        self.color_transform_mut().a_multiply = Fixed8::from_f64(value);
        changed
    }

    fn clip_depth(&self) -> Depth {
        self.clip_depth
    }

    fn set_clip_depth(&mut self, depth: Depth) {
        self.clip_depth = depth;
    }

    fn parent(&self) -> Option<DisplayObject<'gc>> {
        self.parent
    }

    /// You should almost always use `DisplayObject.set_parent` instead, which
    /// properly handles 'orphan' movie clips
    fn set_parent_ignoring_orphan_list(&mut self, parent: Option<DisplayObject<'gc>>) {
        self.parent = parent;
    }

    fn next_avm1_clip(&self) -> Option<DisplayObject<'gc>> {
        self.next_avm1_clip
    }

    fn set_next_avm1_clip(&mut self, node: Option<DisplayObject<'gc>>) {
        self.next_avm1_clip = node;
    }

    fn avm1_removed(&self) -> bool {
        self.flags.contains(DisplayObjectFlags::AVM1_REMOVED)
    }

    fn avm1_pending_removal(&self) -> bool {
        self.flags
            .contains(DisplayObjectFlags::AVM1_PENDING_REMOVAL)
    }

    pub fn should_skip_next_enter_frame(&self) -> bool {
        self.flags
            .contains(DisplayObjectFlags::SKIP_NEXT_ENTER_FRAME)
    }

    pub fn set_skip_next_enter_frame(&mut self, skip: bool) {
        self.flags
            .set(DisplayObjectFlags::SKIP_NEXT_ENTER_FRAME, skip);
    }

    fn set_avm1_removed(&mut self, value: bool) {
        self.flags.set(DisplayObjectFlags::AVM1_REMOVED, value);
    }

    fn set_avm1_pending_removal(&mut self, value: bool) {
        self.flags
            .set(DisplayObjectFlags::AVM1_PENDING_REMOVAL, value);
    }

    fn scale_rotation_cached(&self) -> bool {
        self.flags
            .contains(DisplayObjectFlags::SCALE_ROTATION_CACHED)
    }

    fn set_scale_rotation_cached(&mut self, set_flag: bool) {
        if set_flag {
            self.flags |= DisplayObjectFlags::SCALE_ROTATION_CACHED;
        } else {
            self.flags -= DisplayObjectFlags::SCALE_ROTATION_CACHED;
        }
    }

    pub fn sound_transform(&self) -> &SoundTransform {
        &self.sound_transform
    }

    pub fn set_sound_transform(&mut self, sound_transform: SoundTransform) {
        self.sound_transform = sound_transform;
    }

    fn visible(&self) -> bool {
        self.flags.contains(DisplayObjectFlags::VISIBLE)
    }

    fn set_visible(&mut self, value: bool) -> bool {
        let changed = self.visible() != value;
        self.flags.set(DisplayObjectFlags::VISIBLE, value);
        changed
    }

    fn blend_mode(&self) -> ExtendedBlendMode {
        self.blend_mode
    }

    fn set_blend_mode(&mut self, value: ExtendedBlendMode) -> bool {
        let changed = self.blend_mode != value;
        self.blend_mode = value;
        changed
    }

    fn blend_shader(&self) -> Option<PixelBenderShaderHandle> {
        self.blend_shader.clone()
    }

    fn set_blend_shader(&mut self, value: Option<PixelBenderShaderHandle>) {
        self.blend_shader = value;
    }

    /// The opaque background color of this display object.
    /// The bounding box of the display object will be filled with this color.
    fn opaque_background(&self) -> Option<Color> {
        self.opaque_background
    }

    /// The opaque background color of this display object.
    /// The bounding box of the display object will be filled with the given color. This also
    /// triggers cache-as-bitmap behavior. Only solid backgrounds are supported; the alpha channel
    /// is ignored.
    fn set_opaque_background(&mut self, value: Option<Color>) -> bool {
        let value = value.map(|mut color| {
            color.a = 255;
            color
        });
        let changed = self.opaque_background != value;
        self.opaque_background = value;
        changed
    }

    fn is_root(&self) -> bool {
        self.flags.contains(DisplayObjectFlags::IS_ROOT)
    }

    fn set_is_root(&mut self, value: bool) {
        self.flags.set(DisplayObjectFlags::IS_ROOT, value);
    }

    fn lock_root(&self) -> bool {
        self.flags.contains(DisplayObjectFlags::LOCK_ROOT)
    }

    fn set_lock_root(&mut self, value: bool) {
        self.flags.set(DisplayObjectFlags::LOCK_ROOT, value);
    }

    fn transformed_by_script(&self) -> bool {
        self.flags
            .contains(DisplayObjectFlags::TRANSFORMED_BY_SCRIPT)
    }

    fn set_transformed_by_script(&mut self, value: bool) {
        self.flags
            .set(DisplayObjectFlags::TRANSFORMED_BY_SCRIPT, value);
    }

    fn placed_by_script(&self) -> bool {
        self.flags.contains(DisplayObjectFlags::PLACED_BY_SCRIPT)
    }

    fn set_placed_by_script(&mut self, value: bool) {
        self.flags.set(DisplayObjectFlags::PLACED_BY_SCRIPT, value);
    }

    fn is_bitmap_cached_preference(&self) -> bool {
        self.flags.contains(DisplayObjectFlags::CACHE_AS_BITMAP)
    }

    fn set_bitmap_cached_preference(&mut self, value: bool) {
        self.flags.set(DisplayObjectFlags::CACHE_AS_BITMAP, value);
        self.recheck_cache_as_bitmap();
    }

    fn bitmap_cache_mut(&mut self) -> Option<&mut BitmapCache> {
        self.cache.as_mut()
    }

    /// Invalidates a cached bitmap, if it exists.
    /// This may only be called once per frame - the first call will return true, regardless of
    /// if there was a cache.
    /// Any subsequent calls will return false, indicating that you do not need to invalidate the ancestors.
    /// This is reset during rendering.
    fn invalidate_cached_bitmap(&mut self) -> bool {
        if self.flags.contains(DisplayObjectFlags::CACHE_INVALIDATED) {
            return false;
        }
        if let Some(cache) = &mut self.cache {
            cache.make_dirty();
        }
        self.flags.insert(DisplayObjectFlags::CACHE_INVALIDATED);
        true
    }

    fn clear_invalidate_flag(&mut self) {
        self.flags.remove(DisplayObjectFlags::CACHE_INVALIDATED);
    }

    fn recheck_cache_as_bitmap(&mut self) {
        let should_cache = self.is_bitmap_cached_preference() || !self.filters.is_empty();
        if should_cache && self.cache.is_none() {
            self.cache = Some(Default::default());
        } else if !should_cache && self.cache.is_some() {
            self.cache = None;
        }
    }

    fn instantiated_by_timeline(&self) -> bool {
        self.flags
            .contains(DisplayObjectFlags::INSTANTIATED_BY_TIMELINE)
    }

    fn set_instantiated_by_timeline(&mut self, value: bool) {
        self.flags
            .set(DisplayObjectFlags::INSTANTIATED_BY_TIMELINE, value);
    }

    fn has_scroll_rect(&self) -> bool {
        self.flags.contains(DisplayObjectFlags::HAS_SCROLL_RECT)
    }

    fn set_has_scroll_rect(&mut self, value: bool) {
        self.flags.set(DisplayObjectFlags::HAS_SCROLL_RECT, value);
    }

    fn has_explicit_name(&self) -> bool {
        self.flags.contains(DisplayObjectFlags::HAS_EXPLICIT_NAME)
    }

    fn set_has_explicit_name(&mut self, value: bool) {
        self.flags.set(DisplayObjectFlags::HAS_EXPLICIT_NAME, value);
    }

    fn masker(&self) -> Option<DisplayObject<'gc>> {
        self.masker
    }

    fn set_masker(&mut self, node: Option<DisplayObject<'gc>>) {
        self.masker = node;
    }

    fn maskee(&self) -> Option<DisplayObject<'gc>> {
        self.maskee
    }

    fn set_maskee(&mut self, node: Option<DisplayObject<'gc>>) {
        self.maskee = node;
    }

    fn meta_data(&self) -> Option<Avm2Object<'gc>> {
        self.meta_data
    }

    fn set_meta_data(&mut self, value: Avm2Object<'gc>) {
        self.meta_data = Some(value);
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

pub fn render_base<'gc>(this: DisplayObject<'gc>, context: &mut RenderContext<'_, 'gc>) {
    if this.maskee().is_some() {
        return;
    }
    context.transform_stack.push(this.base().transform());
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
        let mut filters: Vec<Filter> = this.filters();
        let swf_version = this.swf_version();
        filters.retain(|f| !f.impotent());

        if let Some(cache) = this.base_mut(context.gc_context).bitmap_cache_mut() {
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
        apply_standard_mask_and_scroll(this, context, |context| {
            context.commands.render_bitmap(
                cache_info.handle,
                Transform {
                    matrix: Matrix {
                        tx: context.transform_stack.transform().matrix.tx + offset_x,
                        ty: context.transform_stack.transform().matrix.ty + offset_y,
                        ..Default::default()
                    },
                    color_transform: cache_info.base_transform.color_transform,
                },
                true,
                PixelSnapping::Always, // cacheAsBitmap forces pixel snapping
            )
        });
    } else {
        if let Some(background) = this.opaque_background() {
            // This is intended for use with cacheAsBitmap, but can be set for non-cached objects too
            // It wants the entire bounding box to be cleared before any draws happen
            let bounds: Rectangle<Twips> = this.render_bounds_with_transform(
                &context.transform_stack.transform().matrix,
                true,
                &context.stage.view_matrix(),
            );
            context.commands.draw_rect(
                background,
                Matrix::create_box(
                    bounds.width().to_pixels() as f32,
                    bounds.height().to_pixels() as f32,
                    bounds.x_min,
                    bounds.y_min,
                ),
            );
        }
        apply_standard_mask_and_scroll(this, context, |context| this.render_self(context));
    }

    if let Some(original_commands) = original_commands {
        let sub_commands = std::mem::replace(&mut context.commands, original_commands);
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

    context.transform_stack.pop();
}

/// This applies the **standard** method of `mask` and `scrollRect`.
///
/// It uses the stencil buffer so that any pixel drawn in the mask will allow the inner contents to show.
/// This is what is used for most cases, except for cacheAsBitmap-on-cacheAsBitmap.
pub fn apply_standard_mask_and_scroll<'gc, F>(
    this: DisplayObject<'gc>,
    context: &mut RenderContext<'_, 'gc>,
    draw: F,
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
        });
    }

    let mask = this.masker();
    let mut mask_transform = ruffle_render::transform::Transform::default();
    if let Some(m) = mask {
        mask_transform.matrix = this.global_to_local_matrix().unwrap_or_default();
        mask_transform.matrix *= m.local_to_global_matrix();
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

    draw(context);

    if let Some(rect_mat) = scroll_rect_matrix {
        // Draw the rectangle again after deactivating the mask,
        // to reset the stencil buffer.
        context.commands.deactivate_mask();
        context.commands.draw_rect(Color::WHITE, rect_mat);
        context.commands.pop_mask();
    }

    if let Some(m) = mask {
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
    'gc + Clone + Copy + Collect + Debug + Into<DisplayObject<'gc>>
{
    fn base<'a>(&'a self) -> Ref<'a, DisplayObjectBase<'gc>>;
    fn base_mut<'a>(&'a self, mc: &Mutation<'gc>) -> RefMut<'a, DisplayObjectBase<'gc>>;

    /// The `SCALE_ROTATION_CACHED` flag should only be set in SWFv5+.
    /// So scaling/rotation values always have to get recalculated from the matrix in SWFv4.
    fn set_scale_rotation_cached(&self, gc_context: &Mutation<'gc>) {
        if self.swf_version() >= 5 {
            self.base_mut(gc_context).set_scale_rotation_cached(true);
        }
    }

    fn id(&self) -> CharacterId;
    fn depth(&self) -> Depth {
        self.base().depth()
    }

    fn set_depth(&self, gc_context: &Mutation<'gc>, depth: Depth) {
        self.base_mut(gc_context).set_depth(depth)
    }

    /// The untransformed inherent bounding box of this object.
    /// These bounds do **not** include child DisplayObjects.
    /// To get the bounds including children, use `bounds`, `local_bounds`, or `world_bounds`.
    ///
    /// Implementors must override this method.
    /// Leaf DisplayObjects should return their bounds.
    /// Composite DisplayObjects that only contain children should return `&Default::default()`
    fn self_bounds(&self) -> Rectangle<Twips>;

    /// The untransformed bounding box of this object including children.
    fn bounds(&self) -> Rectangle<Twips> {
        self.bounds_with_transform(&Matrix::default())
    }

    /// The local bounding box of this object including children, in its parent's coordinate system.
    fn local_bounds(&self) -> Rectangle<Twips> {
        self.bounds_with_transform(self.base().matrix())
    }

    /// The world bounding box of this object including children, relative to the stage.
    fn world_bounds(&self) -> Rectangle<Twips> {
        self.bounds_with_transform(&self.local_to_global_matrix())
    }

    /// Bounds used for drawing debug rects and picking objects.
    fn debug_rect_bounds(&self) -> Rectangle<Twips> {
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
    fn bounds_with_transform(&self, matrix: &Matrix) -> Rectangle<Twips> {
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
                let matrix = *matrix * *child.base().matrix();
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
        &self,
        matrix: &Matrix,
        include_own_filters: bool,
        view_matrix: &Matrix,
    ) -> Rectangle<Twips> {
        let mut bounds = *matrix * self.self_bounds();

        if let Some(ctr) = self.as_container() {
            for child in ctr.iter_render_list() {
                let matrix = *matrix * *child.base().matrix();
                bounds =
                    bounds.union(&child.render_bounds_with_transform(&matrix, true, view_matrix));
            }
        }

        if include_own_filters {
            let filters = self.filters();
            for mut filter in filters {
                filter.scale(view_matrix.a, view_matrix.d);
                bounds = filter.calculate_dest_rect(bounds);
            }
        }

        bounds
    }

    fn place_frame(&self) -> u16 {
        self.base().place_frame()
    }
    fn set_place_frame(&self, gc_context: &Mutation<'gc>, frame: u16) {
        self.base_mut(gc_context).set_place_frame(frame)
    }

    /// Sets the matrix of this object.
    /// This does NOT invalidate the cache, as it's often used with other operations.
    /// It is the callers responsibility to do so.
    fn set_matrix(&self, gc_context: &Mutation<'gc>, matrix: Matrix) {
        self.base_mut(gc_context).set_matrix(matrix);
    }

    /// Sets the color transform of this object.
    /// This does NOT invalidate the cache, as it's often used with other operations.
    /// It is the callers responsibility to do so.
    fn set_color_transform(&self, gc_context: &Mutation<'gc>, color_transform: ColorTransform) {
        self.base_mut(gc_context)
            .set_color_transform(color_transform)
    }

    /// Should only be used to implement 'Transform.concatenatedMatrix'
    fn local_to_global_matrix_without_own_scroll_rect(&self) -> Matrix {
        let mut node = self.parent();
        let mut matrix = *self.base().matrix();
        while let Some(display_object) = node {
            // We want to transform to Stage-local coordinates,
            // so do *not* apply the Stage's matrix
            if display_object.as_stage().is_some() {
                break;
            }
            if let Some(rect) = display_object.scroll_rect() {
                matrix = Matrix::translate(-rect.x_min, -rect.y_min) * matrix;
            }
            matrix = *display_object.base().matrix() * matrix;
            node = display_object.parent();
        }
        matrix
    }

    /// Returns the matrix for transforming from this object's local space to global stage space.
    fn local_to_global_matrix(&self) -> Matrix {
        let mut matrix = Matrix::IDENTITY;
        if let Some(rect) = self.scroll_rect() {
            matrix = Matrix::translate(-rect.x_min, -rect.y_min) * matrix;
        }
        self.local_to_global_matrix_without_own_scroll_rect() * matrix
    }

    /// Returns the matrix for transforming from global stage to this object's local space.
    /// `None` is returned if the object has zero scale.
    fn global_to_local_matrix(&self) -> Option<Matrix> {
        self.local_to_global_matrix().inverse()
    }

    /// Converts a local position to a global stage position
    fn local_to_global(&self, local: Point<Twips>) -> Point<Twips> {
        self.local_to_global_matrix() * local
    }

    /// Converts a local position on the stage to a local position on this display object
    /// Returns `None` if the object has zero scale.
    fn global_to_local(&self, global: Point<Twips>) -> Option<Point<Twips>> {
        self.global_to_local_matrix().map(|matrix| matrix * global)
    }

    /// Converts the mouse position on the stage to a local position on this display object.
    /// If the object has zero scale, then the stage `TWIPS_TO_PIXELS` matrix will be used.
    /// This matches Flash's behavior for `mouseX`/`mouseY` on an object with zero scale.
    fn local_mouse_position(&self, context: &UpdateContext<'gc>) -> Point<Twips> {
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
    fn x(&self) -> Twips {
        self.base().x()
    }

    /// Sets the `x` position in pixels of this display object in local space.
    /// Set by the `_x`/`x` ActionScript properties.
    /// This invalidates any ancestors cacheAsBitmap automatically.
    fn set_x(&self, gc_context: &Mutation<'gc>, x: Twips) {
        if self.base_mut(gc_context).set_x(x) {
            if let Some(parent) = self.parent() {
                // Self-transform changes are automatically handled,
                // we only want to inform ancestors to avoid unnecessary invalidations for tx/ty
                parent.invalidate_cached_bitmap(gc_context);
            }
        }
    }

    /// The `y` position in pixels of this display object in local space.
    /// Returned by the `_y`/`y` ActionScript properties.
    fn y(&self) -> Twips {
        self.base().y()
    }

    /// Sets the `y` position in pixels of this display object in local space.
    /// Set by the `_y`/`y` ActionScript properties.
    /// This invalidates any ancestors cacheAsBitmap automatically.
    fn set_y(&self, gc_context: &Mutation<'gc>, y: Twips) {
        if self.base_mut(gc_context).set_y(y) {
            if let Some(parent) = self.parent() {
                // Self-transform changes are automatically handled,
                // we only want to inform ancestors to avoid unnecessary invalidations for tx/ty
                parent.invalidate_cached_bitmap(gc_context);
            }
        }
    }

    /// The rotation in degrees this display object in local space.
    /// Returned by the `_rotation`/`rotation` ActionScript properties.
    fn rotation(&self, gc_context: &Mutation<'gc>) -> Degrees {
        let degrees = self.base_mut(gc_context).rotation();
        self.set_scale_rotation_cached(gc_context);
        degrees
    }

    /// Sets the rotation in degrees this display object in local space.
    /// Set by the `_rotation`/`rotation` ActionScript properties.
    /// This invalidates any ancestors cacheAsBitmap automatically.
    fn set_rotation(&self, gc_context: &Mutation<'gc>, radians: Degrees) {
        if self.base_mut(gc_context).set_rotation(radians) {
            self.set_scale_rotation_cached(gc_context);
            if let Some(parent) = self.parent() {
                // Self-transform changes are automatically handled,
                // we only want to inform ancestors to avoid unnecessary invalidations for tx/ty
                parent.invalidate_cached_bitmap(gc_context);
            }
        }
    }

    /// The X axis scale for this display object in local space.
    /// Returned by the `_xscale`/`scaleX` ActionScript properties.
    fn scale_x(&self, gc_context: &Mutation<'gc>) -> Percent {
        let percent = self.base_mut(gc_context).scale_x();
        self.set_scale_rotation_cached(gc_context);
        percent
    }

    /// Sets the X axis scale for this display object in local space.
    /// Set by the `_xscale`/`scaleX` ActionScript properties.
    /// This invalidates any ancestors cacheAsBitmap automatically.
    fn set_scale_x(&self, gc_context: &Mutation<'gc>, value: Percent) {
        if self.base_mut(gc_context).set_scale_x(value) {
            self.set_scale_rotation_cached(gc_context);
            if let Some(parent) = self.parent() {
                // Self-transform changes are automatically handled,
                // we only want to inform ancestors to avoid unnecessary invalidations for tx/ty
                parent.invalidate_cached_bitmap(gc_context);
            }
        }
    }

    /// The Y axis scale for this display object in local space.
    /// Returned by the `_yscale`/`scaleY` ActionScript properties.
    fn scale_y(&self, gc_context: &Mutation<'gc>) -> Percent {
        let percent = self.base_mut(gc_context).scale_y();
        self.set_scale_rotation_cached(gc_context);
        percent
    }

    /// Sets the Y axis scale for this display object in local space.
    /// Returned by the `_yscale`/`scaleY` ActionScript properties.
    /// This invalidates any ancestors cacheAsBitmap automatically.
    fn set_scale_y(&self, gc_context: &Mutation<'gc>, value: Percent) {
        if self.base_mut(gc_context).set_scale_y(value) {
            self.set_scale_rotation_cached(gc_context);
            if let Some(parent) = self.parent() {
                // Self-transform changes are automatically handled,
                // we only want to inform ancestors to avoid unnecessary invalidations for tx/ty
                parent.invalidate_cached_bitmap(gc_context);
            }
        }
    }

    /// Gets the pixel width of the AABB containing this display object in local space.
    /// Returned by the ActionScript `_width`/`width` properties.
    fn width(&self) -> f64 {
        self.local_bounds().width().to_pixels()
    }

    /// Sets the pixel width of this display object in local space.
    /// The width is based on the AABB of the object.
    /// Set by the ActionScript `_width`/`width` properties.
    /// This does odd things on rotated clips to match the behavior of Flash.
    fn set_width(&self, context: &mut UpdateContext<'gc>, value: f64) {
        let gc_context = context.gc_context;
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
        let prev_scale_x = self.scale_x(gc_context).unit();
        let prev_scale_y = self.scale_y(gc_context).unit();
        let rotation = self.rotation(gc_context);
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

        self.set_scale_x(gc_context, Percent::from_unit(new_scale_x));
        self.set_scale_y(gc_context, Percent::from_unit(new_scale_y));
    }

    /// Gets the pixel height of the AABB containing this display object in local space.
    /// Returned by the ActionScript `_height`/`height` properties.
    fn height(&self) -> f64 {
        self.local_bounds().height().to_pixels()
    }

    /// Sets the pixel height of this display object in local space.
    /// Set by the ActionScript `_height`/`height` properties.
    /// This does odd things on rotated clips to match the behavior of Flash.
    fn set_height(&self, context: &mut UpdateContext<'gc>, value: f64) {
        let gc_context = context.gc_context;
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
        let prev_scale_x = self.scale_x(gc_context).unit();
        let prev_scale_y = self.scale_y(gc_context).unit();
        let rotation = self.rotation(gc_context);
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

        self.set_scale_x(gc_context, Percent::from_unit(new_scale_x));
        self.set_scale_y(gc_context, Percent::from_unit(new_scale_y));
    }

    /// The opacity of this display object.
    /// 1 is fully opaque.
    /// Returned by the `_alpha`/`alpha` ActionScript properties.
    fn alpha(&self) -> f64 {
        self.base().alpha()
    }

    /// Sets the opacity of this display object.
    /// 1 is fully opaque.
    /// Set by the `_alpha`/`alpha` ActionScript properties.
    /// This invalidates any cacheAsBitmap automatically.
    fn set_alpha(&self, gc_context: &Mutation<'gc>, value: f64) {
        if self.base_mut(gc_context).set_alpha(value) {
            if let Some(parent) = self.parent() {
                // Self-transform changes are automatically handled
                parent.invalidate_cached_bitmap(gc_context);
            }
        }
    }

    fn name(&self) -> AvmString<'gc> {
        self.base().name().unwrap_or_default()
    }
    fn name_optional(&self) -> Option<AvmString<'gc>> {
        self.base().name()
    }
    fn set_name(&self, gc_context: &Mutation<'gc>, name: AvmString<'gc>) {
        self.base_mut(gc_context).set_name(name)
    }

    fn filters(&self) -> Vec<Filter> {
        self.base().filters()
    }

    fn set_filters(&self, gc_context: &Mutation<'gc>, filters: Vec<Filter>) {
        if self.base_mut(gc_context).set_filters(filters) {
            self.invalidate_cached_bitmap(gc_context);
        }
    }

    /// Returns the dot-syntax path to this display object, e.g. `_level0.foo.clip`
    fn path(&self) -> WString {
        if let Some(parent) = self.avm1_parent() {
            let mut path = parent.path();
            path.push_byte(b'.');
            path.push_str(&self.name());
            path
        } else {
            WString::from_utf8_owned(format!("_level{}", self.depth()))
        }
    }

    /// Returns the Flash 4 slash-syntax path to this display object, e.g. `/foo/clip`.
    /// Returned by the `_target` property in AVM1.
    fn slash_path(&self) -> WString {
        fn build_slash_path(object: DisplayObject<'_>) -> WString {
            if let Some(parent) = object.avm1_parent() {
                let mut path = build_slash_path(parent);
                path.push_byte(b'/');
                path.push_str(&object.name());
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
            build_slash_path((*self).into())
        } else {
            // _target of _level0 should just be '/'.
            WString::from_unit(b'/'.into())
        }
    }

    fn clip_depth(&self) -> Depth {
        self.base().clip_depth()
    }
    fn set_clip_depth(&self, gc_context: &Mutation<'gc>, depth: Depth) {
        self.base_mut(gc_context).set_clip_depth(depth);
    }

    /// Retrieve the parent of this display object.
    ///
    /// This version of the function merely exposes the display object parent,
    /// without any further filtering.
    fn parent(&self) -> Option<DisplayObject<'gc>> {
        self.base().parent()
    }

    /// Set the parent of this display object.
    fn set_parent(&self, context: &mut UpdateContext<'gc>, parent: Option<DisplayObject<'gc>>) {
        let had_parent = self.parent().is_some();
        self.base_mut(context.gc_context)
            .set_parent_ignoring_orphan_list(parent);
        let has_parent = self.parent().is_some();
        let parent_removed = had_parent && !has_parent;

        if parent_removed {
            if let Some(int) = self.as_interactive() {
                int.drop_focus(context);
            }

            self.on_parent_removed(context);
        }
    }

    /// This method is called when the parent is removed.
    /// It may be overwritten to inject some implementation-specific behavior.
    fn on_parent_removed(&self, _context: &mut UpdateContext<'gc>) {}

    /// Retrieve the parent of this display object.
    ///
    /// This version of the function implements the concept of parenthood as
    /// seen in AVM1. Notably, it disallows access to the `Stage` and to
    /// non-AVM1 DisplayObjects; for an unfiltered concept of parent,
    /// use the `parent` method.
    fn avm1_parent(&self) -> Option<DisplayObject<'gc>> {
        self.parent()
            .filter(|p| p.as_stage().is_none())
            .filter(|p| !p.movie().is_action_script_3())
    }

    /// Retrieve the parent of this display object.
    ///
    /// This version of the function implements the concept of parenthood as
    /// seen in AVM2. Notably, it disallows access to non-container parents.
    fn avm2_parent(&self) -> Option<DisplayObject<'gc>> {
        self.parent().filter(|p| p.as_container().is_some())
    }

    fn next_avm1_clip(&self) -> Option<DisplayObject<'gc>> {
        self.base().next_avm1_clip()
    }
    fn set_next_avm1_clip(&self, gc_context: &Mutation<'gc>, node: Option<DisplayObject<'gc>>) {
        self.base_mut(gc_context).set_next_avm1_clip(node);
    }
    fn masker(&self) -> Option<DisplayObject<'gc>> {
        self.base().masker()
    }
    fn set_masker(
        &self,
        gc_context: &Mutation<'gc>,
        node: Option<DisplayObject<'gc>>,
        remove_old_link: bool,
    ) {
        if remove_old_link {
            let old_masker = self.base().masker();
            if let Some(old_masker) = old_masker {
                old_masker.set_maskee(gc_context, None, false);
            }
            if let Some(parent) = self.parent() {
                // Masks are natively handled by cacheAsBitmap - don't invalidate self, only parents
                parent.invalidate_cached_bitmap(gc_context);
            }
        }
        self.base_mut(gc_context).set_masker(node);
    }
    fn maskee(&self) -> Option<DisplayObject<'gc>> {
        self.base().maskee()
    }
    fn set_maskee(
        &self,
        gc_context: &Mutation<'gc>,
        node: Option<DisplayObject<'gc>>,
        remove_old_link: bool,
    ) {
        if remove_old_link {
            let old_maskee = self.base().maskee();
            if let Some(old_maskee) = old_maskee {
                old_maskee.set_masker(gc_context, None, false);
            }
            self.invalidate_cached_bitmap(gc_context);
        }
        self.base_mut(gc_context).set_maskee(node);
    }

    fn scroll_rect(&self) -> Option<Rectangle<Twips>> {
        self.base().scroll_rect.clone()
    }

    fn next_scroll_rect(&self) -> Rectangle<Twips> {
        self.base().next_scroll_rect.clone()
    }

    fn set_next_scroll_rect(&self, gc_context: &Mutation<'gc>, rectangle: Rectangle<Twips>) {
        self.base_mut(gc_context).next_scroll_rect = rectangle;

        // Scroll rect is natively handled by cacheAsBitmap - don't invalidate self, only parents
        if let Some(parent) = self.parent() {
            parent.invalidate_cached_bitmap(gc_context);
        }
    }

    fn scaling_grid(&self) -> Rectangle<Twips> {
        self.base().scaling_grid.clone()
    }

    fn set_scaling_grid(&self, gc_context: &Mutation<'gc>, rect: Rectangle<Twips>) {
        self.base_mut(gc_context).scaling_grid = rect;
    }

    /// Whether this object has been removed. Only applies to AVM1.
    fn avm1_removed(&self) -> bool {
        self.base().avm1_removed()
    }

    // Sets whether this object has been removed. Only applies to AVM1
    fn set_avm1_removed(&self, gc_context: &Mutation<'gc>, value: bool) {
        self.base_mut(gc_context).set_avm1_removed(value)
    }

    /// Is this object waiting to be removed on the start of the next frame
    fn avm1_pending_removal(&self) -> bool {
        self.base().avm1_pending_removal()
    }

    fn set_avm1_pending_removal(&self, gc_context: &Mutation<'gc>, value: bool) {
        self.base_mut(gc_context).set_avm1_pending_removal(value)
    }

    /// Whether this display object is visible.
    /// Invisible objects are not rendered, but otherwise continue to exist normally.
    /// Returned by the `_visible`/`visible` ActionScript properties.
    fn visible(&self) -> bool {
        self.base().visible()
    }

    /// Sets whether this display object will be visible.
    /// Invisible objects are not rendered, but otherwise continue to exist normally.
    /// Returned by the `_visible`/`visible` ActionScript properties.
    fn set_visible(&self, context: &mut UpdateContext<'gc>, value: bool) {
        if self.base_mut(context.gc()).set_visible(value) {
            if let Some(parent) = self.parent() {
                // We don't need to invalidate ourselves, we're just toggling if the bitmap is rendered.
                parent.invalidate_cached_bitmap(context.gc());
            }
        }

        if !value {
            if let Some(int) = self.as_interactive() {
                // The focus is dropped when it's made invisible.
                int.drop_focus(context);
            }
        }
    }

    fn meta_data(&self) -> Option<Avm2Object<'gc>> {
        self.base().meta_data()
    }

    fn set_meta_data(&self, gc_context: &Mutation<'gc>, value: Avm2Object<'gc>) {
        self.base_mut(gc_context).set_meta_data(value);
    }

    /// The blend mode used when rendering this display object.
    /// Values other than the default `BlendMode::Normal` implicitly cause cache-as-bitmap behavior.
    fn blend_mode(&self) -> ExtendedBlendMode {
        self.base().blend_mode()
    }

    /// Sets the blend mode used when rendering this display object.
    /// Values other than the default `BlendMode::Normal` implicitly cause cache-as-bitmap behavior.
    fn set_blend_mode(&self, gc_context: &Mutation<'gc>, value: ExtendedBlendMode) {
        if self.base_mut(gc_context).set_blend_mode(value) {
            if let Some(parent) = self.parent() {
                // We don't need to invalidate ourselves, we're just toggling how the bitmap is rendered.

                // Note that Flash does not always invalidate on changing the blend mode;
                // but that's a bug we don't need to copy :)
                parent.invalidate_cached_bitmap(gc_context);
            }
        }
    }

    fn blend_shader(&self) -> Option<PixelBenderShaderHandle> {
        self.base().blend_shader()
    }

    fn set_blend_shader(&self, gc_context: &Mutation<'gc>, value: Option<PixelBenderShaderHandle>) {
        self.base_mut(gc_context).set_blend_shader(value);
        self.set_blend_mode(gc_context, ExtendedBlendMode::Shader);
    }

    /// The opaque background color of this display object.
    fn opaque_background(&self) -> Option<Color> {
        self.base().opaque_background()
    }

    /// Sets the opaque background color of this display object.
    /// The bounding box of the display object will be filled with the given color. This also
    /// triggers cache-as-bitmap behavior. Only solid backgrounds are supported; the alpha channel
    /// is ignored.
    fn set_opaque_background(&self, gc_context: &Mutation<'gc>, value: Option<Color>) {
        if self.base_mut(gc_context).set_opaque_background(value) {
            self.invalidate_cached_bitmap(gc_context);
        }
    }

    /// Whether this display object represents the root of loaded content.
    fn is_root(&self) -> bool {
        self.base().is_root()
    }

    /// Sets whether this display object represents the root of loaded content.
    fn set_is_root(&self, gc_context: &Mutation<'gc>, value: bool) {
        self.base_mut(gc_context).set_is_root(value);
    }

    /// The sound transform for sounds played inside this display object.
    fn set_sound_transform(
        &self,
        context: &mut UpdateContext<'gc>,
        sound_transform: SoundTransform,
    ) {
        self.base_mut(context.gc_context)
            .set_sound_transform(sound_transform);
        context.set_sound_transforms_dirty();
    }

    /// Whether this display object is used as the _root of itself and its children.
    /// Returned by the `_lockroot` ActionScript property.
    fn lock_root(&self) -> bool {
        self.base().lock_root()
    }

    /// Sets whether this display object is used as the _root of itself and its children.
    /// Returned by the `_lockroot` ActionScript property.
    fn set_lock_root(&self, gc_context: &Mutation<'gc>, value: bool) {
        self.base_mut(gc_context).set_lock_root(value);
    }

    /// Whether this display object has been transformed by ActionScript.
    /// When this flag is set, changes from SWF `PlaceObject` tags are ignored.
    fn transformed_by_script(&self) -> bool {
        self.base().transformed_by_script()
    }

    /// Sets whether this display object has been transformed by ActionScript.
    /// When this flag is set, changes from SWF `PlaceObject` tags are ignored.
    fn set_transformed_by_script(&self, gc_context: &Mutation<'gc>, value: bool) {
        self.base_mut(gc_context).set_transformed_by_script(value)
    }

    /// Whether this display object prefers to be cached into a bitmap rendering.
    /// This is the PlaceObject `cacheAsBitmap` flag - and may be overridden if filters are applied.
    /// Consider `is_bitmap_cached` for if a bitmap cache is actually in use.
    fn is_bitmap_cached_preference(&self) -> bool {
        self.base().is_bitmap_cached_preference()
    }

    /// Whether this display object is using a bitmap cache, whether by preference or necessity.
    fn is_bitmap_cached(&self) -> bool {
        self.base().cache.is_some()
    }

    /// Explicitly sets the preference of this display object to be cached into a bitmap rendering.
    /// Note that the object will still be bitmap cached if a filter is active.
    fn set_bitmap_cached_preference(&self, gc_context: &Mutation<'gc>, value: bool) {
        self.base_mut(gc_context)
            .set_bitmap_cached_preference(value)
    }

    /// Whether this display object has a scroll rectangle applied.
    fn has_scroll_rect(&self) -> bool {
        self.base().has_scroll_rect()
    }

    /// Sets whether this display object has a scroll rectangle applied.
    fn set_has_scroll_rect(&self, gc_context: &Mutation<'gc>, value: bool) {
        self.base_mut(gc_context).set_has_scroll_rect(value)
    }

    /// Whether this display object has been created by ActionScript 3.
    /// When this flag is set, changes from SWF `RemoveObject` tags are
    /// ignored.
    fn placed_by_script(&self) -> bool {
        self.base().placed_by_script()
    }

    /// Sets whether this display object has been created by ActionScript 3.
    /// When this flag is set, changes from SWF `RemoveObject` tags are
    /// ignored.
    fn set_placed_by_script(&self, gc_context: &Mutation<'gc>, value: bool) {
        self.base_mut(gc_context).set_placed_by_script(value)
    }

    /// Whether this display object has been instantiated by the timeline.
    /// When this flag is set, attempts to change the object's name from AVM2
    /// throw an exception.
    fn instantiated_by_timeline(&self) -> bool {
        self.base().instantiated_by_timeline()
    }

    /// Sets whether this display object has been instantiated by the timeline.
    /// When this flag is set, attempts to change the object's name from AVM2
    /// throw an exception.
    fn set_instantiated_by_timeline(&self, gc_context: &Mutation<'gc>, value: bool) {
        self.base_mut(gc_context)
            .set_instantiated_by_timeline(value);
    }

    /// Whether this display object was placed by a SWF tag with an explicit
    /// name.
    ///
    /// When this flag is set, the object will attempt to set a dynamic property
    /// on the parent with the same name as itself.
    fn has_explicit_name(&self) -> bool {
        self.base().has_explicit_name()
    }

    /// Sets whether this display object was placed by a SWF tag with an
    /// explicit name.
    ///
    /// When this flag is set, the object will attempt to set a dynamic property
    /// on the parent with the same name as itself.
    fn set_has_explicit_name(&self, gc_context: &Mutation<'gc>, value: bool) {
        self.base_mut(gc_context).set_has_explicit_name(value);
    }
    fn state(&self) -> Option<ButtonState> {
        None
    }
    fn set_state(self, _context: &mut UpdateContext<'gc>, _state: ButtonState) {}
    /// Run any start-of-frame actions for this display object.
    ///
    /// When fired on `Stage`, this also emits the AVM2 `enterFrame` broadcast.
    fn enter_frame(&self, _context: &mut UpdateContext<'gc>) {}

    /// Construct all display objects that the timeline indicates should exist
    /// this frame, and their children.
    ///
    /// This function should ensure the following, from the point of view of
    /// downstream VMs:
    ///
    /// 1. That the object itself has been allocated, if not constructed
    /// 2. That newly created children have been instantiated and are present
    ///    as properties on the class
    fn construct_frame(&self, _context: &mut UpdateContext<'gc>) {}

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
    #[inline(never)]
    fn on_construction_complete(&self, context: &mut UpdateContext<'gc>) {
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
                .as_object()
                .expect("MovieClip object should have been constructed");
            let movieclip_class = context.avm2.classes().movieclip.inner_class_definition();
            // It's possible to have a DefineSprite tag with multiple frames, but have
            // the corresponding `SymbolClass` *not* extend `MovieClip` (e.g. extending `Sprite` directly.)
            // When this occurs, Flash Player will run the first frame, and immediately stop.
            // However, Flash Player runs frames for the root movie clip, even if it doesn't extend `MovieClip`.
            if !obj.is_of_type(movieclip_class) && !movie.is_root() {
                movie.stop(context);
            }
            movie.set_initialized(context.gc_context);
        }
    }

    fn fire_added_events(&self, context: &mut UpdateContext<'gc>) {
        if !self.placed_by_script() {
            // Since we construct AVM2 display objects after they are
            // allocated and placed on the render list, we have to emit all
            // events after this point.
            //
            // Children added to buttons by the timeline do not emit events.
            if self.parent().and_then(|p| p.as_avm2_button()).is_none() {
                dispatch_added_event_only((*self).into(), context);
                if self.avm2_stage(context).is_some() {
                    dispatch_added_to_stage_event_only((*self).into(), context);
                }
            }
        }
    }

    fn set_on_parent_field(&self, context: &mut UpdateContext<'gc>) {
        //TODO: Don't report missing property errors.
        //TODO: Don't attempt to set properties if object was placed without a name.
        if self.has_explicit_name() {
            if let Some(Avm2Value::Object(p)) = self.parent().map(|p| p.object2()) {
                if let Avm2Value::Object(c) = self.object2() {
                    let domain = context
                        .library
                        .library_for_movie(self.movie())
                        .unwrap()
                        .avm2_domain();
                    let mut activation = Avm2Activation::from_domain(context, domain);
                    let name =
                        Avm2Multiname::new(activation.avm2().find_public_namespace(), self.name());
                    if let Err(e) = p.init_property(&name, c.into(), &mut activation) {
                        tracing::error!(
                            "Got error when setting AVM2 child named \"{}\": {}",
                            &self.name(),
                            e
                        );
                    }
                }
            }
        }
    }

    /// Execute all other timeline actions on this object.
    fn run_frame_avm1(&self, _context: &mut UpdateContext<'gc>) {}

    /// Emit a `frameConstructed` event on this DisplayObject and any children it
    /// may have.
    fn frame_constructed(&self, context: &mut UpdateContext<'gc>) {
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
    fn exit_frame(&self, context: &mut UpdateContext<'gc>) {
        let exit_frame_evt = Avm2EventObject::bare_default_event(context, "exitFrame");
        let dobject_constr = context.avm2.classes().display_object;
        Avm2::broadcast_event(context, exit_frame_evt, dobject_constr);

        self.on_exit_frame(context);
    }

    fn on_exit_frame(&self, context: &mut UpdateContext<'gc>) {
        if let Some(container) = self.as_container() {
            for child in container.iter_render_list() {
                child.on_exit_frame(context);
            }
        }
    }

    /// Called before the child is about to be rendered.
    /// Note that this happens even if the child is invisible
    /// (as long as the child is still on a render list)
    fn pre_render(&self, context: &mut RenderContext<'_, 'gc>) {
        let mut this = self.base_mut(context.gc_context);
        this.clear_invalidate_flag();
        this.scroll_rect = this
            .has_scroll_rect()
            .then(|| this.next_scroll_rect.clone());
    }

    fn render_self(&self, _context: &mut RenderContext<'_, 'gc>) {}

    fn render(&self, context: &mut RenderContext<'_, 'gc>) {
        render_base((*self).into(), context)
    }

    #[cfg(not(feature = "avm_debug"))]
    fn display_render_tree(&self, _depth: usize) {}

    #[cfg(feature = "avm_debug")]
    fn display_render_tree(&self, depth: usize) {
        let mut self_str = &*format!("{self:?}");
        if let Some(end_char) = self_str.find(|c: char| !c.is_ascii_alphanumeric()) {
            self_str = &self_str[..end_char];
        }

        let bounds = self.world_bounds();

        let mut classname = "".to_string();
        if let Some(o) = self.object2().as_object() {
            classname = format!("{:?}", o.base().debug_class_name());
        }

        println!(
            "{} rel({},{}) abs({},{}) {} {} {} id={} depth={}",
            " ".repeat(depth),
            self.x(),
            self.y(),
            bounds.x_min.to_pixels(),
            bounds.y_min.to_pixels(),
            classname,
            self.name(),
            self_str,
            self.id(),
            depth
        );

        if let Some(ctr) = self.as_container() {
            ctr.recurse_render_tree(depth + 1);
        }
    }

    fn avm1_unload(&self, context: &mut UpdateContext<'gc>) {
        // Unload children.
        if let Some(ctr) = self.as_container() {
            for child in ctr.iter_render_list() {
                child.avm1_unload(context);
            }
        }

        if let Some(node) = self.maskee() {
            node.set_masker(context.gc_context, None, true);
        } else if let Some(node) = self.masker() {
            node.set_maskee(context.gc_context, None, true);
        }

        // Unregister any text field variable bindings, and replace them on the unbound list.
        if let Avm1Value::Object(object) = self.object() {
            if let Some(stage_object) = object.as_stage_object() {
                stage_object.unregister_text_field_bindings(context);
            }
        }

        context
            .audio_manager
            .stop_sounds_with_display_object(context.audio, (*self).into());

        self.set_avm1_removed(context.gc_context, true);
    }

    fn as_stage(&self) -> Option<Stage<'gc>> {
        None
    }
    fn as_avm1_button(&self) -> Option<Avm1Button<'gc>> {
        None
    }
    fn as_avm2_button(&self) -> Option<Avm2Button<'gc>> {
        None
    }
    fn as_movie_clip(&self) -> Option<MovieClip<'gc>> {
        None
    }
    fn as_edit_text(&self) -> Option<EditText<'gc>> {
        None
    }
    fn as_morph_shape(&self) -> Option<MorphShape<'gc>> {
        None
    }
    fn as_container(self) -> Option<DisplayObjectContainer<'gc>> {
        None
    }
    fn as_video(self) -> Option<Video<'gc>> {
        None
    }
    fn as_drawing(&self, _gc_context: &Mutation<'gc>) -> Option<RefMut<'_, Drawing>> {
        None
    }
    fn as_bitmap(self) -> Option<Bitmap<'gc>> {
        None
    }
    fn as_interactive(self) -> Option<InteractiveObject<'gc>> {
        None
    }

    fn apply_place_object(
        &self,
        context: &mut UpdateContext<'gc>,
        place_object: &swf::PlaceObject,
    ) {
        // PlaceObject tags only apply if this object has not been dynamically moved by AS code.
        if !self.transformed_by_script() {
            if let Some(matrix) = place_object.matrix {
                self.set_matrix(context.gc_context, matrix.into());
                if let Some(parent) = self.parent() {
                    // Self-transform changes are automatically handled,
                    // we only want to inform ancestors to avoid unnecessary invalidations for tx/ty
                    parent.invalidate_cached_bitmap(context.gc_context);
                }
            }
            if let Some(color_transform) = &place_object.color_transform {
                self.set_color_transform(context.gc_context, *color_transform);
                if let Some(parent) = self.parent() {
                    parent.invalidate_cached_bitmap(context.gc_context);
                }
            }
            if let Some(ratio) = place_object.ratio {
                if let Some(mut morph_shape) = self.as_morph_shape() {
                    morph_shape.set_ratio(context.gc_context, ratio);
                } else if let Some(video) = self.as_video() {
                    video.seek(context, ratio.into());
                }
            }
            if let Some(is_bitmap_cached) = place_object.is_bitmap_cached {
                self.set_bitmap_cached_preference(context.gc_context, is_bitmap_cached);
            }
            if let Some(blend_mode) = place_object.blend_mode {
                self.set_blend_mode(context.gc_context, blend_mode.into());
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
                    self.set_opaque_background(context.gc_context, color);
                }
            }
            if let Some(filters) = &place_object.filters {
                self.set_filters(
                    context.gc_context,
                    filters.iter().map(Filter::from).collect(),
                );
            }
            // Purposely omitted properties:
            // name, clip_depth, clip_actions
            // These properties are only set on initial placement in `MovieClip::instantiate_child`
            // and can not be modified by subsequent PlaceObject tags.
        }
    }

    /// Called when this object should be replaced by a PlaceObject tag.
    fn replace_with(&self, _context: &mut UpdateContext<'gc>, _id: CharacterId) {
        // Noop for most symbols; only shapes can replace their innards with another Graphic.
    }

    fn object(&self) -> Avm1Value<'gc> {
        Avm1Value::Undefined // TODO: Implement for every type and delete this fallback.
    }

    fn object2(&self) -> Avm2Value<'gc> {
        Avm2Value::Undefined // TODO: See above. Also, unconstructed objects should return null.
    }

    fn set_object2(&self, _context: &mut UpdateContext<'gc>, _to: Avm2Object<'gc>) {}

    /// Tests if a given stage position point intersects with the world bounds of this object.
    fn hit_test_bounds(&self, point: Point<Twips>) -> bool {
        self.world_bounds().contains(point)
    }

    /// Tests if a given object's world bounds intersects with the world bounds
    /// of this object.
    fn hit_test_object(&self, other: DisplayObject<'gc>) -> bool {
        self.world_bounds().intersects(&other.world_bounds())
    }

    /// Tests if a given stage position point intersects within this object, considering the art.
    fn hit_test_shape(
        &self,
        _context: &mut UpdateContext<'gc>,
        point: Point<Twips>,
        options: HitTestOptions,
    ) -> bool {
        // Default to using bounding box.
        (!options.contains(HitTestOptions::SKIP_INVISIBLE) || self.visible())
            && self.hit_test_bounds(point)
    }

    fn post_instantiation(
        &self,
        context: &mut UpdateContext<'gc>,
        _init_object: Option<Avm1Object<'gc>>,
        _instantiated_by: Instantiator,
        run_frame: bool,
    ) {
        if run_frame && !self.movie().is_action_script_3() {
            self.run_frame_avm1(context);
        }
    }

    /// Return the version of the SWF that created this movie clip.
    fn swf_version(&self) -> u8 {
        self.movie().version()
    }

    /// Return the SWF that defines this display object.
    fn movie(&self) -> Arc<SwfMovie>;

    fn loader_info(&self) -> Option<Avm2Object<'gc>> {
        None
    }

    fn instantiate(&self, gc_context: &Mutation<'gc>) -> DisplayObject<'gc>;
    fn as_ptr(&self) -> *const DisplayObjectPtr;

    /// Whether this object can be used as a mask.
    /// If this returns false and this object is used as a mask, the mask will not be applied.
    /// This is used by movie clips to disable the mask when there are no children, for example.
    fn allow_as_mask(&self) -> bool {
        true
    }

    /// Obtain the top-most non-Stage parent of the display tree hierarchy.
    ///
    /// This function implements the AVM1 concept of root clips. For the AVM2
    /// version, see `avm2_root`.
    fn avm1_root(&self) -> DisplayObject<'gc> {
        let mut root = (*self).into();
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
    fn avm1_root_no_lock(&self) -> DisplayObject<'gc> {
        let mut root = (*self).into();
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
    fn avm1_stage(&self) -> DisplayObject<'gc> {
        let mut root = (*self).into();
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
    fn avm2_root(&self) -> Option<DisplayObject<'gc>> {
        let mut parent = Some((*self).into());
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
    fn avm2_stage(&self, _context: &UpdateContext<'gc>) -> Option<DisplayObject<'gc>> {
        let mut parent = Some((*self).into());
        while let Some(p) = parent {
            if p.as_stage().is_some() {
                return parent;
            }
            parent = p.parent();
        }
        None
    }

    /// Determine if this display object is currently on the stage.
    fn is_on_stage(self, context: &UpdateContext<'gc>) -> bool {
        let mut ancestor = self.avm2_parent();
        while let Some(parent) = ancestor {
            if parent.avm2_parent().is_some() {
                ancestor = parent.avm2_parent();
            } else {
                break;
            }
        }

        let ancestor = ancestor.unwrap_or_else(|| self.into());
        DisplayObject::ptr_eq(ancestor, context.stage.into())
    }

    /// Assigns a default instance name `instanceN` to this object.
    fn set_default_instance_name(&self, context: &mut UpdateContext<'gc>) {
        if self.base().name().is_none() {
            let name = format!("instance{}", *context.instance_counter);
            self.set_name(
                context.gc_context,
                AvmString::new_utf8(context.gc_context, name),
            );
            *context.instance_counter = context.instance_counter.wrapping_add(1);
        }
    }

    /// Assigns a default root name to this object.
    ///
    /// The default root names change based on the AVM configuration of the
    /// clip; AVM2 clips get `rootN` while AVM1 clips get blank strings.
    fn set_default_root_name(&self, context: &mut UpdateContext<'gc>) {
        if self.movie().is_action_script_3() {
            let name = AvmString::new_utf8(context.gc_context, format!("root{}", self.depth() + 1));
            self.set_name(context.gc_context, name);
        } else {
            self.set_name(context.gc_context, Default::default());
        }
    }

    fn bind_text_field_variables(&self, activation: &mut Activation<'_, 'gc>) {
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

    /// Inform this object and its ancestors that it has visually changed and must be redrawn.
    /// If this object or any ancestor is marked as cacheAsBitmap, it will invalidate that cache.
    fn invalidate_cached_bitmap(&self, mc: &Mutation<'gc>) {
        if self.base_mut(mc).invalidate_cached_bitmap() {
            // Don't inform ancestors if we've already done so this frame
            if let Some(parent) = self.parent() {
                parent.invalidate_cached_bitmap(mc);
            }
        }
    }

    /// Retrieve a named property from the AVM1 object.
    ///
    /// This is required as some boolean properties in AVM1 can in fact hold any value.
    fn get_avm1_boolean_property<F>(
        self,
        context: &mut UpdateContext<'gc>,
        name: &'static str,
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

    fn set_avm1_property(
        self,
        context: &mut UpdateContext<'gc>,
        name: &'static str,
        value: Avm1Value<'gc>,
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
}

pub enum DisplayObjectPtr {}

impl<'gc> DisplayObject<'gc> {
    pub fn ptr_eq(a: DisplayObject<'gc>, b: DisplayObject<'gc>) -> bool {
        a.as_ptr() == b.as_ptr()
    }

    pub fn option_ptr_eq(a: Option<DisplayObject<'gc>>, b: Option<DisplayObject<'gc>>) -> bool {
        a.map(|o| o.as_ptr()) == b.map(|o| o.as_ptr())
    }

    pub fn downgrade(self) -> DisplayObjectWeak<'gc> {
        match self {
            DisplayObject::MovieClip(mc) => DisplayObjectWeak::MovieClip(mc.downgrade()),
            DisplayObject::LoaderDisplay(l) => DisplayObjectWeak::LoaderDisplay(l.downgrade()),
            DisplayObject::Bitmap(b) => DisplayObjectWeak::Bitmap(b.downgrade()),
            _ => panic!("Downgrade not yet implemented for {:?}", self),
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

        /// The options used for `hitTest` calls in ActionScript.
        const AVM_HIT_TEST = Self::SKIP_MASK.bits();

        /// The options used for mouse picking, such as clicking on buttons.
        const MOUSE_PICK = Self::SKIP_MASK.bits() | Self::SKIP_INVISIBLE.bits();
    }
}

/// Represents the sound transform of sounds played inside a Flash MovieClip.
/// Every value is a percentage (0-100), but out of range values are allowed.
/// In AVM1, this is returned by `Sound.getTransform`.
/// In AVM2, this is returned by `Sprite.soundTransform`.
#[derive(Debug, PartialEq, Eq, Clone)]
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
    pub fn concat(&mut self, other: &SoundTransform) {
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

    /// Sets this transform of this pan.
    /// -100 is full left and 100 is full right.
    /// This matches the behavior of AVM1 `Sound.setPan()`.
    pub fn set_pan(&mut self, pan: i32) {
        if pan >= 0 {
            self.left_to_left = Self::MAX_VOLUME - pan;
            self.right_to_right = Self::MAX_VOLUME;
        } else {
            self.left_to_left = Self::MAX_VOLUME;
            self.right_to_right = Self::MAX_VOLUME + pan;
        }
        self.left_to_right = 0;
        self.right_to_left = 0;
    }

    pub fn from_avm2_object<'gc>(
        activation: &mut Avm2Activation<'_, 'gc>,
        as3_st: Avm2Object<'gc>,
    ) -> Result<Self, Avm2Error<'gc>> {
        Ok(SoundTransform {
            left_to_left: (as3_st
                .get_public_property("leftToLeft", activation)?
                .coerce_to_number(activation)?
                * 100.0) as i32,
            left_to_right: (as3_st
                .get_public_property("leftToRight", activation)?
                .coerce_to_number(activation)?
                * 100.0) as i32,
            right_to_left: (as3_st
                .get_public_property("rightToLeft", activation)?
                .coerce_to_number(activation)?
                * 100.0) as i32,
            right_to_right: (as3_st
                .get_public_property("rightToRight", activation)?
                .coerce_to_number(activation)?
                * 100.0) as i32,
            volume: (as3_st
                .get_public_property("volume", activation)?
                .coerce_to_number(activation)?
                * 100.0) as i32,
        })
    }

    pub fn into_avm2_object<'gc>(
        self,
        activation: &mut Avm2Activation<'_, 'gc>,
    ) -> Result<Avm2Object<'gc>, Avm2Error<'gc>> {
        let as3_st = activation
            .avm2()
            .classes()
            .soundtransform
            .construct(activation, &[])?;

        as3_st.set_public_property(
            "leftToLeft",
            (self.left_to_left as f64 / 100.0).into(),
            activation,
        )?;
        as3_st.set_public_property(
            "leftToRight",
            (self.left_to_right as f64 / 100.0).into(),
            activation,
        )?;
        as3_st.set_public_property(
            "rightToLeft",
            (self.right_to_left as f64 / 100.0).into(),
            activation,
        )?;
        as3_st.set_public_property(
            "rightToRight",
            (self.right_to_right as f64 / 100.0).into(),
            activation,
        )?;
        as3_st.set_public_property("volume", (self.volume as f64 / 100.0).into(), activation)?;

        Ok(as3_st)
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
