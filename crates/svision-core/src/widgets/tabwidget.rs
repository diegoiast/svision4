use crate::{
    Event, EventCtx, Rect, Size, SizePolicy, SizePolicy2D, impl_widget_any,
    widget::{ArrangeCtx, MeasureCtx, PaintCtx, Widget},
    widgets::{Stack, TAB_HEIGHT, TabBar},
};

type TabChangeHandler = Box<dyn FnMut(usize, &mut EventCtx)>;

/// A tabbed container: a tab bar on top and a stack of pages below.
///
/// The widget is assembled from two parts:
///
/// * a [`TabBar`] that draws the tabs and tracks the selection, and
/// * a [`Stack`] that holds the pages and shows only the selected one.
///
/// The pages are the children of the tab widget and are shown in the order they
/// were added; the tab titles are given to [`TabWidget::new`].
///
/// ```
/// use svision_core::prelude::*;
///
/// let ui = Ui::new();
/// let root = ui.set_root(Column::new());
/// let tabs = ui.add_child(&root, TabWidget::new(["Songs", "Albums"]));
/// ui.add_child(&tabs, Label::new("Songs"));
/// ui.add_child(&tabs, Label::new("Albums"));
/// ```
pub struct TabWidget {
    tab_bar: TabBar,
    stack: Stack,
    on_change: Option<TabChangeHandler>,
}

impl TabWidget {
    /// Creates a tab widget with the given tab titles.
    ///
    /// One child widget must be added per tab; the children are the pages and
    /// are shown in the order they were added.
    pub fn new<I, S>(titles: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            tab_bar: TabBar::new(titles),
            stack: Stack::new(),
            on_change: None,
        }
    }

    /// The tab titles.
    pub fn titles(&self) -> &[String] {
        self.tab_bar.titles()
    }

    /// Replaces the tab titles.
    pub fn set_titles<I, S>(&mut self, titles: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.tab_bar.set_titles(titles);
    }

    /// Index of the selected tab.
    pub fn selected(&self) -> usize {
        self.tab_bar.selected()
    }

    /// Selects a tab programmatically. Does not fire the change callback.
    pub fn set_selected(&mut self, index: usize) -> bool {
        let changed = self.tab_bar.set_selected(index);
        if changed {
            self.stack.set_current(index);
        }
        changed
    }

    /// Callback fired when the user selects another tab.
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: FnMut(usize, &mut EventCtx) + 'static,
    {
        self.on_change = Some(Box::new(callback));
        self
    }

    /// Height of the tab strip.
    pub const fn tab_bar_height() -> f32 {
        TAB_HEIGHT
    }

    /// The rectangle of a tab, if it has been laid out.
    pub fn tab_rect(&self, index: usize) -> Option<Rect> {
        self.tab_bar.tab_rect(index)
    }

    fn strip_rect(&self, bounds: Rect) -> Rect {
        let height = TAB_HEIGHT.min(bounds.height);
        Rect::new(bounds.x, bounds.y, bounds.width, height)
    }

    fn content_rect(&self, bounds: Rect) -> Rect {
        let height = TAB_HEIGHT.min(bounds.height);
        Rect::new(
            bounds.x,
            bounds.y + height,
            bounds.width,
            (bounds.height - height).max(0.0),
        )
    }

    /// Keeps the stack in sync with the tab bar and reports the change.
    fn sync_selection(&mut self, previous: usize, ctx: &mut EventCtx) {
        let current = self.tab_bar.selected();
        if current == previous {
            return;
        }
        self.stack.set_current(current);
        if let Some(callback) = self.on_change.as_mut() {
            callback(current, ctx);
        }
        ctx.invalidate();
    }
}

impl Widget for TabWidget {
    fn size_policy(&self) -> SizePolicy2D {
        SizePolicy2D::new(SizePolicy::Expanding, SizePolicy::Expanding)
    }

    fn min_size(&mut self, ctx: &mut MeasureCtx, _available: Size) -> Size {
        let strip = self.tab_bar.measure(ctx, Size::ZERO);
        let content = self.stack.min_size(ctx, Size::ZERO);
        Size::new(
            strip.width.max(content.width),
            strip.height + content.height,
        )
    }

    fn measure(&mut self, ctx: &mut MeasureCtx, available: Size) -> Size {
        let strip = self.tab_bar.measure(ctx, available);
        let content_height = if available.height.is_finite() {
            (available.height - strip.height).max(0.0)
        } else {
            f32::INFINITY
        };
        let content = self
            .stack
            .measure(ctx, Size::new(available.width, content_height));
        Size::new(
            strip.width.max(content.width),
            strip.height + content.height,
        )
    }

    fn arrange(&mut self, ctx: &mut ArrangeCtx, bounds: Rect) {
        // The tab bar needs the measured titles to lay out the tabs. Measure
        // them here as well: the root widget is arranged without ever being
        // measured, so `measure` may not have run.
        self.tab_bar.measure_titles(&ctx.ui);

        let strip = self.strip_rect(bounds);
        self.tab_bar.layout_tabs(strip);
        let content = self.content_rect(bounds);
        self.stack.arrange(ctx, content);

        // The stack clamps itself to the number of pages; keep the tab bar in
        // sync so a tab without a page is not shown as selected.
        let current = self.stack.current_index();
        if self.tab_bar.selected() != current {
            self.tab_bar.set_selected(current);
        }
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let theme = ctx.theme;
        let content = self.content_rect(ctx.bounds);
        ctx.rasterizer
            .fill_round_rect(content, theme.radius(), theme.surface());

        let strip = self.strip_rect(ctx.bounds);
        let mut strip_ctx = PaintCtx {
            rasterizer: &mut *ctx.rasterizer,
            ui: ctx.ui,
            theme: ctx.theme,
            bounds: strip,
            state: ctx.state,
        };
        self.tab_bar.paint(&mut strip_ctx);
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event) -> bool {
        if !ctx.state.enabled {
            return false;
        }

        let strip = self.strip_rect(ctx.bounds);
        let mut strip_ctx = EventCtx {
            ui: ctx.ui.clone(),
            id: ctx.id(),
            bounds: strip,
            state: ctx.state,
        };
        let previous = self.tab_bar.selected();
        let handled = self.tab_bar.event(&mut strip_ctx, event);
        self.sync_selection(previous, ctx);
        handled
    }

    fn focusable(&self) -> bool {
        true
    }

    impl_widget_any!();
}
