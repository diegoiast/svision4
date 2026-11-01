use crate::{
    impl_widget_any,
    widget::{EventCtx, MeasureCtx, PaintCtx, Widget},
    ButtonState, Event, FontSpec, Insets, Key, MouseButton, Point, Size, SizePolicy, SizePolicy2D,
};
use std::time::{Duration, Instant};

type ClickHandler = Box<dyn FnMut(&mut EventCtx)>;

/// A push button with a stored click callback.
pub struct Button {
    text: String,
    font_size: Option<f32>,
    bold: bool,
    min_width: f32,
    padding: Insets,
    primary: bool,
    toggle: bool,
    checked: bool,
    auto_repeat: bool,
    auto_repeat_delay: f32,
    auto_repeat_interval: f32,
    auto_repeat_pressed_at: Option<Instant>,
    auto_repeat_last_fired: Option<Instant>,
    on_click: Option<ClickHandler>,
    measured_text: Size,
}

impl Button {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            font_size: None,
            bold: false,
            min_width: 0.0,
            padding: Insets::symmetric(18.0, 10.0),
            primary: false,
            toggle: false,
            checked: false,
            auto_repeat: false,
            auto_repeat_delay: 0.5,
            auto_repeat_interval: 0.4,
            auto_repeat_pressed_at: None,
            auto_repeat_last_fired: None,
            on_click: None,
            measured_text: Size::ZERO,
        }
    }

    pub fn primary(mut self) -> Self {
        self.primary = true;
        self
    }

    pub fn toggle(mut self, toggle: bool) -> Self {
        self.toggle = toggle;
        self
    }

    pub fn auto_repeat(mut self, enabled: bool) -> Self {
        self.auto_repeat = enabled;
        self
    }

    pub fn auto_repeat_delay(mut self, delay: f32) -> Self {
        self.auto_repeat_delay = delay;
        self
    }

    pub fn auto_repeat_interval(mut self, interval: f32) -> Self {
        self.auto_repeat_interval = interval;
        self
    }

    pub fn set_auto_repeat(&mut self, enabled: bool) {
        self.auto_repeat = enabled;
    }

    pub fn set_auto_repeat_delay(&mut self, delay: f32) {
        self.auto_repeat_delay = delay;
    }

    pub fn set_auto_repeat_interval(&mut self, interval: f32) {
        self.auto_repeat_interval = interval;
    }

    pub fn set_auto_repeat_config(&mut self, enabled: bool, delay: f32, interval: f32) {
        self.auto_repeat = enabled;
        self.auto_repeat_delay = delay;
        self.auto_repeat_interval = interval;
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = Some(size);
        self
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = Insets::all(padding);
        self
    }

    /// Stores a callback. The callback may capture widget handles and mutate
    /// other widgets; it must not construct a new event dispatch.
    pub fn on_click<F>(mut self, callback: F) -> Self
    where
        F: FnMut(&mut EventCtx) + 'static,
    {
        self.on_click = Some(Box::new(callback));
        self
    }

    pub fn set_on_click<F>(&mut self, callback: F)
    where
        F: FnMut(&mut EventCtx) + 'static,
    {
        self.on_click = Some(Box::new(callback));
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    pub fn set_toggle(&mut self, toggle: bool) {
        self.toggle = toggle;
    }

    pub fn is_toggle(&self) -> bool {
        self.toggle
    }

    pub fn set_checked(&mut self, checked: bool) -> &mut Self {
        self.checked = checked;
        self
    }

    pub fn is_checked(&self) -> bool {
        self.checked
    }

    fn font(&self, default_size: f32) -> FontSpec {
        FontSpec::new(self.font_size.unwrap_or(default_size)).bold(self.bold)
    }

    fn fire(&mut self, ctx: &mut EventCtx) {
        if self.toggle {
            self.checked = !self.checked;
        }
        if let Some(callback) = self.on_click.as_mut() {
            callback(ctx);
        }
    }

    pub fn auto_repeat_tick(&mut self, ctx: &mut EventCtx, now: Instant) -> Option<Instant> {
        if !self.auto_repeat {
            return None;
        }
        let pressed_at = self.auto_repeat_pressed_at?;
        if ctx.state.interaction != ButtonState::ClickedInside {
            self.auto_repeat_pressed_at = None;
            self.auto_repeat_last_fired = None;
            return None;
        }

        let delay = Duration::from_secs_f32(self.auto_repeat_delay.max(0.0));
        let interval = Duration::from_secs_f32(self.auto_repeat_interval.max(0.001));
        let first_due = pressed_at + delay;
        if now < first_due {
            return Some(first_due);
        }

        let last_fired = self.auto_repeat_last_fired.unwrap_or(pressed_at);
        let next_due = last_fired + interval;
        if now >= next_due {
            self.fire(ctx);
            ctx.invalidate();
            self.auto_repeat_last_fired = Some(now);
            Some(now + interval)
        } else {
            Some(next_due)
        }
    }

    pub fn stop_auto_repeat(&mut self) {
        self.auto_repeat_pressed_at = None;
        self.auto_repeat_last_fired = None;
    }
}

impl Widget for Button {
    fn size_policy(&self) -> SizePolicy2D {
        SizePolicy2D::new(SizePolicy::Fixed, SizePolicy::Fixed)
    }

    fn measure(&mut self, ctx: &mut MeasureCtx, _available: Size) -> Size {
        let theme = ctx.theme();
        let font = self.font(theme.font_size());
        let text = ctx.measure_text(&self.text, &font);
        self.measured_text = text;
        Size::new(
            (text.width + self.padding.horizontal()).max(self.min_width),
            (text.height + self.padding.vertical()).max(36.0),
        )
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let theme = ctx.theme;
        let enabled = ctx.state.enabled;
        let checked = if self.toggle { self.checked } else { false };

        let background = if !enabled {
            theme.surface()
        } else if checked {
            if ctx.state.is_pressed() {
                theme.accent_pressed()
            } else if self.primary && ctx.state.is_hovered() {
                theme.accent()
            } else if ctx.state.is_hovered() {
                theme.accent_hover()
            } else {
                theme.accent()
            }
        } else if self.primary && ctx.state.is_pressed() {
            theme.accent_pressed()
        } else if ctx.state.is_pressed() {
            theme.surface_pressed()
        } else if self.primary && ctx.state.is_hovered() {
            theme.accent()
        } else if ctx.state.is_hovered() {
            theme.accent_hover()
        } else if self.primary {
            theme.accent()
        } else {
            theme.surface()
        };

        ctx.rasterizer
            .fill_round_rect(ctx.bounds, theme.radius(), background);

        if !self.primary {
            ctx.rasterizer
                .stroke_round_rect(ctx.bounds, theme.radius(), theme.border(), 1.0);
        }

        if ctx.state.focused && ctx.state.window_active {
            ctx.rasterizer.stroke_round_rect(
                ctx.bounds.inset_all(2.0),
                (theme.radius() - 2.0).max(0.0),
                if self.primary {
                    theme.focused()
                } else {
                    theme.focus()
                },
                1.5,
            );
        }

        let text_color = if !enabled {
            theme.text_muted()
        } else if checked || self.primary {
            theme.accent_text()
        } else {
            theme.text()
        };
        let font = self.font(theme.font_size());
        let x = ctx.bounds.x + (ctx.bounds.width - self.measured_text.width) * 0.5;
        let y = ctx.bounds.y + (ctx.bounds.height - self.measured_text.height) * 0.5;
        ctx.rasterizer
            .draw_text(&self.text, Point::new(x, y), &font, text_color);
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
                if self.auto_repeat {
                    self.auto_repeat_pressed_at = Some(Instant::now());
                    self.auto_repeat_last_fired = Some(Instant::now());
                    self.fire(ctx);
                    ctx.invalidate();
                }
                true
            }
            Event::MouseUp {
                position,
                button: MouseButton::Left,
            } if ctx.state.interaction == ButtonState::ClickedInside => {
                if self.auto_repeat {
                    self.stop_auto_repeat();
                } else {
                    self.fire(ctx);
                    ctx.invalidate();
                }
                true
            }
            Event::KeyDown {
                key: Key::Enter | Key::Space,
                ..
            } if ctx.state.focused => {
                self.fire(ctx);
                ctx.invalidate();
                true
            }
            _ => false,
        }
    }

    impl_widget_any!();
}

impl std::fmt::Debug for Button {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Button")
            .field("text", &self.text)
            .field("primary", &self.primary)
            .field("has_click_handler", &self.on_click.is_some())
            .finish()
    }
}
