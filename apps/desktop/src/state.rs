//! State Manager - Centralized browser state management
//!
//! Following AWS service-oriented architecture:
//! - **Single Source of Truth**: All application state centralized
//! - **Immutable State**: State updates tracked
//! - **Simple API**: Clear state mutation methods
//!
//! # Responsibilities
//!
//! - Tab lifecycle (create, close, switch)
//! - Active tab tracking
//! - Tab metadata (URL, title, navigation state)
//! - Settings management (future)

// Allow dead code temporarily - APIs will be integrated in Week 2
#![allow(dead_code)]

use std::collections::HashMap;

/// Unique tab identifier
pub type TabId = usize;

/// Tab state information
#[derive(Debug, Clone)]
pub struct TabState {
    /// Unique tab ID
    pub id: TabId,

    /// Current URL
    pub url: String,

    /// Page title
    pub title: String,

    /// Can navigate back
    pub can_go_back: bool,

    /// Can navigate forward
    pub can_go_forward: bool,

    /// Currently loading
    pub is_loading: bool,
}

impl TabState {
    /// Create new tab state
    pub fn new(id: TabId, url: String) -> Self {
        Self {
            id,
            url: url.clone(),
            title: Self::extract_title(&url),
            can_go_back: false,
            can_go_forward: false,
            is_loading: false,
        }
    }

    /// Extract title from URL (domain as fallback)
    fn extract_title(url: &str) -> String {
        url.split('/').nth(2).unwrap_or("New Tab").to_string()
    }

    /// Update URL and title
    pub fn set_url(&mut self, url: String) {
        self.title = Self::extract_title(&url);
        self.url = url;
    }

    /// Update title
    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }
}

/// State Manager - Single source of truth for browser state
///
/// AWS pattern: Centralized state management for observability
/// and consistency.
pub struct StateManager {
    /// All tabs indexed by ID
    tabs: HashMap<TabId, TabState>,

    /// Active tab ID
    active_tab_id: Option<TabId>,

    /// Next tab ID (monotonically increasing)
    next_tab_id: TabId,

    /// Application settings (future extension)
    settings: HashMap<String, String>,
}

impl StateManager {
    /// Create new state manager
    pub fn new() -> Self {
        Self {
            tabs: HashMap::new(),
            active_tab_id: None,
            next_tab_id: 0,
            settings: HashMap::new(),
        }
    }

    /// Create new tab
    ///
    /// # Arguments
    /// * `url` - Initial URL for tab
    ///
    /// # Returns
    /// Tab ID of created tab
    pub fn create_tab(&mut self, url: String) -> TabId {
        let tab_id = self.next_tab_id;
        self.next_tab_id += 1;

        let tab = TabState::new(tab_id, url);
        self.tabs.insert(tab_id, tab);

        // Set as active if first tab
        if self.active_tab_id.is_none() {
            self.active_tab_id = Some(tab_id);
        }

        tab_id
    }

    /// Close tab
    ///
    /// # Arguments
    /// * `id` - Tab ID to close
    ///
    /// # Returns
    /// Ok if tab existed and was closed
    ///
    /// # Side Effects
    /// If closing active tab, switches to another tab
    pub fn close_tab(&mut self, id: TabId) -> Result<(), String> {
        if self.tabs.remove(&id).is_none() {
            return Err(format!("Tab {} not found", id));
        }

        // If closing active tab, switch to another
        if self.active_tab_id == Some(id) {
            self.active_tab_id = self.tabs.keys().next().copied();
        }

        Ok(())
    }

    /// Switch to tab
    ///
    /// # Arguments
    /// * `id` - Tab ID to switch to
    ///
    /// # Returns
    /// Ok if tab exists
    pub fn switch_tab(&mut self, id: TabId) -> Result<(), String> {
        if !self.tabs.contains_key(&id) {
            return Err(format!("Tab {} not found", id));
        }

        self.active_tab_id = Some(id);
        Ok(())
    }

    /// Get active tab
    pub fn get_active_tab(&self) -> Option<&TabState> {
        self.active_tab_id.and_then(|id| self.tabs.get(&id))
    }

    /// Get active tab (mutable)
    pub fn get_active_tab_mut(&mut self) -> Option<&mut TabState> {
        self.active_tab_id.and_then(|id| self.tabs.get_mut(&id))
    }

    /// Get tab by ID
    pub fn get_tab(&self, id: TabId) -> Option<&TabState> {
        self.tabs.get(&id)
    }

    /// Get tab by ID (mutable)
    pub fn get_tab_mut(&mut self, id: TabId) -> Option<&mut TabState> {
        self.tabs.get_mut(&id)
    }

    /// Get all tabs
    pub fn get_all_tabs(&self) -> Vec<&TabState> {
        self.tabs.values().collect()
    }

    /// Get tab count
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Get active tab ID
    pub fn active_tab_id(&self) -> Option<TabId> {
        self.active_tab_id
    }

    /// Update tab title
    pub fn update_tab_title(&mut self, id: TabId, title: String) {
        if let Some(tab) = self.tabs.get_mut(&id) {
            tab.set_title(title);
        }
    }

    /// Update tab URL
    pub fn update_tab_url(&mut self, id: TabId, url: String) {
        if let Some(tab) = self.tabs.get_mut(&id) {
            tab.set_url(url);
        }
    }

    /// Update tab loading state
    pub fn set_tab_loading(&mut self, id: TabId, loading: bool) {
        if let Some(tab) = self.tabs.get_mut(&id) {
            tab.is_loading = loading;
        }
    }

    /// Update tab navigation state
    pub fn set_tab_nav_state(&mut self, id: TabId, can_go_back: bool, can_go_forward: bool) {
        if let Some(tab) = self.tabs.get_mut(&id) {
            tab.can_go_back = can_go_back;
            tab.can_go_forward = can_go_forward;
        }
    }

    /// Get setting
    pub fn get_setting(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    /// Set setting
    pub fn set_setting(&mut self, key: String, value: String) {
        self.settings.insert(key, value);
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_tab() {
        let mut state = StateManager::new();

        let tab_id = state.create_tab("https://example.com".to_string());
        assert_eq!(tab_id, 0);
        assert_eq!(state.tab_count(), 1);
        assert_eq!(state.active_tab_id(), Some(0));
    }

    #[test]
    fn test_multiple_tabs() {
        let mut state = StateManager::new();

        let tab1 = state.create_tab("https://example.com".to_string());
        let tab2 = state.create_tab("https://github.com".to_string());
        let tab3 = state.create_tab("https://google.com".to_string());

        assert_eq!(state.tab_count(), 3);
        assert_eq!(tab1, 0);
        assert_eq!(tab2, 1);
        assert_eq!(tab3, 2);
    }

    #[test]
    fn test_switch_tab() {
        let mut state = StateManager::new();

        let tab1 = state.create_tab("https://example.com".to_string());
        let tab2 = state.create_tab("https://github.com".to_string());

        assert_eq!(state.active_tab_id(), Some(tab1));

        state.switch_tab(tab2).unwrap();
        assert_eq!(state.active_tab_id(), Some(tab2));

        let active = state.get_active_tab().unwrap();
        assert_eq!(active.url, "https://github.com");
    }

    #[test]
    fn test_close_tab() {
        let mut state = StateManager::new();

        let tab1 = state.create_tab("https://example.com".to_string());
        let tab2 = state.create_tab("https://github.com".to_string());

        assert_eq!(state.tab_count(), 2);

        state.close_tab(tab1).unwrap();
        assert_eq!(state.tab_count(), 1);
        assert!(state.get_tab(tab1).is_none());
        assert!(state.get_tab(tab2).is_some());
    }

    #[test]
    fn test_close_active_tab_switches() {
        let mut state = StateManager::new();

        let tab1 = state.create_tab("https://example.com".to_string());
        let tab2 = state.create_tab("https://github.com".to_string());

        state.switch_tab(tab1).unwrap();
        assert_eq!(state.active_tab_id(), Some(tab1));

        state.close_tab(tab1).unwrap();
        // Should auto-switch to remaining tab
        assert_eq!(state.active_tab_id(), Some(tab2));
    }

    #[test]
    fn test_update_tab_title() {
        let mut state = StateManager::new();

        let tab_id = state.create_tab("https://example.com".to_string());
        state.update_tab_title(tab_id, "Example Domain".to_string());

        let tab = state.get_tab(tab_id).unwrap();
        assert_eq!(tab.title, "Example Domain");
    }

    #[test]
    fn test_update_tab_url() {
        let mut state = StateManager::new();

        let tab_id = state.create_tab("https://example.com".to_string());
        state.update_tab_url(tab_id, "https://github.com".to_string());

        let tab = state.get_tab(tab_id).unwrap();
        assert_eq!(tab.url, "https://github.com");
        assert_eq!(tab.title, "github.com"); // Auto-extracted
    }

    #[test]
    fn test_tab_loading_state() {
        let mut state = StateManager::new();

        let tab_id = state.create_tab("https://example.com".to_string());
        assert!(!state.get_tab(tab_id).unwrap().is_loading);

        state.set_tab_loading(tab_id, true);
        assert!(state.get_tab(tab_id).unwrap().is_loading);

        state.set_tab_loading(tab_id, false);
        assert!(!state.get_tab(tab_id).unwrap().is_loading);
    }

    #[test]
    fn test_navigation_state() {
        let mut state = StateManager::new();

        let tab_id = state.create_tab("https://example.com".to_string());
        let tab = state.get_tab(tab_id).unwrap();
        assert!(!tab.can_go_back);
        assert!(!tab.can_go_forward);

        state.set_tab_nav_state(tab_id, true, false);
        let tab = state.get_tab(tab_id).unwrap();
        assert!(tab.can_go_back);
        assert!(!tab.can_go_forward);
    }

    #[test]
    fn test_settings() {
        let mut state = StateManager::new();

        state.set_setting("theme".to_string(), "dark".to_string());
        assert_eq!(state.get_setting("theme"), Some(&"dark".to_string()));

        state.set_setting("theme".to_string(), "light".to_string());
        assert_eq!(state.get_setting("theme"), Some(&"light".to_string()));
    }

    #[test]
    fn test_get_all_tabs() {
        let mut state = StateManager::new();

        state.create_tab("https://example.com".to_string());
        state.create_tab("https://github.com".to_string());
        state.create_tab("https://google.com".to_string());

        let tabs = state.get_all_tabs();
        assert_eq!(tabs.len(), 3);
    }
}
