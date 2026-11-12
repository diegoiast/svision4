use crate::{
    Color, FontSpec, Point, Rect,
    button_state::ButtonState,
    painter::Painter,
    text::TextRasterizer,
    theme::{CheckboxSpec, FrameKind, Palette, RadioButtonSpec, TabSpec, Theme},
};

/// The Windows 95 light palette: silver surfaces, hard edges, navy accent.
pub const LIGHT_PALETTE: Palette = Palette {
    background: Color::hex(0xc0c0c0),
    surface: Color::hex(0xffffff),
    surface_hover: Color::hex(0xd4d0c8),
    surface_pressed: Color::hex(0xb0aca4),
    border: Color::hex(0x808080),
    frame: Color::hex(0x808080),
    frame_highlight: Color::hex(0xffffff),
    frame_shadow: Color::hex(0x808080),
    text: Color::hex(0x1b1b1b),
    text_muted: Color::hex(0x616161),
    text_pressed: Color::hex(0x000000),
    accent: Color::hex(0x0000a8),
    accent_hover: Color::hex(0x3a6ea5),
    accent_pressed: Color::hex(0x000080),
    accent_text: Color::hex(0xffffff),
    focus: Color::hex(0x0000a8),
    focused: Color::hex(0xffffff),
};

/// The Windows 95 dark palette: the high-contrast black scheme.
///
/// Same hard edges and 3D bevels as the light scheme, but with dark grey
/// surfaces and light text.
pub const DARK_PALETTE: Palette = Palette {
    background: Color::hex(0x1a1a1a),
    surface: Color::hex(0x262626),
    surface_hover: Color::hex(0x333333),
    surface_pressed: Color::hex(0x1a1a1a),
    border: Color::hex(0x3a3a3a),
    frame: Color::hex(0x3a3a3a),
    frame_highlight: Color::hex(0x808080),
    frame_shadow: Color::hex(0x000000),
    text: Color::hex(0xe8e8e8),
    text_muted: Color::hex(0x999999),
    text_pressed: Color::hex(0xffffff),
    accent: Color::hex(0x0827f5),
    accent_hover: Color::hex(0x1a3fd0),
    accent_pressed: Color::hex(0x0622a8),
    accent_text: Color::hex(0xffffff),
    focus: Color::hex(0x0827f5),
    focused: Color::hex(0xffffff),
};

/// Windows 95 has square corners.
pub const RADIUS: f32 = 0.0;

/// The Windows 95 theme: raised, square, classic controls.
#[derive(Clone, Copy, Debug)]
pub struct Windows95Theme {
    palette: Palette,
}

impl Windows95Theme {
    /// The Windows 95 theme in the light appearance.
    pub fn light() -> Self {
        Self {
            palette: LIGHT_PALETTE,
        }
    }

    /// The Windows 95 theme in the dark (high contrast black) appearance.
    pub fn dark() -> Self {
        Self {
            palette: DARK_PALETTE,
        }
    }

    /// The Windows 95 theme with an explicit palette.
    pub fn with_palette(palette: Palette) -> Self {
        Self { palette }
    }
}

impl Theme for Windows95Theme {
    fn palette(&self) -> Palette {
        self.palette
    }

    fn radius(&self) -> f32 {
        RADIUS
    }

    fn popup_surface(&self) -> Color {
        // Classic popups were plain white.
        Color::hex(0xffffff)
    }

    fn draw_frame(
        &self,
        painter: &mut dyn Painter,
        rect: Rect,
        kind: FrameKind,
        _focused: bool,
    ) {
        // Windows 95: raised border — top/left edges are light (highlight),
        // bottom/right edges are dark (shadow). Double frame for normal
        // buttons, triple frame for default buttons.
        let width = 1.0;
        let layers = match kind {
            FrameKind::Normal => 2,
            FrameKind::Default => 3,
        };
        for i in 0..layers {
            let inset = i as f32 * width;
            let r = rect.inset_all(inset);
            // Default buttons draw their outermost frame in black.
            let black = kind == FrameKind::Default && i == 0;
            let color = if black {
                Color::BLACK
            } else {
                self.palette().frame_highlight
            };
            painter.draw_line(
                Point::new(r.x, r.y),
                Point::new(r.right(), r.y),
                color,
                width,
            );
            painter.draw_line(
                Point::new(r.x, r.y),
                Point::new(r.x, r.bottom()),
                color,
                width,
            );
            painter.draw_line(
                Point::new(r.x, r.bottom()),
                Point::new(r.right(), r.bottom()),
                if black {
                    Color::BLACK
                } else {
                    self.palette().frame_shadow
                },
                width,
            );
            painter.draw_line(
                Point::new(r.right(), r.y),
                Point::new(r.right(), r.bottom()),
                if black {
                    Color::BLACK
                } else {
                    self.palette().frame_shadow
                },
                width,
            );
        }
    }

    fn draw_panel(&self, rasterizer: &mut dyn TextRasterizer, rect: Rect) {
        // Classic panels have square corners.
        rasterizer.fill_rect(rect, self.surface());
    }

    fn draw_checkbox(
        &self,
        rasterizer: &mut dyn TextRasterizer,
        rect: Rect,
        spec: &CheckboxSpec<'_>,
    ) {
        let CheckboxSpec {
            label,
            label_size,
            checked,
            enabled,
            interaction,
            focused,
        } = *spec;

        let gap = 4.0;
        let side = rect.height.min(rect.width).min(28.0);
        let x = rect.x;
        let y = rect.y + (rect.height - side) / 2.0;
        let box_rect = Rect::new(x, y, side, side);

        // Classic check boxes are sunken: the top/left edges are dark and the
        // bottom/right edges are light.
        let bg = if checked {
            self.surface()
        } else {
            match interaction {
                ButtonState::ClickedInside => self.surface_pressed(),
                ButtonState::Hovered => self.surface_hover(),
                _ => self.surface(),
            }
        };
        rasterizer.fill_rect(box_rect, bg);

        let shadow = self.palette().frame_shadow;
        let highlight = self.palette().frame_highlight;
        rasterizer.draw_line(
            Point::new(box_rect.x, box_rect.y),
            Point::new(box_rect.right(), box_rect.y),
            shadow,
            1.0,
        );
        rasterizer.draw_line(
            Point::new(box_rect.x, box_rect.y),
            Point::new(box_rect.x, box_rect.bottom()),
            shadow,
            1.0,
        );
        rasterizer.draw_line(
            Point::new(box_rect.x, box_rect.bottom()),
            Point::new(box_rect.right(), box_rect.bottom()),
            highlight,
            1.0,
        );
        rasterizer.draw_line(
            Point::new(box_rect.right(), box_rect.y),
            Point::new(box_rect.right(), box_rect.bottom()),
            highlight,
            1.0,
        );

        // Windows 95 marks a checked box with a cross.
        if checked {
            let cx = rect.x + side / 2.0;
            let cy = rect.y + side / 2.0;
            let arm = side * 0.24;
            let mark_color = if enabled {
                self.text()
            } else {
                self.text_muted()
            };
            rasterizer.draw_line(
                Point::new(cx - arm, cy - arm),
                Point::new(cx + arm, cy + arm),
                mark_color,
                2.0,
            );
            rasterizer.draw_line(
                Point::new(cx + arm, cy - arm),
                Point::new(cx - arm, cy + arm),
                mark_color,
                2.0,
            );
        }

        // The label text to the right of the square.
        if !label.is_empty() {
            let font = FontSpec::new(self.font_size());
            let text_x = rect.x + side + gap;
            let text_y = rect.y + (rect.height - label_size.height) / 2.0;
            let color = if enabled {
                self.text()
            } else {
                self.text_muted()
            };
            rasterizer.draw_text(label, Point::new(text_x, text_y), &font, color);
        }

        // Windows 95 does not use a dotted focus frame.
        let _ = focused;
    }

    fn draw_radio_button(
        &self,
        rasterizer: &mut dyn TextRasterizer,
        rect: Rect,
        spec: &RadioButtonSpec<'_>,
    ) {
        let RadioButtonSpec {
            label,
            label_size,
            checked,
            enabled,
            interaction,
            focused,
        } = *spec;

        let gap = 8.0;

        // Circle mark on the left.
        let side = rect.height.min(rect.width).min(24.0);
        let outer = Rect::new(rect.x, rect.y + (rect.height - side) / 2.0, side, side);
        let pad = side * 0.30;
        let inner = Rect::new(
            rect.x + pad,
            rect.y + (rect.height - side) / 2.0 + pad,
            side - pad * 2.0,
            side - pad * 2.0,
        );

        // The outer ring is sunken, the inner dot marks the selection.
        let outer_color = if checked {
            self.accent()
        } else {
            self.palette().frame_shadow
        };
        rasterizer.fill_ellipse(outer, outer_color);

        let inner_bg = if checked {
            match interaction {
                ButtonState::ClickedInside => self.accent_pressed(),
                ButtonState::Hovered => self.accent_hover(),
                _ => self.accent(),
            }
        } else {
            self.surface()
        };
        rasterizer.fill_ellipse(inner, inner_bg);

        // The label text to the right of the circle.
        if !label.is_empty() {
            let font = FontSpec::new(self.font_size());
            let text_x = rect.x + side + gap;
            let text_y = rect.y + (rect.height - label_size.height) / 2.0;
            let color = if enabled {
                self.text()
            } else {
                self.text_muted()
            };
            rasterizer.draw_text(label, Point::new(text_x, text_y), &font, color);
        }

        // Windows 95 does not use a dotted focus frame.
        let _ = focused;
    }

    fn draw_tabs(
        &self,
        rasterizer: &mut dyn TextRasterizer,
        tabs: &[TabSpec<'_>],
        selected: usize,
        hovered: Option<usize>,
        font: &FontSpec,
    ) {
        // Classic tabs are raised frames that merge into the content below.
        for (index, tab) in tabs.iter().enumerate() {
            let is_selected = index == selected;
            let is_hovered = hovered == Some(index);

            rasterizer.fill_rect(tab.rect, self.surface());

            // Raised edges: light on top/left, dark on bottom/right. The
            // selected tab leaves its bottom open so it joins the content.
            let highlight = self.palette().frame_highlight;
            let shadow = self.palette().frame_shadow;
            rasterizer.draw_line(
                Point::new(tab.rect.x, tab.rect.y),
                Point::new(tab.rect.right(), tab.rect.y),
                highlight,
                1.0,
            );
            rasterizer.draw_line(
                Point::new(tab.rect.x, tab.rect.y),
                Point::new(tab.rect.x, tab.rect.bottom()),
                highlight,
                1.0,
            );
            rasterizer.draw_line(
                Point::new(tab.rect.right(), tab.rect.y),
                Point::new(tab.rect.right(), tab.rect.bottom()),
                shadow,
                1.0,
            );
            if !is_selected {
                rasterizer.draw_line(
                    Point::new(tab.rect.x, tab.rect.bottom()),
                    Point::new(tab.rect.right(), tab.rect.bottom()),
                    shadow,
                    1.0,
                );
            }

            let text_x = tab.rect.x + (tab.rect.width - tab.size.width) * 0.5;
            let text_y = tab.rect.y + (tab.rect.height - tab.size.height) * 0.5;
            let color = if is_selected || is_hovered {
                self.text()
            } else {
                self.text_muted()
            };
            rasterizer.draw_text(tab.title, Point::new(text_x, text_y), font, color);
        }
    }
}
