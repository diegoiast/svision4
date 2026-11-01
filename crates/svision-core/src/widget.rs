use std::any::Any;

use crate::{
    button_state::ButtonState,
    event::Event,
    text::TextRasterizer,
    theme::Theme,
    ui::{Ui, WidgetHandle, WidgetId},
    FontSpec, Insets, Rect, Size,
};

/// Qt-inspired size policy for one dimension.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SizePolicy {
    Fixed,
    Minimum,
    Maximum,
    #[default]
    Preferred,
    Expanding,
    MinimumExpanding,
    Ignored,
}

/// Size policy for both axes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SizePolicy2D {
    pub horizontal: SizePolicy,
    pub vertical: SizePolicy,
}

impl SizePolicy2D {
    pub const fn new(horizontal: SizePolicy, vertical: SizePolicy) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }
}

/// Per-widget interaction state, supplied by the UI on paint/event.
#[derive(Clone, Copy, Debug)]
pub struct WidgetState {
    pub interaction: ButtonState,
    pub focused: bool,
    pub enabled: bool,
    pub window_active: bool,
    pub checked: bool,
    pub layout_debug: bool,
}

impl Default for WidgetState {
    fn default() -> Self {
        Self {
            interaction: ButtonState::Normal,
            focused: false,
            enabled: true,
            window_active: true,
            checked: false,
            layout_debug: true,
        }
    }
}

impl WidgetState {
    pub fn is_hovered(&self) -> bool {
        matches!(
            self.interaction,
            ButtonState::Hovered | ButtonState::ClickedInside
        )
    }

    pub fn is_pressed(&self) -> bool {
        self.interaction == ButtonState::ClickedInside
    }
}

/// Context passed to [`Widget::measure`].
pub struct MeasureCtx {
    pub(crate) ui: Ui,
    pub(crate) id: WidgetId,
    pub(crate) children: Vec<WidgetId>,
}

impl MeasureCtx {
    pub fn id(&self) -> WidgetId {
        self.id
    }

    pub fn children(&self) -> &[WidgetId] {
        &self.children
    }

    /// Measures a child (and caches nothing; layout is intentionally simple).
    pub fn measure_child(&mut self, child: WidgetId, available: Size) -> Size {
        self.ui.measure_node(child, available)
    }

    pub fn measure_text(&self, text: &str, font: &FontSpec) -> Size {
        self.ui.measure_text(text, font)
    }

    pub fn theme(&self) -> Theme {
        self.ui.theme()
    }

    pub fn ui_size(&self) -> Size {
        self.ui.size()
    }
}

/// Context passed to [`Widget::arrange`].
pub struct ArrangeCtx {
    pub(crate) ui: Ui,
    pub(crate) id: WidgetId,
    pub(crate) children: Vec<WidgetId>,
}

impl ArrangeCtx {
    pub fn id(&self) -> WidgetId {
        self.id
    }

    pub fn children(&self) -> &[WidgetId] {
        &self.children
    }

    pub fn measure_child(&mut self, child: WidgetId, available: Size) -> Size {
        self.ui.measure_node(child, available)
    }

    pub fn arrange_child(&mut self, child: WidgetId, bounds: Rect) {
        self.ui.arrange_node(child, bounds);
    }

    pub fn theme(&self) -> Theme {
        self.ui.theme()
    }

    pub fn ui_size(&self) -> Size {
        self.ui.size()
    }
}

/// Context passed to [`Widget::paint`].
pub struct PaintCtx<'a> {
    pub rasterizer: &'a mut dyn TextRasterizer,
    pub theme: &'a Theme,
    pub bounds: Rect,
    pub state: WidgetState,
}

/// Context passed to [`Widget::event`].
pub struct EventCtx {
    pub(crate) ui: Ui,
    pub(crate) id: WidgetId,
    pub bounds: Rect,
    pub state: WidgetState,
}

impl EventCtx {
    pub fn id(&self) -> WidgetId {
        self.id
    }

    pub fn handle(&self) -> WidgetHandle {
        WidgetHandle::new(self.ui.clone(), self.id)
    }

    pub fn invalidate(&self) {
        self.ui.invalidate();
    }

    pub fn request_focus(&self) {
        self.ui.set_focus(Some(self.id));
    }

    pub fn is_hovered(&self) -> bool {
        self.state.is_hovered()
    }

    pub fn is_pressed(&self) -> bool {
        self.state.is_pressed()
    }

    pub fn is_focused(&self) -> bool {
        self.state.focused
    }

    pub fn is_enabled(&self) -> bool {
        self.state.enabled
    }
}

/// A retained UI object.
///
/// Widgets are owned by the [`Ui`] tree, not by their parents or the caller.
/// Callers interact with them through cloneable [`WidgetHandle`]s, which keeps
/// the ownership graph acyclic (apart from callbacks that capture handles).
pub trait Widget: Any {
    fn measure(&mut self, ctx: &mut MeasureCtx, available: Size) -> Size;

    fn arrange(&mut self, _ctx: &mut ArrangeCtx, _bounds: Rect) {}

    fn paint(&self, _ctx: &mut PaintCtx) {}

    fn min_size(&mut self, ctx: &mut MeasureCtx, _available: Size) -> Size {
        self.measure(ctx, Size::ZERO)
    }

    fn size_policy(&self) -> SizePolicy2D {
        SizePolicy2D::default()
    }

    fn margin(&self, _theme: &Theme) -> Insets {
        Insets::ZERO
    }

    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}
