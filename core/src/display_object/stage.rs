//! Root stage impl

use crate::backend::ui::UiBackend;
use crate::collect::CollectWrapper;
use crate::config::Letterbox;
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::container::{
    ChildContainer, DisplayObjectContainer, TDisplayObjectContainer,
};
use crate::display_object::{render_base, DisplayObject, DisplayObjectBase, TDisplayObject};
use crate::prelude::*;
use crate::types::{Degrees, Percent};
use gc_arena::{Collect, GcCell, MutationContext};

/// The Stage is the root of the display object hierarchy. It contains all AVM1
/// levels as well as AVM2 movies.
#[derive(Clone, Debug, Collect, Copy)]
#[collect(no_drop)]
pub struct Stage<'gc>(GcCell<'gc, StageData<'gc>>);

#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct StageData<'gc> {
    /// Base properties for all display objects.
    ///
    /// This particular base has additional constraints currently not
    /// expressable by the type system. Notably, this should never have a
    /// parent, as the stage does not respect it.
    base: DisplayObjectBase<'gc>,

    /// The list of all children of the stage.
    ///
    /// Stage children are exposed to AVM1 as `_level*n*` on all stage objects.
    child: ChildContainer<'gc>,

    /// The stage background.
    ///
    /// If the background color is not specified, it should be white.
    background_color: CollectWrapper<Option<Color>>,

    /// Determines how player content is resized to fit the stage.
    letterbox: Letterbox,

    /// The dimensions of the stage.
    stage_size: CollectWrapper<(u32, u32)>,

    /// The dimensions of the stage's containing viewport.
    viewport_size: CollectWrapper<(u32, u32)>,

    /// The bounds of the current viewport in twips, used for culling.
    view_bounds: BoundingBox,
}

impl<'gc> Stage<'gc> {
    pub fn empty(gc_context: MutationContext<'gc, '_>, width: u32, height: u32) -> Stage<'gc> {
        Self(GcCell::allocate(
            gc_context,
            StageData {
                base: Default::default(),
                child: Default::default(),
                background_color: CollectWrapper(None),
                letterbox: Letterbox::Fullscreen,
                stage_size: CollectWrapper((width, height)),
                viewport_size: CollectWrapper((width, height)),
                view_bounds: Default::default(),
            },
        ))
    }

    pub fn background_color(self) -> Option<Color> {
        self.0.read().background_color.0.clone()
    }

    pub fn set_background_color(self, gc_context: MutationContext<'gc, '_>, color: Option<Color>) {
        self.0.write(gc_context).background_color.0 = color;
    }

    pub fn inverse_view_matrix(self) -> Matrix {
        let mut inverse_view_matrix = *(self.matrix());
        inverse_view_matrix.invert();

        inverse_view_matrix
    }

    pub fn letterbox(self) -> Letterbox {
        self.0.read().letterbox
    }

    pub fn set_letterbox(self, gc_context: MutationContext<'gc, '_>, letterbox: Letterbox) {
        self.0.write(gc_context).letterbox = letterbox
    }

    /// Get the current stage size.
    pub fn stage_size(self) -> (u32, u32) {
        self.0.read().stage_size.0
    }

    /// Set the current stage size.
    pub fn set_stage_size(self, gc_context: MutationContext<'gc, '_>, width: u32, height: u32) {
        self.0.write(gc_context).stage_size.0 = (width, height);
    }

    /// Get the current viewport size.
    pub fn viewport_size(self) -> (u32, u32) {
        self.0.read().viewport_size.0
    }

    /// Set the current viewport size.
    pub fn set_viewport_size(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        width: u32,
        height: u32,
    ) {
        self.0.write(context.gc_context).viewport_size.0 = (width, height);
        self.build_matrices(context);
    }

    pub fn view_bounds(self) -> BoundingBox {
        self.0.read().view_bounds.clone()
    }

    /// Determine if we should letterbox the stage content.
    fn should_letterbox(self, ui: &mut dyn UiBackend) -> bool {
        let letterbox = self.letterbox();

        letterbox == Letterbox::On || (letterbox == Letterbox::Fullscreen && ui.is_fullscreen())
    }

    /// Update the stage's transform matrix in response to a root movie change.
    pub fn build_matrices(self, context: &mut UpdateContext<'_, 'gc, '_>) {
        // Create view matrix to scale stage into viewport area.
        let (movie_width, movie_height) = self.0.read().stage_size.0;
        let movie_width = movie_width as f64;
        let movie_height = movie_height as f64;

        let (viewport_width, viewport_height) = self.0.read().viewport_size.0;
        let viewport_width = viewport_width as f64;
        let viewport_height = viewport_height as f64;

        let movie_aspect = movie_width / movie_height;
        let viewport_aspect = viewport_width / viewport_height;
        let (scale, margin_width, margin_height) = if viewport_aspect > movie_aspect {
            let scale = viewport_height / movie_height;
            (scale, (viewport_width - movie_width * scale) / 2.0, 0.0)
        } else {
            let scale = viewport_width / movie_width;
            (scale, 0.0, (viewport_height - movie_height * scale) / 2.0)
        };
        *self.matrix_mut(context.gc_context) = Matrix {
            a: scale as f32,
            b: 0.0,
            c: 0.0,
            d: scale as f32,
            tx: Twips::from_pixels(margin_width),
            ty: Twips::from_pixels(margin_height),
        };

        self.0.write(context.gc_context).view_bounds = if self.should_letterbox(context.ui) {
            // No letterbox: movie area
            BoundingBox {
                x_min: Twips::zero(),
                y_min: Twips::zero(),
                x_max: Twips::from_pixels(movie_width),
                y_max: Twips::from_pixels(movie_height),
                valid: true,
            }
        } else {
            // No letterbox: full visible stage area
            let margin_width = margin_width / scale;
            let margin_height = margin_height / scale;
            BoundingBox {
                x_min: Twips::from_pixels(-margin_width),
                y_min: Twips::from_pixels(-margin_height),
                x_max: Twips::from_pixels(movie_width + margin_width),
                y_max: Twips::from_pixels(movie_height + margin_height),
                valid: true,
            }
        };
    }

    /// Draw the stage's letterbox.
    fn draw_letterbox(&self, context: &mut RenderContext<'_, 'gc>) {
        let black = Color::from_rgb(0, 255);
        let (viewport_width, viewport_height) = self.0.read().viewport_size.0;
        let viewport_width = viewport_width as f32;
        let viewport_height = viewport_height as f32;

        let view_matrix = self.matrix();

        let margin_width = view_matrix.tx.to_pixels() as f32;
        let margin_height = view_matrix.ty.to_pixels() as f32;
        if margin_height > 0.0 {
            context.renderer.draw_rect(
                black.clone(),
                &Matrix::create_box(
                    viewport_width,
                    margin_height,
                    0.0,
                    Twips::default(),
                    Twips::default(),
                ),
            );
            context.renderer.draw_rect(
                black,
                &Matrix::create_box(
                    viewport_width,
                    margin_height,
                    0.0,
                    Twips::default(),
                    Twips::from_pixels((viewport_height - margin_height) as f64),
                ),
            );
        } else if margin_width > 0.0 {
            context.renderer.draw_rect(
                black.clone(),
                &Matrix::create_box(
                    margin_width,
                    viewport_height,
                    0.0,
                    Twips::default(),
                    Twips::default(),
                ),
            );
            context.renderer.draw_rect(
                black,
                &Matrix::create_box(
                    margin_width,
                    viewport_height,
                    0.0,
                    Twips::from_pixels((viewport_width - margin_width) as f64),
                    Twips::default(),
                ),
            );
        }
    }
}

impl<'gc> TDisplayObject<'gc> for Stage<'gc> {
    impl_display_object!(base);

    fn id(&self) -> CharacterId {
        u16::MAX
    }

    fn self_bounds(&self) -> BoundingBox {
        Default::default()
    }

    fn as_container(self) -> Option<DisplayObjectContainer<'gc>> {
        Some(self.into())
    }

    fn as_stage(&self) -> Option<Stage<'gc>> {
        Some(*self)
    }

    fn render_self(&self, context: &mut RenderContext<'_, 'gc>) {
        self.render_children(context);
    }

    fn render(&self, context: &mut RenderContext<'_, 'gc>) {
        let background_color = self
            .background_color()
            .unwrap_or_else(|| Color::from_rgb(0xffffff, 255));

        context.renderer.begin_frame(background_color);

        render_base((*self).into(), context);

        if self.should_letterbox(context.ui) {
            self.draw_letterbox(context);
        }

        context.renderer.end_frame();
    }

    fn construct_frame(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        for child in self.iter_execution_list() {
            child.construct_frame(context);
        }
    }

    fn run_frame(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        for child in self.iter_execution_list() {
            child.run_frame(context);
        }
    }
}

impl<'gc> TDisplayObjectContainer<'gc> for Stage<'gc> {
    impl_display_object_container!(child);
}
