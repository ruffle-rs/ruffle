//! Root stage impl

use crate::avm1::Object as Avm1Object;
use crate::avm2::object::Stage3DObject;
use crate::avm2::{
    Activation as Avm2Activation, Avm2, EventObject as Avm2EventObject, LoaderInfoObject,
    Object as Avm2Object, StageObject as Avm2StageObject,
};
use crate::backend::ui::MouseCursor;
use crate::config::Letterbox;
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::container::ChildContainer;
use crate::display_object::interactive::{InteractiveObjectBase, TInteractiveObject};
use crate::display_object::{render_base, DisplayObjectBase, RenderOptions};
use crate::events::{ClipEvent, ClipEventResult};
use crate::focus_tracker::FocusTracker;
use crate::prelude::*;
use crate::string::{FromWStr, WStr};
use crate::tag_utils::SwfMovie;
use crate::utils::HasPrefixField;
use crate::vminterface::Instantiator;
use bitflags::bitflags;
use gc_arena::barrier::unlock;
use gc_arena::{Collect, Gc, Lock, Mutation, RefLock};
use ruffle_macros::istr;
use ruffle_render::backend::ViewportDimensions;
use ruffle_render::commands::CommandHandler;
use ruffle_render::perspective_projection::PerspectiveProjection;
use ruffle_render::quality::StageQuality;
use ruffle_render::transform::Transform;
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use std::sync::Arc;

/// The Stage is the root of the display object hierarchy. It contains all AVM1
/// levels as well as AVM2 movies.
#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct Stage<'gc>(Gc<'gc, StageData<'gc>>);

impl fmt::Debug for Stage<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stage")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Clone, Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct StageData<'gc> {
    /// Base properties for interactive display objects.
    ///
    /// This particular base has additional constraints currently not
    /// expressible by the type system. Notably, this should never have a
    /// parent, as the stage does not respect it.
    base: InteractiveObjectBase<'gc>,

    /// The list of all children of the stage.
    ///
    /// Stage children are exposed to AVM1 as `_level*n*` on all stage objects.
    child: RefLock<ChildContainer<'gc>>,

    /// The AVM2 view of this stage object.
    avm2_object: Lock<Option<Avm2StageObject<'gc>>>,

    /// The AVM2 'LoaderInfo' object for this stage object
    loader_info: Lock<Option<LoaderInfoObject<'gc>>>,

    /// An array of AVM2 'Stage3D' instances
    stage3ds: RefLock<Vec<Avm2Object<'gc>>>,

    /// A tracker for the current keyboard focused element
    focus_tracker: FocusTracker<'gc>,

    /// The swf that registered this stage
    movie: RefCell<Arc<SwfMovie>>,

    /// The dimensions of the SWF file.
    movie_size: Cell<(u32, u32)>,

    /// The final viewport transformation matrix applied
    /// when rendering the stage. This includes the HiDPI scale factor,
    /// and stage alignment translation. Neither of those are included
    /// in the ActionScript-exposed `Stage.matrix` (which is always the
    /// identity matrix unless explicitly set from ActionScript)
    viewport_matrix: Cell<Matrix>,

    /// Matrix used for rendering the letterbox.
    ///
    /// It represents the transformation of the whole window into the
    /// letterboxed area.  Note that it's different from the viewport matrix, as
    /// it doesn't include any additional transformations of the content.
    letterbox_matrix: Cell<Matrix>,

    /// The bounds of the current viewport in twips, used for culling.
    view_bounds: Cell<Rectangle<Twips>>,

    /// The stage background.
    ///
    /// If the background color is not specified, it should be white.
    background_color: Cell<Option<Color>>,

    /// Determines how player content is resized to fit the stage.
    letterbox: Cell<Letterbox>,

    /// The quality settings of the stage.
    quality: Cell<StageQuality>,

    /// The dimensions of the stage, as reported to ActionScript.
    stage_size: Cell<(u32, u32)>,

    /// The scale mode of the stage.
    scale_mode: Cell<StageScaleMode>,

    /// Whether to prevent movies from changing the stage scale mode.
    forced_scale_mode: Cell<bool>,

    /// The display state of the stage.
    display_state: Cell<StageDisplayState>,

    /// The alignment of the stage.
    align: Cell<StageAlign>,

    /// Whether to prevent movies from the changing the stage alignment
    forced_align: Cell<bool>,

    /// Whether to allow the stage's displayState to be changed.
    allow_fullscreen: Cell<bool>,

    /// Whether or not a RENDER event should be dispatched on the next render
    invalidated: Cell<bool>,

    /// Whether to use high quality downsampling for bitmaps.
    ///
    /// This is usually implied by `quality` being `Best` or higher, but the AVM1
    /// `ToggleHighQuality` op can adjust stage quality independently of this flag.
    /// This setting is currently ignored in Ruffle.
    use_bitmap_downsampling: Cell<bool>,

    /// The window mode of the viewport.
    ///
    /// Only used on web to control how the Flash content layers with other content on the page.
    window_mode: Cell<WindowMode>,

    /// Whether objects display a glowing border when they have focus.
    stage_focus_rect: Cell<bool>,

    /// Whether to show default context menu items
    show_menu: Cell<bool>,
}

impl<'gc> Stage<'gc> {
    pub fn empty(gc_context: &Mutation<'gc>, fullscreen: bool, movie: Arc<SwfMovie>) -> Stage<'gc> {
        let stage = Self(Gc::new(
            gc_context,
            StageData {
                base: Default::default(),
                child: RefLock::new(ChildContainer::new(&movie)),
                background_color: Cell::new(None),
                letterbox: Cell::new(Letterbox::Fullscreen),
                // This is updated when we set the root movie
                movie_size: Cell::new((0, 0)),
                quality: Default::default(),
                // This is updated in `build_matrices`
                stage_size: Cell::new((0, 0)),
                scale_mode: Default::default(),
                forced_scale_mode: Cell::new(false),
                display_state: Cell::new(if fullscreen {
                    StageDisplayState::FullScreen
                } else {
                    StageDisplayState::Normal
                }),
                invalidated: Cell::new(false),
                align: Default::default(),
                forced_align: Cell::new(false),
                allow_fullscreen: Cell::new(true),
                use_bitmap_downsampling: Cell::new(false),
                view_bounds: Default::default(),
                window_mode: Default::default(),
                show_menu: Cell::new(true),
                stage_focus_rect: Cell::new(true),
                avm2_object: Lock::new(None),
                loader_info: Lock::new(None),
                stage3ds: RefLock::new(vec![]),
                movie: RefCell::new(movie),
                viewport_matrix: Cell::new(Matrix::IDENTITY),
                letterbox_matrix: Cell::new(Matrix::IDENTITY),
                focus_tracker: FocusTracker::new(gc_context),
            },
        ));
        stage.set_is_root(true);
        stage.set_perspective_projection(None); // Set default PerspectiveProjection
        stage
    }

    pub fn background_color(self) -> Option<Color> {
        self.0.background_color.get()
    }

    pub fn set_background_color(self, color: Option<Color>) {
        self.0.background_color.set(color);
    }

    pub fn inverse_view_matrix(self) -> Matrix {
        self.0
            .viewport_matrix
            .get()
            .inverse()
            .unwrap_or(Matrix::ZERO)
    }

    pub fn view_matrix(self) -> Matrix {
        self.0.viewport_matrix.get()
    }

    pub fn letterbox(self) -> Letterbox {
        self.0.letterbox.get()
    }

    pub fn set_letterbox(self, letterbox: Letterbox) {
        self.0.letterbox.set(letterbox)
    }

    /// Get the size of the SWF file.
    pub fn movie_size(self) -> (u32, u32) {
        self.0.movie_size.get()
    }

    /// Set the size of the SWF file.
    pub fn set_movie_size(self, width: u32, height: u32) {
        self.0.movie_size.set((width, height));
    }

    pub fn set_movie(self, gc_context: &Mutation<'gc>, movie: Arc<SwfMovie>) {
        // Stage is the only DO that has a fake movie set and then gets the real movie set.
        // NOTE: Make sure to NOT reset any state here, AVM1 depends on it.

        let is_action_script_3 = movie.is_action_script_3();
        self.0.movie.replace(movie);
        unlock!(Gc::write(gc_context, self.0), StageData, child)
            .borrow_mut()
            .set_is_action_script_3(is_action_script_3);
    }

    pub fn set_loader_info(self, gc_context: &Mutation<'gc>, loader_info: LoaderInfoObject<'gc>) {
        unlock!(Gc::write(gc_context, self.0), StageData, loader_info).set(Some(loader_info));
    }

    // Get the invalidation state
    pub fn invalidated(self) -> bool {
        self.0.invalidated.get()
    }

    // Set the invalidation state
    pub fn set_invalidated(self, value: bool) {
        self.0.invalidated.set(value);
    }

    /// Returns the quality setting of the stage.
    ///
    /// In the Flash Player, the quality setting affects anti-aliasing and smoothing of bitmaps.
    /// This setting is currently ignored in Ruffle.
    /// Used by AVM1 `stage.quality` and AVM2 `Stage.quality` properties.
    pub fn quality(self) -> StageQuality {
        self.0.quality.get()
    }

    /// Sets the quality setting of the stage.
    ///
    /// In the Flash Player, the quality setting affects anti-aliasing and smoothing of bitmaps.
    /// This setting is currently ignored in Ruffle.
    /// Used by AVM1 `stage.quality` and AVM2 `Stage.quality` properties.
    pub fn set_quality(self, context: &mut UpdateContext<'gc>, quality: StageQuality) {
        self.0.quality.set(quality);
        self.0.use_bitmap_downsampling.set(matches!(
            quality,
            StageQuality::Best
                | StageQuality::High8x8
                | StageQuality::High8x8Linear
                | StageQuality::High16x16
                | StageQuality::High16x16Linear
        ));
        context.renderer.set_quality(quality);
    }

    pub fn stage3ds(&self) -> Ref<'_, Vec<Avm2Object<'gc>>> {
        self.0.stage3ds.borrow()
    }

    /// Get the boolean flag which determines whether objects display a glowing border
    /// when they have focus.
    pub fn stage_focus_rect(self) -> bool {
        self.0.stage_focus_rect.get()
    }

    /// Set the boolean flag which determines whether objects display a glowing border
    /// when they have focus.
    pub fn set_stage_focus_rect(self, value: bool) {
        self.0.stage_focus_rect.set(value);
    }

    /// Get the size of the stage.
    /// Used by AVM1 `stage.width`/`height` and AVM2 `Stage.stageWidth`/`stageHeight` properties.
    /// If `scale_mode` is `StageScaleMode::NO_SCALE`, this returns the size of the viewport.
    /// Otherwise, this returns the size of the SWF file.
    pub fn stage_size(self) -> (u32, u32) {
        self.0.stage_size.get()
    }

    /// Get the stage mode.
    /// This controls how the content scales to fill the viewport.
    pub fn scale_mode(self) -> StageScaleMode {
        self.0.scale_mode.get()
    }

    /// Set the stage scale mode.
    pub fn set_scale_mode(
        self,
        context: &mut UpdateContext<'gc>,
        scale_mode: StageScaleMode,
        respect_forced: bool,
    ) {
        if respect_forced && self.forced_scale_mode() {
            return;
        }

        self.0.scale_mode.set(scale_mode);
        self.build_matrices(context);
    }

    /// Get whether movies are prevented from changing the stage scale mode.
    pub fn forced_scale_mode(self) -> bool {
        self.0.forced_scale_mode.get()
    }

    /// Set whether movies are prevented from changing the stage scale mode.
    pub fn set_forced_scale_mode(self, force: bool) {
        self.0.forced_scale_mode.set(force);
    }

    /// Get whether the Stage's display state can be changed.
    pub fn allow_fullscreen(self) -> bool {
        self.0.allow_fullscreen.get()
    }

    /// Set whether the Stage's display state can be changed.
    pub fn set_allow_fullscreen(self, allow: bool) {
        self.0.allow_fullscreen.set(allow);
    }

    fn is_fullscreen_state(display_state: StageDisplayState) -> bool {
        display_state == StageDisplayState::FullScreen
            || display_state == StageDisplayState::FullScreenInteractive
    }

    /// Gets whether the stage is in fullscreen
    pub fn is_fullscreen(self) -> bool {
        let display_state = self.display_state();
        Self::is_fullscreen_state(display_state)
    }

    /// Get the stage display state.
    /// This controls the fullscreen state.
    pub fn display_state(self) -> StageDisplayState {
        self.0.display_state.get()
    }

    /// Toggles display state between fullscreen and normal
    pub fn toggle_display_state(self, context: &mut UpdateContext<'gc>) {
        if self.is_fullscreen() {
            self.set_display_state(context, StageDisplayState::Normal);
        } else {
            self.set_display_state(context, StageDisplayState::FullScreen);
        }
    }

    /// Set the stage display state.
    pub fn set_display_state(
        self,
        context: &mut UpdateContext<'gc>,
        display_state: StageDisplayState,
    ) {
        if display_state == self.display_state()
            || (Self::is_fullscreen_state(display_state) && self.is_fullscreen())
            || !self.allow_fullscreen()
        {
            return;
        }

        let result = if display_state == StageDisplayState::FullScreen
            || display_state == StageDisplayState::FullScreenInteractive
        {
            context.ui.set_fullscreen(true)
        } else {
            context.ui.set_fullscreen(false)
        };

        if result.is_ok() {
            self.0.display_state.set(display_state);
            self.fire_fullscreen_event(context);
        }
    }

    /// Get the stage alignment.
    pub fn align(self) -> StageAlign {
        self.0.align.get()
    }

    /// Set the stage alignment.
    /// This only has an effect if the scale mode is not `StageScaleMode::ExactFit`.
    pub fn set_align(self, context: &mut UpdateContext<'gc>, align: StageAlign) {
        if !self.forced_align() {
            self.0.align.set(align);
            self.build_matrices(context);
        }
    }

    /// Get whether movies are prevented from changing the stage alignment.
    pub fn forced_align(self) -> bool {
        self.0.forced_align.get()
    }

    /// Set whether movies are prevented from changing the stage alignment.
    pub fn set_forced_align(self, force: bool) {
        self.0.forced_align.set(force);
    }

    /// Returns whether bitmaps will use high quality downsampling when scaled down.
    /// This setting is currently ignored in Ruffle.
    pub fn use_bitmap_downsampling(self) -> bool {
        self.0.use_bitmap_downsampling.get()
    }

    /// Sets whether bitmaps will use high quality downsampling when scaled down.
    /// This setting is currently ignored in Ruffle.
    pub fn set_use_bitmap_downsampling(self, value: bool) {
        self.0.use_bitmap_downsampling.set(value);
    }

    /// Get the stage mode.
    /// This controls how the content layers with other content on the page.
    /// Only used on web.
    pub fn window_mode(self) -> WindowMode {
        self.0.window_mode.get()
    }

    /// Sets the window mode.
    pub fn set_window_mode(self, window_mode: WindowMode) {
        self.0.window_mode.set(window_mode);
    }

    pub fn view_bounds(self) -> Rectangle<Twips> {
        self.0.view_bounds.get()
    }

    pub fn show_menu(self) -> bool {
        self.0.show_menu.get()
    }

    pub fn set_show_menu(self, show_menu: bool) {
        self.0.show_menu.set(show_menu);
    }

    /// Determine if we should letterbox the stage content.
    fn should_letterbox(self) -> bool {
        // Only enable letterbox in the default `ShowAll` scale mode.
        // If content changes the scale mode or alignment, it signals that it is size-aware.
        // For example, `NoScale` is used to make responsive layouts; don't letterbox over it.
        let letterbox = self.0.letterbox.get();
        self.0.scale_mode.get() == StageScaleMode::ShowAll
            && self.0.align.get().is_empty()
            && self.0.window_mode.get() != WindowMode::Transparent
            && (letterbox == Letterbox::On
                || (letterbox == Letterbox::Fullscreen && self.is_fullscreen()))
    }

    /// Update the stage's transform matrix in response to a root movie change.
    pub fn build_matrices(self, context: &mut UpdateContext<'gc>) {
        let scale_mode = self.0.scale_mode.get();
        let align = self.0.align.get();
        let prev_stage_size = self.0.stage_size.get();
        let viewport_size = context.renderer.viewport_dimensions();

        // Update stage size based on scale mode and DPI.
        let new_stage_size = if self.0.scale_mode.get() == StageScaleMode::NoScale {
            // Viewport size is adjusted for HiDPI.
            let width = f64::from(viewport_size.width) / viewport_size.scale_factor;
            let height = f64::from(viewport_size.height) / viewport_size.scale_factor;
            (width.round() as u32, height.round() as u32)
        } else {
            self.0.movie_size.get()
        };
        self.0.stage_size.set(new_stage_size);
        let stage_size_changed = prev_stage_size != new_stage_size;

        // Create view matrix to scale stage into viewport area.
        let (movie_width, movie_height) = self.0.movie_size.get();
        let movie_width = movie_width as f64;
        let movie_height = movie_height as f64;

        let viewport_width = viewport_size.width as f64;
        let viewport_height = viewport_size.height as f64;

        let movie_aspect = movie_width / movie_height;
        let viewport_aspect = viewport_width / viewport_height;

        let (scale_x, scale_y) = match scale_mode {
            StageScaleMode::ShowAll => {
                // Keep aspect ratio, padding the edges.
                let scale = if viewport_aspect > movie_aspect {
                    viewport_height / movie_height
                } else {
                    viewport_width / movie_width
                };
                (scale, scale)
            }
            StageScaleMode::NoBorder => {
                // Keep aspect ratio, cropping off the edges.
                let scale = if viewport_aspect < movie_aspect {
                    viewport_height / movie_height
                } else {
                    viewport_width / movie_width
                };
                (scale, scale)
            }
            StageScaleMode::ExactFit => {
                // Stretch to fill container.
                (viewport_width / movie_width, viewport_height / movie_height)
            }
            StageScaleMode::NoScale => {
                // No adjustment.
                (viewport_size.scale_factor, viewport_size.scale_factor)
            }
        };

        let width_delta = viewport_width - movie_width * scale_x;
        let height_delta = viewport_height - movie_height * scale_y;
        // The precedence is important here to match Flash behavior.
        // L > R > "", T > B > "".
        let tx = if align.contains(StageAlign::LEFT) {
            0.0
        } else if align.contains(StageAlign::RIGHT) {
            width_delta
        } else {
            width_delta / 2.0
        };
        let ty = if align.contains(StageAlign::TOP) {
            0.0
        } else if align.contains(StageAlign::BOTTOM) {
            height_delta
        } else {
            height_delta / 2.0
        };

        // The viewport can be additionally translated from within the SWF header.
        let stage_tx = {
            let movie = self.movie();
            let stage_size = movie.stage_size();
            Matrix::translate(stage_size.x_min, stage_size.y_min)
        };

        let letterbox_matrix = Matrix {
            a: scale_x as f32,
            b: 0.0,
            c: 0.0,
            d: scale_y as f32,
            tx: Twips::from_pixels(tx),
            ty: Twips::from_pixels(ty),
        };
        self.0.letterbox_matrix.set(letterbox_matrix);
        self.0
            .viewport_matrix
            .set(letterbox_matrix * stage_tx.inverse().unwrap());

        let view_bounds = if self.should_letterbox() {
            // Letterbox: movie area
            Rectangle {
                x_min: Twips::ZERO,
                y_min: Twips::ZERO,
                x_max: Twips::from_pixels(movie_width),
                y_max: Twips::from_pixels(movie_height),
            }
        } else {
            // No letterbox: full visible stage area
            let margin_left = tx / scale_x;
            let margin_right = (width_delta - tx) / scale_x;
            let margin_top = ty / scale_y;
            let margin_bottom = (height_delta - ty) / scale_y;
            Rectangle {
                x_min: Twips::from_pixels(-margin_left),
                y_min: Twips::from_pixels(-margin_top),
                x_max: Twips::from_pixels(movie_width + margin_right),
                y_max: Twips::from_pixels(movie_height + margin_bottom),
            }
        };

        self.0.view_bounds.set(stage_tx * view_bounds);

        // Fire resize handler if stage size has changed.
        if scale_mode == StageScaleMode::NoScale && stage_size_changed {
            self.fire_resize_event(context);
        }
    }

    /// Draw the stage's letterbox.
    fn draw_letterbox(self, context: &mut RenderContext<'_, 'gc>) {
        let ViewportDimensions {
            width: viewport_width,
            height: viewport_height,
            scale_factor: _,
        } = context.renderer.viewport_dimensions();
        let viewport_width = viewport_width as f32;
        let viewport_height = viewport_height as f32;

        let letterbox_matrix = self.0.letterbox_matrix.get();

        let (movie_width, movie_height) = self.0.movie_size.get();
        let movie_width = movie_width as f32 * letterbox_matrix.a;
        let movie_height = movie_height as f32 * letterbox_matrix.d;

        let margin_left = letterbox_matrix.tx.to_pixels() as f32;
        let margin_right = viewport_width - movie_width - margin_left;
        let margin_top = letterbox_matrix.ty.to_pixels() as f32;
        let margin_bottom = viewport_height - movie_height - margin_top;

        // Letterboxing only occurs in `StageScaleMode::ShowAll`, and they would only appear on the top+bottom or left+right.
        if margin_top + margin_bottom > margin_left + margin_right {
            // Top + bottom
            if margin_top > 0.0 {
                context.commands.draw_rect(
                    Color::BLACK,
                    Matrix::create_box(
                        viewport_width,
                        margin_top,
                        Twips::default(),
                        Twips::default(),
                    ),
                );
            }
            if margin_bottom > 0.0 {
                context.commands.draw_rect(
                    Color::BLACK,
                    Matrix::create_box(
                        viewport_width,
                        margin_bottom,
                        Twips::default(),
                        Twips::from_pixels((viewport_height - margin_bottom) as f64),
                    ),
                );
            }
        } else {
            // Left + right
            if margin_left > 0.0 {
                context.commands.draw_rect(
                    Color::BLACK,
                    Matrix::create_box(
                        margin_left,
                        viewport_height,
                        Twips::default(),
                        Twips::default(),
                    ),
                );
            }
            if margin_right > 0.0 {
                context.commands.draw_rect(
                    Color::BLACK,
                    Matrix::create_box(
                        margin_right,
                        viewport_height,
                        Twips::from_pixels((viewport_width - margin_right) as f64),
                        Twips::default(),
                    ),
                );
            }
        }
    }

    /// Obtain the root movie on the stage.
    ///
    /// It is not a guarantee that the root clip exists, as it can be deliberately removed.
    pub fn root_clip(self) -> Option<DisplayObject<'gc>> {
        self.child_by_depth(0)
    }

    /// Fires `Stage.onResize` in AVM1 or `Event.RESIZE` in AVM2.
    fn fire_resize_event(self, context: &mut UpdateContext<'gc>) {
        // This event fires immediately when scaleMode is changed;
        // it doesn't queue up.
        if !self.movie().is_action_script_3() {
            if let Some(root_clip) = self.root_clip() {
                crate::avm1::Avm1::notify_system_listeners(
                    root_clip,
                    istr!(context, "Stage"),
                    istr!(context, "onResize"),
                    &[],
                    context,
                );
            }
        } else if let Some(stage) = self.object2() {
            let resized_event = Avm2EventObject::bare_default_event(context, "resize");
            Avm2::dispatch_event(context, resized_event, stage.into());
        }
    }

    /// Broadcast the 'render' event
    ///
    /// TODO: Need additional check as Flash Player does not
    /// broadcast the 'render' event on the first render
    pub fn broadcast_render(self, context: &mut UpdateContext<'gc>) {
        let render_evt = Avm2EventObject::bare_default_event(context, "render");
        let dobject_constr = context.avm2.classes().display_object;
        Avm2::broadcast_event(context, render_evt, dobject_constr);

        self.set_invalidated(false);
    }

    /// Fires `Stage.onFullScreen` in AVM1 or `Event.FULLSCREEN` in AVM2.
    pub fn fire_fullscreen_event(self, context: &mut UpdateContext<'gc>) {
        if !self.movie().is_action_script_3() {
            if let Some(root_clip) = self.root_clip() {
                crate::avm1::Avm1::notify_system_listeners(
                    root_clip,
                    istr!(context, "Stage"),
                    istr!(context, "onFullScreen"),
                    &[self.is_fullscreen().into()],
                    context,
                );
            }
        } else if let Some(stage) = self.object2() {
            let mut activation = Avm2Activation::from_nothing(context);

            let full_screen_event_cls = activation.avm2().classes().fullscreenevent;
            let full_screen_string = istr!("fullScreen");
            let full_screen_event = Avm2EventObject::from_class_and_args(
                &mut activation,
                full_screen_event_cls,
                &[
                    full_screen_string.into(),
                    false.into(),
                    false.into(),
                    self.is_fullscreen().into(),
                    true.into(),
                ],
            );

            Avm2::dispatch_event(context, full_screen_event, stage.into());
        }
    }

    pub fn focus_tracker(self) -> FocusTracker<'gc> {
        self.0.focus_tracker
    }
}

impl<'gc> TDisplayObject<'gc> for Stage<'gc> {
    fn base(self) -> Gc<'gc, DisplayObjectBase<'gc>> {
        HasPrefixField::as_prefix_gc(self.raw_interactive())
    }

    fn instantiate(self, gc_context: &Mutation<'gc>) -> DisplayObject<'gc> {
        Self(Gc::new(gc_context, self.0.as_ref().clone())).into()
    }

    fn local_to_global_matrix(self) -> Matrix {
        // The stage is in Stage coordinates by definition
        Default::default()
    }

    fn post_instantiation(
        self,
        context: &mut UpdateContext<'gc>,
        _init_object: Option<Avm1Object<'gc>>,
        _instantiated_by: Instantiator,
        _run_frame: bool,
    ) {
        let stage_constr = context.avm2.classes().stage;

        // TODO: Replace this when we have a convenience method for constructing AVM2 native objects.
        // TODO: We should only do this if the movie is actually an AVM2 movie.
        // This is necessary for DisplayObject and EventDispatcher super-constructors to run.
        let global_domain = context.avm2.stage_domain();
        let mut activation = Avm2Activation::from_domain(context, global_domain);
        let avm2_stage = Avm2StageObject::for_display_object_childless(
            &mut activation,
            self.into(),
            stage_constr,
        );

        match avm2_stage {
            Ok(avm2_stage) => {
                // Always create 4 Stage3D instances for now, which matches the flash projector behavior
                let stage3ds: Vec<Avm2Object<'gc>> = (0..4)
                    .map(|_| Stage3DObject::new(&mut activation).into())
                    .collect();

                let write = Gc::write(activation.gc(), self.0);
                unlock!(write, StageData, avm2_object).set(Some(avm2_stage));
                unlock!(write, StageData, stage3ds).replace(stage3ds);
            }
            Err(e) => tracing::error!("Unable to construct AVM2 Stage: {}", e),
        }
    }

    fn id(self) -> CharacterId {
        u16::MAX
    }

    fn self_bounds(self) -> Rectangle<Twips> {
        Default::default()
    }

    fn render_self(self, context: &mut RenderContext<'_, 'gc>) {
        self.render_children(context);
    }

    fn render_with_options(self, context: &mut RenderContext<'_, 'gc>, options: RenderOptions) {
        context.transform_stack.push(&Transform {
            matrix: self.0.viewport_matrix.get(),
            color_transform: Default::default(),
            // TODO: Verify perspective_projection when its rendering is implemented.
            perspective_projection: self.as_displayobject().base().perspective_projection(),
            tz: 0.0,
        });

        // All of our Stage3D instances get rendered *underneath* the main stage.
        // Note that the stage background color is actually the lowest possible layer,
        // and get applied when we start the frame (before `render` is called).
        for stage3d in self.stage3ds().iter() {
            let stage3d = stage3d.as_stage_3d().unwrap();
            if stage3d.visible() {
                if let Some(context3d) = stage3d.context3d() {
                    context3d.as_context_3d().unwrap().render(context);
                }
            }
        }

        render_base(self.into(), context, options);

        self.focus_tracker().render_highlight(context);

        if self.should_letterbox() {
            self.draw_letterbox(context);
        }

        context.transform_stack.pop();
    }

    fn enter_frame(self, context: &mut UpdateContext<'gc>) {
        for child in self.iter_render_list() {
            child.enter_frame(context);
        }

        let enter_frame_evt = Avm2EventObject::bare_default_event(context, "enterFrame");
        let dobject_constr = context.avm2.classes().display_object;
        Avm2::broadcast_event(context, enter_frame_evt, dobject_constr);
    }

    fn construct_frame(self, context: &mut UpdateContext<'gc>) {
        for child in self.iter_render_list() {
            child.construct_frame(context);
        }
    }

    fn object2(self) -> Option<Avm2StageObject<'gc>> {
        self.0.avm2_object.get()
    }

    fn set_perspective_projection(self, mut perspective_projection: Option<PerspectiveProjection>) {
        if perspective_projection.is_none() {
            // `stage` doesn't allow null PerspectiveProjection.
            perspective_projection = Some(Default::default());
        }
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

    fn loader_info(self) -> Option<LoaderInfoObject<'gc>> {
        self.0.loader_info.get()
    }

    fn movie(self) -> Arc<SwfMovie> {
        self.0.movie.borrow().clone()
    }
}

impl<'gc> TDisplayObjectContainer<'gc> for Stage<'gc> {
    fn raw_container(&self) -> Ref<'_, ChildContainer<'gc>> {
        self.0.child.borrow()
    }

    fn raw_container_mut(&self, gc_context: &Mutation<'gc>) -> RefMut<'_, ChildContainer<'gc>> {
        unlock!(Gc::write(gc_context, self.0), StageData, child).borrow_mut()
    }
}

impl<'gc> TInteractiveObject<'gc> for Stage<'gc> {
    fn raw_interactive(self) -> Gc<'gc, InteractiveObjectBase<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }

    fn as_displayobject(self) -> DisplayObject<'gc> {
        self.into()
    }

    fn filter_clip_event(
        self,
        _context: &mut UpdateContext<'gc>,
        _event: ClipEvent,
    ) -> ClipEventResult {
        ClipEventResult::Handled
    }

    fn event_dispatch(
        self,
        _context: &mut UpdateContext<'gc>,
        _event: ClipEvent<'gc>,
    ) -> ClipEventResult {
        ClipEventResult::NotHandled
    }

    fn mouse_cursor(self, _context: &mut UpdateContext<'gc>) -> MouseCursor {
        MouseCursor::Arrow
    }

    fn is_highlightable(self, _context: &mut UpdateContext<'gc>) -> bool {
        // Stage cannot be highlighted.
        false
    }
}

pub struct ParseEnumError;

/// The scale mode of a stage.
/// This controls the behavior when the player viewport size differs from the SWF size.
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageScaleMode {
    /// The movie will be stretched to fit the container.
    ExactFit,

    /// The movie will maintain its aspect ratio, but will be cropped.
    NoBorder,

    /// The movie is not scaled to fit the container.
    /// With this scale mode, `Stage.stageWidth` and `stageHeight` will return the dimensions of the container.
    /// SWF content uses this scale mode to resize dynamically and create responsive layouts.
    NoScale,

    /// The movie will scale to fill the container and maintain its aspect ratio, but will be letterboxed.
    /// This is the default scale mode.
    #[default]
    ShowAll,
}

impl Display for StageScaleMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Match string values returned by AS.
        let s = match *self {
            StageScaleMode::ExactFit => "exactFit",
            StageScaleMode::NoBorder => "noBorder",
            StageScaleMode::NoScale => "noScale",
            StageScaleMode::ShowAll => "showAll",
        };
        f.write_str(s)
    }
}

impl FromStr for StageScaleMode {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let scale_mode = match s {
            "exact_fit" => StageScaleMode::ExactFit,
            "no_border" => StageScaleMode::NoBorder,
            "no_scale" => StageScaleMode::NoScale,
            "show_all" => StageScaleMode::ShowAll,
            _ => return Err(ParseEnumError),
        };
        Ok(scale_mode)
    }
}

impl FromWStr for StageScaleMode {
    type Err = ParseEnumError;

    fn from_wstr(s: &WStr) -> Result<Self, Self::Err> {
        if s.eq_ignore_case(WStr::from_units(b"exactfit")) {
            Ok(StageScaleMode::ExactFit)
        } else if s.eq_ignore_case(WStr::from_units(b"noborder")) {
            Ok(StageScaleMode::NoBorder)
        } else if s.eq_ignore_case(WStr::from_units(b"noscale")) {
            Ok(StageScaleMode::NoScale)
        } else if s.eq_ignore_case(WStr::from_units(b"showall")) {
            Ok(StageScaleMode::ShowAll)
        } else {
            Err(ParseEnumError)
        }
    }
}

/// The scale mode of a stage.
/// This controls the behavior when the player viewport size differs from the SWF size.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageDisplayState {
    /// Sets AIR application or content in Flash Player to expand the stage over the user's entire screen.
    /// Keyboard input is disabled, with the exception of a limited set of non-printing keys.
    FullScreen,

    /// Sets the application to expand the stage over the user's entire screen, with keyboard input allowed.
    /// (Available in AIR and Flash Player, beginning with Flash Player 11.3.)
    FullScreenInteractive,

    /// Sets the stage back to the standard stage display mode.
    #[default]
    Normal,
}

impl Display for StageDisplayState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Match string values returned by AS.
        let s = match *self {
            StageDisplayState::FullScreen => "fullScreen",
            StageDisplayState::FullScreenInteractive => "fullScreenInteractive",
            StageDisplayState::Normal => "normal",
        };
        f.write_str(s)
    }
}

impl FromStr for StageDisplayState {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let display_state = match s.to_ascii_lowercase().as_str() {
            "fullscreen" => StageDisplayState::FullScreen,
            "fullscreeninteractive" => StageDisplayState::FullScreenInteractive,
            "normal" => StageDisplayState::Normal,
            _ => return Err(ParseEnumError),
        };
        Ok(display_state)
    }
}

impl FromWStr for StageDisplayState {
    type Err = ParseEnumError;

    fn from_wstr(s: &WStr) -> Result<Self, Self::Err> {
        if s.eq_ignore_case(WStr::from_units(b"fullscreen")) {
            Ok(StageDisplayState::FullScreen)
        } else if s.eq_ignore_case(WStr::from_units(b"fullscreeninteractive")) {
            Ok(StageDisplayState::FullScreenInteractive)
        } else if s.eq_ignore_case(WStr::from_units(b"normal")) {
            Ok(StageDisplayState::Normal)
        } else {
            Err(ParseEnumError)
        }
    }
}

bitflags! {
    /// The alignment of the stage.
    /// This controls the position of the movie after scaling to fill the viewport.
    /// The default alignment is centered (no bits set).
    ///
    /// This is a bitflags instead of an enum to mimic Flash Player behavior.
    /// You can theoretically have both TOP and BOTTOM bits set, for example.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct StageAlign: u8 {
        /// Align to the top of the viewport.
        const TOP    = 1 << 0;

        /// Align to the bottom of the viewport.
        const BOTTOM = 1 << 1;

        /// Align to the left of the viewport.
        const LEFT   = 1 << 2;

        /// Align to the right of the viewport.;
        const RIGHT  = 1 << 3;
    }
}

impl FromStr for StageAlign {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let align = match s {
            "bottom" => StageAlign::BOTTOM,
            "bottom_left" => StageAlign::BOTTOM | StageAlign::LEFT,
            "bottom_right" => StageAlign::BOTTOM | StageAlign::RIGHT,
            "left" => StageAlign::LEFT,
            "right" => StageAlign::RIGHT,
            "top" => StageAlign::TOP,
            "top_left" => StageAlign::TOP | StageAlign::LEFT,
            "top_right" => StageAlign::TOP | StageAlign::RIGHT,
            "center" => StageAlign::empty(),
            _ => return Err(ParseEnumError),
        };
        Ok(align)
    }
}

impl FromWStr for StageAlign {
    type Err = std::convert::Infallible;

    fn from_wstr(s: &WStr) -> Result<Self, Self::Err> {
        // Chars get converted into flags.
        // This means "tbbtlbltblbrllrbltlrtbl" is valid, resulting in "TBLR".
        let mut align = StageAlign::default();
        for c in s.iter() {
            match u8::try_from(c).map(|c| c.to_ascii_uppercase()) {
                Ok(b'T') => align.insert(StageAlign::TOP),
                Ok(b'B') => align.insert(StageAlign::BOTTOM),
                Ok(b'L') => align.insert(StageAlign::LEFT),
                Ok(b'R') => align.insert(StageAlign::RIGHT),
                _ => (),
            }
        }
        Ok(align)
    }
}

/// The window mode of the Ruffle player.
///
/// This setting controls how the Ruffle container is layered and rendered with other content on
/// the page. This setting is only used on web.
///
/// [Apply OBJECT and EMBED tag attributes in Adobe Flash Professional](https://helpx.adobe.com/flash/kb/flash-object-embed-tag-attributes.html)
#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub enum WindowMode {
    /// The Flash content is rendered in its own window and layering is done with the browser's
    /// default behavior.
    ///
    /// In Ruffle, this mode functions like `WindowMode::Opaque` and will layer the Flash content
    /// together with other HTML elements.
    #[default]
    Window,

    /// The Flash content is layered together with other HTML elements, and the stage color is
    /// opaque. Content can render above or below Ruffle based on CSS rendering order.
    Opaque,

    /// The Flash content is layered together with other HTML elements, and the stage color is
    /// transparent. Content beneath Ruffle will be visible through transparent areas.
    Transparent,

    /// Request compositing with hardware acceleration when possible.
    ///
    /// This mode has no effect in Ruffle and will function like `WindowMode::Opaque`.
    Gpu,

    /// Request a direct rendering path, bypassing browser compositing when possible.
    ///
    /// This mode has no effect in Ruffle and will function like `WindowMode::Opaque`.
    Direct,
}

impl Display for WindowMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match *self {
            WindowMode::Window => "window",
            WindowMode::Opaque => "opaque",
            WindowMode::Transparent => "transparent",
            WindowMode::Direct => "direct",
            WindowMode::Gpu => "gpu",
        };
        f.write_str(s)
    }
}

impl FromStr for WindowMode {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let window_mode = match s.to_ascii_lowercase().as_str() {
            "window" => WindowMode::Window,
            "opaque" => WindowMode::Opaque,
            "transparent" => WindowMode::Transparent,
            "direct" => WindowMode::Direct,
            "gpu" => WindowMode::Gpu,
            _ => return Err(ParseEnumError),
        };
        Ok(window_mode)
    }
}
