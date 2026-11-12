use crate::{
    Align, Color, Insets, Rect, Size, SizePolicy, SizePolicy2D, Theme, WidgetHandle,
    impl_widget_any,
    widget::{ArrangeCtx, MeasureCtx, PaintCtx, Widget},
};

/// Overlays its children so that only one of them is visible at a time.
///
/// The children are laid out on top of each other (like a deck of cards) and
/// the child at [`Stack::current_index`] is the visible one. The others are
/// hidden: they are neither painted nor hit-tested, but they keep their place
/// in the tree so that switching pages is cheap.
///
/// [`TabBar`](crate::TabBar) is usually placed next to a `Stack` to switch
/// between its pages; [`TabWidget`](crate::TabWidget) bundles both.
pub struct Stack {
    current: usize,
    align: Align,
    padding: Option<Insets>,
    margin: Option<Insets>,
    background: Option<Color>,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            current: 0,
            align: Align::Stretch,
            padding: None,
            margin: None,
            background: None,
        }
    }

    /// Selects the page shown by default.
    pub fn current(mut self, index: usize) -> Self {
        self.current = index;
        self
    }

    /// Selects the page that is currently shown.
    pub fn set_current(&mut self, index: usize) {
        self.current = index;
    }

    /// Index of the page that is currently shown.
    pub fn current_index(&self) -> usize {
        self.current
    }

    /// Alignment of the pages inside the stack area.
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = Some(Insets::all(padding));
        self
    }

    pub fn padding_xy(mut self, horizontal: f32, vertical: f32) -> Self {
        self.padding = Some(Insets::symmetric(horizontal, vertical));
        self
    }

    pub fn insets(mut self, insets: Insets) -> Self {
        self.padding = Some(insets);
        self
    }

    pub fn set_padding(&mut self, padding: Insets) {
        self.padding = Some(padding);
    }

    pub fn margin(mut self, margin: f32) -> Self {
        self.margin = Some(Insets::all(margin));
        self
    }

    pub fn set_margin(&mut self, margin: Insets) {
        self.margin = Some(margin);
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    pub fn set_background(&mut self, color: Color) {
        self.background = Some(color);
    }

    pub fn resolved_padding(&self, _theme: &dyn Theme) -> Insets {
        self.padding.unwrap_or(Insets::ZERO)
    }

    pub fn resolved_margin(&self, theme: &dyn Theme) -> Insets {
        self.margin.unwrap_or_else(|| Insets::all(theme.spacing()))
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Stack {
    fn size_policy(&self) -> SizePolicy2D {
        SizePolicy2D::new(SizePolicy::Expanding, SizePolicy::Expanding)
    }

    fn margin(&self, theme: &dyn Theme) -> Insets {
        self.resolved_margin(theme)
    }

    fn min_size(&mut self, ctx: &mut MeasureCtx, _available: Size) -> Size {
        let theme = ctx.theme();
        let padding = self.resolved_padding(theme.as_ref());
        let mut width = 0.0f32;
        let mut height = 0.0f32;
        for child in ctx.children().to_vec() {
            let child_min = ctx
                .ui
                .min_size_node(child, Size::new(f32::INFINITY, f32::INFINITY));
            width = width.max(child_min.width);
            height = height.max(child_min.height);
        }
        Size::new(
            (width + padding.horizontal()).max(0.0),
            (height + padding.vertical()).max(0.0),
        )
    }

    fn measure(&mut self, ctx: &mut MeasureCtx, available: Size) -> Size {
        let theme = ctx.theme();
        let padding = self.resolved_padding(theme.as_ref());
        let content_width = if available.width.is_finite() {
            (available.width - padding.horizontal()).max(0.0)
        } else {
            f32::INFINITY
        };

        let mut width = 0.0f32;
        let mut height = 0.0f32;
        for child in ctx.children().to_vec() {
            let child_size =
                ctx.measure_child(child, Size::new(content_width, f32::INFINITY));
            width = width.max(child_size.width);
            height = height.max(child_size.height);
        }

        Size::new(
            (width + padding.horizontal()).max(0.0),
            (height + padding.vertical()).max(0.0),
        )
    }

    fn arrange(&mut self, ctx: &mut ArrangeCtx, bounds: Rect) {
        let theme = ctx.theme();
        let padding = self.resolved_padding(theme.as_ref());
        let content = bounds.inset(padding);
        let children = ctx.children().to_vec();
        if children.is_empty() {
            return;
        }
        if self.current >= children.len() {
            self.current = children.len() - 1;
        }

        for (index, child) in children.into_iter().enumerate() {
            // Only the current page is visible; the others keep their tree
            // position but are skipped by painting and hit testing.
            let handle = WidgetHandle::new(ctx.ui.clone(), child);
            handle.set_visible(index == self.current);

            let size = match self.align {
                Align::Stretch => Size::new(content.width, content.height),
                _ => {
                    let preferred =
                        ctx.measure_child(child, Size::new(content.width, f32::INFINITY));
                    Size::new(
                        preferred.width.min(content.width),
                        preferred.height.min(content.height),
                    )
                }
            };
            let x = match self.align {
                Align::Center => content.x + (content.width - size.width) * 0.5,
                Align::End => content.x + content.width - size.width,
                Align::Start | Align::Stretch => content.x,
            };
            let y = match self.align {
                Align::Center => content.y + (content.height - size.height) * 0.5,
                Align::End => content.y + content.height - size.height,
                Align::Start | Align::Stretch => content.y,
            };
            ctx.arrange_child(child, Rect::new(x, y, size.width, size.height));
        }
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        if let Some(color) = self.background {
            ctx.rasterizer.fill_rect(ctx.bounds, color);
        }
    }

    impl_widget_any!();
}
