//! CPU rasterizer for `svision`, built on tiny-skia + fontdue.
//!
//! This is one concrete [`Rasterizer`](svision_core::Rasterizer)
//! implementation. Nothing in `svision-core` depends on it; swapping in GDI+,
//! OpenGL, or another drawing engine only requires another implementation of
//! the trait.

mod font;
mod rasterizer;

pub use font::{load_default_font, FontdueTextEngine};
pub use rasterizer::SoftRasterizer;
