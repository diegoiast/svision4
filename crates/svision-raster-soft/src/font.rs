use std::{env, fs, path::PathBuf, rc::Rc};

use fontdue::{Font, FontSettings};
use svision_core::{FontSpec, Size, TextEngine};

/// Loads a reasonable platform UI font.
///
/// The rasterizer intentionally keeps a single font for now; [`FontSpec::family`]
/// is retained for future multi-font support.
pub fn load_default_font() -> Option<Rc<Font>> {
    for path in font_candidates() {
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };
        if let Ok(font) = Font::from_bytes(bytes, FontSettings::default()) {
            return Some(Rc::new(font));
        }
    }
    None
}

fn font_candidates() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(windir) = env::var("WINDIR") {
        let fonts = PathBuf::from(windir).join("Fonts");
        paths.push(fonts.join("segoeui.ttf"));
        paths.push(fonts.join("arial.ttf"));
        paths.push(fonts.join("tahoma.ttf"));
    }

    paths.push(PathBuf::from("C:/Windows/Fonts/segoeui.ttf"));
    paths.push(PathBuf::from("C:/Windows/Fonts/arial.ttf"));

    #[cfg(target_os = "linux")]
    {
        paths.push(PathBuf::from(
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        ));
        paths.push(PathBuf::from(
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        ));
        paths.push(PathBuf::from("/usr/share/fonts/TTF/DejaVuSans.ttf"));
    }

    #[cfg(target_os = "macos")]
    {
        paths.push(PathBuf::from(
            "/System/Library/Fonts/Supplemental/Arial.ttf",
        ));
        paths.push(PathBuf::from("/Library/Fonts/Arial.ttf"));
    }

    paths
}

/// [`TextEngine`] backed by fontdue metrics.
#[derive(Clone)]
pub struct FontdueTextEngine {
    font: Rc<Font>,
}

impl FontdueTextEngine {
    pub fn new() -> Option<Self> {
        load_default_font().map(|font| Self { font })
    }

    pub fn from_font(font: Rc<Font>) -> Self {
        Self { font }
    }

    pub fn font(&self) -> &Font {
        &self.font
    }
}

impl TextEngine for FontdueTextEngine {
    fn measure(&self, text: &str, font: &FontSpec) -> Size {
        measure_with_font(&self.font, text, font.size)
    }
}

pub(crate) fn measure_with_font(font: &Font, text: &str, px: f32) -> Size {
    let px = px.max(1.0);
    let metrics = font.horizontal_line_metrics(px);
    let line_height = metrics
        .as_ref()
        .map(|metrics| metrics.new_line_size)
        .unwrap_or(px * 1.25)
        .max(px * 0.5);

    let mut max_width = 0.0f32;
    let mut lines = 0usize;

    for line in text.lines() {
        lines += 1;
        let mut width = 0.0f32;
        for character in line.chars() {
            width += font.metrics(character, px).advance_width;
        }
        max_width = max_width.max(width);
    }

    if lines == 0 {
        lines = 1;
    }

    Size::new(max_width, lines as f32 * line_height)
}
