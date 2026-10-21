//! Mobile display surface management for Android ANativeWindow and headless presentation.
//!
//! Provides zero-copy or stride-matched frame presentation from rendered RGBA images
//! to the Android OS compositor.

use effectcraft_raster::Image;

use crate::error::{MobileError, Result};

pub struct MobileSurface {
    pub width: u32,
    pub height: u32,
    #[cfg(target_os = "android")]
    native_window: Option<ndk::native_window::NativeWindow>,
    #[cfg(not(target_os = "android"))]
    mock_buffer: Vec<u8>,
}

impl MobileSurface {
    /// Create an unattached mobile surface.
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            #[cfg(target_os = "android")]
            native_window: None,
            #[cfg(not(target_os = "android"))]
            mock_buffer: Vec::new(),
        }
    }

    /// Attach an Android native window from an ANativeWindow pointer.
    #[cfg(target_os = "android")]
    pub unsafe fn set_android_surface(&mut self, window_ptr: *mut ndk_sys::ANativeWindow, width: u32, height: u32) -> Result<()> {
        if window_ptr.is_null() {
            self.native_window = None;
            self.width = 0;
            self.height = 0;
            return Ok(());
        }
        let win = ndk::native_window::NativeWindow::from_ptr(std::ptr::NonNull::new(window_ptr).ok_or_else(|| {
            MobileError::Surface("Null native window pointer".into())
        })?);
        
        // Configure window format to RGBA_8888 for high-fidelity composition
        let _ = win.set_buffers_geometry(width as i32, height as i32, ndk::hardware_buffer::HardwareBufferFormat::R8G8B8A8_UNORM);
        
        self.native_window = Some(win);
        self.width = width;
        self.height = height;
        Ok(())
    }

    /// Attach a mock buffer for unit testing on non-Android hosts.
    #[cfg(not(target_os = "android"))]
    pub fn set_mock_surface(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.mock_buffer = vec![0u8; (width as usize).saturating_mul(height as usize).saturating_mul(4)];
    }

    /// Destroy / detach the surface.
    pub fn destroy(&mut self) {
        #[cfg(target_os = "android")]
        {
            self.native_window = None;
        }
        #[cfg(not(target_os = "android"))]
        {
            self.mock_buffer.clear();
        }
        self.width = 0;
        self.height = 0;
    }

    /// Present a rendered frame to the surface.
    pub fn present(&mut self, image: &Image) -> Result<()> {
        if self.width == 0 || self.height == 0 {
            return Err(MobileError::Surface("Surface has 0 dimensions or is unattached".into()));
        }

        #[cfg(target_os = "android")]
        {
            let Some(win) = self.native_window.as_ref() else {
                return Err(MobileError::Surface("No native window attached".into()));
            };

            // Lock the window buffer for writing
            let mut buffer = win.lock(None).map_err(|e| {
                MobileError::Surface(format!("Failed to lock ANativeWindow: {:?}", e))
            })?;

            let stride = buffer.stride() as usize;
            let buf_width = buffer.width() as usize;
            let buf_height = buffer.height() as usize;
            let dst_slice = buffer.bits();

            let img_w = image.width as usize;
            let img_h = image.height as usize;
            let src_data = &image.data;

            let copy_h = buf_height.min(img_h);
            let copy_w = buf_width.min(img_w);

            for y in 0..copy_h {
                let src_offset = y.saturating_mul(img_w).saturating_mul(4);
                let dst_offset = y.saturating_mul(stride).saturating_mul(4);
                let row_bytes = copy_w.saturating_mul(4);

                if let (Some(src_row), Some(dst_row)) = (
                    src_data.get(src_offset..src_offset.saturating_add(row_bytes)),
                    dst_slice.get_mut(dst_offset..dst_offset.saturating_add(row_bytes)),
                ) {
                    dst_row.copy_from_slice(src_row);
                }
            }

            // Window drops lock and posts on exit of buffer scope (ndk NativeWindowBuffer drop)
            Ok(())
        }

        #[cfg(not(target_os = "android"))]
        {
            let copy_len = self.mock_buffer.len().min(image.data.len());
            if let (Some(dst), Some(src)) = (self.mock_buffer.get_mut(..copy_len), image.data.get(..copy_len)) {
                dst.copy_from_slice(src);
            }
            Ok(())
        }
    }
}

impl Default for MobileSurface {
    fn default() -> Self {
        Self::new()
    }
}
