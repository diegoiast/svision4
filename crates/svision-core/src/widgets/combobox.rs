use crate::{
    Color, Event, FontSpec, Key, MouseButton, Point, Rect, Size, SizePolicy, SizePolicy2D,
    impl_widget_any,
    widget::{EventCtx, MeasureCtx, PaintCtx, Widget},
};

const HEADER_HEIGHT: f32 = 36.0;
const ITEM_HEIGHT: f32 = 32.0;

type ChangeHandler = Box<dyn FnMut(usize, &mut EventCtx)>;

/// A simple string combo box.
#[derive(Default)]
pub struct ComboBox {
    items: Vec<String>,
    selected: Option<usize>,
    placeholder: Option<String>,
    open: bool,
    hovered: Option<usize>,
    min_width: f32,
    on_change: Option<ChangeHandler>,
}

impl ComboBox {
    pub fn new<I, S>(items: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            items: items
                .into_iter()
                .map(|item| item.as_ref().to_string())
                .collect(),
            selected: None,
            placeholder: None,
            open: false,
            hovered: None,
            min_width: 180.0,
            on_change: None,
        }
    }

    pub fn items<I, S>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.items = items
            .into_iter()
            .map(|item| item.as_ref().to_string())
            .collect();
        if self.selected.is_some_and(|index| index >= self.items.len()) {
            self.selected = None;
        }
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        if index < self.items.len() {
            self.selected = Some(index);
        }
        self
    }

    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }

    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: FnMut(usize, &mut EventCtx) + 'static,
    {
        self.on_change = Some(Box::new(callback));
        self
    }

    pub fn set_on_change<F>(&mut self, callback: F)
    where
        F: FnMut(usize, &mut EventCtx) + 'static,
    {
        self.on_change = Some(Box::new(callback));
    }

    pub fn on_select<F>(self, callback: F) -> Self
    where
        F: FnMut(usize, &mut EventCtx) + 'static,
    {
        self.on_change(callback)
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    pub fn selected_text(&self) -> Option<&str> {
        self.selected
            .and_then(|index| self.items.get(index).map(String::as_str))
    }

    pub fn items_list(&self) -> &[String] {
        &self.items
    }

    pub fn items_ref(&self) -> &[String] {
        &self.items
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    pub fn placeholder_text(&self) -> Option<&str> {
        self.placeholder.as_deref()
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn set_open(&mut self, open: bool) {
        self.open = open;
        if open {
            self.hovered = self.selected;
        } else {
            self.hovered = None;
        }
    }

    pub fn set_selected(&mut self, index: Option<usize>) {
        if index.is_some_and(|value| value >= self.items.len()) {
            return;
        }
        self.selected = index;
    }

    fn header_rect(bounds: Rect) -> Rect {
        Rect::new(
            bounds.x,
            bounds.y,
            bounds.width,
            HEADER_HEIGHT.min(bounds.height),
        )
    }

    fn item_rect(bounds: Rect, index: usize) -> Rect {
        Rect::new(
            bounds.x + 1.0,
            bounds.y + HEADER_HEIGHT + index as f32 * ITEM_HEIGHT,
            (bounds.width - 2.0).max(0.0),
            ITEM_HEIGHT,
        )
    }

    fn item_index_at(bounds: Rect, position: Point) -> Option<usize> {
        if position.y < bounds.y + HEADER_HEIGHT {
            return None;
        }
        let index = ((position.y - (bounds.y + HEADER_HEIGHT)) / ITEM_HEIGHT).floor() as usize;
        Some(index)
    }

    fn select(&mut self, index: usize, ctx: &mut EventCtx) {
        if index >= self.items.len() {
            return;
        }
        self.selected = Some(index);
        self.open = false;
        self.hovered = None;
        if let Some(callback) = self.on_change.as_mut() {
            callback(index, ctx);
        }
        ctx.invalidate();
    }

    fn notify_change(&mut self, ctx: &mut EventCtx) {
        if let Some(index) = self.selected {
            if let Some(callback) = self.on_change.as_mut() {
                callback(index, ctx);
            }
        }
    }

    fn toggle_open(&mut self) {
        self.open = !self.open;
        if self.open {
            self.hovered = self.selected;
        } else {
            self.hovered = None;
        }
    }

    fn move_hover(&mut self, delta: i32) {
        if self.items.is_empty() {
            return;
        }
        let current = self.hovered.or(self.selected).unwrap_or(0) as i32;
        let next = (current + delta).clamp(0, self.items.len() as i32 - 1) as usize;
        self.hovered = Some(next);
    }

    fn move_selected(&mut self, delta: i32) {
        if self.items.is_empty() {
            return;
        }
        let current = self.selected.unwrap_or(0) as i32;
        let next = (current + delta).clamp(0, self.items.len() as i32 - 1) as usize;
        self.selected = Some(next);
    }
}

impl Widget for ComboBox {
    fn size_policy(&self) -> SizePolicy2D {
        SizePolicy2D::new(SizePolicy::Expanding, SizePolicy::Minimum)
    }

    fn measure(&mut self, ctx: &mut MeasureCtx, available: Size) -> Size {
        let theme = ctx.theme();
        let font = FontSpec::new(theme.font_size());
        let text = self
            .selected_text()
            .or(self.placeholder.as_deref())
            .unwrap_or("Select...");
        let text_size = ctx.measure_text(text, &font);
        let width = (text_size.width + 12.0 + 12.0 + 20.0).max(self.min_width);
        let width = if available.width.is_finite() {
            width.min(available.width).max(0.0)
        } else {
            width
        };
        let height = if self.open {
            HEADER_HEIGHT + self.items.len() as f32 * ITEM_HEIGHT
        } else {
            HEADER_HEIGHT
        };
        Size::new(width, height)
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let theme = ctx.theme;
        let bounds = ctx.bounds;
        let header = Self::header_rect(bounds);

        let background = if self.open {
            theme.surface()
        } else {
            theme.accent()
        };
        ctx.rasterizer
            .fill_round_rect(header, theme.radius(), background);
        if !self.open {
            ctx.rasterizer
                .stroke_round_rect(header, theme.radius(), theme.accent(), 1.0);
        }

        let text = self
            .selected_text()
            .or(self.placeholder.as_deref())
            .unwrap_or("Select...");
        let color = if self.open {
            theme.text()
        } else {
            theme.accent_text()
        };
        let font = FontSpec::new(theme.font_size());
        let x = header.x + 12.0;
        let y = header.y + (header.height - theme.font_size() * 1.25) * 0.5;
        ctx.rasterizer
            .draw_text(text, Point::new(x, y), &font, color);

        let chevron_x = header.right() - 16.0;
        let chevron_y = header.y + header.height * 0.5;
        ctx.rasterizer.draw_line(
            Point::new(chevron_x - 5.0, chevron_y - 2.0),
            Point::new(chevron_x, chevron_y + 3.0),
            color,
            1.5,
        );
        ctx.rasterizer.draw_line(
            Point::new(chevron_x, chevron_y + 3.0),
            Point::new(chevron_x + 5.0, chevron_y - 2.0),
            color,
            1.5,
        );

        if self.open {
            let list = Rect::new(
                bounds.x,
                header.bottom(),
                bounds.width,
                (bounds.height - HEADER_HEIGHT).max(0.0),
            );
            ctx.rasterizer.fill_rect(list, Color::hex(0xffffff));

            for (index, item) in self.items.iter().enumerate() {
                let item_rect = Self::item_rect(bounds, index);
                let is_selected = self.selected == Some(index);
                let is_hovered = self.hovered == Some(index);
                let background = if is_selected {
                    theme.accent()
                } else if is_hovered {
                    theme.surface_hover()
                } else {
                    theme.surface()
                };
                ctx.rasterizer.fill_rect(item_rect, background);

                let text_color = if is_selected {
                    theme.accent_text()
                } else {
                    theme.text()
                };
                let text_y = item_rect.y + (item_rect.height - theme.font_size() * 1.25) * 0.5;
                ctx.rasterizer.draw_text(
                    item,
                    Point::new(item_rect.x + 12.0, text_y),
                    &font,
                    text_color,
                );
            }

            ctx.rasterizer
                .stroke_round_rect(bounds, theme.radius(), theme.border(), 1.0);
        }

        if ctx.state.focused && ctx.state.window_active {
            ctx.rasterizer.stroke_round_rect(
                header.inset_all(2.0),
                (theme.radius() - 2.0).max(0.0),
                theme.focus(),
                1.5,
            );
        }
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event) -> bool {
        if !ctx.state.enabled {
            return false;
        }

        match event {
            Event::CursorMoved { position } if ctx.bounds.contains(*position) => {
                if self.open {
                    let index = Self::item_index_at(ctx.bounds, *position);
                    let index = index.filter(|value| *value < self.items.len());
                    if index != self.hovered {
                        self.hovered = index;
                        ctx.invalidate();
                    }
                }
                true
            }
            Event::MouseDown {
                position,
                button: MouseButton::Left,
            } if ctx.bounds.contains(*position) => {
                ctx.request_focus();
                let header = Self::header_rect(ctx.bounds);
                if header.contains(*position) {
                    self.toggle_open();
                    ctx.invalidate();
                    true
                } else if self.open {
                    if let Some(index) = Self::item_index_at(ctx.bounds, *position) {
                        if index < self.items.len() {
                            self.select(index, ctx);
                        }
                    }
                    true
                } else {
                    true
                }
            }
            Event::KeyDown { key, .. } if ctx.state.focused => match key {
                Key::Enter | Key::Space => {
                    if self.open {
                        let index = self.hovered.or(self.selected).unwrap_or(0);
                        self.select(index, ctx);
                    } else {
                        self.toggle_open();
                        ctx.invalidate();
                    }
                    true
                }
                Key::Up if self.open => {
                    self.move_hover(-1);
                    ctx.invalidate();
                    true
                }
                Key::Down if self.open => {
                    self.move_hover(1);
                    ctx.invalidate();
                    true
                }
                Key::Escape if self.open => {
                    self.open = false;
                    self.hovered = None;
                    ctx.invalidate();
                    true
                }
                Key::Up => {
                    let before = self.selected;
                    self.move_selected(-1);
                    if self.selected != before {
                        self.notify_change(ctx);
                    }
                    ctx.invalidate();
                    true
                }
                Key::Down => {
                    let before = self.selected;
                    self.move_selected(1);
                    if self.selected != before {
                        self.notify_change(ctx);
                    }
                    ctx.invalidate();
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    impl_widget_any!();
}

impl std::fmt::Debug for ComboBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComboBox")
            .field("items", &self.items.len())
            .field("selected", &self.selected)
            .field("open", &self.open)
            .finish()
    }
}
