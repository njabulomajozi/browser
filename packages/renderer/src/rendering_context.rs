//! Rendering context for Servo integration
//!
//! Servo requires a RenderingContext for OpenGL rendering. For MVP, we use
//! SoftwareRenderingContext (CPU-based) to avoid window handle complexity.
//!
//! Future: Upgrade to OffscreenRenderingContext with GL for performance.

use crate::{RendererError, Result};
use servo::SoftwareRenderingContext;
use std::rc::Rc;
use winit::dpi::PhysicalSize;

/// Creates a SoftwareRenderingContext for Servo
///
/// This is a CPU-based fallback renderer that doesn't require OpenGL.
/// It's slower than GL but simpler to integrate.
///
/// # Arguments
/// * `width` - Rendering width in pixels
/// * `height` - Rendering height in pixels
pub fn create_software_rendering_context(
    width: u32,
    height: u32,
) -> Result<Rc<SoftwareRenderingContext>> {
    let size = PhysicalSize::new(width, height);

    SoftwareRenderingContext::new(size)
        .map(Rc::new)
        .map_err(|e| {
            RendererError::InitializationFailed(format!(
                "Failed to create SoftwareRenderingContext: {:?}",
                e
            ))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_software_context() {
        let context = create_software_rendering_context(800, 600);
        assert!(context.is_ok(), "Should create software rendering context");
    }

    #[test]
    fn test_small_size() {
        // Minimum size is 1x1
        let context = create_software_rendering_context(1, 1);
        assert!(context.is_ok(), "Should support minimum size");
    }

    #[test]
    fn test_large_size() {
        let context = create_software_rendering_context(3840, 2160);
        assert!(context.is_ok(), "Should support 4K resolution");
    }
}
