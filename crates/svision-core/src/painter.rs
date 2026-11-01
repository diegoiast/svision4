use crate::{Color, Point, Rect, Size};

/// Pluggable drawing surface.
///
/// A backend owns (or borrows) one of these and feeds it to
/// [`Ui::paint`](crate::Ui::paint). Implementations can be CPU rasterizers,
/// GDI+, OpenGL/Skia, etc.
///
/// The painter owns the surface and draws the primitives (clearing, fills,
/// strokes, lines, clip). Text rendering is layered on top through a
/// [`TextRasterizer`](crate::TextRasterizer).
pub trait Painter {
    /// Prepares the surface for a new frame. Called once before painting.
    fn begin_frame(&mut self, size: Size);

    /// Clears the whole surface, ignoring the current clip.
    fn clear(&mut self, color: Color);

    fn fill_rect(&mut self, rect: Rect, color: Color);

    fn stroke_rect(&mut self, rect: Rect, color: Color, width: f32);

    fn fill_round_rect(&mut self, rect: Rect, radius: f32, color: Color);

    fn stroke_round_rect(&mut self, rect: Rect, radius: f32, color: Color, width: f32);

    fn fill_ellipse(&mut self, rect: Rect, color: Color);

    fn draw_line(&mut self, from: Point, to: Point, color: Color, width: f32);

    fn push_clip(&mut self, rect: Rect);

    fn pop_clip(&mut self);
}
