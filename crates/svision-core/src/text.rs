use std::rc::Rc;

use crate::{Color, Point, Size, painter::Painter};

/// Description of a font request.
///
/// The built-in software text rasterizer currently resolves all requests to the
/// platform UI font; family/bold/italic are retained so richer rasterizers can
/// honour them.
#[derive(Clone, Debug)]
pub struct FontSpec {
    pub family: Option<String>,
    pub size: f32,
    pub bold: bool,
    pub italic: bool,
}

impl FontSpec {
    pub fn new(size: f32) -> Self {
        Self {
            family: None,
            size,
            bold: false,
            italic: false,
        }
    }

    pub fn family(mut self, family: impl Into<String>) -> Self {
        self.family = Some(family.into());
        self
    }

    pub fn bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }

    pub fn italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }
}

/// Pluggable text measurement used during layout.
///
/// A graphics backend can provide a real implementation (see
/// `svision-raster-soft::FontdueTextEngine`) through
/// [`TextRasterizer::text_engine`](crate::TextRasterizer::text_engine).
pub trait TextEngine {
    fn measure(&self, text: &str, font: &FontSpec) -> Size;
}

/// Dependency-free approximation, used until a backend supplies a real engine.
#[derive(Clone, Copy, Debug, Default)]
pub struct ApproxTextEngine;

impl TextEngine for ApproxTextEngine {
    fn measure(&self, text: &str, font: &FontSpec) -> Size {
        let size = font.size.max(1.0);
        let mut max_chars = 0usize;
        let mut lines = 0usize;
        for line in text.lines() {
            lines += 1;
            max_chars = max_chars.max(line.chars().count());
        }
        if lines == 0 {
            lines = 1;
        }
        let char_width = size * if font.bold { 0.58 } else { 0.54 };
        Size::new(max_chars as f32 * char_width, lines as f32 * size * 1.25)
    }
}

/// Text drawing capability layered on top of a [`Painter`].
///
/// Implementations rasterize glyphs into the underlying painting surface via
/// [`TextRasterizer::draw_text`]. The optional [`TextEngine`] supplies real
/// text metrics for layout.
pub trait TextRasterizer: Painter {
    /// Draws text with its top-left corner at `origin`.
    fn draw_text(&mut self, text: &str, origin: Point, font: &FontSpec, color: Color);

    /// Optional real text metrics implementation for layout.
    fn text_engine(&self) -> Option<Rc<dyn TextEngine>> {
        None
    }
}
