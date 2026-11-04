use crate::{
    Color, Event, FontSpec, Key, MouseButton, Painter,
    button_state::ButtonState,
    geometry::{Point, Rect, Size},
    widget::{EventCtx, MeasureCtx, PaintCtx, SizePolicy, SizePolicy2D, Widget},
};

/// Draws a dotted (dashed) rectangle outline using short line segments.
///
/// This is used for focus indicators on Windows themes, where Windows 95
/// (zero corner radius) intentionally does not get a dotted frame.
fn draw_dotted_rect(rasterizer: &mut dyn Painter, rect: Rect, color: Color, width: f32) {
    let dash = 4.0;
    let gap = 4.0;

    // Top and bottom edges.
    let mut x = rect.x;
    while x < rect.right() {
        let x1 = x;
        let x2 = (x + dash).min(rect.right());
        rasterizer.draw_line(Point::new(x1, rect.y), Point::new(x2, rect.y), color, width);
        rasterizer.draw_line(
            Point::new(x1, rect.bottom()),
            Point::new(x2, rect.bottom()),
            color,
            width,
        );
        x += dash + gap;
    }

    // Left and right edges.
    let mut y = rect.y;
    while y < rect.bottom() {
        let y1 = y;
        let y2 = (y + dash).min(rect.bottom());
        rasterizer.draw_line(Point::new(rect.x, y1), Point::new(rect.x, y2), color, width);
        rasterizer.draw_line(
            Point::new(rect.right(), y1),
            Point::new(rect.right(), y2),
            color,
            width,
        );
        y += dash + gap;
    }
}

/// A checkbox widget that can be toggled on/off.
/// It displays a square box that is filled when checked,
/// and optional label text to the right (the label is stored but not rasterized here;
/// backends may render it or the caller can use the `label` field).
pub struct Checkbox {
    label: String,
    checked: bool,
    measured: Size,
    on_change: Option<Box<dyn FnMut(&mut EventCtx)>>,
}

impl Checkbox {
    /// Create a new checkbox with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            checked: false,
            measured: Size::ZERO,
            on_change: None,
        }
    }

    /// Set the initial checked state.
    pub fn selected(mut self, selected: bool) -> Self {
        self.checked = selected;
        self
    }

    pub fn is_checked(&self) -> bool {
        self.checked
    }

    /// Set a callback that is called when the checkbox is clicked.
    /// The callback receives the event context and can query the new state.
    pub fn on_click<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut EventCtx) + 'static,
    {
        self.on_change = Some(Box::new(f));
        self
    }
}

impl Widget for Checkbox {
    fn size_policy(&self) -> SizePolicy2D {
        // Fixed size: a 16×16 square plus label space.
        SizePolicy2D::new(SizePolicy::Fixed, SizePolicy::Fixed)
    }

    fn measure(&mut self, ctx: &mut MeasureCtx, _available: Size) -> Size {
        // Square side: smaller than full height, min 24px.
        let gap = 4.0;
        let theme = ctx.theme();
        let font = FontSpec::new(theme.font_size());
        let text_size = if self.label.is_empty() {
            Size::ZERO
        } else {
            ctx.measure_text(&self.label, &font)
        };
        let side = (text_size.height * 0.6).max(24.0);
        self.measured = text_size;
        Size::new(side + gap + text_size.width, side.max(text_size.height))
    }

    fn min_size(&mut self, ctx: &mut MeasureCtx, _available: Size) -> Size {
        self.measure(ctx, Size::new(0.0, 0.0))
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let theme = ctx.theme;
        let b = ctx.bounds;
        let state = ctx.state.interaction;

        // Square mark on the left side.
        let gap = 4.0;
        let side = b.height.min(b.width).min(28.0);
        let x = b.x;
        let y = b.y + (b.height - side) / 2.0;
        let box_b = Rect::new(x, y, side, side);

        // Background: accent when checked, surface/hover/pressed otherwise.
        let bg = if self.checked {
            match state {
                ButtonState::ClickedInside => theme.accent_pressed(),
                ButtonState::Hovered => theme.accent_hover(),
                _ => theme.accent(),
            }
        } else {
            match state {
                ButtonState::ClickedInside => theme.surface_pressed(),
                ButtonState::Hovered => theme.surface_hover(),
                _ => theme.surface(),
            }
        };
        ctx.rasterizer.fill_rect(box_b, bg);

        // Border: accent when checked/hovered, border otherwise.
        let border_color = match state {
            ButtonState::ClickedInside => theme.accent_pressed(),
            ButtonState::Hovered => theme.accent_hover(),
            _ => {
                if self.checked {
                    theme.accent()
                } else {
                    theme.border()
                }
            }
        };
        ctx.rasterizer.stroke_rect(box_b, border_color, 1.0);

        // Draw a "v" check mark inside the square when checked.
        if self.checked {
            let cx = b.x + side / 2.0;
            let cy = b.y + side / 2.0;
            let p1 = Point::new(cx - side * 0.28, cy - side * 0.10);
            let p2 = Point::new(cx - side * 0.08, cy + side * 0.22);
            let p3 = Point::new(cx + side * 0.28, cy - side * 0.30);
            ctx.rasterizer.draw_line(p1, p2, theme.accent_text(), 2.0);
            ctx.rasterizer.draw_line(p2, p3, theme.accent_text(), 2.0);
        }

        // Draw label text to the right of the square.
        if !self.label.is_empty() {
            let font = FontSpec::new(theme.font_size());
            let text_x = b.x + side + gap;
            let text_y = b.y + (b.height - self.measured.height) / 2.0;
            ctx.rasterizer
                .draw_text(&self.label, Point::new(text_x, text_y), &font, theme.text());
        }

        // Dotted focus frame on Windows themes (non-zero radius).
        // Windows 95 (radius == 0) does not get a dotted frame.
        if ctx.state.focused && ctx.state.window_active && theme.radius() > 0.0 {
            draw_dotted_rect(&mut *ctx.rasterizer, ctx.bounds, theme.focus(), 1.0);
        }
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event) -> bool {
        match event {
            Event::KeyDown {
                key: Key::Space | Key::Enter,
                ..
            } => {
                self.checked = !self.checked;
                ctx.invalidate();
                if let Some(ref mut f) = self.on_change {
                    f(ctx);
                }
                true
            }
            Event::MouseDown {
                position,
                button: MouseButton::Left,
            } if ctx.bounds.contains(*position) => {
                // Show pressed state on button down; ClickedInside middle state is
                // reflected visually via ButtonState in paint. Toggle happens on release.
                ctx.invalidate();
                true
            }
            Event::MouseUp {
                position,
                button: MouseButton::Left,
            } if ctx.bounds.contains(*position) => {
                self.checked = !self.checked;
                ctx.invalidate();
                if let Some(ref mut f) = self.on_change {
                    f(ctx);
                }
                true
            }
            _ => false,
        }
    }

    fn focusable(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
