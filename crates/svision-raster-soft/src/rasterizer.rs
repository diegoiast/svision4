use std::rc::Rc;

use fontdue::Font;
use svision_core::{Color, FontSpec, Point, Painter, Rect, Size, TextEngine, TextRasterizer};
use tiny_skia::{
    FillRule, LineCap, LineJoin, Mask, Paint, Path, PathBuilder, Pixmap, Stroke, Transform,
};

use crate::font::{load_default_font, FontdueTextEngine};

/// tiny-skia + fontdue CPU rasterizer.
pub struct SoftRasterizer {
    pixmap: Pixmap,
    font: Option<Rc<Font>>,
    engine: Option<Rc<FontdueTextEngine>>,
    clip_stack: Vec<Rect>,
    clip_mask: Option<Mask>,
    size: Size,
}

impl SoftRasterizer {
    pub fn new() -> Self {
        let font = load_default_font();
        let engine = font
            .as_ref()
            .map(|font| Rc::new(FontdueTextEngine::from_font(font.clone())));
        Self {
            pixmap: Pixmap::new(1, 1).expect("1x1 pixmap"),
            font,
            engine,
            clip_stack: Vec::new(),
            clip_mask: None,
            size: Size::ZERO,
        }
    }

    pub fn with_font(font: Rc<Font>) -> Self {
        let engine = Rc::new(FontdueTextEngine::from_font(font.clone()));
        Self {
            pixmap: Pixmap::new(1, 1).expect("1x1 pixmap"),
            font: Some(font),
            engine: Some(engine),
            clip_stack: Vec::new(),
            clip_mask: None,
            size: Size::ZERO,
        }
    }

    pub fn pixmap(&self) -> &Pixmap {
        &self.pixmap
    }

    pub fn font(&self) -> Option<&Font> {
        self.font.as_deref()
    }

    fn fill_path(&mut self, path: &Path, color: Color) {
        let paint = solid_paint(color);
        let mask = self.clip_mask.as_ref();
        self.pixmap
            .fill_path(path, &paint, FillRule::Winding, Transform::identity(), mask);
    }

    fn stroke_path(&mut self, path: &Path, color: Color, width: f32) {
        let paint = solid_paint(color);
        let stroke = Stroke {
            width: width.max(0.01),
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            ..Stroke::default()
        };
        let mask = self.clip_mask.as_ref();
        self.pixmap
            .stroke_path(path, &paint, &stroke, Transform::identity(), mask);
    }

    fn rebuild_clip_mask(&mut self) {
        let clip = self.clip_stack.last().copied();
        self.clip_mask = None;
        if let Some(rect) = clip {
            self.clip_mask = Some(clip_mask(self.pixmap.width(), self.pixmap.height(), rect));
        }
    }

    fn blend_glyph(
        &mut self,
        origin: (i32, i32),
        width: usize,
        height: usize,
        bitmap: &[u8],
        color: Color,
        clip: Option<Rect>,
    ) {
        let (x0, y0) = origin;
        let pixmap_width = self.pixmap.width() as i32;
        let pixmap_height = self.pixmap.height() as i32;
        let color_alpha = color.a as f32 / 255.0;
        if color_alpha <= 0.0 {
            return;
        }

        let data = self.pixmap.data_mut();

        for row in 0..height as i32 {
            let py = y0 + row;
            if py < 0 || py >= pixmap_height {
                continue;
            }
            for col in 0..width as i32 {
                let px = x0 + col;
                if px < 0 || px >= pixmap_width {
                    continue;
                }
                if let Some(clip) = clip {
                    if !clip.contains(Point::new(px as f32 + 0.5, py as f32 + 0.5)) {
                        continue;
                    }
                }

                let coverage = bitmap[row as usize * width + col as usize] as f32 / 255.0;
                let alpha = coverage * color_alpha;
                if alpha <= 0.0 {
                    continue;
                }

                let index = ((py * pixmap_width + px) * 4) as usize;
                let inverse = 1.0 - alpha;
                let destination = [
                    data[index] as f32 * inverse,
                    data[index + 1] as f32 * inverse,
                    data[index + 2] as f32 * inverse,
                    data[index + 3] as f32 * inverse,
                ];
                data[index] = (color.r as f32 * alpha + destination[0])
                    .round()
                    .clamp(0.0, 255.0) as u8;
                data[index + 1] = (color.g as f32 * alpha + destination[1])
                    .round()
                    .clamp(0.0, 255.0) as u8;
                data[index + 2] = (color.b as f32 * alpha + destination[2])
                    .round()
                    .clamp(0.0, 255.0) as u8;
                data[index + 3] = (255.0 * alpha + destination[3]).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
}

impl Default for SoftRasterizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Painter for SoftRasterizer {
    fn begin_frame(&mut self, size: Size) {
        let width = size.width.max(1.0).round() as u32;
        let height = size.height.max(1.0).round() as u32;
        if self.pixmap.width() != width || self.pixmap.height() != height {
            if let Some(pixmap) = Pixmap::new(width, height) {
                self.pixmap = pixmap;
            }
        }
        self.size = size;
        self.clip_stack.clear();
        self.clip_mask = None;
    }

    fn clear(&mut self, color: Color) {
        self.pixmap.fill(skia_color(color));
    }

    fn fill_rect(&mut self, rect: Rect, color: Color) {
        let Some(skia_rect) = skia_rect(rect) else {
            return;
        };
        let paint = solid_paint(color);
        let mask = self.clip_mask.as_ref();
        self.pixmap
            .fill_rect(skia_rect, &paint, Transform::identity(), mask);
    }

    fn stroke_rect(&mut self, rect: Rect, color: Color, width: f32) {
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return;
        }
        let inset = (width * 0.5).max(0.0);
        let Some(path) = rounded_rect_path(rect.inset_all(inset), 0.0) else {
            return;
        };
        self.stroke_path(&path, color, width);
    }

    fn fill_round_rect(&mut self, rect: Rect, radius: f32, color: Color) {
        let Some(path) = rounded_rect_path(rect, radius) else {
            return;
        };
        self.fill_path(&path, color);
    }

    fn stroke_round_rect(&mut self, rect: Rect, radius: f32, color: Color, width: f32) {
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return;
        }
        let inset = (width * 0.5).max(0.0);
        let Some(path) = rounded_rect_path(rect.inset_all(inset), radius) else {
            return;
        };
        self.stroke_path(&path, color, width);
    }

    fn fill_ellipse(&mut self, rect: Rect, color: Color) {
        let Some(skia_rect) = skia_rect(rect) else {
            return;
        };
        let Some(path) = PathBuilder::from_oval(skia_rect) else {
            return;
        };
        self.fill_path(&path, color);
    }

    fn draw_line(&mut self, from: Point, to: Point, color: Color, width: f32) {
        let mut builder = PathBuilder::new();
        builder.move_to(from.x, from.y);
        builder.line_to(to.x, to.y);
        let Some(path) = builder.finish() else {
            return;
        };
        self.stroke_path(&path, color, width);
    }

    fn push_clip(&mut self, rect: Rect) {
        let rect = match self.clip_stack.last().copied() {
            Some(current) => current.intersect(rect).unwrap_or(Rect::ZERO),
            None => rect,
        };
        self.clip_stack.push(rect);
        self.rebuild_clip_mask();
    }

    fn pop_clip(&mut self) {
        self.clip_stack.pop();
        self.rebuild_clip_mask();
    }
}

impl TextRasterizer for SoftRasterizer {
    fn draw_text(&mut self, text: &str, origin: Point, font: &FontSpec, color: Color) {
        let Some(face) = self.font.clone() else {
            return;
        };
        let px = font.size.max(1.0);
        let metrics = face.horizontal_line_metrics(px);
        let ascent = metrics
            .as_ref()
            .map(|metrics| metrics.ascent)
            .unwrap_or(px * 0.8);
        let lin
