use crate::context::{RenderContext, UpdateContext};
use crate::font::{DefaultFont, Font, FontType};
use gc_arena::Mutation;
use ruffle_common::tag_utils::SwfMovie;
use std::sync::Arc;

/// Provides access to the font resolution operations needed to lay out text,
/// common to both `UpdateContext` and `RenderContext`.
pub trait LayoutContext<'gc> {
    /// Retrieve the current GC context.
    fn gc(&self) -> &'gc Mutation<'gc>;

    /// Find an embedded font by its name and parameters.
    fn get_embedded_font_by_name(
        &self,
        name: &str,
        font_type: FontType,
        is_bold: bool,
        is_italic: bool,
        movie: Option<Arc<SwfMovie>>,
    ) -> Option<Font<'gc>>;

    /// Returns the default font implementations behind a built-in name.
    fn default_font(&mut self, name: DefaultFont, is_bold: bool, is_italic: bool)
    -> Vec<Font<'gc>>;

    /// Returns the device font(s) best matching the given name and parameters.
    fn get_or_sort_device_fonts(
        &mut self,
        name: &str,
        is_bold: bool,
        is_italic: bool,
    ) -> Vec<Font<'gc>>;
}

impl<'gc> LayoutContext<'gc> for UpdateContext<'gc> {
    #[inline(always)]
    fn gc(&self) -> &'gc Mutation<'gc> {
        self.gc_context
    }

    fn get_embedded_font_by_name(
        &self,
        name: &str,
        font_type: FontType,
        is_bold: bool,
        is_italic: bool,
        movie: Option<Arc<SwfMovie>>,
    ) -> Option<Font<'gc>> {
        self.library
            .get_embedded_font_by_name(name, font_type, is_bold, is_italic, movie)
    }

    fn default_font(
        &mut self,
        name: DefaultFont,
        is_bold: bool,
        is_italic: bool,
    ) -> Vec<Font<'gc>> {
        self.library.default_font(
            name,
            is_bold,
            is_italic,
            self.ui,
            self.renderer,
            self.gc_context,
        )
    }

    fn get_or_sort_device_fonts(
        &mut self,
        name: &str,
        is_bold: bool,
        is_italic: bool,
    ) -> Vec<Font<'gc>> {
        self.library.get_or_sort_device_fonts(
            name,
            is_bold,
            is_italic,
            self.ui,
            self.renderer,
            self.gc_context,
        )
    }
}

impl<'gc> LayoutContext<'gc> for RenderContext<'_, 'gc> {
    #[inline(always)]
    fn gc(&self) -> &'gc Mutation<'gc> {
        self.gc_context
    }

    fn get_embedded_font_by_name(
        &self,
        name: &str,
        font_type: FontType,
        is_bold: bool,
        is_italic: bool,
        movie: Option<Arc<SwfMovie>>,
    ) -> Option<Font<'gc>> {
        self.library
            .get_embedded_font_by_name(name, font_type, is_bold, is_italic, movie)
    }

    fn default_font(
        &mut self,
        name: DefaultFont,
        is_bold: bool,
        is_italic: bool,
    ) -> Vec<Font<'gc>> {
        self.library.default_font(
            name,
            is_bold,
            is_italic,
            self.ui,
            self.renderer,
            self.gc_context,
        )
    }

    fn get_or_sort_device_fonts(
        &mut self,
        name: &str,
        is_bold: bool,
        is_italic: bool,
    ) -> Vec<Font<'gc>> {
        self.library.get_or_sort_device_fonts(
            name,
            is_bold,
            is_italic,
            self.ui,
            self.renderer,
            self.gc_context,
        )
    }
}
