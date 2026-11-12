pub mod windows11;
pub mod windows95;

use std::rc::Rc;

use crate::{Palette, Theme};

/// A visual style: a family of related themes.
///
/// A style picks the implementation and the appearance. The concrete themes
/// live next to this module and own everything that differs between operating
/// systems.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Style {
    /// The modern Fluent look: rounded corners and a blue accent.
    Windows11,
    /// The classic look: square corners, silver surfaces and navy accent.
    Windows95,
}

impl Style {
    /// The default palette for this style in the requested appearance.
    pub fn palette(self, dark: bool) -> Palette {
        match self {
            Style::Windows11 => {
                if dark {
                    windows11::DARK_PALETTE
                } else {
                    windows11::LIGHT_PALETTE
                }
            }
            Style::Windows95 => {
                if dark {
                    windows95::DARK_PALETTE
                } else {
                    windows95::LIGHT_PALETTE
                }
            }
        }
    }

    /// The corner radius used by this style.
    pub fn radius(self) -> f32 {
        match self {
            Style::Windows11 => windows11::RADIUS,
            Style::Windows95 => windows95::RADIUS,
        }
    }

    /// The theme for this style in the requested appearance.
    pub fn theme(self, dark: bool) -> Rc<dyn Theme> {
        match self {
            Style::Windows11 => {
                if dark {
                    Rc::new(windows11::Windows11Theme::dark())
                } else {
                    Rc::new(windows11::Windows11Theme::light())
                }
            }
            Style::Windows95 => {
                if dark {
                    Rc::new(windows95::Windows95Theme::dark())
                } else {
                    Rc::new(windows95::Windows95Theme::light())
                }
            }
        }
    }
}

/// The theme used when none is requested: Windows 11, light.
pub fn default_theme() -> Rc<dyn Theme> {
    Style::Windows11.theme(false)
}
