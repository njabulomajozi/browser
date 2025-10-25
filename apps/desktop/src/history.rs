//! Browser navigation history
//!
//! Implements back/forward navigation using the Array with Index pattern.
//! This is simpler than two-stack approach and easier to persist later.
//!
//! # Architecture
//!
//! - HistoryEntry: Single navigation entry (URL, title, timestamp)
//! - TabHistory: Manages history stack with current position
//!
//! # Navigation Behavior
//!
//! - Navigate forward: Add entry at current_index+1, truncate future entries
//! - Go back: Decrement current_index
//! - Go forward: Increment current_index
//!
//! # Example
//!
//! ```
//! let mut history = TabHistory::new();
//! history.push("https://example.com", Some("Example"));
//! history.push("https://wikipedia.org", Some("Wikipedia"));
//!
//! // Now at wikipedia.org
//! assert!(history.can_go_back());
//! assert!(!history.can_go_forward());
//!
//! // Go back to example.com
//! let entry = history.go_back().unwrap();
//! assert_eq!(entry.url, "https://example.com");
//!
//! // Now can go forward
//! assert!(history.can_go_forward());
//! ```

use std::time::Instant;

/// A single entry in the navigation history
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    /// The URL of the page
    pub url: String,
    /// The page title (if available)
    pub title: Option<String>,
    /// When this entry was created (for future persistence in Milestone 1.6)
    #[allow(dead_code)]
    pub visit_time: Instant,
}

impl HistoryEntry {
    /// Create a new history entry
    pub fn new(url: String, title: Option<String>) -> Self {
        Self {
            url,
            title,
            visit_time: Instant::now(),
        }
    }
}

/// Navigation history for a browser tab
///
/// Uses array-with-index pattern: stores entries in a Vec with current_index
/// pointing to the current position.
#[derive(Debug, Clone)]
pub struct TabHistory {
    /// All history entries
    entries: Vec<HistoryEntry>,
    /// Index of current entry (0-based)
    /// Invariant: if entries is non-empty, current_index < entries.len()
    current_index: Option<usize>,
}

impl TabHistory {
    /// Create a new empty history
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            current_index: None,
        }
    }

    /// Add a new entry to history (navigate forward)
    ///
    /// This truncates any forward history if we're in the middle of the stack.
    /// For example:
    /// - If history is [A, B, C] and current is B
    /// - Navigating to D results in [A, B, D] with current = D
    pub fn push(&mut self, url: String, title: Option<String>) {
        let entry = HistoryEntry::new(url, title);

        match self.current_index {
            None => {
                // First entry
                self.entries.push(entry);
                self.current_index = Some(0);
            }
            Some(index) => {
                // Truncate any forward history
                self.entries.truncate(index + 1);
                // Add new entry
                self.entries.push(entry);
                // Move to new entry
                self.current_index = Some(index + 1);
            }
        }
    }

    /// Check if we can go back in history
    pub fn can_go_back(&self) -> bool {
        self.current_index.is_some_and(|idx| idx > 0)
    }

    /// Check if we can go forward in history
    pub fn can_go_forward(&self) -> bool {
        self.current_index
            .is_some_and(|idx| idx < self.entries.len().saturating_sub(1))
    }

    /// Go back one entry in history
    ///
    /// Returns the previous entry if available, None if already at beginning
    pub fn go_back(&mut self) -> Option<&HistoryEntry> {
        if let Some(index) = self.current_index {
            if index > 0 {
                self.current_index = Some(index - 1);
                return self.entries.get(index - 1);
            }
        }
        None
    }

    /// Go forward one entry in history
    ///
    /// Returns the next entry if available, None if already at end
    pub fn go_forward(&mut self) -> Option<&HistoryEntry> {
        if let Some(index) = self.current_index {
            if index < self.entries.len() - 1 {
                self.current_index = Some(index + 1);
                return self.entries.get(index + 1);
            }
        }
        None
    }

    /// Get the current history entry
    #[allow(dead_code)]
    pub fn current(&self) -> Option<&HistoryEntry> {
        self.current_index
            .and_then(|idx| self.entries.get(idx))
    }

    /// Get the current URL (convenience method)
    #[allow(dead_code)]
    pub fn current_url(&self) -> Option<&str> {
        self.current().map(|entry| entry.url.as_str())
    }

    /// Get the current title (convenience method)
    #[allow(dead_code)]
    pub fn current_title(&self) -> Option<&str> {
        self.current()
            .and_then(|entry| entry.title.as_deref())
    }

    /// Get the total number of history entries
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if history is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get all entries (for debugging or display)
    #[allow(dead_code)]
    pub fn entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    /// Get the current index (for debugging)
    #[allow(dead_code)]
    pub fn current_index(&self) -> Option<usize> {
        self.current_index
    }
}

impl Default for TabHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_history_is_empty() {
        let history = TabHistory::new();
        assert!(history.is_empty());
        assert_eq!(history.len(), 0);
        assert!(history.current().is_none());
        assert!(!history.can_go_back());
        assert!(!history.can_go_forward());
    }

    #[test]
    fn test_push_first_entry() {
        let mut history = TabHistory::new();
        history.push("https://example.com".to_string(), Some("Example".to_string()));

        assert_eq!(history.len(), 1);
        assert_eq!(history.current_url(), Some("https://example.com"));
        assert_eq!(history.current_title(), Some("Example"));
        assert!(!history.can_go_back());
        assert!(!history.can_go_forward());
    }

    #[test]
    fn test_push_multiple_entries() {
        let mut history = TabHistory::new();
        history.push("https://example.com".to_string(), Some("Example".to_string()));
        history.push("https://wikipedia.org".to_string(), Some("Wikipedia".to_string()));
        history.push("https://github.com".to_string(), Some("GitHub".to_string()));

        assert_eq!(history.len(), 3);
        assert_eq!(history.current_url(), Some("https://github.com"));
        assert_eq!(history.current_title(), Some("GitHub"));
        assert!(history.can_go_back());
        assert!(!history.can_go_forward());
    }

    #[test]
    fn test_go_back() {
        let mut history = TabHistory::new();
        history.push("https://a.com".to_string(), Some("A".to_string()));
        history.push("https://b.com".to_string(), Some("B".to_string()));
        history.push("https://c.com".to_string(), Some("C".to_string()));

        // Go back from C to B
        let entry = history.go_back().unwrap();
        assert_eq!(entry.url, "https://b.com");
        assert_eq!(history.current_url(), Some("https://b.com"));
        assert!(history.can_go_back());
        assert!(history.can_go_forward());

        // Go back from B to A
        let entry = history.go_back().unwrap();
        assert_eq!(entry.url, "https://a.com");
        assert_eq!(history.current_url(), Some("https://a.com"));
        assert!(!history.can_go_back());
        assert!(history.can_go_forward());

        // Try to go back from A (should fail)
        assert!(history.go_back().is_none());
        assert_eq!(history.current_url(), Some("https://a.com"));
    }

    #[test]
    fn test_go_forward() {
        let mut history = TabHistory::new();
        history.push("https://a.com".to_string(), Some("A".to_string()));
        history.push("https://b.com".to_string(), Some("B".to_string()));
        history.push("https://c.com".to_string(), Some("C".to_string()));

        // Go back twice (C -> B -> A)
        history.go_back();
        history.go_back();
        assert_eq!(history.current_url(), Some("https://a.com"));

        // Go forward from A to B
        let entry = history.go_forward().unwrap();
        assert_eq!(entry.url, "https://b.com");
        assert_eq!(history.current_url(), Some("https://b.com"));
        assert!(history.can_go_back());
        assert!(history.can_go_forward());

        // Go forward from B to C
        let entry = history.go_forward().unwrap();
        assert_eq!(entry.url, "https://c.com");
        assert_eq!(history.current_url(), Some("https://c.com"));
        assert!(history.can_go_back());
        assert!(!history.can_go_forward());

        // Try to go forward from C (should fail)
        assert!(history.go_forward().is_none());
        assert_eq!(history.current_url(), Some("https://c.com"));
    }

    #[test]
    fn test_navigate_from_middle_truncates_forward() {
        let mut history = TabHistory::new();
        history.push("https://a.com".to_string(), Some("A".to_string()));
        history.push("https://b.com".to_string(), Some("B".to_string()));
        history.push("https://c.com".to_string(), Some("C".to_string()));

        // Go back to B
        history.go_back();
        assert_eq!(history.current_url(), Some("https://b.com"));
        assert_eq!(history.len(), 3);

        // Navigate to D from B - should truncate C
        history.push("https://d.com".to_string(), Some("D".to_string()));
        assert_eq!(history.len(), 3); // A, B, D
        assert_eq!(history.current_url(), Some("https://d.com"));
        assert!(history.can_go_back());
        assert!(!history.can_go_forward());

        // Verify C is gone
        history.go_back(); // D -> B
        assert_eq!(history.current_url(), Some("https://b.com"));
        history.go_back(); // B -> A
        assert_eq!(history.current_url(), Some("https://a.com"));
        assert!(!history.can_go_back());
    }

    #[test]
    fn test_back_forward_back_forward_sequence() {
        let mut history = TabHistory::new();
        history.push("https://1.com".to_string(), None);
        history.push("https://2.com".to_string(), None);
        history.push("https://3.com".to_string(), None);

        // At 3
        assert_eq!(history.current_url(), Some("https://3.com"));

        // Back to 2
        history.go_back();
        assert_eq!(history.current_url(), Some("https://2.com"));

        // Forward to 3
        history.go_forward();
        assert_eq!(history.current_url(), Some("https://3.com"));

        // Back to 2 again
        history.go_back();
        assert_eq!(history.current_url(), Some("https://2.com"));

        // Back to 1
        history.go_back();
        assert_eq!(history.current_url(), Some("https://1.com"));

        // Forward to 2
        history.go_forward();
        assert_eq!(history.current_url(), Some("https://2.com"));
    }

    #[test]
    fn test_can_go_back_forward_edge_cases() {
        let mut history = TabHistory::new();

        // Empty history
        assert!(!history.can_go_back());
        assert!(!history.can_go_forward());

        // Single entry
        history.push("https://a.com".to_string(), None);
        assert!(!history.can_go_back());
        assert!(!history.can_go_forward());

        // Two entries, at second
        history.push("https://b.com".to_string(), None);
        assert!(history.can_go_back());
        assert!(!history.can_go_forward());

        // Two entries, at first
        history.go_back();
        assert!(!history.can_go_back());
        assert!(history.can_go_forward());
    }

    #[test]
    fn test_entries_access() {
        let mut history = TabHistory::new();
        history.push("https://a.com".to_string(), Some("A".to_string()));
        history.push("https://b.com".to_string(), Some("B".to_string()));

        let entries = history.entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].url, "https://a.com");
        assert_eq!(entries[1].url, "https://b.com");
    }

    #[test]
    fn test_current_index() {
        let mut history = TabHistory::new();
        assert_eq!(history.current_index(), None);

        history.push("https://a.com".to_string(), None);
        assert_eq!(history.current_index(), Some(0));

        history.push("https://b.com".to_string(), None);
        assert_eq!(history.current_index(), Some(1));

        history.go_back();
        assert_eq!(history.current_index(), Some(0));

        history.go_forward();
        assert_eq!(history.current_index(), Some(1));
    }
}
