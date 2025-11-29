#[cfg(feature = "freetype")]
mod freetype_font_renderer;

#[cfg(feature = "freetype")]
pub use freetype_font_renderer::FreetypeFontRenderer;
