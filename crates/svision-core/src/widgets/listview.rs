use crate::{
    Event, FrameKind, Key, MouseButton, Point, Rect, ScrollBar, Size, SizePolicy, SizePolicy2D,
    impl_widget_any,
    widget::{ArrangeCtx, EventCtx, MeasureCtx, PaintCtx, Widget},
};
use std::cell::RefCell;
use std::time::Instant;

const ITEM_HEIGHT: f32 = 32.0;
const DEFAULT_HEIGHT: f32 = 120.0;
const SCROLLBAR_WIDTH: f32 = 16.0;

type SelectionHandler = Box<dyn FnMut(Option<usize>)>;

pub struct ListView {
    items: Vec<String>,
    filter: String,
    visible: Vec<usize>,
    selected: Option<usize>,
    hovered: Option<usize>,
    scroll_offset: f32,
    viewport_height: f32,
    min_width: f32,
    alternating: bool,
    on_selection_changed: Option<SelectionHandler>,
    scrollbar: RefCell<ScrollBar>,
}

impl ListView {
    pub fn new<I, S>(items: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut list = Self {
            items: items
                .into_iter()
                .map(|item| item.as_ref().to_string())
                .collect(),
            filter: String::new(),
            visible: Vec::new(),
            selected: None,
            hovered: None,
            scroll_offset: 0.0,
            viewport_height: DEFAULT_HEIGHT,
            min_width: 160.0,
            alternating: false,
            on_selection_changed: None,
            scrollbar: RefCell::new(ScrollBar::vertical()),
        };
        list.apply_filter();
        list
    }

    pub fn items<I, S>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.set_items(items);
        self
    }

    pub fn set_items<I, S>(&mut self, items: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.items = items
            .into_iter()
            .map(|item| item.as_ref().to_string())
            .collect();
        self.apply_filter();
        self.scroll_offset = 0.0;
    }

    /// Shows only the items containing `query`.
    ///
    /// The match is case-insensitive; an empty query shows every item.
    pub fn set_filter(&mut self, query: impl Into<String>) {
        self.filter = query.into();
        self.apply_filter();
        self.scroll_offset = 0.0;
    }

    /// The current filter text.
    pub fn filter(&self) -> &str {
        &self.filter
    }

    /// Clears the filter and shows every item again.
    pub fn clear_filter(&mut self) {
        self.set_filter(String::new());
    }

    /// Recomputes the visible items from the current filter.
    fn apply_filter(&mut self) {
        let needle = self.filter.trim().to_lowercase();
        self.visible = if needle.is_empty() {
            (0..self.items.len()).collect()
        } else {
            self.items
                .iter()
                .enumerate()
                .filter(|(_, item)| item.to_lowercase().contains(&needle))
                .map(|(index, _)| index)
                .collect()
        };
        if self.selected.is_some_and(|index| index >= self.visible.len()) {
            self.selected = None;
        }
    }

    pub fn selected(mut self, index: usize) -> Self {
        if index < self.visible.len() {
            self.selected = Some(index);
        }
        self
    }

    /// The items currently shown in the list.
    pub fn items_list(&self) -> &[String] {
        &self.items
    }

    /// The items matching the current filter, in display order.
    pub fn visible_items(&self) -> Vec<String> {
        self.visible
            .iter()
            .filter_map(|index| self.items.get(*index).cloned())
            .collect()
    }

    pub fn selected_index_option(mut self, index: Option<usize>) -> Self {
        if index.is_some_and(|value| value >= self.visible.len()) {
            return self;
        }
        self.selected = index;
        self
    }

    pub fn set_selected(&mut self, index: Option<usize>) {
        if index.is_some_and(|value| value >= self.visible.len()) {
            return;
        }
        self.selected = index;
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    pub fn alternating(mut self, enabled: bool) -> Self {
        self.alternating = enabled;
        self
    }

    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }

    pub fn on_selection_changed<F>(mut self, callback: F) -> Self
    where
        F: FnMut(Option<usize>) + 'static,
    {
        self.on_selection_changed = Some(Box::new(callback));
        self
    }

    pub fn on_select<F>(self, callback: F) -> Self
    where
        F: FnMut(Option<usize>) + 'static,
    {
        self.on_selection_changed(callback)
    }

    pub fn scroll_offset(&self) -> f32 {
        self.scroll_offset
    }

    pub fn set_scroll_offset(&mut self, offset: f32) {
        let max_offset = self.max_scroll_offset();
        self.scroll_offset = offset.clamp(0.0, max_offset);
    }

    pub fn auto_repeat_tick(&mut self, ctx: &mut EventCtx, now: Instant) -> Option<Instant> {
        if !self.is_scrollable() {
            return None;
        }
        let scrollbar_bounds = self.scrollbar_bounds(ctx.bounds);
        let mut scrollbar_ctx = EventCtx {
            ui: ctx.ui.clone(),
            id: ctx.id(),
            bounds: scrollbar_bounds,
            state: ctx.state,
        };
        let deadline = self
            .scrollbar
            .borrow_mut()
            .auto_repeat_tick(&mut scrollbar_ctx, now);
        if deadline.is_some() {
            self.scroll_offset = self.scrollbar.borrow().value_ref();
        }
        deadline
    }

    fn max_scroll_offset(&self) -> f32 {
        (self.visible.len() as f32 * ITEM_HEIGHT - self.viewport_height).max(0.0)
    }

    fn content_height(&self) -> f32 {
        self.visible.len() as f32 * ITEM_HEIGHT
    }

    fn is_scrollable(&self) -> bool {
        self.content_height() > self.viewport_height + 0.5
    }

    fn scrollbar_width(&self) -> f32 {
        if self.is_scrollable() {
            SCROLLBAR_WIDTH
        } else {
            0.0
        }
    }

    fn content_bounds(&self, bounds: Rect) -> Rect {
        Rect::new(
            bounds.x,
            bounds.y,
            (bounds.width - self.scrollbar_width()).max(0.0),
            bounds.height,
        )
    }

    fn scrollbar_bounds(&self, bounds: Rect) -> Rect {
        Rect::new(
            bounds.x + bounds.width - self.scrollbar_width(),
            bounds.y,
            self.scrollbar_width(),
            bounds.height,
        )
    }

    fn item_rect(&self, bounds: Rect, index: usize) -> Rect {
        let content = self.content_bounds(bounds);
        Rect::new(
            content.x,
            bounds.y + index as f32 * ITEM_HEIGHT - self.scroll_offset,
            content.width,
            ITEM_HEIGHT,
        )
    }

    fn item_index_at(&self, bounds: Rect, position: Point) -> Option<usize> {
        if position.y < bounds.y || position.y >= bounds.bottom() {
            return None;
        }
        let index = ((position.y - bounds.y + self.scroll_offset) / ITEM_HEIGHT).floor() as usize;
        if index < self.visible.len() {
            Some(index)
        } else {
            None
        }
    }

    fn scroll_to(&mut self, index: usize) {
        let item_top = index as f32 * ITEM_HEIGHT;
        let item_bottom = item_top + ITEM_HEIGHT;
        if item_bottom > self.scroll_offset + self.viewport_height {
            self.scroll_offset = item_bottom - self.viewport_height;
        }
        if item_top < self.scroll_offset {
            self.scroll_offset = item_top;
        }
        self.scroll_offset = self.scroll_offset.clamp(0.0, self.max_scroll_offset());
    }

    fn select(&mut self, index: usize, ctx: &mut EventCtx) {
        if index >= self.visible.len() {
            return;
        }
        self.selected = Some(index);
        if let Some(callback) = self.on_selection_changed.as_mut() {
            callback(self.selected);
        }
        ctx.invalidate();
    }

    fn sync_scrollbar(&self) {
        let mut scrollbar = self.scrollbar.borrow_mut();
        scrollbar.set_range(0.0, self.max_scroll_offset());
        scrollbar.set_value(self.scroll_offset);
    }
}

impl Widget for ListView {
    fn min_size(&mut self, ctx: &mut MeasureCtx, _available: Size) -> Size {
        let theme = ctx.theme();
        let font = crate::FontSpec::new(theme.font_size());
        let mut max_text_width = 0.0f32;
        for index in &self.visible {
            let size = ctx.measure_text(&self.items[*index], &font);
            max_text_width = max_text_width.max(size.width);
        }
        Size::new(
            (max_text_width + 24.0 + SCROLLBAR_WIDTH).max(self.min_width),
            ITEM_HEIGHT,
        )
    }

    fn arrange(&mut self, _ctx: &mut ArrangeCtx, bounds: Rect) {
        self.viewport_height = bounds.height.max(1.0);
        self.set_scroll_offset(self.scroll_offset);
        self.sync_scrollbar();
    }

    fn size_policy(&self) -> SizePolicy2D {
        SizePolicy2D::new(SizePolicy::Expanding, SizePolicy::Expanding)
    }

    fn measure(&mut self, ctx: &mut MeasureCtx, available: Size) -> Size {
        let theme = ctx.theme();
        let font = crate::FontSpec::new(theme.font_size());
        let mut max_text_width: f32 = 0.0;
        for index in &self.visible {
            let size = ctx.measure_text(&self.items[*index], &font);
            max_text_width = max_text_width.max(size.width);
        }

        let viewport_height = if available.height.is_finite() && available.height > 0.0 {
            available.height
        } else {
            DEFAULT_HEIGHT
        };
        self.viewport_height = viewport_height;

        let content_height = self.content_height();
        let scrollable = content_height > viewport_height + 0.5;
        let width = (max_text_width + 24.0 + if scrollable { SCROLLBAR_WIDTH } else { 0.0 })
            .max(self.min_width);
        let width = if available.width.is_finite() {
            width.min(available.width).max(0.0)
        } else {
            width
        };
        let height = if available.height.is_finite() {
            available.height
        } else {
            content_height.clamp(ITEM_HEIGHT, DEFAULT_HEIGHT)
        };
        Size::new(width, height)
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let theme = ctx.theme.clone();
        let bounds = ctx.bounds;
        self.sync_scrollbar();

        let is_focused = ctx.state.focused && ctx.state.window_active;

        ctx.rasterizer
            .fill_round_rect(bounds, theme.radius(), theme.surface());
        theme.draw_frame(&mut *ctx.rasterizer, bounds, FrameKind::Normal, is_focused);

        let content = self.content_bounds(bounds);
        ctx.rasterizer.push_clip(content);

        let first = (self.scroll_offset / ITEM_HEIGHT).floor() as usize;
        let last = (((self.scroll_offset + bounds.height) / ITEM_HEIGHT).ceil() as usize)
            .min(self.visible.len());
        for index in first..last {
            let item_rect = self.item_rect(bounds, index);
            if item_rect.bottom() < bounds.y || item_rect.y > bounds.bottom() {
                continue;
            }
            let is_selected = self.selected == Some(index);
            let is_hovered = self.hovered == Some(index);
            let background = if is_selected {
                theme.accent()
            } else if is_hovered {
                theme.surface_hover()
            } else if self.alternating && index % 2 == 1 {
                theme.surface_pressed()
            } else {
                theme.surface()
            };

            // Fill an inset rect so the selected item has smaller internals and
            // a visible border around it.
            if is_selected {
                let fill = item_rect.inset_all(2.0);
                ctx.rasterizer.fill_rect(fill, background);
                // Use accent_text (white on all themes) for a visible border
                // regardless of how close accent and focus colors are.
                ctx.rasterizer
                    .stroke_rect(item_rect, theme.accent_text(), 1.0);
            } else {
                ctx.rasterizer.fill_rect(item_rect, background);
            }

            let text_color = if is_selected {
                theme.accent_text()
            } else {
                theme.text()
            };
            let font = crate::FontSpec::new(theme.font_size());
            let text_y = item_rect.y + (item_rect.height - theme.font_size() * 1.25) * 0.5;
            ctx.rasterizer.draw_text(
                &self.items[self.visible[index]],
                crate::Point::new(item_rect.x + 12.0, text_y),
                &font,
                text_color,
            );
        }

        ctx.rasterizer.pop_clip();

        if self.is_scrollable() {
            let scrollbar_bounds = self.scrollbar_bounds(bounds);
            let mut scrollbar_ctx = PaintCtx {
                rasterizer: &mut *ctx.rasterizer,
                ui: ctx.ui,
                theme: ctx.theme.clone(),
                bounds: scrollbar_bounds,
                state: ctx.state,
            };
            self.scrollbar.borrow().paint(&mut scrollbar_ctx);
        }
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event) -> bool {
        if !ctx.state.enabled {
            return false;
        }

        let bounds = ctx.bounds;
        let scrollbar_bounds = self.scrollbar_bounds(bounds);

        match event {
            Event::CursorMoved { position }
                if bounds.contains(*position) || self.scrollbar.borrow().is_dragging() =>
            {
                let scrollbar_controls = {
                    let scrollbar = self.scrollbar.borrow();
                    scrollbar
                        .start_button_rect(scrollbar_bounds)
                        .contains(*position)
                        || scrollbar
                            .end_button_rect(scrollbar_bounds)
                            .contains(*position)
                        || scrollbar.thumb_rect(scrollbar_bounds).contains(*position)
                };
                if scrollbar_controls || self.scrollbar.borrow().is_dragging() {
                    let mut scrollbar_ctx = EventCtx {
                        ui: ctx.ui.clone(),
                        id: ctx.id(),
                        bounds: scrollbar_bounds,
                        state: ctx.state,
                    };
                    let mut scrollbar = self.scrollbar.borrow_mut();
                    let _ = scrollbar.event(&mut scrollbar_ctx, event);
                    self.scroll_offset = scrollbar.value_ref();
                    ctx.invalidate();
                    return true;
                }

                let hovered = self.item_index_at(bounds, *position);
                if hovered != self.hovered {
                    self.hovered = hovered;
                    ctx.invalidate();
                }
                true
            }
            Event::MouseWheel {
                position, delta_y, ..
            } if bounds.contains(*position) => {
                self.set_scroll_offset(self.scroll_offset - delta_y * 24.0);
                self.sync_scrollbar();
                ctx.invalidate();
                true
            }
            Event::MouseDown {
                position,
                button: MouseButton::Left,
            } if bounds.contains(*position) => {
                ctx.request_focus();
                let scrollbar_controls = {
                    let scrollbar = self.scrollbar.borrow();
                    scrollbar
                        .start_button_rect(scrollbar_bounds)
                        .contains(*position)
                        || scrollbar
                            .end_button_rect(scrollbar_bounds)
                            .contains(*position)
                        || scrollbar.thumb_rect(scrollbar_bounds).contains(*position)
                };
                if scrollbar_bounds.contains(*position) && scrollbar_controls {
                    let mut scrollbar_ctx = EventCtx {
                        ui: ctx.ui.clone(),
                        id: ctx.id(),
                        bounds: scrollbar_bounds,
                        state: ctx.state,
                    };
                    let mut scrollbar = self.scrollbar.borrow_mut();
                    let _ = scrollbar.event(&mut scrollbar_ctx, event);
                    self.scroll_offset = scrollbar.value_ref();
                    ctx.invalidate();
                    return true;
                }
                if let Some(index) = self.item_index_at(bounds, *position) {
                    self.select(index, ctx);
                } else if scrollbar_bounds.contains(*position) {
                    let mut scrollbar_ctx = EventCtx {
                        ui: ctx.ui.clone(),
                        id: ctx.id(),
                        bounds: scrollbar_bounds,
                        state: ctx.state,
                    };
                    let mut scrollbar = self.scrollbar.borrow_mut();
                    let _ = scrollbar.event(&mut scrollbar_ctx, event);
                    self.scroll_offset = scrollbar.value_ref();
                    ctx.invalidate();
                    return true;
                }
                true
            }
            Event::MouseUp {
                position,
                button: MouseButton::Left,
            } => {
                let scrollbar_active =
                    self.scrollbar.borrow().is_dragging() || scrollbar_bounds.contains(*position);
                if scrollbar_active {
                    let was_dragging = self.scrollbar.borrow().is_dragging();
                    let mut scrollbar = self.scrollbar.borrow_mut();
                    let handled = scrollbar.event(ctx, event);
                    self.scroll_offset = scrollbar.value_ref();
                    if was_dragging || handled {
                        ctx.invalidate();
                        return true;
                    }
                }
                false
            }
            Event::KeyDown { key, .. } if ctx.state.focused => match key {
                Key::Up => {
                    if let Some(index) = self.selected {
                        if index > 0 {
                            self.select(index - 1, ctx);
                            self.scroll_to(index - 1);
                        }
                    } else if !self.visible.is_empty() {
                        self.select(0, ctx);
                        self.scroll_to(0);
                    }
                    true
                }
                Key::Down => {
                    if let Some(index) = self.selected {
                        if index + 1 < self.visible.len() {
                            self.select(index + 1, ctx);
                            self.scroll_to(index + 1);
                        }
                    } else if !self.visible.is_empty() {
                        self.select(0, ctx);
                        self.scroll_to(0);
                    }
                    true
                }
                Key::Home => {
                    if !self.visible.is_empty() {
                        self.select(0, ctx);
                        self.scroll_to(0);
                    }
                    true
                }
                Key::End => {
                    if !self.visible.is_empty() {
                        let last = self.visible.len() - 1;
                        self.select(last, ctx);
                        self.scroll_to(last);
                    }
                    true
                }
                Key::PageUp => {
                    let page = (self.viewport_height / ITEM_HEIGHT).max(1.0) as usize;
                    let index = self.selected.unwrap_or(0).saturating_sub(page);
                    if !self.visible.is_empty() {
                        self.select(index, ctx);
                        self.scroll_to(index);
                    }
                    true
                }
                Key::PageDown => {
                    let page = (self.viewport_height / ITEM_HEIGHT).max(1.0) as usize;
                    let index = (self.selected.unwrap_or(0) + page)
                        .min(self.visible.len().saturating_sub(1));
                    if !self.visible.is_empty() {
                        self.select(index, ctx);
                        self.scroll_to(index);
                    }
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn focusable(&self) -> bool {
        true
    }

    impl_widget_any!();
}

impl std::fmt::Debug for ListView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ListView")
            .field("items", &self.items.len())
            .field("selected", &self.selected)
            .field("scroll_offset", &self.scroll_offset)
            .finish()
    }
}
