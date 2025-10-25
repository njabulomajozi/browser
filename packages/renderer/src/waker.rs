//! Event loop waker for Servo
//!
//! Servo runs compositor on a separate thread and needs to wake
//! the main event loop when rendering updates are ready.

use servo::EventLoopWaker;
use std::sync::{Arc, Mutex};

/// Event loop waker that Servo uses to signal the main thread
///
/// For now, this is a simple stub. In a full implementation with winit,
/// you would use winit's EventLoopProxy to wake the event loop.
#[derive(Clone)]
pub struct EguiEventLoopWaker {
    /// Callback to invoke when wakeup is requested
    #[allow(dead_code)] // Will be used when Servo is integrated
    #[allow(clippy::type_complexity)]
    callback: Arc<Mutex<Option<Box<dyn Fn() + Send + 'static>>>>,
}

impl EguiEventLoopWaker {
    /// Create a new event loop waker
    pub fn new() -> Self {
        Self {
            callback: Arc::new(Mutex::new(None)),
        }
    }

    /// Set the wakeup callback
    ///
    /// This callback will be invoked when Servo requests a wakeup
    #[allow(dead_code)] // Will be used when Servo is integrated
    pub fn set_callback<F>(&self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        *self.callback.lock().unwrap() = Some(Box::new(callback));
    }

    /// Wake up the event loop
    #[allow(dead_code)] // Will be used when Servo is integrated
    pub fn wake(&self) {
        if let Some(ref callback) = *self.callback.lock().unwrap() {
            callback();
        }
    }
}

impl Default for EguiEventLoopWaker {
    fn default() -> Self {
        Self::new()
    }
}

// Implement Servo's EventLoopWaker trait
impl EventLoopWaker for EguiEventLoopWaker {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(self.clone())
    }

    fn wake(&self) {
        // Invoke registered callback to wake the event loop
        if let Some(ref callback) = *self.callback.lock().unwrap() {
            callback();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_waker_callback() {
        let waker = EguiEventLoopWaker::new();
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();

        waker.set_callback(move || {
            called_clone.store(true, Ordering::SeqCst);
        });

        waker.wake();
        assert!(called.load(Ordering::SeqCst));
    }
}
