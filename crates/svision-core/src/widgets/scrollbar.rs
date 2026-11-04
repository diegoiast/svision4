use crate::{
    Event, Key, MouseButton, Point, Rect, Size, SizePolicy, SizePolicy2D, impl_widget_any,
    widget::{EventCtx, MeasureCtx, PaintCtx, Widget},
};

/// Scroll bar orientation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Orientation {
    #[default]
    Horizontal,
    Vertical,
}

const DEFAULT_LENGTH: f32 = 100.0;
const DEFAULT_THICKNESS: f32 = 16.0;
const MIN_THUMB_SIZE: f32 = 20.0;

type ChangeHandler = Box<dyn FnMut(f32)>;

pub struct ScrollBar {
    orientation: Orientation,
    value: f32,
    min: f32,
    max: f32,
    step_small: f32,
    step_page: f32,
    length: f32,
    thickness: f32,
    dragging: bool,
    drag_start_mouse: f32,
    drag_start_value: f32,
    hovered_start: bool,
    pressed_start: bool,
    hovered_end: bool,
    pressed_end: bool,
    hovered_thumb: bool,
    auto_repeat_target: Option<i8>,
    auto_repeat_started: Option<std::time::Instant>,
    auto_repeat_last_fired: Option<std::time::Instant>,
    auto_repeat_delay: f32,
    auto_repeat_interval: f32,
    on_change: Option<ChangeHandler>,
}

impl ScrollBar {
    pub fn new(orientation: Orientation) -> Self {
        Self {
            orientation,
            value: 0.0,
            min: 0.0,
            max: 100.0,
            step_small: 1.0,
            step_page: 10.0,
            length: DEFAULT_LENGTH,
            thickness: DEFAULT_THICKNESS,
            dragging: false,
            drag_start_mouse: 0.0,
            drag_start_value: 0.0,
            hovered_start: false,
            pressed_start: false,
            hovered_end: false,
            pressed_end: false,
            hovered_thumb: false,
            auto_repeat_target: None,
            auto_repeat_started: None,
            auto_repeat_last_fired: None,
            auto_repeat_delay: 0.5,
            auto_repeat_interval: 0.1,
            on_change: None,
        }
    }

    pub fn horizontal() -> Self {
        Self::new(Orientation::Horizontal)
    }

    pub fn vertical() -> Self {
        Self::new(Orientation::Vertical)
    }

    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn value(mut self, value: f32) -> Self {
        self.value = value.clamp(self.min, self.max);
        self
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self.value = self.value.clamp(self.min, self.max);
        self
    }

    pub fn min(mut self, min: f32) -> Self {
        self.min = min;
        self.value = self.value.clamp(self.min, self.max);
        self
    }

    pub fn max(mut self, max: f32) -> Self {
        self.max = max;
        self.value = self.value.clamp(self.min, self.max);
        self
    }

    pub fn step(mut self, step: f32) -> Self {
        self.step_small = step;
        self
    }

    pub fn step_small(mut self, step: f32) -> Self {
        self.step_small = step;
        self
    }

    pub fn step_page(mut self, step: f32) -> Self {
        self.step_page = step;
        self
    }

    pub fn page_step(mut self, step: f32) -> Self {
        self.step_page = step;
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.length = size;
        self
    }

    pub fn length(mut self, length: f32) -> Self {
        self.length = length;
        self
    }

    pub fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: FnMut(f32) + 'static,
    {
        self.on_change = Some(Box::new(callback));
        self
    }

    pub fn value_ref(&self) -> f32 {
        self.value
    }

    pub fn min_ref(&self) -> f32 {
        self.min
    }

    pub fn max_ref(&self) -> f32 {
        self.max
    }

    pub fn auto_repeat_tick(
        &mut self,
        ctx: &mut EventCtx,
        now: std::time::Instant,
    ) -> Option<std::time::Instant> {
        let target = self.auto_repeat_target?;
        if !(self.pressed_start || self.pressed_end) {
            self.auto_repeat_target = None;
            self.auto_repeat_started = None;
            self.auto_repeat_last_fired = None;
            return None;
        }
        let started = self.auto_repeat_started?;
        let delay = std::time::Duration::from_secs_f32(self.auto_repeat_delay.max(0.0));
        let interval = std::time::Duration::from_secs_f32(self.auto_repeat_interval.max(0.001));
        let first_due = started + delay;
        if now < first_due {
            return Some(first_due);
        }
        let last_fired = self.auto_repeat_last_fired.unwrap_or(started);
        let next_due = last_fired + interval;
        if now >= next_due {
            if target < 0 {
                self.set_value(self.value - self.step_small);
            } else {
                self.set_value(self.value + self.step_small);
            }
            ctx.invalidate();
            self.auto_repeat_last_fired = Some(now);
            Some(now + interval)
        } else {
            Some(next_due)
        }
    }

    pub fn is_dragging(&self) -> bool {
        self.dragging
    }

    pub fn set_value(&mut self, value: f32) {
        let value = value.clamp(self.min, self.max);
        if (value - self.value).abs() > f32::EPSILON {
            self.value = value;
            if let Some(callback) = self.on_change.as_mut() {
                callback(self.value);
            }
        }
    }

    pub fn set_range(&mut self, min: f32, max: f32) {
        self.min = min;
        self.max = max;
        self.set_value(self.value);
    }

    pub fn normalized_value(&self) -> f32 {
        let range = self.max - self.min;
        if range <= 0.0 {
            0.0
        } else {
            (self.value - self.min) / range
        }
    }

    fn is_horizontal(&self) -> bool {
        matches!(self.orientation, Orientation::Horizontal)
    }

    fn button_size(&self) -> f32 {
        self.thickness.min(16.0)
    }

    pub fn start_button_rect(&self, bounds: Rect) -> Rect {
        let size = self.button_size();
        if self.is_horizontal() {
            Rect::new(
                bounds.x,
                bounds.y + (bounds.height - size) * 0.5,
                size,
                size,
            )
        } else {
            Rect::new(bounds.x + (bounds.width - size) * 0.5, bounds.y, size, size)
        }
    }

    pub fn end_button_rect(&self, bounds: Rect) -> Rect {
        let size = self.button_size();
        if self.is_horizontal() {
            Rect::new(
                bounds.right() - size,
                bounds.y + (bounds.height - size) * 0.5,
                size,
                size,
            )
        } else {
            Rect::new(
                bounds.x + (bounds.width - size) * 0.5,
                bounds.bottom() - size,
                size,
                size,
            )
        }
    }

    pub fn track_rect(&self, bounds: Rect) -> Rect {
        let size = self.button_size();
        if self.is_horizontal() {
            Rect::new(
                bounds.x + size,
                bounds.y + (bounds.height - self.thickness) * 0.5,
                (bounds.width - size * 2.0).max(0.0),
                self.thickness,
            )
        } else {
            Rect::new(
                bounds.x + (bounds.width - self.thickness) * 0.5,
                bounds.y + size,
                self.thickness,
                (bounds.height - size * 2.0).max(0.0),
            )
        }
    }

    pub fn thumb_rect(&self, bounds: Rect) -> Rect {
        let track = self.track_rect(bounds);
        let normalized = self.normalized_value();
        if self.is_horizontal() {
            let thumb_width = (track.width * 0.2)
                .max(MIN_THUMB_SIZE)
                .min(track.width.max(MIN_THUMB_SIZE));
            let travel = (track.width - thumb_width).max(0.0);
            let x = track.x + normalized * travel;
            Rect::new(x, track.y, thumb_width, track.height)
        } else {
            let thumb_height = (track.height * 0.2)
                .max(MIN_THUMB_SIZE)
                .min(track.height.max(MIN_THUMB_SIZE));
            let travel = (track.height - thumb_height).max(0.0);
            let y = track.y + normalized * travel;
            Rect::new(track.x, y, track.width, thumb_height)
        }
    }

    fn set_from_mouse(&mut self, bounds: Rect, mouse: f32) {
        let track = self.track_rect(bounds);
        let thumb = self.thumb_rect(bounds);
        if self.is_horizontal() {
            let travel = (track.width - thumb.width).max(1.0);
            let relative = (mouse - track.x - thumb.width * 0.5).clamp(0.0, travel);
            self.set_value(self.min + (relative / travel) * (self.max - self.min));
        } else {
            let travel = (track.height - thumb.height).max(1.0);
            let relative = (mouse - track.y - thumb.height * 0.5).clamp(0.0, travel);
            self.set_value(self.min + (relative / travel) * (self.max - self.min));
        }
    }
}

impl Widget for ScrollBar {
    fn size_policy(&self) -> SizePolicy2D {
        if self.is_horizontal() {
            SizePolicy2D::new(SizePolicy::Expanding, SizePolicy::Fixed)
        } else {
            SizePolicy2D::new(SizePolicy::Fixed, SizePolicy::Expanding)
        }
    }

    fn measure(&mut self, _ctx: &mut MeasureCtx, available: Size) -> Size {
        if self.is_horizontal() {
            let width = if available.width.is_finite() && available.width > 0.0 {
                available.width
            } else {
                self.length
            };
            Size::new(width, self.thickness)
        } else {
            let height = if available.height.is_finite() && available.height > 0.0 {
                available.height
            } else {
                self.length
            };
            Size::new(self.thickness, height)
        }
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let theme = ctx.theme;
        let bounds = ctx.bounds;

        ctx.rasterizer
            .fill_round_rect(bounds, theme.radius(), theme.surface());

        let track = self.track_rect(bounds);
        ctx.rasterizer
            .fill_round_rect(track, theme.radius(), theme.surface_pressed());

        let thumb = self.thumb_rect(bounds);
        ctx.rasterizer
            .fill_round_rect(thumb, theme.radius(), theme.accent());

        let start = self.start_button_rect(bounds);
        let end = self.end_button_rect(bounds);
        ctx.rasterizer
            .fill_round_rect(start, theme.radius(), theme.surface_hover());
        ctx.rasterizer
            .fill_round_rect(end, theme.radius(), theme.surface_hover());

        let arrow_color = theme.text();
        if self.is_horizontal() {
            let center_y = start.y + start.height * 0.5;
            ctx.rasterizer.draw_line(
                Point::new(start.x + start.width * 0.65, center_y),
                Point::new(start.x + start.width * 0.35, center_y - 3.0),
                arrow_color,
                1.5,
            );
            ctx.rasterizer.draw_line(
                Point::new(start.x + start.width * 0.35, center_y - 3.0),
                Point::new(start.x + start.width * 0.35, center_y + 3.0),
                arrow_color,
                1.5,
            );
            let center_y = end.y + end.height * 0.5;
            ctx.rasterizer.draw_line(
                Point::new(end.x + end.width * 0.35, center_y),
                Point::new(end.x + end.width * 0.65, center_y - 3.0),
                arrow_color,
                1.5,
            );
            ctx.rasterizer.draw_line(
                Point::new(end.x + end.width * 0.65, center_y - 3.0),
                Point::new(end.x + end.width * 0.65, center_y + 3.0),
                arrow_color,
                1.5,
            );
        } else {
            let center_x = start.x + start.width * 0.5;
            ctx.rasterizer.draw_line(
                Point::new(center_x, start.y + start.height * 0.35),
                Point::new(center_x - 3.0, start.y + start.height * 0.65),
                arrow_color,
                1.5,
            );
            ctx.rasterizer.draw_line(
                Point::new(center_x - 3.0, start.y + start.height * 0.65),
                Point::new(center_x + 3.0, start.y + start.height * 0.65),
                arrow_color,
                1.5,
            );
            let center_x = end.x + end.width * 0.5;
            ctx.rasterizer.draw_line(
                Point::new(center_x, end.y + end.height * 0.65),
                Point::new(center_x - 3.0, end.y + end.height * 0.35),
                arrow_color,
                1.5,
            );
            ctx.rasterizer.draw_line(
                Point::new(center_x - 3.0, end.y + end.height * 0.35),
                Point::new(center_x + 3.0, end.y + end.height * 0.35),
                arrow_color,
                1.5,
            );
        }

        if ctx.state.focused && ctx.state.window_active {
            ctx.rasterizer.stroke_round_rect(
                bounds.inset_all(1.0),
                (theme.radius() - 1.0).max(0.0),
                theme.focus(),
                1.0,
            );
        }
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event) -> bool {
        match event {
            Event::CursorMoved { position } if ctx.bounds.contains(*position) => {
                if !self.dragging {
                    self.hovered_start = self.start_button_rect(ctx.bounds).contains(*position);
                    self.hovered_end = self.end_button_rect(ctx.bounds).contains(*position);
                    self.hovered_thumb = self.thumb_rect(ctx.bounds).contains(*position);
                }
                if self.dragging {
                    let mouse = if self.is_horizontal() {
                        position.x
                    } else {
                        position.y
                    };
                    self.set_from_mouse(ctx.bounds, mouse);
                    ctx.invalidate();
                }
                true
            }
            Event::MouseDown {
                position,
                button: MouseButton::Left,
            } if ctx.bounds.contains(*position) => {
                ctx.request_focus();
                let start = self.start_button_rect(ctx.bounds);
                let end = self.end_button_rect(ctx.bounds);
                let thumb = self.thumb_rect(ctx.bounds);
                if start.contains(*position) {
                    self.pressed_start = true;
                    self.auto_repeat_target = Some(-1);
                    let now = std::time::Instant::now();
                    self.auto_repeat_started = Some(now);
                    self.auto_repeat_last_fired = Some(now);
                    self.set_value(self.value - self.step_small);
                    ctx.invalidate();
                    true
                } else if end.contains(*position) {
                    self.pressed_end = true;
                    self.auto_repeat_target = Some(1);
                    let now = std::time::Instant::now();
                    self.auto_repeat_started = Some(now);
                    self.auto_repeat_last_fired = Some(now);
                    self.set_value(self.value + self.step_small);
                    ctx.invalidate();
                    true
                } else if thumb.contains(*position) {
                    self.dragging = true;
                    self.drag_start_mouse = if self.is_horizontal() {
                        position.x
                    } else {
                        position.y
                    };
                    self.drag_start_value = self.value;
                    ctx.invalidate();
                    true
                } else {
                    let track = self.track_rect(ctx.bounds);
                    if self.is_horizontal() {
                        if position.x < thumb.x {
                            self.set_value(self.value - self.step_page);
                        } else {
                            self.set_value(self.value + self.step_page);
                        }
                    } else if position.y < thumb.y {
                        self.set_value(self.value - self.step_page);
                    } else {
                        self.set_value(self.value + self.step_page);
                    }
                    let _ = track;
                    ctx.invalidate();
                    true
                }
            }
            Event::MouseUp {
                position: _,
                button: MouseButton::Left,
            } if self.dragging || self.pressed_start || self.pressed_end => {
                self.dragging = false;
                self.pressed_start = false;
                self.pressed_end = false;
                self.auto_repeat_target = None;
                self.auto_repeat_started = None;
                self.auto_repeat_last_fired = None;
                self.hovered_start = false;
                self.hovered_end = false;
                self.hovered_thumb = false;
                ctx.invalidate();
                true
            }
            Event::CursorMoved { position } if self.dragging => {
                let mouse = if self.is_horizontal() {
                    position.x
                } else {
                    position.y
                };
                let delta = mouse - self.drag_start_mouse;
                let bounds = ctx.bounds;
                let track = self.track_rect(bounds);
                let thumb = self.thumb_rect(bounds);
                if self.is_horizontal() {
                    let travel = (track.width - thumb.width).max(1.0);
                    let normalized = (self.drag_start_value - self.min)
                        / (self.max - self.min).max(1.0)
                        + delta / travel;
                    self.set_value(self.min + normalized.clamp(0.0, 1.0) * (self.max - self.min));
                } else {
                    let travel = (track.height - thumb.height).max(1.0);
                    let normalized = (self.drag_start_value - self.min)
                        / (self.max - self.min).max(1.0)
                        + delta / travel;
                    self.set_value(self.min + normalized.clamp(0.0, 1.0) * (self.max - self.min));
                }
                ctx.invalidate();
                true
            }
            Event::KeyDown { key, .. } if ctx.state.focused => {
                let horizontal = self.is_horizontal();
                let prev = if horizontal { Key::Left } else { Key::Up };
                let next = if horizontal { Key::Right } else { Key::Down };
                match key {
                    key if *key == prev => {
                        self.set_value(self.value - self.step_small);
                        ctx.invalidate();
                        true
                    }
                    key if *key == next => {
                        self.set_value(self.value + self.step_small);
                        ctx.invalidate();
                        true
                    }
                    Key::Home => {
                        self.set_value(self.min);
                        ctx.invalidate();
                        true
                    }
                    Key::End => {
                        self.set_value(self.max);
                        ctx.invalidate();
                        true
                    }
                    Key::PageUp => {
                        self.set_value(self.value - self.step_page);
                        ctx.invalidate();
                        true
                    }
                    Key::PageDown => {
                        self.set_value(self.value + self.step_page);
                        ctx.invalidate();
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn focusable(&self) -> bool {
        true
    }

    impl_widget_any!();
}

impl std::fmt::Debug for ScrollBar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScrollBar")
            .field("orientation", &self.orientation)
            .field("value", &self.value)
            .field("range", &(self.min, self.max))
            .finish()
    }
}
