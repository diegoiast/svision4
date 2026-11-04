use crate::{
    Size, SizePolicy, SizePolicy2D, impl_widget_any,
    widget::{MeasureCtx, Widget},
};

/// An invisible, fixed-size gap.
#[derive(Clone, Copy, Debug, Default)]
pub struct Spacer {
    size: Size,
}

impl Spacer {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            size: Size::new(width, height),
        }
    }

    pub fn vertical(height: f32) -> Self {
        Self::new(0.0, height)
    }

    pub fn horizontal(width: f32) -> Self {
        Self::new(width, 0.0)
    }

    pub fn size(&self) -> Size {
        self.size
    }
}

impl Widget for Spacer {
    fn size_policy(&self) -> SizePolicy2D {
        SizePolicy2D::new(SizePolicy::Fixed, SizePolicy::Fixed)
    }

    fn measure(&mut self, _ctx: &mut MeasureCtx, _available: Size) -> Size {
        self.size
    }

    impl_widget_any!();
}
