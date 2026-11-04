use std::{cell::RefCell, collections::HashMap, fmt, marker::PhantomData, rc::Rc, time::Instant};

use crate::{
    Insets, Key, Point, Rect, Size,
    button_state::ButtonStateHandler,
    event::Event,
    text::{ApproxTextEngine, FontSpec, TextEngine, TextRasterizer},
    theme::Theme,
    widget::{ArrangeCtx, EventCtx, MeasureCtx, PaintCtx, SizePolicy2D, Widget, WidgetState},
};

/// Stable identifier for a widget inside a [`Ui`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WidgetId(pub(crate) u64);

impl fmt::Display for WidgetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WidgetId({})", self.0)
    }
}

#[derive(Clone, Debug)]
struct Node {
    parent: Option<WidgetId>,
    children: Vec<WidgetId>,
    rect: Rect,
    visible: bool,
    enabled: bool,
    interaction: ButtonStateHandler,
}

pub(crate) struct UiInner {
    widgets: HashMap<WidgetId, Box<dyn Widget>>,
    nodes: HashMap<WidgetId, Node>,
    root: Option<WidgetId>,
    next_id: u64,
    size: Size,
    theme: Theme,
    text_engine: Rc<dyn TextEngine>,
    dirty: bool,
    needs_layout: bool,
    hovered: Option<WidgetId>,
    capture: Option<WidgetId>,
    focus: Option<WidgetId>,
    window_active: bool,
    layout_debug: bool,
}

/// The retained widget tree.
///
/// `Ui` is a cloneable handle to shared state. All mutation is single-threaded
/// and coordinated by the host (normally on the winit event loop thread).
#[derive(Clone)]
pub struct Ui {
    pub(crate) inner: Rc<RefCell<UiInner>>,
}

impl fmt::Debug for Ui {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ui")
            .field("size", &self.size())
            .field("dirty", &self.inner.borrow().dirty)
            .finish()
    }
}

impl Default for Ui {
    fn default() -> Self {
        Self::new()
    }
}

impl Ui {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(UiInner {
                widgets: HashMap::new(),
                nodes: HashMap::new(),
                root: None,
                next_id: 1,
                size: Size::ZERO,
                theme: Theme::default(),
                text_engine: Rc::new(ApproxTextEngine),
                dirty: true,
                needs_layout: true,
                hovered: None,
                capture: None,
                focus: None,
                window_active: true,
                layout_debug: false,
            })),
        }
    }

    /// Replaces the whole tree with a single root widget.
    pub fn set_root<W: Widget>(&self, widget: W) -> WidgetHandle {
        let id = {
            let mut inner = self.inner.borrow_mut();
            inner.widgets.clear();
            inner.nodes.clear();
            inner.root = None;
            inner.hovered = None;
            inner.capture = None;
            inner.focus = None;
            let id = alloc_node(&mut inner, Box::new(widget), None);
            inner.root = Some(id);
            inner.needs_layout = true;
            inner.dirty = true;
            id
        };
        WidgetHandle::new(self.clone(), id)
    }

    /// Adds `child` to `parent` and returns a reference to the new widget.
    pub fn add_child<W: Widget>(&self, parent: &WidgetHandle, child: W) -> WidgetHandle {
        let id = {
            let mut inner = self.inner.borrow_mut();
            let parent_id = parent.id;
            assert!(
                inner.nodes.contains_key(&parent_id),
                "cannot add a child to a widget that is no longer in the tree"
            );
            let id = alloc_node(&mut inner, Box::new(child), Some(parent_id));
            if let Some(node) = inner.nodes.get_mut(&parent_id) {
                node.children.push(id);
            }
            inner.needs_layout = true;
            inner.dirty = true;
            id
        };
        WidgetHandle::new(self.clone(), id)
    }

    pub fn root(&self) -> Option<WidgetHandle> {
        self.inner
            .borrow()
            .root
            .map(|id| WidgetHandle::new(self.clone(), id))
    }

    pub fn handle(&self, id: WidgetId) -> Option<WidgetHandle> {
        if self.inner.borrow().nodes.contains_key(&id) {
            Some(WidgetHandle::new(self.clone(), id))
        } else {
            None
        }
    }

    pub fn is_valid(&self, id: WidgetId) -> bool {
        self.inner.borrow().nodes.contains_key(&id)
    }

    pub fn remove(&self, handle: &WidgetHandle) {
        self.remove_id(handle.id);
    }

    fn remove_id(&self, id: WidgetId) {
        let mut inner = self.inner.borrow_mut();
        let Some(node) = inner.nodes.get(&id) else {
            return;
        };
        let parent = node.parent;
        if let Some(parent) = parent {
            if let Some(parent_node) = inner.nodes.get_mut(&parent) {
                parent_node.children.retain(|child| *child != id);
            }
        }

        let mut stack = vec![id];
        let mut removed = Vec::new();
        while let Some(current) = stack.pop() {
            if let Some(node) = inner.nodes.remove(&current) {
                stack.extend(node.children.iter().copied());
                removed.push(current);
            }
        }
        for removed_id in &removed {
            inner.widgets.remove(removed_id);
        }
        if inner.root == Some(id) {
            inner.root = None;
        }
        clear_if_removed(&mut inner.hovered, &removed);
        clear_if_removed(&mut inner.capture, &removed);
        clear_if_removed(&mut inner.focus, &removed);
        inner.needs_layout = true;
        inner.dirty = true;
    }

    pub fn size(&self) -> Size {
        self.inner.borrow().size
    }

    pub fn min_size(&self) -> Size {
        let root = self.inner.borrow().root;
        root.map(|root| self.min_size_node(root, Size::ZERO))
            .unwrap_or(Size::ZERO)
    }

    pub fn process_auto_repeat(&self, now: Instant) -> Option<Instant> {
        let mut next = None;
        let root = self.inner.borrow().root;
        if let Some(root) = root {
            process_auto_repeat_node(self, root, now, &mut next);
        }
        next
    }

    pub fn set_size(&self, size: Size) {
        let size = Size::new(size.width.max(0.0), size.height.max(0.0));
        let mut inner = self.inner.borrow_mut();
        if inner.size != size {
            inner.size = size;
            inner.needs_layout = true;
            inner.dirty = true;
        }
    }

    pub fn theme(&self) -> Theme {
        self.inner.borrow().theme.clone()
    }

    pub fn layout_debug(&self) -> bool {
        self.inner.borrow().layout_debug
    }

    pub fn set_layout_debug(&self, debug: bool) {
        let mut inner = self.inner.borrow_mut();
        if inner.layout_debug != debug {
            inner.layout_debug = debug;
            inner.dirty = true;
        }
    }

    pub fn set_theme(&self, theme: Theme) {
        let mut inner = self.inner.borrow_mut();
        inner.theme = theme;
        inner.needs_layout = true;
        inner.dirty = true;
    }

    pub fn set_text_engine(&self, engine: Rc<dyn TextEngine>) {
        let mut inner = self.inner.borrow_mut();
        inner.text_engine = engine;
        inner.needs_layout = true;
        inner.dirty = true;
    }

    pub fn invalidate(&self) {
        let mut inner = self.inner.borrow_mut();
        inner.dirty = true;
        inner.needs_layout = true;
    }

    pub fn mark_dirty(&self) {
        self.inner.borrow_mut().dirty = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.inner.borrow().dirty
    }

    /// Returns and clears the dirty flag.
    pub fn take_dirty(&self) -> bool {
        let mut inner = self.inner.borrow_mut();
        std::mem::replace(&mut inner.dirty, false)
    }

    pub(crate) fn measure_text(&self, text: &str, font: &FontSpec) -> Size {
        let engine = self.inner.borrow().text_engine.clone();
        engine.measure(text, font)
    }

    pub fn set_focus(&self, focus: Option<WidgetId>) {
        let mut inner = self.inner.borrow_mut();
        if inner.focus != focus {
            inner.focus = focus;
            inner.dirty = true;
        }
    }

    pub fn focus(&self) -> Option<WidgetHandle> {
        self.inner
            .borrow()
            .focus
            .map(|id| WidgetHandle::new(self.clone(), id))
    }

    pub fn focused(&self) -> Option<WidgetId> {
        self.inner.borrow().focus
    }

    pub fn focus_next(&self) -> Option<WidgetHandle> {
        self.focus_in_direction(true)
            .map(|id| WidgetHandle::new(self.clone(), id))
    }

    pub fn focus_previous(&self) -> Option<WidgetHandle> {
        self.focus_in_direction(false)
            .map(|id| WidgetHandle::new(self.clone(), id))
    }

    fn focus_in_direction(&self, forward: bool) -> Option<WidgetId> {
        let focusable = self.focusable_widget_ids();
        if focusable.is_empty() {
            return None;
        }

        let next_index = if let Some(current) = self.focused() {
            let index = focusable
                .iter()
                .position(|id| *id == current)
                .unwrap_or(if forward { 0 } else { focusable.len() - 1 });
            if forward {
                (index + 1) % focusable.len()
            } else {
                (index + focusable.len() - 1) % focusable.len()
            }
        } else if forward {
            0
        } else {
            focusable.len() - 1
        };
        Some(focusable[next_index])
    }

    fn focusable_widget_ids(&self) -> Vec<WidgetId> {
        let inner = self.inner.borrow();
        let mut result = Vec::new();
        fn visit(inner: &UiInner, id: WidgetId, result: &mut Vec<WidgetId>) {
            let Some(node) = inner.nodes.get(&id) else {
                return;
            };
            if let Some(widget) = inner.widgets.get(&id) {
                if (widget.as_any().is::<crate::Button>()
                    || widget.as_any().is::<crate::ComboBox>()
                    || widget.as_any().is::<crate::ListView>()
                    || widget.as_any().is::<crate::ScrollBar>())
                    && node.enabled
                    && node.visible
                {
                    result.push(id);
                }
            }
            for child in node.children.clone() {
                visit(inner, child, result);
            }
        }
        if let Some(root) = inner.root {
            visit(&inner, root, &mut result);
        }
        result
    }

    fn set_window_active(&self, active: bool) {
        let mut inner = self.inner.borrow_mut();
        if inner.window_active != active {
            inner.window_active = active;
            inner.dirty = true;
        }
    }

    fn capture(&self) -> Option<WidgetId> {
        self.inner.borrow().capture
    }

    fn set_capture(&self, capture: Option<WidgetId>) {
        self.inner.borrow_mut().capture = capture;
    }

    fn hovered(&self) -> Option<WidgetId> {
        self.inner.borrow().hovered
    }

    fn change_hover(&self, hovered: Option<WidgetId>) {
        let previous = self.hovered();
        if previous == hovered {
            return;
        }

        if let Some(previous) = previous {
            self.update_interaction(previous, |handler| handler.on_mouse_leave());
        }
        if let Some(next) = hovered {
            self.update_interaction(next, |handler| handler.on_mouse_enter());
        }

        let mut inner = self.inner.borrow_mut();
        inner.hovered = hovered;
        inner.dirty = true;
    }

    fn update_interaction(
        &self,
        id: WidgetId,
        change: impl FnOnce(&mut ButtonStateHandler) -> bool,
    ) {
        let mut inner = self.inner.borrow_mut();
        let Some(node) = inner.nodes.get_mut(&id) else {
            return;
        };
        if change(&mut node.interaction) {
            inner.dirty = true;
        }
    }

    fn press_interaction(&self, id: WidgetId) {
        self.update_interaction(id, |handler| {
            handler.on_mouse_click(true);
            true
        });
    }

    fn release_interaction(&self, id: WidgetId) {
        self.update_interaction(id, |handler| {
            handler.on_mouse_click(false);
            true
        });
    }

    fn parent_of(&self, id: WidgetId) -> Option<WidgetId> {
        self.inner
            .borrow()
            .nodes
            .get(&id)
            .and_then(|node| node.parent)
    }

    fn hit_test(&self, point: Point) -> Option<WidgetId> {
        let inner = self.inner.borrow();
        let root = inner.root?;
        Self::hit_test_node(&inner, root, point)
    }

    fn hit_test_node(inner: &UiInner, id: WidgetId, point: Point) -> Option<WidgetId> {
        let node = inner.nodes.get(&id)?;
        if !node.visible || !node.rect.contains(point) {
            return None;
        }
        for child in node.children.iter().rev() {
            if let Some(hit) = Self::hit_test_node(inner, *child, point) {
                return Some(hit);
            }
        }
        Some(id)
    }

    pub(crate) fn min_size_node(&self, id: WidgetId, available: Size) -> Size {
        let (mut widget, children) = {
            let mut inner = self.inner.borrow_mut();
            let Some(widget) = inner.widgets.remove(&id) else {
                return Size::ZERO;
            };
            let children = inner
                .nodes
                .get(&id)
                .map(|node| node.children.clone())
                .unwrap_or_default();
            (widget, children)
        };

        let mut ctx = MeasureCtx {
            ui: self.clone(),
            id,
            children,
        };
        let size = widget.min_size(&mut ctx, available);

        let mut inner = self.inner.borrow_mut();
        if inner.nodes.contains_key(&id) {
            inner.widgets.insert(id, widget);
        }
        size
    }

    pub(crate) fn size_policy_node(&self, id: WidgetId) -> SizePolicy2D {
        let inner = self.inner.borrow();
        inner
            .widgets
            .get(&id)
            .map(|widget| widget.size_policy())
            .unwrap_or_default()
    }

    pub(crate) fn margin_node(&self, id: WidgetId, theme: &Theme) -> Insets {
        let inner = self.inner.borrow();
        inner
            .widgets
            .get(&id)
            .map(|widget| widget.margin(theme))
            .unwrap_or_default()
    }

    pub(crate) fn measure_node(&self, id: WidgetId, available: Size) -> Size {
        let (mut widget, children) = {
            let mut inner = self.inner.borrow_mut();
            let Some(widget) = inner.widgets.remove(&id) else {
                return Size::ZERO;
            };
            let children = inner
                .nodes
                .get(&id)
                .map(|node| node.children.clone())
                .unwrap_or_default();
            (widget, children)
        };

        let mut ctx = MeasureCtx {
            ui: self.clone(),
            id,
            children,
        };
        let size = widget.measure(&mut ctx, available);

        let mut inner = self.inner.borrow_mut();
        if inner.nodes.contains_key(&id) {
            inner.widgets.insert(id, widget);
        }
        size
    }

    pub(crate) fn arrange_node(&self, id: WidgetId, bounds: Rect) {
        let (mut widget, children) = {
            let mut inner = self.inner.borrow_mut();
            let Some(widget) = inner.widgets.remove(&id) else {
                return;
            };
            if let Some(node) = inner.nodes.get_mut(&id) {
                node.rect = bounds;
            }
            let children = inner
                .nodes
                .get(&id)
                .map(|node| node.children.clone())
                .unwrap_or_default();
            (widget, children)
        };

        let mut ctx = ArrangeCtx {
            ui: self.clone(),
            id,
            children,
        };
        widget.arrange(&mut ctx, bounds);

        let mut inner = self.inner.borrow_mut();
        if inner.nodes.contains_key(&id) {
            inner.widgets.insert(id, widget);
        }
    }

    /// Runs layout if the tree is marked as needing it.
    pub fn layout(&self) {
        let (needs_layout, root) = {
            let inner = self.inner.borrow();
            (inner.needs_layout, inner.root)
        };
        if !needs_layout {
            return;
        }

        if let Some(root) = root {
            let bounds = Rect::from_size(self.size());
            self.arrange_node(root, bounds);
        }
        self.inner.borrow_mut().needs_layout = false;
    }

    /// Paints the whole tree into `rasterizer` (clearing it first).
    pub fn paint(&self, rasterizer: &mut dyn TextRasterizer) {
        self.layout();

        let inner = self.inner.borrow();
        rasterizer.begin_frame(inner.size);
        rasterizer.clear(self.theme().background());

        let Some(root) = inner.root else {
            return;
        };

        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            let Some(node) = inner.nodes.get(&id) else {
                continue;
            };
            if !node.visible {
                continue;
            }
            if let Some(widget) = inner.widgets.get(&id) {
                let mut ctx = PaintCtx {
                    rasterizer: &mut *rasterizer,
                    ui: self,
                    theme: &inner.theme,
                    bounds: node.rect,
                    state: WidgetState {
                        interaction: node.interaction.state,
                        focused: inner.focus == Some(id),
                        enabled: node.enabled,
                        window_active: inner.window_active,
                        checked: false,
                        layout_debug: inner.layout_debug,
                    },
                };
                widget.paint(&mut ctx);
            }
            for child in node.children.iter().rev() {
                stack.push(*child);
            }
        }
    }

    /// Routes a platform-independent event through the retained tree.
    pub fn dispatch(&self, event: Event) {
        match &event {
            Event::CursorMoved { position } => {
                let hit = self.hit_test(*position);
                self.change_hover(hit);
                if let Some(hit) = hit {
                    self.deliver_chain(hit, &event);
                }
            }
            Event::MouseDown { position, .. } => {
                if let Some(focus) = self.focused() {
                    let mut inner = self.inner.borrow_mut();
                    let contains = inner
                        .nodes
                        .get(&focus)
                        .map(|node| node.rect.contains(*position))
                        .unwrap_or(false);
                    if let Some(widget) = inner.widgets.get_mut(&focus) {
                        if let Some(combo) = widget.as_any_mut().downcast_mut::<crate::ComboBox>() {
                            if combo.is_open() && !contains {
                                combo.set_open(false);
                                inner.dirty = true;
                            }
                        }
                    }
                }
                let hit = self.hit_test(*position);
                self.change_hover(hit);
                let target = hit.and_then(|hit| self.deliver_chain(hit, &event));
                if let Some(target) = target {
                    self.press_interaction(target);
                    self.set_capture(Some(target));
                    self.set_focus(Some(target));
                } else {
                    self.set_focus(None);
                }
            }
            Event::MouseUp { position, .. } => {
                let hit = self.hit_test(*position);
                self.change_hover(hit);
                let target = self.capture().or(hit);
                if let Some(target) = target {
                    self.deliver_chain(target, &event);
                    self.release_interaction(target);
                }
                self.set_capture(None);
            }
            Event::CursorLeft => {
                self.change_hover(None);
            }
            Event::MouseWheel { position, .. } => {
                if let Some(hit) = self.hit_test(*position) {
                    self.deliver_chain(hit, &event);
                }
            }
            Event::KeyDown { key, modifiers } => {
                if *key == Key::Tab {
                    let next = if modifiers.shift {
                        self.focus_previous()
                    } else {
                        self.focus_next()
                    };
                    self.set_focus(next.map(|handle| handle.id()));
                } else if let Some(focus) = self.focused() {
                    self.deliver_chain(focus, &event);
                }
            }
            Event::KeyUp { .. } | Event::TextInput(_) => {
                if let Some(focus) = self.focused() {
                    self.deliver_chain(focus, &event);
                }
            }
            Event::FocusGained => self.set_window_active(true),
            Event::FocusLost => self.set_window_active(false),
        }
    }

    fn deliver_chain(&self, target: WidgetId, event: &Event) -> Option<WidgetId> {
        let mut current = Some(target);
        while let Some(id) = current {
            if self.deliver(id, event) {
                return Some(id);
            }
            current = self.parent_of(id);
        }
        None
    }

    fn deliver(&self, id: WidgetId, event: &Event) -> bool {
        let (mut widget, bounds, enabled, focused, interaction, window_active) = {
            let mut inner = self.inner.borrow_mut();
            let Some(widget) = inner.widgets.remove(&id) else {
                return false;
            };
            let Some(node) = inner.nodes.get(&id) else {
                inner.widgets.insert(id, widget);
                return false;
            };
            let bounds = node.rect;
            let enabled = node.enabled;
            let focused = inner.focus == Some(id);
            let interaction = node.interaction.state;
            let window_active = inner.window_active;
            (widget, bounds, enabled, focused, interaction, window_active)
        };

        let mut ctx = EventCtx {
            ui: self.clone(),
            id,
            bounds,
            state: WidgetState {
                interaction,
                focused,
                enabled,
                window_active,
                checked: false,
                layout_debug: self.inner.borrow().layout_debug,
            },
        };
        let handled = widget.event(&mut ctx, event);

        let mut inner = self.inner.borrow_mut();
        if inner.nodes.contains_key(&id) {
            inner.widgets.insert(id, widget);
        }
        handled
    }
}

fn clear_if_removed(slot: &mut Option<WidgetId>, removed: &[WidgetId]) {
    if let Some(value) = *slot {
        if removed.contains(&value) {
            *slot = None;
        }
    }
}

fn alloc_node(inner: &mut UiInner, widget: Box<dyn Widget>, parent: Option<WidgetId>) -> WidgetId {
    let id = WidgetId(inner.next_id);
    inner.next_id += 1;
    inner.nodes.insert(
        id,
        Node {
            parent,
            children: Vec::new(),
            rect: Rect::ZERO,
            visible: true,
            enabled: true,
            interaction: ButtonStateHandler::new(),
        },
    );
    inner.widgets.insert(id, widget);
    id
}

/// A cheap, cloneable reference to a widget in a [`Ui`].
#[derive(Clone)]
pub struct WidgetHandle {
    ui: Ui,
    id: WidgetId,
}

impl fmt::Debug for WidgetHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WidgetHandle")
            .field("id", &self.id)
            .field("valid", &self.is_valid())
            .finish()
    }
}

impl WidgetHandle {
    pub(crate) fn new(ui: Ui, id: WidgetId) -> Self {
        Self { ui, id }
    }

    pub fn id(&self) -> WidgetId {
        self.id
    }

    pub fn ui(&self) -> &Ui {
        &self.ui
    }

    pub fn is_valid(&self) -> bool {
        self.ui.is_valid(self.id)
    }

    /// Read-only access to the type-erased widget.
    pub fn with<R>(&self, f: impl FnOnce(&dyn Widget) -> R) -> Option<R> {
        let inner = self.ui.inner.borrow();
        inner.widgets.get(&self.id).map(|widget| f(widget.as_ref()))
    }

    /// Mutable access to the type-erased widget. Does not invalidate; see [`edit`](Self::edit).
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut dyn Widget) -> R) -> Option<R> {
        let mut inner = self.ui.inner.borrow_mut();
        inner
            .widgets
            .get_mut(&self.id)
            .map(|widget| f(widget.as_mut()))
    }

    pub fn with_concrete<T: Widget, R>(&self, f: impl FnOnce(&T) -> R) -> Option<R> {
        self.with(|widget| widget.as_any().downcast_ref::<T>().map(f))
            .flatten()
    }

    /// Mutates a concrete widget type and schedules a repaint + relayout.
    pub fn edit<T: Widget, R>(&self, f: impl FnOnce(&mut T) -> R) -> Option<R> {
        let result = {
            let mut inner = self.ui.inner.borrow_mut();
            let widget = inner.widgets.get_mut(&self.id)?;
            f(widget.as_any_mut().downcast_mut::<T>()?)
        };
        self.ui.invalidate();
        Some(result)
    }

    pub fn cast<T: Widget>(&self) -> Option<TypedWidgetHandle<T>> {
        let inner = self.ui.inner.borrow();
        let widget = inner.widgets.get(&self.id)?;
        if widget.as_any().is::<T>() {
            Some(TypedWidgetHandle {
                handle: self.clone(),
                _marker: PhantomData,
            })
        } else {
            None
        }
    }

    pub fn invalidate(&self) {
        self.ui.invalidate();
    }

    pub fn request_focus(&self) {
        self.ui.set_focus(Some(self.id));
    }

    pub fn is_focused(&self) -> bool {
        self.ui.focused() == Some(self.id)
    }

    pub fn set_visible(&self, visible: bool) {
        let mut inner = self.ui.inner.borrow_mut();
        if let Some(node) = inner.nodes.get_mut(&self.id) {
            if node.visible != visible {
                node.visible = visible;
                inner.dirty = true;
            }
        }
    }

    pub fn is_visible(&self) -> bool {
        self.ui
            .inner
            .borrow()
            .nodes
            .get(&self.id)
            .map(|node| node.visible)
            .unwrap_or(false)
    }

    pub fn set_enabled(&self, enabled: bool) {
        let mut inner = self.ui.inner.borrow_mut();
        if let Some(node) = inner.nodes.get_mut(&self.id) {
            if node.enabled != enabled {
                node.enabled = enabled;
                inner.dirty = true;
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.ui
            .inner
            .borrow()
            .nodes
            .get(&self.id)
            .map(|node| node.enabled)
            .unwrap_or(false)
    }

    pub fn parent(&self) -> Option<WidgetHandle> {
        self.ui
            .inner
            .borrow()
            .nodes
            .get(&self.id)
            .and_then(|node| node.parent)
            .map(|id| WidgetHandle::new(self.ui.clone(), id))
    }

    pub fn children(&self) -> Vec<WidgetHandle> {
        let inner = self.ui.inner.borrow();
        inner
            .nodes
            .get(&self.id)
            .map(|node| {
                node.children
                    .iter()
                    .map(|id| WidgetHandle::new(self.ui.clone(), *id))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn child_count(&self) -> usize {
        self.ui
            .inner
            .borrow()
            .nodes
            .get(&self.id)
            .map(|node| node.children.len())
            .unwrap_or(0)
    }

    pub fn bounds(&self) -> Rect {
        self.ui
            .inner
            .borrow()
            .nodes
            .get(&self.id)
            .map(|node| node.rect)
            .unwrap_or(Rect::ZERO)
    }

    pub fn remove(&self) {
        self.ui.remove(self);
    }
}

/// A [`WidgetHandle`] statically narrowed to a concrete widget type.
pub struct TypedWidgetHandle<T: Widget> {
    handle: WidgetHandle,
    _marker: PhantomData<fn() -> T>,
}

impl<T: Widget> Clone for TypedWidgetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T: Widget> fmt::Debug for TypedWidgetHandle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypedWidgetHandle")
            .field("id", &self.handle.id)
            .finish()
    }
}

impl<T: Widget> TypedWidgetHandle<T> {
    pub fn handle(&self) -> WidgetHandle {
        self.handle.clone()
    }

    pub fn id(&self) -> WidgetId {
        self.handle.id
    }

    pub fn is_valid(&self) -> bool {
        self.handle.is_valid()
    }

    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.handle
            .with_concrete(f)
            .expect("typed widget handle points to a removed or retyped widget")
    }

    pub fn edit<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        self.handle
            .edit(f)
            .expect("typed widget handle points to a removed or retyped widget")
    }

    pub fn invalidate(&self) {
        self.handle.invalidate();
    }

    pub fn request_focus(&self) {
        self.handle.request_focus();
    }

    pub fn is_focused(&self) -> bool {
        self.handle.is_focused()
    }

    pub fn set_visible(&self, visible: bool) {
        self.handle.set_visible(visible);
    }

    pub fn is_visible(&self) -> bool {
        self.handle.is_visible()
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.handle.set_enabled(enabled);
    }

    pub fn is_enabled(&self) -> bool {
        self.handle.is_enabled()
    }

    pub fn bounds(&self) -> Rect {
        self.handle.bounds()
    }

    pub fn parent(&self) -> Option<WidgetHandle> {
        self.handle.parent()
    }

    pub fn children(&self) -> Vec<WidgetHandle> {
        self.handle.children()
    }

    pub fn remove(&self) {
        self.handle.remove();
    }
}

fn process_auto_repeat_node(ui: &Ui, id: WidgetId, now: Instant, next: &mut Option<Instant>) {
    let (mut widget, node, children, focus, window_active, layout_debug) = {
        let mut inner = ui.inner.borrow_mut();
        let Some(widget) = inner.widgets.remove(&id) else {
            let children = inner
                .nodes
                .get(&id)
                .map(|node| node.children.clone())
                .unwrap_or_default();
            drop(inner);
            process_auto_repeat_children(ui, children, now, next);
            return;
        };
        let node = inner.nodes.get(&id).cloned();
        let children = node
            .as_ref()
            .map(|node| node.children.clone())
            .unwrap_or_default();
        let focus = inner.focus;
        let window_active = inner.window_active;
        let layout_debug = inner.layout_debug;
        (widget, node, children, focus, window_active, layout_debug)
    };

    let interaction = node
        .as_ref()
        .map(|node| node.interaction.state)
        .unwrap_or_default();
    let enabled = node.as_ref().map(|node| node.enabled).unwrap_or(true);
    let focused = focus == Some(id);
    let bounds = node.as_ref().map(|node| node.rect).unwrap_or(Rect::ZERO);
    let mut ctx = EventCtx {
        ui: ui.clone(),
        id,
        bounds,
        state: WidgetState {
            interaction,
            focused,
            enabled,
            window_active,
            checked: false,
            layout_debug,
        },
    };

    let mut handled_deadline = None;
    if let Some(button) = widget.as_any_mut().downcast_mut::<crate::Button>() {
        if let Some(deadline) = button.auto_repeat_tick(&mut ctx, now) {
            handled_deadline = Some(deadline);
        }
    }
    if let Some(scrollbar) = widget.as_any_mut().downcast_mut::<crate::ScrollBar>() {
        if let Some(deadline) = scrollbar.auto_repeat_tick(&mut ctx, now) {
            handled_deadline = Some(deadline);
        }
    }
    if let Some(list_view) = widget.as_any_mut().downcast_mut::<crate::ListView>() {
        if let Some(deadline) = list_view.auto_repeat_tick(&mut ctx, now) {
            handled_deadline = Some(deadline);
        }
    }

    {
        let mut inner = ui.inner.borrow_mut();
        if inner.nodes.contains_key(&id) {
            inner.widgets.insert(id, widget);
        }
    }

    if let Some(deadline) = handled_deadline {
        *next = Some(match *next {
            Some(existing) => existing.min(deadline),
            None => deadline,
        });
    }

    process_auto_repeat_children(ui, children, now, next);
}

fn process_auto_repeat_children(
    ui: &Ui,
    children: Vec<WidgetId>,
    now: Instant,
    next: &mut Option<Instant>,
) {
    for child in children {
        process_auto_repeat_node(ui, child, now, next);
    }
}
