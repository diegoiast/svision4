use crate::{
    Color, FontSpec, Point, Rect,
    button_state::ButtonState,
    painter::Painter,
    text::TextRasterizer,
    theme::{CheckboxSpec, FrameKind, Palette, RadioButtonSpec, TabSpec, Theme, draw_dotted_rect},
};

/// The Windows 11 light palette.
pub const LIGHT_PALETTE: Palette = Palette {
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
    text_pressed: Color::hex(0x000000),
    accent: Color::hex(0x0078d4),
    accent_hover: Color::hex(0x1e88e5),
    accent_pressed: Color::hex(0x005a9e),
    accent_text: Color::hex(0xffffff),
    focus: Color::hex(0x0078d4),
    focused: Color::hex(0xffffff),
};

/// The Windows 11 dark palette.
pub const DARK_PALETTE: Palette = Palette {
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
    text_pressed: Color::hex(0xffffff),
    accent: Color::hex(0x0078d4),
    accent_hover: Color::hex(0x1e88e5),
    accent_pressed: Color::hex(0x005a9e),
    accent_text: Color::hex(0xffffff),
    focus: Color::hex(0x0078d4),
    focused: Color::hex(0xffffff),
};

/// Windows 11 uses slightly tighter corners than the modern default.
pub const RADIUS: f32 = 5.0;

/// The Windows 11 theme: flat, rounded, accent-coloured controls.
#[derive(Clone, Copy, Debug)]
pub struct Windows11Theme {
    palette: Palette,
}

impl Windows11Theme {
    /// The Windows 11 theme in the light appearance.
    pub fn light() -> Self {
        Self {
            palette: LIGHT_PALETTE,
        }
    }

    /// The Windows 11 theme in the dark appearance.
    pub fn dark() -> Self {
        Self {
            palette: DARK_PALETTE,
        }
    }

    /// The Windows 11 theme with an explicit palette.
    pub fn with_palette(palette: Palette) -> Self {
        Self { palette }
    }
}

impl Theme for Windows11Theme {
    fn palette(&self) -> Palette {
        self.palette
    }

    fn radius(&self) -> f32 {
        RADIUS
    }

    fn draw_frame(
        &self,
        painter: &mut dyn Painter,
        rect: Rect,
        _kind: FrameKind,
        focused: bool,
    ) {
        // Windows / modern: a single lined frame. When focused, the frame uses
        // the focus colour.
        let color = if focused {
            self.focus()
        } else {
            self.palette().frame
        };
        painter.stroke_round_rect(rect, self.radius(), color, 1.0);
    }

    fn draw_panel(&self, rasterizer: &mut dyn TextRasterizer, rect: Rect) {
        rasterizer.fill_round_rect(rect, self.radius(), self.surface());
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

        // Background: accent when checked, surface/hover/pressed otherwise.
        // A disabled box is drawn in muted tones so the whole control reads as
        // disabled, not just its label.
        let bg = if !enabled {
            if checked {
                self.surface_pressed()
            } else {
                self.surface()
            }
        } else if checked {
            match interaction {
                ButtonState::ClickedInside => self.accent_pressed(),
                ButtonState::Hovered => self.accent_hover(),
                _ => self.accent(),
            }
        } else {
            match interaction {
                ButtonState::ClickedInside => self.surface_pressed(),
                ButtonState::Hovered => self.surface_hover(),
                _ => self.surface(),
            }
        };
        rasterizer.fill_rect(box_rect, bg);

        // Border: accent when checked/hovered, border otherwise.
        let border_color = if !enabled {
            self.border()
        } else {
            match interaction {
                ButtonState::ClickedInside => self.accent_pressed(),
                ButtonState::Hovered => self.accent_hover(),
                _ => {
                    if checked {
                        self.accent()
                    } else {
                        self.border()
                    }
                }
            }
        };
        rasterizer.stroke_rect(box_rect, border_color, 1.0);

        // A check mark inside the square when checked.
        if checked {
            let cx = rect.x + side / 2.0;
            let cy = rect.y + side / 2.0;
            let p1 = Point::new(cx - side * 0.28, cy - side * 0.10);
            let p2 = Point::new(cx - side * 0.08, cy + side * 0.22);
            let p3 = Point::new(cx + side * 0.28, cy - side * 0.30);
            let mark_color = if enabled {
                self.accent_text()
            } else {
                self.text_muted()
            };
            rasterizer.draw_line(p1, p2, mark_color, 2.0);
            rasterizer.draw_line(p2, p3, mark_color, 2.0);
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

        // Dotted focus frame on Windows themes (non-zero radius).
        if enabled && focused {
            draw_dotted_rect(rasterizer, rect, self.focus(), 1.0);
        }
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

        // Outer ring: accent when checked/hovered/pressed, border otherwise.
        let outer_color = if !enabled {
            self.border()
        } else {
            match interaction {
                ButtonState::ClickedInside => self.accent_pressed(),
                ButtonState::Hovered => self.accent_hover(),
                _ => {
                    if checked {
                        self.accent()
                    } else {
                        self.border()
                    }
                }
            }
        };
        rasterizer.fill_ellipse(outer, outer_color);

        // Inner fill: accent when checked, surface/hover/pressed otherwise.
        let inner_bg = if !enabled {
            self.surface()
        } else if checked {
            match interaction {
                ButtonState::ClickedInside => self.accent_pressed(),
                ButtonState::Hovered => self.accent_hover(),
                _ => self.accent(),
            }
        } else {
            match interaction {
                ButtonState::ClickedInside => self.surface_pressed(),
                ButtonState::Hovered => self.surface_hover(),
                _ => self.surface(),
            }
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

        // Dotted focus frame on Windows themes (non-zero radius).
        if enabled && focused {
            draw_dotted_rect(rasterizer, rect, self.focus(), 1.0);
        }
    }

    fn draw_tabs(
        &self,
        rasterizer: &mut dyn TextRasterizer,
        tabs: &[TabSpec<'_>],
        selected: usize,
        hovered: Option<usize>,
        font: &FontSpec,
    ) {
        // Windows 11 uses flat, connected tabs: the strip has no background of
        // its own, the selected tab is marked by a bold label with an accent
        // underline, and the inactive tabs are muted and separated by a
        // hairline.
        let bold = FontSpec::new(font.size).bold(true);
        for (index, tab) in tabs.iter().enumerate() {
            let is_selected = index == selected;
            let is_hovered = hovered == Some(index);

            let text_x = tab.rect.x + (tab.rect.width - tab.size.width) * 0.5;
            let text_y = tab.rect.y + (tab.rect.height - tab.size.height) * 0.5;

            // Hover is a soft tint behind the label only.
            if is_hovered && !is_selected {
                let hover = Rect::new(
                    text_x - 8.0,
                    tab.rect.y + 4.0,
                    tab.size.width + 16.0,
                    tab.rect.height - 8.0,
                );
                rasterizer.fill_round_rect(hover, self.radius(), self.surface_hover());
            }

            // The selected label is bold, the inactive ones are muted.
            let label_font = if is_selected { &bold } else { font };
            let color = if is_selected || is_hovered {
                self.text()
            } else {
                self.text_muted()
            };
            rasterizer.draw_text(tab.title, Point::new(text_x, text_y), label_font, color);

            // Accent indicator under the selected tab, hugging the label.
            if is_selected {
                let indicator = Rect::new(text_x, tab.rect.bottom() - 3.0, tab.size.width, 3.0);
                rasterizer.fill_round_rect(indicator, 1.5, self.accent());
            }

            // Hairline separator between the inactive tabs.
            if !is_selected {
                if let Some(next) = tabs.get(index + 1) {
                    let separator_x = (tab.rect.right() + next.rect.x) * 0.5;
                    rasterizer.draw_line(
                        Point::new(separator_x, tab.rect.y + 10.0),
                        Point::new(separator_x, tab.rect.bottom() - 10.0),
                        self.border(),
                        1.0,
                    );
                }
            }
        }
    }
}
