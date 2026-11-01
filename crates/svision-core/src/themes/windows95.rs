use crate::{Color, Palette, Theme};

/// Windows 95 theme: classic silver/gray with squared corners. Provides light and dark variants.
pub struct Windows95Theme;

impl Windows95Theme {
    pub const WINDOWS95_DARK_PALETTE: Palette = Palette {
        background: Color::hex(0x1a1a1a),
        surface: Color::hex(0x262626),
        surface_hover: Color::hex(0x333333),
        surface_pressed: Color::hex(0x1a1a1a),
        border: Color::hex(0x3a3a3a),
        text: Color::hex(0xe8e8e8),
        text_muted: Color::hex(0x999999),
        accent: Color::hex(0x0827f5),
        accent_hover: Color::hex(0x1a3fd0),
        accent_pressed: Color::hex(0x0622a8),
        accent_text: Color::hex(0xffffff),
        focus: Color::hex(0x0827f5),
        focused: Color::hex(0xffffff),
    };

    pub const WINDOWS95_LIGHT_PALETTE: Palette = Palette {
        background: Color::hex(0xc0c0c0),
        surface: Color::hex(0xffffff),
        surface_hover: Color::hex(0xeef1f7),
        surface_pressed: Color::hex(0xe2e7f0),
        border: Color::hex(0xd3dae6),
        text: Color::hex(0x1b1b1b),
        text_muted: Color::hex(0x616161),
        //accent: Color::hex(0x0827f5),
        accent: Color::hex(0x0000A8),
        accent_hover: Color::hex(0xeef1f7),
        accent_pressed: Color::hex(0xeef1f7),
        accent_text: Color::hex(0xffffff),
        focus: Color::hex(0x0000A8),
        focused: Color::hex(0xffffff),
    };

    pub const DARK_RADIUS: f32 = 0.0;
    pub const LIGHT_RADIUS: f32 = 0.0;

    pub fn dark_theme() -> Theme {
        Theme {
            palette: Self::WINDOWS95_DARK_PALETTE,
            radius: Self::DARK_RADIUS,
        }
    }

    pub fn light_theme() -> Theme {
        Theme {
            palette: Self::WINDOWS95_LIGHT_PALETTE,
            radius: Self::LIGHT_RADIUS,
        }
    }
}
