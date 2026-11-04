//! CPU rasterizer for `svision`, built on tiny-skia + fontdue.
//!
//! This is one concrete [`TextRasterizer`](svision_core::TextRasterizer)
//! implementation (which is also a [`Painter`](svision_core::Painter)). Nothing in `svision-core` depends on it; swapping in GDI+,
//! OpenGL, or another drawing engine only requires another implementation of
//! the traits.

mod font;
mod rasterizer;

pub use font::{FontdueTextEngine, load_default_font};
pub use rasterizer::SoftRasterizer;
