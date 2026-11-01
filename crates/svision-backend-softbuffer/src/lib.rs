//! softbuffer presentation backend.
//!
//! This pairs [`SoftRasterizer`](svision_raster_soft::SoftRasterizer) with
//! `softbuffer` to present a CPU-rendered frame to a winit window.

use std::{error::Error, num::NonZeroU32, sync::Arc};

use softbuffer::{Context, Surface};
use svision_core::{GraphicsBackend, Rasterizer, Size};
use svision_raster_soft::SoftRasterizer;
use svision_winit::WinitBackend;
use winit::{
    raw_window_handle::{
        DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
    },
    window::Window,
};

#[derive(Clone)]
struct SoftWindow(Arc<Window>);

impl HasWindowHandle for SoftWindow {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        self.0.window_handle()
    }
}

impl HasDisplayHandle for SoftWindow {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        self.0.display_handle()
    }
}

pub struct SoftbufferBackend {
    surface: Surface<SoftWindow, SoftWindow>,
    _context: Context<SoftWindow>,
    rasterizer: SoftRasterizer,
    size: Size,
}

impl SoftbufferBackend {
    fn apply_size(&mut self, size: Size) {
        self.size = size;
        let width = size.width.round() as u32;
        let height = size.height.round() as u32;
        if let (Some(width), Some(height)) = (NonZeroU32::new(width), NonZeroU32::new(height)) {
            let _ = self.surface.resize(width, height);
        }
    }
}

impl WinitBackend for SoftbufferBackend {
    fn create(window: Arc<Window>) -> Result<Self, Box<dyn Error>> {
        let handle = SoftWindow(window);
        let context = Context::new(handle.clone())?;
        let mut surface = Surface::new(&context, handle.clone())?;

        let window_size = handle.0.inner_size();
        let size = Size::new(window_size.width as f32, window_size.height as f32);
        if let (Some(width), Some(height)) = (
            NonZeroU32::new(window_size.width),
            NonZeroU32::new(window_size.height),
        ) {
            surface.resize(width, height)?;
        }

        Ok(Self {
            _context: context,
            surface,
            rasterizer: SoftRasterizer::new(),
            size,
        })
    }
}

impl GraphicsBackend for SoftbufferBackend {
    fn resize(&mut self, size: Size) {
        self.apply_size(size);
    }

    fn rasterizer(&mut self) -> &mut dyn Rasterizer {
        &mut self.rasterizer
    }

    fn present(&mut self) -> Result<(), Box<dyn Error>> {
        let pixmap = self.rasterizer.pixmap();
        let pixmap_width = pixmap.width() as usize;
        let pixmap_height = pixmap.height() as usize;
        if pixmap_width == 0 || pixmap_height == 0 {
            return Ok(());
        }
        let source = pixmap.data();

        let mut buffer = self.surface.buffer_mut()?;
        let buffer_width = buffer.width().get() as usize;
        let buffer_height = buffer.height().get() as usize;
        let width = buffer_width.min(pixmap_width);
        let height = buffer_height.min(pixmap_height);

        for y in 0..height {
            for x in 0..width {
                let source_index = (y * pixmap_width + x) * 4;
                let destination_index = y * buffer_width + x;
                buffer[destination_index] = (u32::from(source[source_index]) << 16)
                    | (u32::from(source[source_index + 1]) << 8)
                    | u32::from(source[source_index + 2]);
            }
        }

        buffer.present()?;
        Ok(())
    }

    fn size(&self) -> Size {
        self.size
    }
}
