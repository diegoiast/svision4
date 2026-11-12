use crate::{
    Align, Color, Insets, Point, Rect, Size, Theme, impl_widget_any,
    widget::{ArrangeCtx, MeasureCtx, PaintCtx, SizePolicy, SizePolicy2D, Widget},
};

pub struct Column {
    padding: Option<Insets>,
    spacing: Option<f32>,
    margin: Option<Insets>,
    background: Option<Color>,
    align: Align,
    debug: bool,
}

impl Column {
    pub fn new() -> Self {
        Self {
            padding: None,
            spacing: None,
            margin: None,
            background: None,
            align: Align::Stretch,
            debug: true,
        }
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

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = Some(spacing);
        self
    }

    pub fn set_spacing(&mut self, spacing: f32) {
        self.spacing = Some(spacing);
    }

    pub fn margin(mut self, margin: f32) -> Self {
        self.margin = Some(Insets::all(margin));
        self
    }

    pub fn margin_xy(mut self, horizontal: f32, vertical: f32) -> Self {
        self.margin = Some(Insets::symmetric(horizontal, vertical));
        self
    }

    pub fn margins(mut self, margins: Insets) -> Self {
        self.margin = Some(margins);
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

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    pub fn debug_enabled(&self) -> bool {
        self.debug
    }

    pub fn resolved_padding(&self, theme: &dyn Theme) -> Insets {
        self.padding.unwrap_or_else(|| Insets::all(theme.padding()))
    }

    pub fn resolved_spacing(&self, theme: &dyn Theme) -> f32 {
        self.spacing.unwrap_or(theme.spacing())
    }

    pub fn resolved_margin(&self, theme: &dyn Theme) -> Insets {
        self.margin.unwrap_or_else(|| Insets::all(theme.spacing()))
    }
}

impl Default for Column {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Column {
    fn min_size(&mut self, ctx: &mut MeasureCtx, _available: Size) -> Size {
        let theme = ctx.theme();
        let padding = self.resolved_padding(theme.as_ref());
        let spacing = self.resolved_spacing(theme.as_ref());
        let mut width = 0.0f32;
        let mut height = padding.vertical();
        let count = ctx.children().len();
        for child in ctx.children().to_vec() {
            let child_min = ctx
                .ui
                .min_size_node(child, Size::new(f32::INFINITY, f32::INFINITY));
            let child_margin = ctx.ui.margin_node(child, theme.as_ref());
            width = width.max(child_min.width + child_margin.horizontal());
            height += child_min.height + child_margin.vertical();
        }
        if count > 1 {
            height += spacing * (count - 1) as f32;
        }
        Size::new((width + padding.horizontal()).max(0.0), height.max(0.0))
    }

    fn size_policy(&self) -> SizePolicy2D {
        SizePolicy2D::default()
    }

    fn margin(&self, theme: &dyn Theme) -> Insets {
        self.resolved_margin(theme)
    }

    fn measure(&mut self, ctx: &mut MeasureCtx, available: Size) -> Size {
        let theme = ctx.theme();
        let padding = self.resolved_padding(theme.as_ref());
        let spacing = self.resolved_spacing(theme.as_ref());
        let content_width = if available.width.is_finite() {
            (available.width - padding.horizontal()).max(0.0)
        } else {
            f32::INFINITY
        };

        let mut content_height = 0.0f32;
        let mut widest = 0.0f32;
        let count = ctx.children().len();

        for child in ctx.children().to_vec() {
            let child_margin = ctx.ui.margin_node(child, theme.as_ref());
            let child_available_width = if content_width.is_finite() {
                (content_width - child_margin.horizontal()).max(0.0)
            } else {
                f32::INFINITY
            };
            let child_size =
                ctx.measure_child(child, Size::new(child_available_width, f32::INFINITY));
            content_height += child_size.height + child_margin.vertical();
            widest = widest.max(child_size.width + child_margin.horizontal());
        }
        if count > 1 {
            content_height += spacing * (count - 1) as f32;
        }

        let width = if self.align == Align::Stretch && available.width.is_finite() {
            available.width
        } else {
            widest + padding.horizontal()
        };

        Size::new(width.max(0.0), content_height + padding.vertical())
    }

    fn arrange(&mut self, ctx: &mut ArrangeCtx, bounds: Rect) {
        let theme = ctx.theme();
        let padding = self.resolved_padding(theme.as_ref());
        let spacing = self.resolved_spacing(theme.as_ref());
        let content = bounds.inset(padding);
        let children = ctx.children().to_vec();
        let count = children.len();
        if count == 0 {
            return;
        }

        let mut min_sizes = Vec::with_capacity(count);
        let mut preferred_sizes = Vec::with_capacity(count);
        let mut policies = Vec::with_capacity(count);
        let mut margins = Vec::with_capacity(count);
        for child in &children {
            let child_margin = ctx.ui.margin_node(*child, theme.as_ref());
            let child_available_width = (content.width - child_margin.horizontal()).max(0.0);
            let min_size = ctx
                .ui
                .min_size_node(*child, Size::new(child_available_width, 0.0));
            let preferred_size =
                ctx.measure_child(*child, Size::new(child_available_width, f32::INFINITY));
            min_sizes.push(min_size);
            preferred_sizes.push(preferred_size);
            policies.push(ctx.ui.size_policy_node(*child));
            margins.push(child_margin);
        }

        let total_spacing = spacing * (count - 1) as f32;
        let total_preferred: f32 = preferred_sizes
            .iter()
            .zip(margins.iter())
            .map(|(size, margin)| size.height + margin.vertical())
            .sum::<f32>()
            + total_spacing;
        let total_min: f32 = min_sizes
            .iter()
            .zip(margins.iter())
            .map(|(size, margin)| size.height + margin.vertical())
            .sum::<f32>()
            + total_spacing;
        let mut heights: Vec<f32> = preferred_sizes.iter().map(|size| size.height).collect();
        let extra = (content.height - total_preferred).max(0.0);
        let expanding: Vec<usize> = policies
            .iter()
            .enumerate()
            .filter(|(_, policy)| {
                matches!(
                    policy.vertical,
                    SizePolicy::Expanding | SizePolicy::MinimumExpanding
                )
            })
            .map(|(index, _)| index)
            .collect();
        if extra > 0.0 && !expanding.is_empty() {
            let share = extra / expanding.len() as f32;
            for index in expanding {
                heights[index] += share;
            }
        }
        if content.height < total_min {
            for index in 0..count {
                heights[index] = min_sizes[index].height;
            }
        }

        let mut y = content.y;
        for (index, child) in children.into_iter().enumerate() {
            let margin = margins[index];
            y += margin.top;
            let width = if matches!(
                policies[index].horizontal,
                SizePolicy::Expanding | SizePolicy::MinimumExpanding
            ) || self.align == Align::Stretch
            {
                (content.width - margin.horizontal()).max(0.0)
            } else {
                preferred_sizes[index]
                    .width
                    .min((content.width - margin.horizontal()).max(0.0))
                    .max(min_sizes[index].width)
            };
            let x = match self.align {
                Align::Center => {
                    content.x + margin.left + ((content.width - margin.horizontal()) - width) * 0.5
                }
                Align::End => content.x + content.width - margin.right - width,
                Align::Start | Align::Stretch => content.x + margin.left,
            };
            ctx.arrange_child(child, Rect::new(x, y, width, heights[index]));
            y += heights[index] + margin.bottom + spacing;
        }
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        if let Some(color) = self.background {
            ctx.rasterizer.fill_rect(ctx.bounds, color);
        }
        if self.debug && ctx.state.layout_debug {
            let color = ctx.theme.border();
            let bounds = ctx.bounds;
            let mut x = bounds.x;
            while x < bounds.right() {
                let end = (x + 4.0).min(bounds.right());
                ctx.rasterizer.draw_line(
                    Point::new(x, bounds.y),
                    Point::new(end, bounds.y),
                    color,
                    1.0,
                );
                x += 8.0;
            }
            let mut x = bounds.x;
            while x < bounds.right() {
                let end = (x + 4.0).min(bounds.right());
                ctx.rasterizer.draw_line(
                    Point::new(x, bounds.bottom()),
                    Point::new(end, bounds.bottom()),
                    color,
                    1.0,
                );
                x += 8.0;
            }
            let mut y = bounds.y;
            while y < bounds.bottom() {
                let end = (y + 4.0).min(bounds.bottom());
                ctx.rasterizer.draw_line(
                    Point::new(bounds.x, y),
                    Point::new(bounds.x, end),
                    color,
                    1.0,
                );
                y += 8.0;
            }
            let mut y = bounds.y;
            while y < bounds.bottom() {
                let end = (y + 4.0).min(bounds.bottom());
                ctx.rasterizer.draw_line(
                    Point::new(bounds.right(), y),
                    Point::new(bounds.right(), end),
                    color,
                    1.0,
                );
                y += 8.0;
            }
        }
    }

    impl_widget_any!();
}
