use crate::{Color, FontSpec, Point, Rect, Size};

use crate::button_state::ButtonState;
use crate::painter::Painter;
use crate::text::TextRasterizer;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Palette {
    pub background: Color,
    pub surface: Color,
    pub surface_hover: Color,
    pub surface_pressed: Color,
    pub border: Color,
    pub frame: Color,
    pub frame_highlight: Color,
    pub frame_shadow: Color,

    pub text: Color,
    pub text_muted: Color,
    pub text_pressed: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_pressed: Color,
    pub accent_text: Color,
    pub focus: Color,
    pub focused: Color,
}

/// Visual constants shared by the built-in widgets.
///
/// Backends may use these or ignore them; widgets read the active theme through
/// their paint/measure contexts.
/// Style of a control frame drawn by [`Theme::draw_frame`].

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FrameKind {
    #[default]
    Normal,
    /// Default button: Windows 95 draws a triple frame.
    Default,
}

/// A tab of a tab strip, as drawn by [`Theme::draw_tabs`].
#[derive(Clone, Copy, Debug)]
pub struct TabSpec<'a> {
    /// The label of the tab.
    pub title: &'a str,
    /// The measured size of the label.
    pub size: Size,
    /// The rectangle the tab occupies in the strip.
    pub rect: Rect,
}

/// A check box, as drawn by [`Theme::draw_checkbox`].
#[derive(Clone, Copy, Debug)]
pub struct CheckboxSpec<'a> {
    /// The label drawn next to the box.
    pub label: &'a str,
    /// The measured size of the label.
    pub label_size: Size,
    /// Whether the box is checked.
    pub checked: bool,
    /// Whether the control accepts input.
    pub enabled: bool,
    /// The current pointer interaction state.
    pub interaction: ButtonState,
    /// Whether the control has keyboard focus.
    pub focused: bool,
}

/// A radio button, as drawn by [`Theme::draw_radio_button`].
#[derive(Clone, Copy, Debug)]
pub struct RadioButtonSpec<'a> {
    /// The label drawn next to the mark.
    pub label: &'a str,
    /// The measured size of the label.
    pub label_size: Size,
    /// Whether the button is selected.
    pub checked: bool,
    /// Whether the control accepts input.
    pub enabled: bool,
    /// The current pointer interaction state.
    pub interaction: ButtonState,
    /// Whether the control has keyboard focus.
    pub focused: bool,
}

/// The look of the user interface.
///
/// This is the base of the theme hierarchy. It declares how each control is
/// drawn; the concrete implementations live in the [`themes`](crate::themes)
/// module and are responsible for everything that differs between operating
/// systems — the shape of a check mark, whether a frame is raised or flat, and
/// so on.
///
/// Colours and metrics have a sensible default derived from the palette, so an
/// implementation only has to provide the parts that are actually special.
pub trait Theme {
    /// The colours of this theme.
    fn palette(&self) -> Palette;

    /// The corner radius used by the controls of this theme.
    ///
    /// A radius of `0` means square corners.
    fn radius(&self) -> f32;

    /// Draws the frame (border) of a control.
    fn draw_frame(
        &self,
        painter: &mut dyn Painter,
        rect: Rect,
        kind: FrameKind,
        focused: bool,
    );

    /// Draws a panel: the background of a content area.
    fn draw_panel(&self, rasterizer: &mut dyn TextRasterizer, rect: Rect);

    /// Draws a check box with an optional label to its right.
    fn draw_checkbox(
        &self,
        rasterizer: &mut dyn TextRasterizer,
        rect: Rect,
        spec: &CheckboxSpec<'_>,
    );

    /// Draws a radio button with an optional label to its right.
    fn draw_radio_button(
        &self,
        rasterizer: &mut dyn TextRasterizer,
        rect: Rect,
        spec: &RadioButtonSpec<'_>,
    );

    /// Draws a strip of tabs.
    fn draw_tabs(
        &self,
        rasterizer: &mut dyn TextRasterizer,
        tabs: &[TabSpec<'_>],
        selected: usize,
        hovered: Option<usize>,
        font: &FontSpec,
    );

    /// The background colour of a popup (a combo box drop-down, a menu).
    fn popup_surface(&self) -> Color {
        self.palette().surface
    }

    fn background(&self) -> Color {
        self.palette().background
    }

    fn surface(&self) -> Color {
        self.palette().surface
    }

    fn surface_hover(&self) -> Color {
        self.palette().surface_hover
    }

    fn surface_pressed(&self) -> Color {
        self.palette().surface_pressed
    }

    fn border(&self) -> Color {
        self.palette().border
    }

    fn text(&self) -> Color {
        self.palette().text
    }

    fn text_muted(&self) -> Color {
        self.palette().text_muted
    }

    fn text_pressed(&self) -> Color {
        self.palette().text_pressed
    }

    fn accent(&self) -> Color {
        self.palette().accent
    }

    fn accent_hover(&self) -> Color {
        self.palette().accent_hover
    }

    fn accent_pressed(&self) -> Color {
        self.palette().accent_pressed
    }

    fn accent_text(&self) -> Color {
        self.palette().accent_text
    }

    fn focus(&self) -> Color {
        self.palette().focus
    }

    fn focus_ring(&self) -> Color {
        self.focus()
    }

    fn focus_color(&self) -> Color {
        self.focus()
    }

    fn focused(&self) -> Color {
        self.palette().focused
    }

    fn font_size(&self) -> f32 {
        16.0
    }

    fn spacing(&self) -> f32 {
        10.0
    }

    fn padding(&self) -> f32 {
        14.0
    }
}

/// Draws a dotted (dashed) rectangle outline using short line segments.
///
/// This is used for focus indicators on Windows themes, where Windows 95
/// (zero corner radius) intentionally does not get a dotted frame.
pub(crate) fn draw_dotted_rect(
    rasterizer: &mut dyn Painter,
    rect: Rect,
    color: Color,
    width: f32,
) {
    let dash = 4.0;
    let gap = 4.0;

    // Top and bottom edges.
    let mut x = rect.x;
    while x < rect.right() {
        let x1 = x;
        let x2 = (x + dash).min(rect.right());
        rasterizer.draw_line(Point::new(x1, rect.y), Point::new(x2, rect.y), color, width);
        rasterizer.draw_line(
            Point::new(x1, rect.bottom()),
            Point::new(x2, rect.bottom()),
            color,
            width,
        );
        x += dash + gap;
    }

    // Left and right edges.
    let mut y = rect.y;
    while y < rect.bottom() {
        let y1 = y;
        let y2 = (y + dash).min(rect.bottom());
        rasterizer.draw_line(Point::new(rect.x, y1), Point::new(rect.x, y2), color, width);
        rasterizer.draw_line(
            Point::new(rect.right(), y1),
            Point::new(rect.right(), y2),
            color,
            width,
        );
        y += dash + gap;
    }
}
