use crate::{
    Event, FontSpec, Key, MouseButton, Rect, Ui, WidgetHandle, WidgetId,
    geometry::Size,
    theme::RadioButtonSpec,
    widget::{ArrangeCtx, EventCtx, MeasureCtx, PaintCtx, SizePolicy, SizePolicy2D, Widget},
};

/// A single radio button.
///
/// Displays a circular outline with a filled dot when checked, followed by the
/// label text. Carries a string `id` so a [`RadioButtonGroup`] can track which
/// one is selected.
pub struct RadioButton {
    id: String,
    label: String,
    checked: bool,
    on_change: Option<Box<dyn FnMut(&mut EventCtx)>>,
    measured_text: Size,
}

impl RadioButton {
    pub fn new(label: impl Into<String>) -> Self {
        let label = label.into();
        Self {
            id: label.clone(),
            label,
            checked: false,
            on_change: None,
            measured_text: Size::ZERO,
        }
    }

    /// Set the string id used to identify this radio inside its group.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Set the initial checked state.
    pub fn selected(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Callback invoked when this radio becomes selected.
    pub fn on_change<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut EventCtx) + 'static,
    {
        self.on_change = Some(Box::new(f));
        self
    }

    pub fn id_str(&self) -> &str {
        &self.id
    }

    pub fn is_checked(&self) -> bool {
        self.checked
    }

    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
    }
}

impl Widget for RadioButton {
    fn size_policy(&self) -> SizePolicy2D {
        SizePolicy2D::new(SizePolicy::Fixed, SizePolicy::Fixed)
    }

    fn measure(&mut self, ctx: &mut MeasureCtx, _available: Size) -> Size {
        let theme = ctx.theme();
        let font = FontSpec::new(theme.font_size());
        let text = ctx.measure_text(&self.label, &font);
        self.measured_text = text;
        let side = (text.height * 0.7).max(20.0);
        let gap = 8.0;
        Size::new(side + gap + text.width, side.max(text.height))
    }

    fn min_size(&mut self, ctx: &mut MeasureCtx, _available: Size) -> Size {
        self.measure(ctx, Size::new(0.0, 0.0))
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let theme = ctx.theme.clone();
        let spec = RadioButtonSpec {
            label: &self.label,
            label_size: self.measured_text,
            checked: self.checked,
            enabled: ctx.state.enabled,
            interaction: ctx.state.interaction,
            focused: ctx.state.focused && ctx.state.window_active,
        };
        theme.draw_radio_button(ctx.rasterizer, ctx.bounds, &spec);
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event) -> bool {
        match event {
            Event::KeyDown {
                key: Key::Space | Key::Enter,
                ..
            } => {
                if !self.checked {
                    self.checked = true;
                    ctx.invalidate();
                    if let Some(ref mut f) = self.on_change {
                        f(ctx);
                    }
                }
                true
            }
            Event::MouseDown {
                position,
                button: MouseButton::Left,
            } if ctx.bounds.contains(*position) => {
                // Show pressed state on button down; the ClickedInside middle state
                // is reflected visually via ButtonState in paint. Selection happens
                // on MouseUp.
                ctx.invalidate();
                true
            }
            Event::MouseUp {
                position,
                button: MouseButton::Left,
            } if ctx.bounds.contains(*position) => {
                if !self.checked {
                    self.checked = true;
                    ctx.invalidate();
                }
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

/// A mutually-exclusive group of radio buttons.
///
/// Arranges its radio children vertically and ensures only one is selected at
/// a time (Windows UI guidelines). Exposes the selected id and an `on_change`
/// callback fired whenever the selection changes.
pub struct RadioButtonGroup {
    selected_id: Option<String>,
    radios: Vec<(WidgetId, String)>,
    on_change: Option<Box<dyn FnMut(String)>>,
    spacing: f32,
    padding: f32,
}

impl RadioButtonGroup {
    pub fn new() -> Self {
        Self {
            selected_id: None,
            radios: Vec::new(),
            on_change: None,
            spacing: 8.0,
            padding: 4.0,
        }
    }

    pub fn on_change<F>(mut self, f: F) -> Self
    where
        F: FnMut(String) + 'static,
    {
        self.on_change = Some(Box::new(f));
        self
    }

    pub fn spacing(mut self, s: f32) -> Self {
        self.spacing = s;
        self
    }

    pub fn padding(mut self, p: f32) -> Self {
        self.padding = p;
        self
    }

    /// Returns the id of the currently selected radio, if any.
    pub fn selected_id(&self) -> Option<&str> {
        self.selected_id.as_deref()
    }

    /// Programmatically select the radio with the given id.
    pub fn select(&mut self, id: &str, ui: &Ui) {
        self.selected_id = Some(id.to_string());
        for (child_id, child_id_str) in &self.radios {
            let handle = WidgetHandle::new(ui.clone(), *child_id);
            let want = child_id_str == id;
            let _ = handle.edit(|rb: &mut RadioButton| {
                rb.set_checked(want);
            });
        }
    }

    /// Sync `selected_id` with the currently checked radio and enforce single
    /// selection. Called during arrange (no borrow conflict with `Ui.inner`).
    fn enforce_single_selection(&mut self, ui: &Ui) {
        // Find which radio was newly clicked: checked but not the current selection.
        let mut newly_checked: Option<String> = None;
        for (child_id, child_id_str) in &self.radios {
            let handle = WidgetHandle::new(ui.clone(), *child_id);
            let is_checked = handle
                .with_concrete::<RadioButton, _>(|rb| rb.is_checked())
                .unwrap_or(false);
            if is_checked && self.selected_id.as_deref() != Some(child_id_str.as_str()) {
                newly_checked = Some(child_id_str.clone());
            }
        }

        // Keep the newly clicked radio, falling back to the current selection.
        let keep_id = newly_checked.or_else(|| self.selected_id.clone());

        // Apply the single-selection state to all radios.
        for (child_id, child_id_str) in &self.radios {
            let handle = WidgetHandle::new(ui.clone(), *child_id);
            let want = keep_id.as_deref() == Some(child_id_str.as_str());
            let _ = handle.edit(|rb: &mut RadioButton| {
                rb.set_checked(want);
            });
        }

        // Fire group on_change when the selection actually changes.
        if keep_id != self.selected_id {
            self.selected_id = keep_id.clone();
            if let Some(ref mut f) = self.on_change {
                if let Some(id) = &keep_id {
                    f(id.clone());
                }
            }
        } else {
            self.selected_id = keep_id;
        }
    }
}

impl Default for RadioButtonGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for RadioButtonGroup {
    fn size_policy(&self) -> SizePolicy2D {
        SizePolicy2D::default()
    }

    fn measure(&mut self, ctx: &mut MeasureCtx, available: Size) -> Size {
        let padding = self.padding;
        let spacing = self.spacing;

        // Discover radio children and their ids.
        self.radios.clear();
        for child in ctx.children() {
            let handle = WidgetHandle::new(ctx.ui.clone(), *child);
            if let Some(radio_id) =
                handle.with_concrete::<RadioButton, _>(|rb| rb.id_str().to_string())
            {
                self.radios.push((*child, radio_id));
            }
        }

        let content_width = if available.width.is_finite() {
            available.width - padding * 2.0
        } else {
            f32::INFINITY
        };

        let mut content_height = 0.0f32;
        let mut widest = 0.0f32;
        for (child_id, _) in &self.radios {
            let child_size = ctx.measure_child(*child_id, Size::new(content_width, f32::INFINITY));
            content_height += child_size.height;
            widest = widest.max(child_size.width);
        }
        if !self.radios.is_empty() {
            content_height += spacing * (self.radios.len() - 1) as f32;
        }

        Size::new(
            (widest + padding * 2.0).max(0.0),
            (content_height + padding * 2.0).max(0.0),
        )
    }

    fn min_size(&mut self, ctx: &mut MeasureCtx, _available: Size) -> Size {
        self.measure(ctx, Size::new(0.0, 0.0))
    }

    fn arrange(&mut self, ctx: &mut ArrangeCtx, bounds: Rect) {
        // Sync selected_id with the currently checked radio.
        self.enforce_single_selection(&ctx.ui);

        let padding = self.padding;
        let spacing = self.spacing;
        let content = bounds.inset_all(padding);
        let mut y = content.y;

        for (child_id, _) in &self.radios {
            let child_size = ctx
                .ui
                .min_size_node(*child_id, Size::new(content.width, 0.0));
            ctx.arrange_child(
                *child_id,
                Rect::new(content.x, y, content.width, child_size.height),
            );
            y += child_size.height + spacing;
        }
    }

    fn paint(&self, _ctx: &mut PaintCtx) {
        // Group draws nothing; children paint themselves.
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
