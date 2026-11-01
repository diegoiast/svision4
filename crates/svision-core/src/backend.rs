use std::error::Error;

use crate::{Size, TextRasterizer};

/// Pluggable presentation backend.
///
/// The backend owns the native window surface (softbuffer, GDI+, an OpenGL
/// context, ...) and exposes a [`TextRasterizer`] that draws into it.
///
/// The host drives the cycle:
///
/// 1. `Ui::paint(backend.rasterizer())`
/// 2. `backend.present()`
pub trait GraphicsBackend {
    /// Notification that the window surface changed size.
    fn resize(&mut self, size: Size);

    /// The rasterizer used for the next frame. Implementations are usually a
    /// [`TextRasterizer`] (which is also a [`Painter`]) drawing into the
    /// backend surface.
    fn rasterizer(&mut self) -> &mut dyn TextRasterizer;

    /// Flushes the rasterized frame to the window.
    fn present(&mut self) -> Result<(), Box<dyn Error>>;

    /// Current surface size in pixels.
    fn size(&self) -> Size;
}
