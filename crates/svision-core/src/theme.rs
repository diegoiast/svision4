use crate::Color;

use crate::themes;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Palette {
    pub background: Color,
    pub surface: Color,
    pub surface_hover: Color,
    pub surface_pressed: Color,
    pub border: Color,

    pub text: Color,
    pub text_muted: Color,
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
#[derive(Clone, Debug)]
pub struct Theme {
    pub palette: Palette,
    pub radius: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}

impl Theme {
    pub fn dark() -> Self {
        themes::dark::DarkTheme::dark_theme()
    }

    pub fn light() -> Self {
        themes::light::LightTheme::light_theme()
    }

    pub fn windows95() -> Self {
        themes::windows95::Windows95Theme::dark_theme()
    }

    pub fn windows95_light() -> Self {
        themes::windows95::Windows95Theme::light_theme()
    }

    pub fn windows11() -> Self {
        themes::windows11::Windows11Theme::dark_theme()
    }

    pub fn palette(&self) -> Palette {
        self.palette
    }

    pub fn background(&self) -> Color {
        self.palette.background
    }

    pub fn surface(&self) -> Color {
        self.palette.surface
    }

    /// Popup background: white on Windows 95 theme, surface color otherwise.
    pub fn popup_surface(&self) -> Color {
        if self.radius == 0.0 {
            Color::hex(0xffffff)
        } else {
            self.palette.surface
        }
    }

    pub fn surface_hover(&self) -> Color {
        self.palette.surface_hover
    }

    pub fn surface_pressed(&self) -> Color {
        self.palette.surface_pressed
    }

    pub fn border(&self) -> Color {
        self.palette.border
    }

    pub fn text(&self) -> Color {
        self.palette.text
    }

    pub fn text_muted(&self) -> Color {
        self.palette.text_muted
    }

    pub fn accent(&self) -> Color {
        self.palette.accent
    }

    pub fn accent_hover(&self) -> Color {
        self.palette.accent_hover
    }

    pub fn accent_pressed(&self) -> Color {
        self.palette.accent_pressed
    }

    pub fn accent_text(&self) -> Color {
        self.palette.accent_text
    }

    pub fn focus(&self) -> Color {
        self.palette.focus
    }

    pub fn focus_ring(&self) -> Color {
        self.focus()
    }

    pub fn focus_color(&self) -> Color {
        self.focus()
    }

    pub fn focused(&self) -> Color {
        self.palette.focused
    }

    pub fn font_size(&self) -> f32 {
        16.0
    }

    pub fn radius(&self) -> f32 {
        self.radius
    }

    pub fn spacing(&self) -> f32 {
        10.0
    }

    pub fn padding(&self) -> f32 {
        14.0
    }
}
