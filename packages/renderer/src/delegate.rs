//! WebView delegate implementation
//!
//! Implements callbacks that Servo invokes during webview lifecycle events.

use crate::waker::EguiEventLoopWaker;
use servo::{LoadStatus, WebView, WebViewDelegate};
use std::sync::{Arc, Mutex};
use tracing::{debug, info};
use url::Url;

/// State shared between the delegate and the renderer
#[derive(Default, Clone)]
pub struct DelegateState {
    /// Current page title
    pub title: Arc<Mutex<Option<String>>>,
    /// Current page URL
    pub url: Arc<Mutex<Option<String>>>,
    /// Loading state
    pub is_loading: Arc<Mutex<bool>>,
    /// Load progress (0.0 to 1.0)
    pub load_progress: Arc<Mutex<f32>>,
}

impl DelegateState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_title(&self, title: Option<String>) {
        *self.title.lock().unwrap() = title;
    }

    pub fn get_title(&self) -> Option<String> {
        self.title.lock().unwrap().clone()
    }

    pub fn set_url(&self, url: Option<String>) {
        *self.url.lock().unwrap() = url;
    }

    pub fn get_url(&self) -> Option<String> {
        self.url.lock().unwrap().clone()
    }

    pub fn set_loading(&self, loading: bool) {
        *self.is_loading.lock().unwrap() = loading;
    }

    pub fn is_loading(&self) -> bool {
        *self.is_loading.lock().unwrap()
    }

    pub fn set_progress(&self, progress: f32) {
        *self.load_progress.lock().unwrap() = progress;
    }

    pub fn get_progress(&self) -> f32 {
        *self.load_progress.lock().unwrap()
    }
}

/// WebView delegate that receives callbacks from Servo
///
/// This implements the callbacks that Servo invokes during page lifecycle.
pub struct BrowserWebViewDelegate {
    state: DelegateState,
    waker: EguiEventLoopWaker,
}

impl BrowserWebViewDelegate {
    pub fn new(state: DelegateState, waker: EguiEventLoopWaker) -> Self {
        Self { state, waker }
    }

    /// Get reference to delegate state
    ///
    /// This is useful for accessing delegate state from outside the delegate
    #[allow(dead_code)] // Reserved for future use
    pub fn state(&self) -> &DelegateState {
        &self.state
    }
}

// Implement Servo's WebViewDelegate trait
impl WebViewDelegate for BrowserWebViewDelegate {
    fn notify_new_frame_ready(&self, _webview: WebView) {
        debug!("New frame ready for rendering");
        // Wake the event loop to trigger a repaint
        self.waker.wake();
    }

    fn notify_page_title_changed(&self, _webview: WebView, title: Option<String>) {
        if let Some(ref t) = title {
            info!("Title changed: {}", t);
        }
        self.state.set_title(title);
        self.waker.wake();
    }

    fn notify_load_status_changed(&self, _webview: WebView, status: LoadStatus) {
        match status {
            LoadStatus::Started => {
                info!("Page started loading");
                self.state.set_loading(true);
                self.state.set_progress(0.1);
            }
            LoadStatus::HeadParsed => {
                debug!("Page head parsed");
                self.state.set_progress(0.5);
            }
            LoadStatus::Complete => {
                info!("Page finished loading");
                self.state.set_loading(false);
                self.state.set_progress(1.0);
            }
        }
        self.waker.wake();
    }

    fn notify_history_changed(&self, _webview: WebView, entries: Vec<Url>, current: usize) {
        debug!(
            "History changed: {} entries, current index {}",
            entries.len(),
            current
        );
        // Update URL from history
        if let Some(url) = entries.get(current) {
            self.state.set_url(Some(url.to_string()));
        }
        self.waker.wake();
    }

    // All other methods use default implementations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delegate_state() {
        let state = DelegateState::new();
        assert!(!state.is_loading());
        assert_eq!(state.get_progress(), 0.0);

        state.set_loading(true);
        state.set_progress(0.5);
        assert!(state.is_loading());
        assert_eq!(state.get_progress(), 0.5);
    }

    #[test]
    fn test_delegate_creation() {
        let state = DelegateState::new();
        let waker = EguiEventLoopWaker::new();
        let delegate = BrowserWebViewDelegate::new(state.clone(), waker);

        assert!(!delegate.state().is_loading());
        assert_eq!(delegate.state().get_progress(), 0.0);
    }
}
