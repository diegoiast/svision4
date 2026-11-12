use crate::{
    CheckboxSpec, Event, FontSpec, Key, MouseButton,
    geometry::Size,
    widget::{EventCtx, MeasureCtx, PaintCtx, SizePolicy, SizePolicy2D, Widget},
};

/// A checkbox widget that can be toggled on/off.
/// It displays a square box that is filled when checked,
/// and optional label text to the right of the box.
pub struct Checkbox {
    label: String,
    checked: bool,
    measured: Size,
    on_change: Option<Box<dyn FnMut(&mut EventCtx)>>,
}

impl Checkbox {
    /// Create a new checkbox with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            checked: false,
            measured: Size::ZERO,
            on_change: None,
        }
    }

    /// Set the initial checked state.
    pub fn selected(mut self, selected: bool) -> Self {
        self.checked = selected;
        self
    }

    pub fn is_checked(&self) -> bool {
        self.checked
    }

    /// Set a callback that is called when the checkbox is clicked.
    /// The callback receives the event context and can query the new state.
    pub fn on_click<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut EventCtx) + 'static,
    {
        self.on_change = Some(Box::new(f));
        self
    }
}

impl Widget for Checkbox {
    fn size_policy(&self) -> SizePolicy2D {
        // Fixed size: a 16×16 square plus label space.
        SizePolicy2D::new(SizePolicy::Fixed, SizePolicy::Fixed)
    }

    fn measure(&mut self, ctx: &mut MeasureCtx, _available: Size) -> Size {
        // Square side: smaller than full height, min 24px.
        let gap = 4.0;
        let theme = ctx.theme();
        let font = FontSpec::new(theme.font_size());
        let text_size = if self.label.is_empty() {
            Size::ZERO
        } else {
            ctx.measure_text(&self.label, &font)
        };
        let side = (text_size.height * 0.6).max(24.0);
        self.measured = text_size;
        Size::new(side + gap + text_size.width, side.max(text_size.height))
    }

    fn min_size(&mut self, ctx: &mut MeasureCtx, _available: Size) -> Size {
        self.measure(ctx, Size::new(0.0, 0.0))
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let theme = ctx.theme.clone();
        let spec = CheckboxSpec {
            label: &self.label,
            label_size: self.measured,
            checked: self.checked,
            enabled: ctx.state.enabled,
            interaction: ctx.state.interaction,
            focused: ctx.state.focused && ctx.state.window_active,
        };
        theme.draw_checkbox(ctx.rasterizer, ctx.bounds, &spec);
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event) -> bool {
        if !ctx.state.enabled {
            return false;
        }

        match event {
            Event::KeyDown {
                key: Key::Space | Key::Enter,
                ..
            } => {
                self.checked = !self.checked;
                ctx.invalidate();
                if let Some(ref mut f) = self.on_change {
                    f(ctx);
                }
                true
            }
            Event::MouseDown {
                position,
                button: MouseButton::Left,
            } if ctx.bounds.contains(*position) => {
                // Show pressed state on button down; ClickedInside middle state is
                // reflected visually via ButtonState in paint. Toggle happens on release.
                ctx.invalidate();
                true
            }
            Event::MouseUp {
                position,
                button: MouseButton::Left,
            } if ctx.bounds.contains(*position) => {
                self.checked = !self.checked;
                ctx.invalidate();
                if let Some(ref mut f) = self.on_change {
                    f(ctx);
                }
                true
            }
            _ => false,
        }
    }

    fn focusable(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
