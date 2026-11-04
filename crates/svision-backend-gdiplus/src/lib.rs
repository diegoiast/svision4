//! GDI+ presentation backend for Windows.
//!
//! This backend draws directly into the window's device context with GDI+.
//! Text measurement is delegated to the fontdue-based engine shipped with
//! `svision-raster-soft` so that layout matches the software backend.

#![cfg(windows)]

use std::{error::Error, ffi::c_void, ptr::null_mut, rc::Rc, sync::Arc, sync::OnceLock};

use svision_core::{
    ApproxTextEngine, Color, FontSpec, GraphicsBackend, Painter, Point, Rect, Size, TextEngine,
    TextRasterizer,
};
use svision_raster_soft::FontdueTextEngine;
use svision_winit::WinitBackend;
use windows::{
    Win32::{
        Foundation::HWND,
        Graphics::{
            Gdi::{GetDC, HDC, ReleaseDC},
            GdiPlus::{
                CombineModeIntersect, FillModeAlternate, FontStyleRegular, GdipAddPathArc,
                GdipAddPathLine, GdipClosePathFigure, GdipCreateFont, GdipCreateFontFamilyFromName,
                GdipCreateFromHDC, GdipCreatePath, GdipCreatePen1, GdipCreateSolidFill,
                GdipCreateStringFormat, GdipDeleteBrush, GdipDeleteFont, GdipDeleteFontFamily,
                GdipDeletePath, GdipDeletePen, GdipDeleteStringFormat, GdipDrawLine, GdipDrawPath,
                GdipDrawRectangle, GdipDrawString, GdipFillEllipse, GdipFillPath,
                GdipFillRectangle, GdipGraphicsClear, GdipRestoreGraphics, GdipSaveGraphics,
                GdipSetClipRectI, GdipSetSmoothingMode, GdipSetStringFormatFlags,
                GdipSetTextRenderingHint, GdiplusStartup, GdiplusStartupInput, GpBrush, GpFont,
                GpFontFamily, GpGraphics, GpPath, GpPen, GpSolidFill, GpStringFormat, Ok as GdipOk,
                RectF, SmoothingModeAntiAlias, StringAlignmentNear, StringFormatFlagsNoWrap,
                TextRenderingHintAntiAliasGridFit, UnitPixel,
            },
        },
    },
    core::HSTRING,
};
use winit::{
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::Window,
};

static STARTUP: OnceLock<Result<(), String>> = OnceLock::new();

fn ensure_gdiplus() -> Result<(), Box<dyn Error>> {
    let result = STARTUP.get_or_init(|| {
        let mut token = 0usize;
        let input = GdiplusStartupInput {
            GdiplusVersion: 1,
            ..Default::default()
        };
        let status = unsafe { GdiplusStartup(&mut token, &input, null_mut()) };
        if status == GdipOk {
            Ok(())
        } else {
            Err(format!("GdiplusStartup failed with status {}", status.0))
        }
    });
    match result {
        Ok(()) => Ok(()),
        Err(message) => Err(message.clone().into()),
    }
}

fn argb(color: Color) -> u32 {
    (u32::from(color.a) << 24)
        | (u32::from(color.r) << 16)
        | (u32::from(color.g) << 8)
        | u32::from(color.b)
}

fn font_style(font: &FontSpec) -> i32 {
    let mut style = FontStyleRegular.0;
    if font.bold {
        style |= 1;
    }
    if font.italic {
        style |= 2;
    }
    style
}

pub struct GdiPlusRasterizer {
    hwnd: HWND,
    hdc: Option<HDC>,
    graphics: Option<*mut GpGraphics>,
    size: Size,
    clip_states: Vec<Option<u32>>,
    text_engine: Option<Rc<FontdueTextEngine>>,
}

impl GdiPlusRasterizer {
    fn new(hwnd: HWND) -> Self {
        Self {
            hwnd,
            hdc: None,
            graphics: None,
            size: Size::ZERO,
            clip_states: Vec::new(),
            text_engine: FontdueTextEngine::new().map(Rc::new),
        }
    }

    fn end_frame(&mut self) {
        self.clip_states.clear();
        if let Some(graphics) = self.graphics.take() {
            unsafe {
                windows::Win32::Graphics::GdiPlus::GdipDeleteGraphics(graphics);
            }
        }
        if let Some(hdc) = self.hdc.take() {
            unsafe {
                ReleaseDC(Some(self.hwnd), hdc);
            }
        }
    }

    fn with_brush(&mut self, color: Color, draw: impl FnOnce(*mut GpGraphics, *mut GpBrush)) {
        let Some(graphics) = self.graphics else {
            return;
        };
        let mut brush: *mut GpSolidFill = null_mut();
        let status = unsafe { GdipCreateSolidFill(argb(color), &mut brush) };
        if status != GdipOk {
            return;
        }
        draw(graphics, brush as *mut GpBrush);
        unsafe {
            GdipDeleteBrush(brush as *mut GpBrush);
        }
    }

    fn with_pen(
        &mut self,
        color: Color,
        width: f32,
        draw: impl FnOnce(*mut GpGraphics, *mut GpPen),
    ) {
        let Some(graphics) = self.graphics else {
            return;
        };
        let mut pen: *mut GpPen = null_mut();
        let status = unsafe { GdipCreatePen1(argb(color), width.max(1.0), UnitPixel, &mut pen) };
        if status != GdipOk {
            return;
        }
        draw(graphics, pen);
        unsafe {
            GdipDeletePen(pen);
        }
    }

    fn rounded_path(rect: Rect, radius: f32) -> Option<*mut GpPath> {
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return None;
        }
        let radius = radius.max(0.0).min(rect.width.min(rect.height) * 0.5);
        let mut path: *mut GpPath = null_mut();
        let status = unsafe { GdipCreatePath(FillModeAlternate, &mut path) };
        if status != GdipOk {
            return None;
        }
        let x = rect.x;
        let y = rect.y;
        let w = rect.width;
        let h = rect.height;
        let d = radius * 2.0;
        unsafe {
            if radius <= 0.0 {
                GdipAddPathLine(path, x, y, x + w, y);
                GdipAddPathLine(path, x + w, y, x + w, y + h);
                GdipAddPathLine(path, x + w, y + h, x, y + h);
                GdipAddPathLine(path, x, y + h, x, y);
            } else {
                GdipAddPathArc(path, x, y, d, d, 180.0, 90.0);
                GdipAddPathLine(path, x + radius, y, x + w - radius, y);
                GdipAddPathArc(path, x + w - d, y, d, d, 270.0, 90.0);
                GdipAddPathLine(path, x + w, y + radius, x + w, y + h - radius);
                GdipAddPathArc(path, x + w - d, y + h - d, d, d, 0.0, 90.0);
                GdipAddPathLine(path, x + w - radius, y + h, x + radius, y + h);
                GdipAddPathArc(path, x, y + h - d, d, d, 90.0, 90.0);
                GdipAddPathLine(path, x, y + h - radius, x, y + radius);
            }
            GdipClosePathFigure(path);
        }
        Some(path)
    }

    fn fill_path(&mut self, path: *mut GpPath, color: Color) {
        self.with_brush(color, |graphics, brush| unsafe {
            GdipFillPath(graphics, brush, path);
        });
    }

    fn stroke_path(&mut self, path: *mut GpPath, color: Color, width: f32) {
        self.with_pen(color, width, |graphics, pen| unsafe {
            GdipDrawPath(graphics, pen, path);
        });
    }

    fn draw_text_inner(&mut self, text: &str, origin: Point, font: &FontSpec, color: Color) {
        let Some(graphics) = self.graphics else {
            return;
        };
        if text.is_empty() {
            return;
        }

        let mut family: *mut GpFontFamily = null_mut();
        let mut family_name = if let Some(family) = &font.family {
            HSTRING::from(family.as_str())
        } else {
            HSTRING::from("Segoe UI")
        };
        let mut status =
            unsafe { GdipCreateFontFamilyFromName(&family_name, null_mut(), &mut family) };
        if status != GdipOk {
            family_name = HSTRING::from("Arial");
            status = unsafe { GdipCreateFontFamilyFromName(&family_name, null_mut(), &mut family) };
        }
        if status != GdipOk {
            return;
        }

        let mut gp_font: *mut GpFont = null_mut();
        status = unsafe {
            GdipCreateFont(
                family,
                font.size.max(1.0),
                font_style(font),
                UnitPixel,
                &mut gp_font,
            )
        };
        if status != GdipOk {
            unsafe {
                GdipDeleteFontFamily(family);
            }
            return;
        }

        let mut format: *mut GpStringFormat = null_mut();
        status = unsafe { GdipCreateStringFormat(0, 0, &mut format) };
        if status != GdipOk {
            unsafe {
                GdipDeleteFont(gp_font);
                GdipDeleteFontFamily(family);
            }
            return;
        }
        unsafe {
            let _ = GdipSetStringFormatFlags(format, StringFormatFlagsNoWrap.0);
            let _ = windows::Win32::Graphics::GdiPlus::GdipSetStringFormatAlign(
                format,
                StringAlignmentNear,
            );
        }

        let measured = match &self.text_engine {
            Some(engine) => engine.measure(text, font),
            None => ApproxTextEngine.measure(text, font),
        };
        let layout = RectF {
            X: origin.x,
            Y: origin.y,
            Width: measured.width.max(1.0) + 4.0,
            Height: measured.height.max(font.size) + 4.0,
        };
        let text_wide = HSTRING::from(text);
        let length = text_wide.len() as i32;
        let mut brush: *mut GpSolidFill = null_mut();
        status = unsafe { GdipCreateSolidFill(argb(color), &mut brush) };
        if status == GdipOk {
            unsafe {
                GdipDrawString(
                    graphics,
                    &text_wide,
                    length,
                    gp_font,
                    &layout,
                    format,
                    brush as *mut GpBrush,
                );
                GdipDeleteBrush(brush as *mut GpBrush);
            }
        }

        unsafe {
            GdipDeleteStringFormat(format);
            GdipDeleteFont(gp_font);
            GdipDeleteFontFamily(family);
        }
    }
}

impl Painter for GdiPlusRasterizer {
    fn begin_frame(&mut self, size: Size) {
        self.end_frame();
        self.size = size;

        let hdc = unsafe { GetDC(Some(self.hwnd)) };
        if hdc.0.is_null() {
            return;
        }
        let mut graphics: *mut GpGraphics = null_mut();
        let status = unsafe { GdipCreateFromHDC(hdc, &mut graphics) };
        if status != GdipOk {
            unsafe {
                ReleaseDC(Some(self.hwnd), hdc);
            }
            return;
        }
        unsafe {
            let _ = GdipSetSmoothingMode(graphics, SmoothingModeAntiAlias);
            let _ = GdipSetTextRenderingHint(graphics, TextRenderingHintAntiAliasGridFit);
        }
        self.hdc = Some(hdc);
        self.graphics = Some(graphics);
    }

    fn clear(&mut self, color: Color) {
        if let Some(graphics) = self.graphics {
            unsafe {
                let _ = GdipGraphicsClear(graphics, argb(color));
            }
        }
    }

    fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.with_brush(color, |graphics, brush| unsafe {
            let _ = GdipFillRectangle(graphics, brush, rect.x, rect.y, rect.width, rect.height);
        });
    }

    fn stroke_rect(&mut self, rect: Rect, color: Color, width: f32) {
        let inset = (width * 0.5).max(0.0);
        let rect = rect.inset_all(inset);
        self.with_pen(color, width, |graphics, pen| unsafe {
            let _ = GdipDrawRectangle(graphics, pen, rect.x, rect.y, rect.width, rect.height);
        });
    }

    fn fill_round_rect(&mut self, rect: Rect, radius: f32, color: Color) {
        if let Some(path) = Self::rounded_path(rect, radius) {
            self.fill_path(path, color);
            unsafe {
                GdipDeletePath(path);
            }
        }
    }

    fn stroke_round_rect(&mut self, rect: Rect, radius: f32, color: Color, width: f32) {
        let inset = (width * 0.5).max(0.0);
        if let Some(path) = Self::rounded_path(rect.inset_all(inset), radius) {
            self.stroke_path(path, color, width);
            unsafe {
                GdipDeletePath(path);
            }
        }
    }

    fn fill_ellipse(&mut self, rect: Rect, color: Color) {
        self.with_brush(color, |graphics, brush| unsafe {
            let _ = GdipFillEllipse(graphics, brush, rect.x, rect.y, rect.width, rect.height);
        });
    }

    fn draw_line(&mut self, from: Point, to: Point, color: Color, width: f32) {
        self.with_pen(color, width, |graphics, pen| unsafe {
            let _ = GdipDrawLine(graphics, pen, from.x, from.y, to.x, to.y);
        });
    }

    fn push_clip(&mut self, rect: Rect) {
        let Some(graph) = self.graphics else {
            self.clip_states.push(None);
            return;
        };
        let mut state = 0u32;
        let status = unsafe { GdipSaveGraphics(graph, &mut state) };
        if status != GdipOk {
            self.clip_states.push(None);
            return;
        }
        let status = unsafe {
            GdipSetClipRectI(
                graph,
                rect.x.floor() as i32,
                rect.y.floor() as i32,
                rect.width.ceil() as i32,
                rect.height.ceil() as i32,
                CombineModeIntersect,
            )
        };
        if status != GdipOk {
            unsafe {
                GdipRestoreGraphics(graph, state);
            }
            self.clip_states.push(None);
            return;
        }
        self.clip_states.push(Some(state));
    }

    fn pop_clip(&mut self) {
        let Some(Some(state)) = self.clip_states.pop() else {
            return;
        };
        if let Some(graphics) = self.graphics {
            unsafe {
                GdipRestoreGraphics(graphics, state);
            }
        }
    }
}

impl TextRasterizer for GdiPlusRasterizer {
    fn draw_text(&mut self, text: &str, origin: Point, font: &FontSpec, color: Color) {
        self.draw_text_inner(text, origin, font, color);
    }

    fn text_engine(&self) -> Option<Rc<dyn svision_core::TextEngine>> {
        self.text_engine
            .clone()
            .map(|engine| engine as Rc<dyn svision_core::TextEngine>)
    }
}

impl Drop for GdiPlusRasterizer {
    fn drop(&mut self) {
        self.end_frame();
    }
}

pub struct GdiPlusBackend {
    _window: Arc<Window>,
    rasterizer: GdiPlusRasterizer,
    size: Size,
}

impl GdiPlusBackend {
    fn hwnd(window: &Window) -> Result<HWND, Box<dyn Error>> {
        let handle = window.window_handle()?;
        let RawWindowHandle::Win32(handle) = handle.as_raw() else {
            return Err("svision-backend-gdiplus requires a Win32 window".into());
        };
        Ok(HWND(handle.hwnd.get() as *mut c_void))
    }
}

impl WinitBackend for GdiPlusBackend {
    fn create(window: Arc<Window>) -> Result<Self, Box<dyn Error>> {
        ensure_gdiplus()?;
        let hwnd = Self::hwnd(&window)?;
        let inner = window.inner_size();
        let size = Size::new(inner.width as f32, inner.height as f32);
        Ok(Self {
            _window: window,
            rasterizer: GdiPlusRasterizer::new(hwnd),
            size,
        })
    }
}

impl GraphicsBackend for GdiPlusBackend {
    fn resize(&mut self, size: Size) {
        self.size = size;
    }

    fn rasterizer(&mut self) -> &mut dyn TextRasterizer {
        &mut self.rasterizer
    }

    fn present(&mut self) -> Result<(), Box<dyn Error>> {
        self.rasterizer.end_frame();
        Ok(())
    }

    fn size(&self) -> Size {
        self.size
    }
}
