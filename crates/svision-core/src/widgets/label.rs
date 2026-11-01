use crate::{
    impl_widget_any,
    widget::{MeasureCtx, PaintCtx, Widget},
    Color, FontSpec, Point, Size, SizePolicy, SizePolicy2D, TextAlign,
};

/// A non-interactive text label.
#[derive(Clone, Debug)]
pub struct Label {
    text: String,
    font_size: Option<f32>,
    bold: bool,
    color: Option<Color>,
    muted: bool,
    align: TextAlign,
    measured: Size,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            font_size: None,
            bold: false,
            color: None,
            muted: false,
            align: TextAlign::Left,
            measured: Size::ZERO,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = Some(size);
        self
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn muted(mut self) -> Self {
        self.muted = true;
        self
    }

    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    fn font(&self, default_size: f32) -> FontSpec {
        FontSpec::new(self.font_size.unwrap_or(default_size)).bold(self.bold)
    }
}

impl Widget for Label {
    fn size_policy(&self) -> SizePolicy2D {
        SizePolicy2D::new(SizePolicy::Fixed, SizePolicy::Fixed)
    }

    fn measure(&mut self, ctx: &mut MeasureCtx, _available: Size) -> Size {
        let theme = ctx.theme();
        let font = self.font(theme.font_size());
        let size = ctx.measure_text(&self.text, &font);
        self.measured = size;
        size
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let font = self.font(ctx.theme.font_size());
        let x = match self.align {
            TextAlign::Left => ctx.bounds.x,
            TextAlign::Center => ctx.bounds.x + (ctx.bounds.width - self.measured.width) * 0.5,
            TextAlign::Right => ctx.bounds.right() - self.measured.width,
        };
        let y = ctx.bounds.y + (ctx.bounds.height - self.measured.height) * 0.5;
        let color = self.color.unwrap_or(if self.muted {
            ctx.theme.text_muted()
        } else {
            ctx.theme.text()
        });
        ctx.rasterizer
            .draw_text(&self.text, Point::new(x, y), &font, color);
    }

    impl_widget_any!();
}
