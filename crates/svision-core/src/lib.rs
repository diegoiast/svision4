//! `svision-core` is a small retained-mode GUI framework.
//!
//! The crate is deliberately platform and rasterizer agnostic:
//!
//! * [`Widget`] is the retained object model. Widgets live in a tree owned by [`Ui`].
//! * [`WidgetHandle`] is a cheap, cloneable reference to a widget (the Qt-style part).
//! * [`Painter`] is the pluggable drawing interface for primitives.
//! * [`TextRasterizer`] rasterizes text on top of a [`Painter`].
//! * [`GraphicsBackend`] is the pluggable presentation interface (softbuffer, GDI+, OpenGL, ...).
//!
//! A host crate (`svision-winit`) drives the event loop and feeds events into [`Ui`].
//!
//! ```no_run
//! use svision_core::prelude::*;
//!
//! let ui = Ui::new();
//! let root = ui.set_root(Column::new().padding(24.0).spacing(12.0));
//! let label = ui.add_child(&root, Label::new("0"));
//! let button = ui.add_child(
//!     &root,
//!     Button::new("Increment").on_click(move |_ctx| {
//!         label.edit(|label: &mut Label| label.set_text("1"));
//!     }),
//! );
//! ```

mod backend;
mod button_state;
mod color;
mod event;
mod geometry;
mod painter;
mod text;
mod theme;
mod themes;
mod ui;
mod widget;
mod widgets;

pub use backend::GraphicsBackend;
pub use button_state::{ButtonState, ButtonStateHandler};
pub use color::Color;
pub use event::{Event, Key, Modifiers, MouseButton};
pub use geometry::{Insets, Point, Rect, Size};
pub use painter::Painter;
pub use text::{ApproxTextEngine, FontSpec, TextEngine, TextRasterizer};
pub use theme::{FrameKind, Palette, Theme};
pub use ui::{TypedWidgetHandle, Ui, WidgetHandle, WidgetId};
pub use widget::{
    ArrangeCtx, EventCtx, MeasureCtx, PaintCtx, SizePolicy, SizePolicy2D, Widget, WidgetState,
};
pub use widgets::{
    Align, Button, Checkbox, Column, ComboBox, CrossAlign, HBoxLayout, Input, Label, ListView,
    Orientation, RadioButton, RadioButtonGroup, Row, ScrollBar, Spacer, TextAlign, VBoxLayout,
    checkbox, radiobutton,
};

/// Implements the `Any` escape hatches required by [`Widget`].
///
/// Every widget implementation needs these two methods; this macro keeps that
/// boilerplate out of user code.
#[macro_export]
macro_rules! impl_widget_any {
    () => {
        fn as_any(&self) -> &dyn ::std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any {
            self
        }
    };
}

/// Common imports.
pub mod prelude {
    pub use crate::{
        Align, ArrangeCtx, Button, ButtonState, ButtonStateHandler, Checkbox, Color, Column,
        ComboBox, CrossAlign, Event, EventCtx, FontSpec, FrameKind, GraphicsBackend, HBoxLayout,
        Insets, Key, Label, ListView, MeasureCtx, Modifiers, MouseButton, Orientation, PaintCtx,
        Painter, Palette, Point, RadioButton, RadioButtonGroup, Rect, Row, ScrollBar, Size,
        SizePolicy, SizePolicy2D, Spacer, TextAlign, TextEngine, TextRasterizer, Theme,
        TypedWidgetHandle, Ui, VBoxLayout, Widget, WidgetHandle, WidgetId, WidgetState, checkbox,
        impl_widget_any, radiobutton,
    };
}
