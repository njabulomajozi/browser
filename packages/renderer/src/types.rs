//! Shared types for the renderer

/// Configuration for the renderer
#[derive(Debug, Clone)]
pub struct RendererConfig {
    /// Initial window width in pixels
    pub width: u32,
    /// Initial window height in pixels
    pub height: u32,
    /// Device pixel ratio (HiDPI scaling)
    pub device_pixel_ratio: f32,
    /// Enable WebGL support
    pub enable_webgl: bool,
    /// Enable JavaScript
    pub enable_javascript: bool,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            width: 1024,
            height: 768,
            device_pixel_ratio: 1.0,
            enable_webgl: true,
            enable_javascript: true,
        }
    }
}

/// A rendered frame from Servo
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RenderedFrame {
    /// Width in device pixels
    pub width: u32,
    /// Height in device pixels
    pub height: u32,
    /// RGBA pixel data (4 bytes per pixel)
    pub pixels: Vec<u8>,
}

#[allow(dead_code)]
impl RenderedFrame {
    /// Create a new rendered frame
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Self {
        assert_eq!(
            pixels.len(),
            (width * height * 4) as usize,
            "Pixel data size mismatch"
        );
        Self {
            width,
            height,
            pixels,
        }
    }

    /// Check if the frame is empty
    pub fn is_empty(&self) -> bool {
        self.pixels.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_config_default() {
        let config = RendererConfig::default();
        assert_eq!(config.width, 1024);
        assert_eq!(config.height, 768);
    }

    #[test]
    fn test_rendered_frame() {
        let pixels = vec![0u8; 1024 * 768 * 4];
        let frame = RenderedFrame::new(1024, 768, pixels);
        assert!(!frame.is_empty());
        assert_eq!(frame.width, 1024);
        assert_eq!(frame.height, 768);
    }

    #[test]
    #[should_panic(expected = "Pixel data size mismatch")]
    fn test_rendered_frame_wrong_size() {
        let pixels = vec![0u8; 100]; // Wrong size
        RenderedFrame::new(1024, 768, pixels);
    }
}
