use crate::{
    Event, FontSpec, FrameKind, Key, MouseButton, Point, Size, SizePolicy, SizePolicy2D,
    impl_widget_any,
    widget::{EventCtx, MeasureCtx, PaintCtx, Widget},
};

/// A single-line text input field.
///
/// The input accepts text input, moves the cursor with the arrow keys, and
/// supports backspace/delete. It requests focus on click and draws a caret
/// when focused.
#[derive(Clone, Debug)]
pub struct Input {
    text: String,
    cursor: usize,
    font_size: Option<f32>,
    readonly: bool,
    measured: Size,
}

impl Input {
    /// Creates an empty input.
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            font_size: None,
            readonly: false,
            measured: Size::ZERO,
        }
    }

    /// Creates an input pre-filled with `text`.
    pub fn with_text(text: impl Into<String>) -> Self {
        let text = text.into();
        let cursor = text.len();
        Self {
            text,
            cursor,
            font_size: None,
            readonly: false,
            measured: Size::ZERO,
        }
    }

    /// Returns the current text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Replaces the current text and moves the cursor to the end.
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.cursor = self.text.len();
    }

    /// Returns the cursor position (byte index into `text`).
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Sets whether the input is read-only.
    pub fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }

    /// Returns whether the input is read-only.
    pub fn is_readonly(&self) -> bool {
        self.readonly
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = Some(size);
        self
    }

    fn font(&self, default_size: f32) -> FontSpec {
        FontSpec::new(self.font_size.unwrap_or(default_size))
    }

    /// Byte index of the previous character boundary from the cursor.
    fn prev_char_boundary(&self) -> usize {
        self.text[..self.cursor]
            .char_indices()
            .rev()
            .next()
            .map(|(byte, _)| byte)
            .unwrap_or(0)
    }

    /// Byte index of the start of the word before the cursor.
    fn prev_word_start(&self) -> usize {
        let mut i = self.cursor;
        while i > 0 && self.text[i - 1..].chars().next().unwrap().is_whitespace() {
            i -= 1;
        }
        while i > 0 && !self.text[i - 1..].chars().next().unwrap().is_whitespace() {
            i -= 1;
        }
        i
    }

    /// Byte index of the end of the word after the cursor.
    fn next_word_end(&self) -> usize {
        let mut i = self.cursor;
        while i < self.text.len() && self.text[i..].chars().next().unwrap().is_whitespace() {
            i += 1;
        }
        while i < self.text.len() && !self.text[i..].chars().next().unwrap().is_whitespace() {
            i += 1;
        }
        i
    }

    /// Byte index of the next character boundary after the cursor.
    fn next_char_boundary(&self) -> usize {
        self.text[self.cursor..]
            .char_indices()
            .next()
            .map(|(byte, _)| self.cursor + byte)
            .unwrap_or(self.text.len())
    }

    /// Byte index of the character closest to `x` (relative to the text start).
    fn cursor_at_position(&self, font: &FontSpec, ui: &crate::Ui, x: f32) -> usize {
        let mut best = 0usize;
        let mut best_dist = f32::INFINITY;
        for (byte, _) in self.text.char_indices() {
            let w = ui.measure_text(&self.text[..byte], font).width;
            let dist = (w - x).abs();
            if dist < best_dist {
                best_dist = dist;
                best = byte;
            }
        }
        let w = ui.measure_text(&self.text, font).width;
        if (w - x).abs() < best_dist {
            best = self.text.len();
        }
        best
    }
}

impl Widget for Input {
    fn size_policy(&self) -> SizePolicy2D {
        SizePolicy2D::new(SizePolicy::Expanding, SizePolicy::Fixed)
    }

    fn measure(&mut self, ctx: &mut MeasureCtx, _available: Size) -> Size {
        let theme = ctx.theme();
        let font = self.font(theme.font_size());
        let size = ctx.measure_text(&self.text, &font);
        self.measured = size;
        Size::new((size.width + 8.0).max(120.0), (size.height + 8.0).max(36.0))
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let theme = ctx.theme;
        let enabled = ctx.state.enabled;

        let background = if enabled {
            theme.surface()
        } else {
            theme.surface()
        };

        ctx.rasterizer
            .fill_round_rect(ctx.bounds, theme.radius(), background);
        theme.frame(&mut *ctx.rasterizer, ctx.bounds, FrameKind::Normal);

        let inner = ctx.bounds.inset_all(1.0);
        if ctx.state.focused && ctx.state.window_active {
            ctx.rasterizer
                .stroke_round_rect(inner, theme.radius(), theme.focused(), 1.0);
        }

        let font = self.font(theme.font_size());
        let text_color = if enabled {
            theme.text()
        } else {
            theme.text_muted()
        };

        let size = ctx.ui.measure_text(&self.text, &font);
        let text_x = inner.x + 4.0;
        let text_y = inner.y + (inner.height - size.height) * 0.5;

        ctx.rasterizer.push_clip(inner);
        ctx.rasterizer
            .draw_text(&self.text, Point::new(text_x, text_y), &font, text_color);
        ctx.rasterizer.pop_clip();

        if ctx.state.focused && ctx.state.window_active {
            let before = ctx.ui.measure_text(&self.text[..self.cursor], &font);
            let cx = text_x + before.width;
            ctx.rasterizer.draw_line(
                Point::new(cx, inner.y + 4.0),
                Point::new(cx, inner.bottom() - 4.0),
                text_color,
                1.0,
            );
        }
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event) -> bool {
        if !ctx.state.enabled {
            return false;
        }

        match event {
            Event::MouseDown {
                position,
                button: MouseButton::Left,
            } if ctx.bounds.contains(*position) => {
                ctx.request_focus();
                let font = self.font(ctx.ui.theme().font_size());
                let rel_x = position.x - (ctx.bounds.x + 4.0);
                self.cursor = self.cursor_at_position(&font, &ctx.ui, rel_x);
                self.cursor = self.cursor.min(self.text.len());
                ctx.invalidate();
                true
            }
            Event::TextInput(s) if !self.readonly => {
                self.text.insert_str(self.cursor, s);
                self.cursor += s.len();
                ctx.invalidate();
                true
            }
            Event::KeyDown { key, modifiers } if !self.readonly => {
                let handled = match key {
                    Key::Backspace => {
                        if self.cursor > 0 {
                            let prev = self.prev_char_boundary();
                            self.text.replace_range(prev..self.cursor, "");
                            self.cursor = prev;
                        }
                        true
                    }
                    Key::Delete => {
                        if self.cursor < self.text.len() {
                            let next = self.next_char_boundary();
                            self.text.replace_range(self.cursor..next, "");
                        }
                        true
                    }
                    Key::Left => {
                        self.cursor = if modifiers.ctrl {
                            self.prev_word_start()
                        } else {
                            self.prev_char_boundary()
                        };
                        true
                    }
                    Key::Right => {
                        self.cursor = if modifiers.ctrl {
                            self.next_word_end()
                        } else {
                            self.next_char_boundary()
                        };
                        true
                    }
                    Key::Home => {
                        self.cursor = 0;
                        true
                    }
                    Key::End => {
                        self.cursor = self.text.len();
                        true
                    }
                    Key::Character(c) => {
                        self.text.insert(self.cursor, *c);
                        self.cursor += c.len_utf8();
                        true
                    }
                    _ => false,
                };
                if handled {
                    ctx.invalidate();
                }
                handled
            }
            Event::FocusGained => {
                ctx.invalidate();
                true
            }
            Event::FocusLost => {
                ctx.invalidate();
                true
            }
            _ => false,
        }
    }

    fn focusable(&self) -> bool {
        true
    }

    impl_widget_any!();
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}
