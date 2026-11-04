use crate::{Color, Palette, Theme};

/// Windows 11 theme: dark charcoal with subtle accent and rounded corners.
pub struct Windows11Theme;

impl Windows11Theme {
    pub const WINDOWS11_DARK_PALETTE: Palette = Palette {
        background: Color::hex(0x121212),
        surface: Color::hex(0x1c1c1c),
        surface_hover: Color::hex(0x2a2a2a),
        surface_pressed: Color::hex(0x0c0c0c),
        border: Color::hex(0x3a3a3a),
        frame: Color::hex(0x3a3a3a),
        frame_highlight: Color::hex(0x3a3a3a),
        frame_shadow: Color::hex(0x3a3a3a),
        text: Color::hex(0xf3f3f3),
        text_muted: Color::hex(0xa0a0a0),
        accent: Color::hex(0x0078d4),
        accent_hover: Color::hex(0x1e88e5),
        accent_pressed: Color::hex(0x005a9e),
        accent_text: Color::hex(0xffffff),
        focus: Color::hex(0x0078d4),
        focused: Color::hex(0xffffff),
    };

    pub const WINDOWS11_LIGHT_PALETTE: Palette = Palette {
        background: Color::hex(0xf5f5f5),
        surface: Color::hex(0xffffff),
        surface_hover: Color::hex(0xf0f0f0),
        surface_pressed: Color::hex(0xe5e5e5),
        border: Color::hex(0xd0d0d0),
        frame: Color::hex(0xd0d0d0),
        frame_highlight: Color::hex(0xd0d0d0),
        frame_shadow: Color::hex(0xd0d0d0),
        text: Color::hex(0x1b1b1b),
        text_muted: Color::hex(0x616161),
        accent: Color::hex(0x0078d4),
        accent_hover: Color::hex(0x1e88e5),
        accent_pressed: Color::hex(0x005a9e),
        accent_text: Color::hex(0xffffff),
        focus: Color::hex(0x0078d4),
        focused: Color::hex(0xffffff),
    };

    pub const DARK_RADIUS: f32 = 5.0;
    pub const LIGHT_RADIUS: f32 = 5.0;

    pub fn dark_theme() -> Theme {
        Theme {
            palette: Self::WINDOWS11_DARK_PALETTE,
            radius: Self::DARK_RADIUS,
        }
    }

    pub fn light_theme() -> Theme {
        Theme {
            palette: Self::WINDOWS11_LIGHT_PALETTE,
            radius: Self::LIGHT_RADIUS,
        }
    }
}
