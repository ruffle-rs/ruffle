//! Root stage impl

use crate::avm1::Object as Avm1Object;
use crate::avm2::object::TObject;
use crate::avm2::{
    Activation as Avm2Activation, Avm2, EventObject as Avm2EventObject, Object as Avm2Object,
    ScriptObject as Avm2ScriptObject, StageObject as Avm2StageObject, Value as Avm2Value,
};
use crate::config::Letterbox;
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::container::{
    ChildContainer, DisplayObjectContainer, TDisplayObjectContainer,
};
use crate::display_object::interactive::{
    InteractiveObject, InteractiveObjectBase, TInteractiveObject,
};
use crate::display_object::{
    render_base, DisplayObject, DisplayObjectBase, DisplayObjectPtr, TDisplayObject,
};
use crate::events::{ClipEvent, ClipEventResult};
use crate::prelude::*;
use crate::string::{FromWStr, WStr};
use crate::vminterface::Instantiator;
use bitflags::bitflags;
use gc_arena::{Collect, GcCell, MutationContext};
use ruffle_render::backend::ViewportDimensions;
use ruffle_render::commands::CommandHandler;
use std::cell::{Ref, RefMut};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

/// The Stage is the root of the display object hierarchy. It contains all AVM1
/// levels as well as AVM2 movies.
#[derive(Clone, Debug, Collect, Copy)]
#[collect(no_drop)]
pub struct Stage<'gc>(GcCell<'gc, StageData<'gc>>);

#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct StageData<'gc> {
    /// Base properties for interactive display objects.
    ///
    /// This particular base has additional constraints currently not
    /// expressable by the type system. Notably, this should never have a
    /// parent, as the stage does not respect it.
    base: InteractiveObjectBase<'gc>,

    /// The list of all children of the stage.
    ///
    /// Stage children are exposed to AVM1 as `_level*n*` on all stage objects.
    child: ChildContainer<'gc>,

    /// The stage background.
    ///
    /// If the background color is not specified, it should be white.
    #[collect(require_static)]
    background_color: Option<Color>,

    /// Determines how player content is resized to fit the stage.
    letterbox: Letterbox,

    /// The dimensions of the SWF file.
    #[collect(require_static)]
    movie_size: (u32, u32),

    /// The quality settings of the stage.
    quality: StageQuality,

    /// The dimensions of the stage, as reported to ActionScript.
    #[collect(require_static)]
    stage_size: (u32, u32),

    /// The scale mode of the stage.
    scale_mode: StageScaleMode,

    /// The display state of the stage.
    display_state: StageDisplayState,

    /// The alignment of the stage.
    align: StageAlign,

    /// Whether to use high quality downsampling for bitmaps.
    ///
    /// This is usally implied by `quality` being `Best` or higher, but the AVM1
    /// `ToggleHighQuality` op can adjust stage quality independently of this flag.
    /// This setting is currently ignored in Ruffle.
    use_bitmap_downsampling: bool,

    /// The bounds of the current viewport in twips, used for culling.
    #[collect(require_static)]
    view_bounds: BoundingBox,

    /// The window mode of the viewport.
    ///
    /// Only used on web to control how the Flash content layers with other content on the page.
    window_mode: WindowMode,

    /// Whether or not objects display a glowing border when they have focus.
    stage_focus_rect: bool,

    /// Whether to show default context menu items
    show_menu: bool,

    /// The AVM2 view of this stage object.
    avm2_object: Avm2Object<'gc>,

    /// The AVM2 'LoaderInfo' object for this stage object
    loader_info: Avm2Object<'gc>,

    /// An array of AVM2 'Stage3D' instances
    stage3ds: Vec<Avm2Object<'gc>>,
}

impl<'gc> Stage<'gc> {
    pub fn empty(gc_context: MutationContext<'gc, '_>, fullscreen: bool) -> Stage<'gc> {
        let stage = Self(GcCell::allocate(
            gc_context,
            StageData {
                base: Default::default(),
                child: Default::default(),
                background_color: None,
                letterbox: Letterbox::Fullscreen,
                // This is updated when we set the root movie
                movie_size: (0, 0),
                quality: Default::default(),
                // This is updated in `build_matrices`
                stage_size: (0, 0),
                scale_mode: Default::default(),
                display_state: if fullscreen {
                    StageDisplayState::FullScreen
                } else {
                    StageDisplayState::Normal
                },
                align: Default::default(),
                use_bitmap_downsampling: false,
                view_bounds: Default::default(),
                window_mode: Default::default(),
                show_menu: true,
                stage_focus_rect: true,
                avm2_object: Avm2ScriptObject::custom_object(gc_context, None, None),
                loader_info: Avm2ScriptObject::custom_object(gc_context, None, None),
                stage3ds: vec![],
            },
        ));
        stage.set_is_root(gc_context, true);
        stage
    }

    pub fn background_color(self) -> Option<Color> {
        self.0.read().background_color.clone()
    }

    pub fn set_background_color(self, gc_context: MutationContext<'gc, '_>, color: Option<Color>) {
        self.0.write(gc_context).background_color = color;
    }

    pub fn inverse_view_matrix(self) -> Matrix {
        let mut inverse_view_matrix = *(self.base().matrix());
        inverse_view_matrix.invert();

        inverse_view_matrix
    }

    pub fn letterbox(self) -> Letterbox {
        self.0.read().letterbox
    }

    pub fn set_letterbox(self, gc_context: MutationContext<'gc, '_>, letterbox: Letterbox) {
        self.0.write(gc_context).letterbox = letterbox
    }

    /// Get the size of the SWF file.
    pub fn movie_size(self) -> (u32, u32) {
        self.0.read().movie_size
    }

    /// Set the size of the SWF file.
    pub fn set_movie_size(self, gc_context: MutationContext<'gc, '_>, width: u32, height: u32) {
        self.0.write(gc_context).movie_size = (width, height);
    }

    pub fn set_loader_info(
        self,
        gc_context: MutationContext<'gc, '_>,
        loader_info: Avm2Object<'gc>,
    ) {
        self.0.write(gc_context).loader_info = loader_info;
    }

    /// Returns the quality setting of the stage.
    ///
    /// In the Flash Player, the quality setting affects anti-aliasing and smoothing of bitmaps.
    /// This setting is currently ignored in Ruffle.
    /// Used by AVM1 `stage.quality` and AVM2 `Stage.quality` properties.
    pub fn quality(self) -> StageQuality {
        self.0.read().quality
    }

    /// Sets the quality setting of the stage.
    ///
    /// In the Flash Player, the quality setting affects anti-aliasing and smoothing of bitmaps.
    /// This setting is currently ignored in Ruffle.
    /// Used by AVM1 `stage.quality` and AVM2 `Stage.quality` properties.
    pub fn set_quality(self, gc_context: MutationContext<'gc, '_>, quality: StageQuality) {
        let mut this = self.0.write(gc_context);
        this.quality = quality;
        this.use_bitmap_downsampling = matches!(
            quality,
            StageQuality::Best
                | StageQuality::High8x8
                | StageQuality::High8x8Linear
                | StageQuality::High16x16
                | StageQuality::High16x16Linear
        );
    }

    pub fn stage3ds(&self) -> Ref<Vec<Avm2Object<'gc>>> {
        Ref::map(self.0.read(), |this| &this.stage3ds)
    }

    /// Get the boolean flag which determines whether or not objects display a glowing border
    /// when they have focus.
    ///
    /// This setting is currently ignored in Ruffle.
    pub fn stage_focus_rect(self) -> bool {
        self.0.read().stage_focus_rect
    }

    /// Set the boolean flag which determines whether or not objects display a glowing border
    /// when they have focus.
    ///
    /// This setting is currently ignored in Ruffle.
    pub fn set_stage_focus_rect(self, gc_context: MutationContext<'gc, '_>, fr: bool) {
        let mut this = self.0.write(gc_context);
        this.stage_focus_rect = fr
    }

    /// Get the size of the stage.
    /// Used by AVM1 `stage.width`/`height` and AVM2 `Stage.stageWidth`/`stageHeight` properties.
    /// If `scale_mode` is `StageScaleMode::NO_SCALE`, this returns the size of the viewport.
    /// Otherwise, this returns the size of the SWF file.
    pub fn stage_size(self) -> (u32, u32) {
        self.0.read().stage_size
    }

    /// Get the stage mode.
    /// This controls how the content scales to fill the viewport.
    pub fn scale_mode(self) -> StageScaleMode {
        self.0.read().scale_mode
    }

    /// Set the stage scale mode.
    pub fn set_scale_mode(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        scale_mode: StageScaleMode,
    ) {
        self.0.write(context.gc_context).scale_mode = scale_mode;
        self.build_matrices(context);
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
        self.0.read().display_state
    }

    /// Toggles display state between fullscreen and normal
    pub fn toggle_display_state(self, context: &mut UpdateContext<'_, 'gc, '_>) {
        if self.is_fullscreen() {
            self.set_display_state(context, StageDisplayState::Normal);
        } else {
            self.set_display_state(context, StageDisplayState::FullScreen);
        }
    }

    /// Set the stage display state.
    pub fn set_display_state(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        display_state: StageDisplayState,
    ) {
        if display_state == self.display_state()
            || (Self::is_fullscreen_state(display_state) && self.is_fullscreen())
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
            self.0.write(context.gc_context).display_state = display_state;
            self.fire_fullscreen_event(context);
        }
    }

    /// Get the stage alignment.
    pub fn align(self) -> StageAlign {
        self.0.read().align
    }

    /// Set the stage alignment.
    /// This only has an effect if the scale mode is not `StageScaleMode::ExactFit`.
    pub fn set_align(self, context: &mut UpdateContext<'_, 'gc, '_>, align: StageAlign) {
        self.0.write(context.gc_context).align = align;
        self.build_matrices(context);
    }

    /// Returns whether bitmaps will use high quality downsampling when scaled down.
    /// This setting is currently ignored in Ruffle.
    pub fn use_bitmap_downsampling(self) -> bool {
        self.0.read().use_bitmap_downsampling
    }

    /// Sets whether bitmaps will use high quality downsampling when scaled down.
    /// This setting is currently ignored in Ruffle.
    pub fn set_use_bitmap_downsampling(self, gc_context: MutationContext<'gc, '_>, value: bool) {
        self.0.write(gc_context).use_bitmap_downsampling = value;
    }

    /// Get the stage mode.
    /// This controls how the content layers with other content on the page.
    /// Only used on web.
    pub fn window_mode(self) -> WindowMode {
        self.0.read().window_mode
    }

    /// Sets the window mode.
    pub fn set_window_mode(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        window_mode: WindowMode,
    ) {
        self.0.write(context.gc_context).window_mode = window_mode;
    }

    pub fn view_bounds(self) -> BoundingBox {
        self.0.read().view_bounds.clone()
    }

    pub fn show_menu(self) -> bool {
        self.0.read().show_menu
    }

    pub fn set_show_menu(self, context: &mut UpdateContext<'_, 'gc, '_>, show_menu: bool) {
        let mut write = self.0.write(context.gc_context);
        write.show_menu = show_menu;
    }

    /// Determine if we should letterbox the stage content.
    fn should_letterbox(self) -> bool {
        // Only enable letterbox in the default `ShowAll` scale mode.
        // If content changes the scale mode or alignment, it signals that it is size-aware.
        // For example, `NoScale` is used to make responsive layouts; don't letterbox over it.
        let stage = self.0.read();
        stage.scale_mode == StageScaleMode::ShowAll
            && stage.align.is_empty()
            && stage.window_mode != WindowMode::Transparent
            && (stage.letterbox == Letterbox::On
                || (stage.letterbox == Letterbox::Fullscreen && self.is_fullscreen()))
    }

    /// Update the stage's transform matrix in response to a root movie change.
    pub fn build_matrices(self, context: &mut UpdateContext<'_, 'gc, '_>) {
        let mut stage = self.0.write(context.gc_context);
        let scale_mode = stage.scale_mode;
        let align = stage.align;
        let prev_stage_size = stage.stage_size;
        let viewport_size = context.renderer.viewport_dimensions();

        // Update stage size based on scale mode and DPI.
        stage.stage_size = if stage.scale_mode == StageScaleMode::NoScale {
            // Viewport size is adjusted for HiDPI.
            let width = f64::from(viewport_size.width) / viewport_size.scale_factor;
            let height = f64::from(viewport_size.height) / viewport_size.scale_factor;
            (width.round() as u32, height.round() as u32)
        } else {
            stage.movie_size
        };
        let stage_size_changed = prev_stage_size != stage.stage_size;

        // Create view matrix to scale stage into viewport area.
        let (movie_width, movie_height) = stage.movie_size;
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
        drop(stage);

        *self.base_mut(context.gc_context).matrix_mut() = Matrix {
            a: scale_x as f32,
            b: 0.0,
            c: 0.0,
            d: scale_y as f32,
            tx: Twips::from_pixels(tx),
            ty: Twips::from_pixels(ty),
        };

        self.0.write(context.gc_context).view_bounds = if self.should_letterbox() {
            // Letterbox: movie area
            BoundingBox {
                x_min: Twips::ZERO,
                y_min: Twips::ZERO,
                x_max: Twips::from_pixels(movie_width),
                y_max: Twips::from_pixels(movie_height),
                valid: true,
            }
        } else {
            // No letterbox: full visible stage area
            let margin_left = tx / scale_x;
            let margin_right = (width_delta - tx) / scale_x;
            let margin_top = ty / scale_y;
            let margin_bottom = (height_delta - ty) / scale_y;
            BoundingBox {
                x_min: Twips::from_pixels(-margin_left),
                y_min: Twips::from_pixels(-margin_top),
                x_max: Twips::from_pixels(movie_width + margin_right),
                y_max: Twips::from_pixels(movie_height + margin_bottom),
                valid: true,
            }
        };

        // Fire resize handler if stage size has changed.
        if scale_mode == StageScaleMode::NoScale && stage_size_changed {
            self.fire_resize_event(context);
        }
    }

    /// Draw the stage's letterbox.
    fn draw_letterbox(&self, context: &mut RenderContext<'_, 'gc, '_>) {
        let ViewportDimensions {
            width: viewport_width,
            height: viewport_height,
            scale_factor: _,
        } = context.renderer.viewport_dimensions();
        let viewport_width = viewport_width as f32;
        let viewport_height = viewport_height as f32;

        let base = self.base();
        let view_matrix = base.matrix();

        let (movie_width, movie_height) = self.0.read().movie_size;
        let movie_width = movie_width as f32 * view_matrix.a;
        let movie_height = movie_height as f32 * view_matrix.d;

        let margin_left = view_matrix.tx.to_pixels() as f32;
        let margin_right = viewport_width - movie_width - margin_left;
        let margin_top = view_matrix.ty.to_pixels() as f32;
        let margin_bottom = viewport_height - movie_height - margin_top;

        // Letterboxing only occurs in `StageScaleMode::ShowAll`, and they would only appear on the top+bottom or left+right.
        if margin_top + margin_bottom > margin_left + margin_right {
            // Top + bottom
            if margin_top > 0.0 {
                context.commands.draw_rect(
                    Color::BLACK,
                    &Matrix::create_box(
                        viewport_width,
                        margin_top,
                        0.0,
                        Twips::default(),
                        Twips::default(),
                    ),
                );
            }
            if margin_bottom > 0.0 {
                context.commands.draw_rect(
                    Color::BLACK,
                    &Matrix::create_box(
                        viewport_width,
                        margin_bottom,
                        0.0,
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
                    &Matrix::create_box(
                        margin_left,
                        viewport_height,
                        0.0,
                        Twips::default(),
                        Twips::default(),
                    ),
                );
            }
            if margin_right > 0.0 {
                context.commands.draw_rect(
                    Color::BLACK,
                    &Matrix::create_box(
                        margin_right,
                        viewport_height,
                        0.0,
                        Twips::from_pixels((viewport_width - margin_right) as f64),
                        Twips::default(),
                    ),
                );
            }
        }
    }

    /// Obtain the root movie on the stage.
    ///
    /// `Stage` guarantees that there is always a movie clip at depth 0.
    pub fn root_clip(self) -> DisplayObject<'gc> {
        self.child_by_depth(0)
            .expect("Stage must always have a root movie")
    }

    /// Fires `Stage.onResize` in AVM1 or `Event.RESIZE` in AVM2.
    fn fire_resize_event(self, context: &mut UpdateContext<'_, 'gc, '_>) {
        // This event fires immediately when scaleMode is changed;
        // it doesn't queue up.
        if !context.is_action_script_3() {
            crate::avm1::Avm1::notify_system_listeners(
                self.root_clip(),
                context,
                "Stage".into(),
                "onResize".into(),
                &[],
            );
        } else if let Avm2Value::Object(stage) = self.object2() {
            let resized_event = Avm2EventObject::bare_default_event(context, "resize");
            if let Err(e) = crate::avm2::Avm2::dispatch_event(context, resized_event, stage) {
                log::error!("Encountered AVM2 error when dispatching event: {}", e);
            }
        }
    }

    /// Fires `Stage.onFullScreen` in AVM1 or `Event.FULLSCREEN` in AVM2.
    pub fn fire_fullscreen_event(self, context: &mut UpdateContext<'_, 'gc, '_>) {
        if !context.is_action_script_3() {
            crate::avm1::Avm1::notify_system_listeners(
                self.root_clip(),
                context,
                "Stage".into(),
                "onFullScreen".into(),
                &[self.is_fullscreen().into()],
            );
        } else if let Avm2Value::Object(stage) = self.object2() {
            let full_screen_event_cls = context.avm2.classes().fullscreenevent;
            let mut activation = Avm2Activation::from_nothing(context.reborrow());
            let full_screen_event = full_screen_event_cls
                .construct(
                    &mut activation,
                    &[
                        "fullScreen".into(),
                        false.into(),
                        false.into(),
                        self.is_fullscreen().into(),
                        true.into(),
                    ],
                )
                .unwrap(); // we don't expect to break here

            if let Err(e) = crate::avm2::Avm2::dispatch_event(context, full_screen_event, stage) {
                log::error!("Encountered AVM2 error when dispatching event: {}", e);
            }
        }
    }
}

impl<'gc> TDisplayObject<'gc> for Stage<'gc> {
    fn base(&self) -> Ref<DisplayObjectBase<'gc>> {
        Ref::map(self.0.read(), |r| &r.base.base)
    }

    fn base_mut<'a>(&'a self, mc: MutationContext<'gc, '_>) -> RefMut<'a, DisplayObjectBase<'gc>> {
        RefMut::map(self.0.write(mc), |w| &mut w.base.base)
    }

    fn instantiate(&self, gc_context: MutationContext<'gc, '_>) -> DisplayObject<'gc> {
        Self(GcCell::allocate(gc_context, self.0.read().clone())).into()
    }

    fn as_ptr(&self) -> *const DisplayObjectPtr {
        self.0.as_ptr() as *const DisplayObjectPtr
    }

    fn local_to_global_matrix(&self) -> Matrix {
        // TODO: See comments in DisplayObject::local_to_global_matrix.
        Default::default()
    }

    fn post_instantiation(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        _init_object: Option<Avm1Object<'gc>>,
        _instantiated_by: Instantiator,
        _run_frame: bool,
    ) {
        let stage_constr = context.avm2.classes().stage;

        // TODO: Replace this when we have a convenience method for constructing AVM2 native objects.
        // TODO: We should only do this if the movie is actually an AVM2 movie.
        // This is necessary for EventDispatcher super-constructor to run.
        let mut activation = Avm2Activation::from_nothing(context.reborrow());
        let avm2_stage = Avm2StageObject::for_display_object_childless(
            &mut activation,
            (*self).into(),
            stage_constr,
        );

        // Just create a single Stage3D for now
        let stage3d = activation
            .avm2()
            .classes()
            .stage3d
            .construct(&mut activation, &[])
            .expect("Failed to construct Stage3D");

        match avm2_stage {
            Ok(avm2_stage) => {
                let mut write = self.0.write(activation.context.gc_context);
                write.avm2_object = avm2_stage.into();
                write.stage3ds = vec![stage3d];
            }
            Err(e) => log::error!("Unable to construct AVM2 Stage: {}", e),
        }
    }

    fn id(&self) -> CharacterId {
        u16::MAX
    }

    fn self_bounds(&self) -> BoundingBox {
        Default::default()
    }

    fn as_container(self) -> Option<DisplayObjectContainer<'gc>> {
        Some(self.into())
    }

    fn as_interactive(self) -> Option<InteractiveObject<'gc>> {
        Some(self.into())
    }

    fn as_stage(&self) -> Option<Stage<'gc>> {
        Some(*self)
    }

    fn render_self(&self, context: &mut RenderContext<'_, 'gc, '_>) {
        self.render_children(context);
    }

    fn render(&self, context: &mut RenderContext<'_, 'gc, '_>) {
        // All of our Stage3D instances get rendered *underneath* the main stage.
        // Note that the stage background color is actually the lowest possible layer,
        // and get applied when we start the frame (before `render` is called).
        for stage3d in self.stage3ds().iter() {
            if let Some(context3d) = stage3d.as_stage_3d().unwrap().context3d() {
                context3d.as_context_3d().unwrap().render(context);
            }
        }

        render_base((*self).into(), context);

        if self.should_letterbox() {
            self.draw_letterbox(context);
        }
    }

    fn enter_frame(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        for child in self.iter_render_list() {
            child.enter_frame(context);
        }

        let enter_frame_evt = Avm2EventObject::bare_default_event(context, "enterFrame");

        let dobject_constr = context.avm2.classes().display_object;

        if let Err(e) = Avm2::broadcast_event(context, enter_frame_evt, dobject_constr) {
            log::error!(
                "Encountered AVM2 error when broadcasting enterFrame event: {}",
                e
            );
        }
    }

    fn construct_frame(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        for child in self.iter_render_list() {
            child.construct_frame(context);
        }
    }

    fn object2(&self) -> Avm2Value<'gc> {
        self.0.read().avm2_object.into()
    }

    fn loader_info(&self) -> Option<Avm2Object<'gc>> {
        Some(self.0.read().loader_info)
    }
}

impl<'gc> TDisplayObjectContainer<'gc> for Stage<'gc> {
    fn raw_container(&self) -> Ref<'_, ChildContainer<'gc>> {
        Ref::map(self.0.read(), |this| &this.child)
    }

    fn raw_container_mut(
        &self,
        gc_context: MutationContext<'gc, '_>,
    ) -> RefMut<'_, ChildContainer<'gc>> {
        RefMut::map(self.0.write(gc_context), |this| &mut this.child)
    }
}

impl<'gc> TInteractiveObject<'gc> for Stage<'gc> {
    fn raw_interactive(&self) -> Ref<InteractiveObjectBase<'gc>> {
        Ref::map(self.0.read(), |r| &r.base)
    }

    fn raw_interactive_mut(
        &self,
        mc: MutationContext<'gc, '_>,
    ) -> RefMut<InteractiveObjectBase<'gc>> {
        RefMut::map(self.0.write(mc), |w| &mut w.base)
    }

    fn as_displayobject(self) -> DisplayObject<'gc> {
        self.into()
    }

    fn filter_clip_event(self, _event: ClipEvent) -> ClipEventResult {
        ClipEventResult::Handled
    }

    fn event_dispatch(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        event: ClipEvent<'gc>,
    ) -> ClipEventResult {
        self.event_dispatch_to_avm2(context, event);

        ClipEventResult::Handled
    }
}

pub struct ParseEnumError;

/// The scale mode of a stage.
/// This controls the behavior when the player viewport size differs from the SWF size.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Collect)]
#[collect(require_static)]
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
        let scale_mode = match s.to_ascii_lowercase().as_str() {
            "exactfit" => StageScaleMode::ExactFit,
            "noborder" => StageScaleMode::NoBorder,
            "noscale" => StageScaleMode::NoScale,
            "showall" => StageScaleMode::ShowAll,
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
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Collect)]
#[collect(require_static)]
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
    #[derive(Default, Collect)]
    #[collect(require_static)]
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
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Chars get converted into flags.
        // This means "tbbtlbltblbrllrbltlrtbl" is valid, resulting in "TBLR".
        let mut align = StageAlign::default();
        for c in s.bytes().map(|c| c.to_ascii_uppercase()) {
            match c {
                b'T' => align.insert(StageAlign::TOP),
                b'B' => align.insert(StageAlign::BOTTOM),
                b'L' => align.insert(StageAlign::LEFT),
                b'R' => align.insert(StageAlign::RIGHT),
                _ => (),
            }
        }
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

/// The quality setting of the `Stage`.
///
/// In the Flash Player, this settings affects anti-aliasing and bitmap smoothing.
/// These settings currently have no effect in Ruffle, but the active setting is still stored.
/// [StageQuality in the AS3 Reference](https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/display/StageQuality.html)
#[derive(Default, Clone, Collect, Copy, Debug, Eq, PartialEq)]
#[collect(require_static)]
pub enum StageQuality {
    /// No anti-aliasing, and bitmaps are never smoothed.
    Low,

    /// 2x anti-aliasing.
    Medium,

    /// 4x anti-aliasing.
    #[default]
    High,

    /// 4x anti-aliasing with high quality downsampling.
    /// Bitmaps will use high quality downsampling when scaled down.
    /// Despite the name, this is not the best quality setting as 8x8 and 16x16 modes were added to
    /// Flash Player 11.3.
    Best,

    /// 8x anti-aliasing.
    /// Bitmaps will use high quality downsampling when scaled down.
    High8x8,

    /// 8x anti-aliasing done in linear sRGB space.
    /// Bitmaps will use high quality downsampling when scaled down.
    High8x8Linear,

    /// 16x anti-aliasing.
    /// Bitmaps will use high quality downsampling when scaled down.
    High16x16,

    /// 16x anti-aliasing done in linear sRGB space.
    /// Bitmaps will use high quality downsampling when scaled down.
    High16x16Linear,
}

impl StageQuality {
    /// Returns the string representing the quality setting as returned by AVM1 `_quality` and
    /// AVM2 `Stage.quality`.
    pub fn into_avm_str(self) -> &'static str {
        // Flash Player always returns quality in uppercase, despite the AVM2 `StageQuality` being
        // lowercase.
        match self {
            StageQuality::Low => "LOW",
            StageQuality::Medium => "MEDIUM",
            StageQuality::High => "HIGH",
            StageQuality::Best => "BEST",
            // The linear sRGB quality settings are not returned even if they are active.
            StageQuality::High8x8 | StageQuality::High8x8Linear => "8X8",
            StageQuality::High16x16 | StageQuality::High16x16Linear => "16X16",
        }
    }
}

impl Display for StageQuality {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Match string values returned by AS.
        let s = match *self {
            StageQuality::Low => "low",
            StageQuality::Medium => "medium",
            StageQuality::High => "high",
            StageQuality::Best => "best",
            StageQuality::High8x8 => "8x8",
            StageQuality::High8x8Linear => "8x8linear",
            StageQuality::High16x16 => "16x16",
            StageQuality::High16x16Linear => "16x16linear",
        };
        f.write_str(s)
    }
}

impl FromStr for StageQuality {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let quality = match s.to_ascii_lowercase().as_str() {
            "low" => StageQuality::Low,
            "medium" => StageQuality::Medium,
            "high" => StageQuality::High,
            "best" => StageQuality::Best,
            "8x8" => StageQuality::High8x8,
            "8x8linear" => StageQuality::High8x8Linear,
            "16x16" => StageQuality::High16x16,
            "16x16linear" => StageQuality::High16x16Linear,
            _ => return Err(ParseEnumError),
        };
        Ok(quality)
    }
}

impl FromWStr for StageQuality {
    type Err = ParseEnumError;

    fn from_wstr(s: &WStr) -> Result<Self, Self::Err> {
        if s.eq_ignore_case(WStr::from_units(b"low")) {
            Ok(StageQuality::Low)
        } else if s.eq_ignore_case(WStr::from_units(b"medium")) {
            Ok(StageQuality::Medium)
        } else if s.eq_ignore_case(WStr::from_units(b"high")) {
            Ok(StageQuality::High)
        } else if s.eq_ignore_case(WStr::from_units(b"best")) {
            Ok(StageQuality::Best)
        } else if s.eq_ignore_case(WStr::from_units(b"8x8")) {
            Ok(StageQuality::High8x8)
        } else if s.eq_ignore_case(WStr::from_units(b"8x8linear")) {
            Ok(StageQuality::High8x8Linear)
        } else if s.eq_ignore_case(WStr::from_units(b"16x16")) {
            Ok(StageQuality::High16x16)
        } else if s.eq_ignore_case(WStr::from_units(b"16x16linear")) {
            Ok(StageQuality::High16x16Linear)
        } else {
            Err(ParseEnumError)
        }
    }
}

/// The window mode of the Ruffle player.
///
/// This setting controls how the Ruffle container is layered and rendered with other content on
/// the page. This setting is only used on web.
///
/// [Apply OBJECT and EMBED tag attributes in Adobe Flash Professional](https://helpx.adobe.com/flash/kb/flash-object-embed-tag-attributes.html)
#[derive(Default, Clone, Collect, Copy, Debug, Eq, PartialEq)]
#[collect(require_static)]
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
