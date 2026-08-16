#[cfg(all(target_os = "linux", feature = "freetype"))]
mod freetype_font_renderer;

#[cfg(all(target_os = "linux", feature = "freetype"))]
pub use freetype_font_renderer::FreetypeFontRenderer;
