use crate::{Color, Palette, Theme};

/// Dark theme: deep grays with blue accent. Provides dark and light variants.
pub struct DarkTheme;

impl DarkTheme {
    pub const DARK_PALETTE: Palette = Palette {
        background: Color::hex(0x0f1115),
        surface: Color::hex(0x191d24),
        surface_hover: Color::hex(0x222834),
        surface_pressed: Color::hex(0x141820),
        border: Color::hex(0x2b3240),
        text: Color::hex(0xe8edf5),
        text_muted: Color::hex(0x95a0b3),
        accent: Color::hex(0x4f8cff),
        accent_hover: Color::hex(0x1e88e5),
        accent_pressed: Color::hex(0x3d74df),
        accent_text: Color::hex(0xffffff),
        focus: Color::hex(0x6ea8ff),
        focused: Color::hex(0xffffff),
    };

    pub const LIGHT_PALETTE: Palette = Palette {
        background: Color::hex(0xf3f5f9),
        surface: Color::hex(0xffffff),
        surface_hover: Color::hex(0xeef1f7),
        surface_pressed: Color::hex(0xe2e7f0),
        border: Color::hex(0xd3dae6),
        text: Color::hex(0x1b1b1b),
        text_muted: Color::hex(0x616161),
        accent: Color::hex(0x0078d4),
        accent_hover: Color::hex(0x1e88e5),
        accent_pressed: Color::hex(0x005a9e),
        accent_text: Color::hex(0xffffff),
        focus: Color::hex(0x005fb8),
        focused: Color::hex(0xffffff),
    };

    pub const DARK_RADIUS: f32 = 9.0;
    pub const LIGHT_RADIUS: f32 = 9.0;

    pub fn dark_theme() -> Theme {
        Theme {
            palette: Self::DARK_PALETTE,
            radius: Self::DARK_RADIUS,
        }
    }

    pub fn light_theme() -> Theme {
        Theme {
            palette: Self::LIGHT_PALETTE,
            radius: Self::LIGHT_RADIUS,
        }
    }
}
