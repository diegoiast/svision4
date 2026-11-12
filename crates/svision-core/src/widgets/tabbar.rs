use crate::{
    Event, FontSpec, Key, MouseButton, Point, Rect, Size, SizePolicy, SizePolicy2D, TabSpec, Ui,
    impl_widget_any,
    widget::{EventCtx, MeasureCtx, PaintCtx, Widget},
};

/// Height of a tab strip.
pub const TAB_HEIGHT: f32 = 34.0;
/// Horizontal padding inside a tab.
const TAB_PADDING_X: f32 = 16.0;
/// Smallest width of a tab.
const TAB_MIN_WIDTH: f32 = 72.0;

type TabChangeHandler = Box<dyn FnMut(usize, &mut EventCtx)>;

/// A horizontal strip of tabs.
///
/// The bar owns its titles and the current selection; it does not use child
/// widgets. Clicking a tab (or using the arrow keys while it has focus)
/// selects it and fires the [`on_change`](Self::on_change) callback.
///
/// A `TabBar` is usually paired with a [`Stack`](crate::Stack) to switch
/// between pages; [`TabWidget`](crate::TabWidget) bundles both.
pub struct TabBar {
    titles: Vec<String>,
    selected: usize,
    hovered: Option<usize>,
    font_size: Option<f32>,
    on_change: Option<TabChangeHandler>,
    measured: Vec<Size>,
    tab_rects: Vec<Rect>,
}

impl TabBar {
    /// Creates a bar with the given tab titles.
    pub fn new<I, S>(titles: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            titles: titles
                .into_iter()
                .map(|title| title.as_ref().to_string())
                .collect(),
            selected: 0,
            hovered: None,
            font_size: None,
            on_change: None,
            measured: Vec::new(),
            tab_rects: Vec::new(),
        }
    }

    /// Appends a tab and returns the bar.
    pub fn add_tab(mut self, title: impl Into<String>) -> Self {
        self.titles.push(title.into());
        self
    }

    /// The tab titles.
    pub fn titles(&self) -> &[String] {
        &self.titles
    }

    /// Replaces the tab titles, keeping the current selection in range.
    pub fn set_titles<I, S>(&mut self, titles: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.titles = titles
            .into_iter()
            .map(|title| title.as_ref().to_string())
            .collect();
        if self.selected >= self.titles.len() {
            self.selected = self.titles.len().saturating_sub(1);
        }
        self.measured.clear();
        self.tab_rects.clear();
    }

    /// Index of the selected tab.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Selects a tab programmatically. Does not fire the change callback.
    ///
    /// Returns `true` when the selection changed.
    pub fn set_selected(&mut self, index: usize) -> bool {
        if index >= self.titles.len() || index == self.selected {
            return false;
        }
        self.selected = index;
        true
    }

    /// Callback fired when the user selects another tab.
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: FnMut(usize, &mut EventCtx) + 'static,
    {
        self.on_change = Some(Box::new(callback));
        self
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = Some(size);
        self
    }

    fn font(&self, default_size: f32) -> FontSpec {
        FontSpec::new(self.font_size.unwrap_or(default_size))
    }

    /// Measures the tab titles against the current theme.
    pub(crate) fn measure_titles(&mut self, ui: &Ui) {
        let font = self.font(ui.theme().font_size());
        self.measured = self
            .titles
            .iter()
            .map(|title| ui.measure_text(title, &font))
            .collect();
    }

    /// Computes the rectangle of every tab inside `bounds`.
    pub(crate) fn layout_tabs(&mut self, bounds: Rect) {
        self.tab_rects.clear();
        let mut x = bounds.x;
        for size in &self.measured {
            let width = (size.width + TAB_PADDING_X * 2.0).max(TAB_MIN_WIDTH);
            self.tab_rects.push(Rect::new(x, bounds.y, width, bounds.height));
            x += width;
        }
    }

    /// The rectangle of a tab, if it has been laid out.
    pub fn tab_rect(&self, index: usize) -> Option<Rect> {
        self.tab_rects.get(index).copied()
    }

    fn tab_index_at(&self, position: Point) -> Option<usize> {
        self.tab_rects
            .iter()
            .position(|rect| rect.contains(position))
    }

    fn select(&mut self, index: usize, ctx: &mut EventCtx) {
        if index >= self.titles.len() || index == self.selected {
            return;
        }
        self.selected = index;
        if let Some(callback) = self.on_change.as_mut() {
            callback(index, ctx);
        }
        ctx.invalidate();
    }
}

impl Widget for TabBar {
    fn size_policy(&self) -> SizePolicy2D {
        SizePolicy2D::new(SizePolicy::Minimum, SizePolicy::Fixed)
    }

    fn measure(&mut self, ctx: &mut MeasureCtx, _available: Size) -> Size {
        self.measure_titles(&ctx.ui);
        let width: f32 = self
            .measured
            .iter()
            .map(|size| (size.width + TAB_PADDING_X * 2.0).max(TAB_MIN_WIDTH))
            .sum();
        Size::new(width, TAB_HEIGHT)
    }

    fn arrange(&mut self, _ctx: &mut crate::widget::ArrangeCtx, bounds: Rect) {
        self.layout_tabs(bounds);
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let theme = ctx.theme.clone();
        let font = self.font(theme.font_size());

        let tabs: Vec<TabSpec<'_>> = self
            .tab_rects
            .iter()
            .enumerate()
            .filter_map(|(index, rect)| {
                let title = self.titles.get(index)?;
                Some(TabSpec {
                    title,
                    size: self.measured.get(index).copied().unwrap_or(Size::ZERO),
                    rect: *rect,
                })
            })
            .collect();

        theme.draw_tabs(
            ctx.rasterizer,
            &tabs,
            self.selected,
            self.hovered,
            &font,
        );
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
                if let Some(index) = self.tab_index_at(*position) {
                    self.select(index, ctx);
                }
                true
            }
            Event::CursorMoved { position } => {
                let hovered = if ctx.bounds.contains(*position) {
                    self.tab_index_at(*position)
                } else {
                    None
                };
                if hovered != self.hovered {
                    self.hovered = hovered;
                    ctx.invalidate();
                }
                false
            }
            Event::KeyDown { key, .. } if ctx.state.focused => {
                let last = self.titles.len().saturating_sub(1);
                match key {
                    Key::Left => {
                        let next = if self.selected == 0 {
                            last
                        } else {
                            self.selected - 1
                        };
                        self.select(next, ctx);
                        true
                    }
                    Key::Right => {
                        let next = if self.selected >= last {
                            0
                        } else {
                            self.selected + 1
                        };
                        self.select(next, ctx);
                        true
                    }
                    Key::Home => {
                        self.select(0, ctx);
                        true
                    }
                    Key::End => {
                        self.select(last, ctx);
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
