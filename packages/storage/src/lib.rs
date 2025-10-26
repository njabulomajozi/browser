//! Browser persistent storage
//!
//! SQLite-based storage for history, bookmarks, and settings.
//!
//! # Architecture
//!
//! - **Database**: Main database manager with connection and migration
//! - **HistoryEntry**: Browsing history record
//! - **Bookmark**: Saved bookmark with folder organization
//! - **Migrations**: Schema versioning system
//!
//! # Usage
//!
//! ```ignore
//! use storage::{Database, HistoryEntry};
//! use std::path::Path;
//!
//! let db = Database::new(Path::new("browser.db"))?;
//!
//! // Add history
//! db.add_history("https://example.com", Some("Example Domain"))?;
//!
//! // Search history
//! let results = db.search_history("example", 10)?;
//! ```

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::Path;
use tracing::{debug, info};

/// Database schema version
const SCHEMA_VERSION: i32 = 1;

/// Browsing history entry
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryEntry {
    pub id: i64,
    pub url: String,
    pub title: Option<String>,
    pub visit_time: DateTime<Utc>,
}

/// Bookmark entry
#[derive(Debug, Clone, PartialEq)]
pub struct Bookmark {
    pub id: i64,
    pub url: String,
    pub title: Option<String>,
    pub folder: String,
    pub created_at: DateTime<Utc>,
}

/// Browser database manager
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create the browser database
    pub fn new(path: &Path) -> Result<Self> {
        info!("Opening database at {:?}", path);

        let conn = Connection::open(path).context("Failed to open SQLite database")?;

        let mut db = Self { conn };
        db.run_migrations()
            .context("Failed to run database migrations")?;

        Ok(db)
    }

    /// Run database migrations
    fn run_migrations(&mut self) -> Result<()> {
        // Create schema_version table if not exists
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY
            )",
            [],
        )?;

        // Get current version
        let current_version: i32 = self
            .conn
            .query_row(
                "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        debug!("Current schema version: {}", current_version);

        if current_version < SCHEMA_VERSION {
            info!(
                "Migrating database from version {} to {}",
                current_version, SCHEMA_VERSION
            );
            self.migrate_to_v1()?;

            // Update schema version
            self.conn.execute(
                "INSERT OR REPLACE INTO schema_version (version) VALUES (?1)",
                params![SCHEMA_VERSION],
            )?;

            info!("Migration complete");
        }

        Ok(())
    }

    /// Migrate to schema version 1
    fn migrate_to_v1(&mut self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL,
                title TEXT,
                visit_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );

            CREATE INDEX IF NOT EXISTS idx_history_url ON history(url);
            CREATE INDEX IF NOT EXISTS idx_history_visit_time ON history(visit_time DESC);

            CREATE TABLE IF NOT EXISTS bookmarks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL UNIQUE,
                title TEXT,
                folder TEXT DEFAULT 'Unsorted',
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );

            CREATE INDEX IF NOT EXISTS idx_bookmarks_folder ON bookmarks(folder);

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT
            );
            "#,
        )?;

        Ok(())
    }

    /// Get a reference to the underlying connection
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    // ========== History Operations ==========

    /// Add a history entry
    pub fn add_history(&self, url: &str, title: Option<&str>) -> Result<i64> {
        let id = self.conn.execute(
            "INSERT INTO history (url, title, visit_time) VALUES (?1, ?2, ?3)",
            params![url, title, Utc::now().to_rfc3339()],
        )?;

        debug!("Added history entry: {} (id={})", url, id);
        Ok(id as i64)
    }

    /// Get recent history (limited by count)
    pub fn get_recent_history(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, url, title, visit_time FROM history
             ORDER BY visit_time DESC LIMIT ?1",
        )?;

        let entries = stmt.query_map(params![limit], |row| {
            Ok(HistoryEntry {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                visit_time: row
                    .get::<_, String>(3)?
                    .parse::<DateTime<Utc>>()
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;

        entries
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect history entries")
    }

    /// Search history by URL or title
    pub fn search_history(&self, query: &str, limit: usize) -> Result<Vec<HistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, url, title, visit_time FROM history
             WHERE url LIKE ?1 OR title LIKE ?1
             ORDER BY visit_time DESC LIMIT ?2",
        )?;

        let search_pattern = format!("%{}%", query);
        let entries = stmt.query_map(params![search_pattern, limit], |row| {
            Ok(HistoryEntry {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                visit_time: row
                    .get::<_, String>(3)?
                    .parse::<DateTime<Utc>>()
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;

        entries
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to search history")
    }

    /// Clear all history
    pub fn clear_history(&self) -> Result<()> {
        self.conn.execute("DELETE FROM history", [])?;
        info!("Cleared all history");
        Ok(())
    }

    // ========== Bookmark Operations ==========

    /// Add a bookmark
    pub fn add_bookmark(
        &self,
        url: &str,
        title: Option<&str>,
        folder: Option<&str>,
    ) -> Result<i64> {
        let folder = folder.unwrap_or("Unsorted");

        let id = self.conn.execute(
            "INSERT INTO bookmarks (url, title, folder, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![url, title, folder, Utc::now().to_rfc3339()],
        )?;

        debug!("Added bookmark: {} in folder '{}' (id={})", url, folder, id);
        Ok(id as i64)
    }

    /// Get all bookmarks
    pub fn get_bookmarks(&self) -> Result<Vec<Bookmark>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, url, title, folder, created_at FROM bookmarks
             ORDER BY created_at DESC",
        )?;

        let bookmarks = stmt.query_map([], |row| {
            Ok(Bookmark {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                folder: row.get(3)?,
                created_at: row
                    .get::<_, String>(4)?
                    .parse::<DateTime<Utc>>()
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;

        bookmarks
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to get bookmarks")
    }

    /// Get bookmarks in a specific folder
    pub fn get_bookmarks_by_folder(&self, folder: &str) -> Result<Vec<Bookmark>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, url, title, folder, created_at FROM bookmarks
             WHERE folder = ?1 ORDER BY created_at DESC",
        )?;

        let bookmarks = stmt.query_map(params![folder], |row| {
            Ok(Bookmark {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                folder: row.get(3)?,
                created_at: row
                    .get::<_, String>(4)?
                    .parse::<DateTime<Utc>>()
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;

        bookmarks
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to get bookmarks by folder")
    }

    /// Remove a bookmark by URL
    pub fn remove_bookmark(&self, url: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM bookmarks WHERE url = ?1", params![url])?;
        debug!("Removed bookmark: {}", url);
        Ok(())
    }

    // ========== Settings Operations ==========

    /// Get a setting value
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let result = self.conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );

        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Set a setting value
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;

        debug!("Set setting: {} = {}", key, value);
        Ok(())
    }

    /// Remove a setting
    pub fn remove_setting(&self, key: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM settings WHERE key = ?1", params![key])?;
        debug!("Removed setting: {}", key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_database_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let _db = Database::new(temp_file.path()).unwrap();
    }

    #[test]
    fn test_tables_exist() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).unwrap();

        // Verify tables were created
        let mut stmt = db
            .connection()
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap();

        let tables: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(Result::ok)
            .collect();

        assert!(tables.contains(&"history".to_string()));
        assert!(tables.contains(&"bookmarks".to_string()));
        assert!(tables.contains(&"settings".to_string()));
        assert!(tables.contains(&"schema_version".to_string()));
    }

    #[test]
    fn test_schema_version() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).unwrap();

        let version: i32 = db
            .connection()
            .query_row(
                "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(version, SCHEMA_VERSION);
    }

    // ========== History Tests ==========

    #[test]
    fn test_add_history() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).unwrap();

        let id = db
            .add_history("https://example.com", Some("Example Domain"))
            .unwrap();
        assert!(id > 0);
    }

    #[test]
    fn test_get_recent_history() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).unwrap();

        // Add multiple entries
        db.add_history("https://example.com", Some("Example"))
            .unwrap();
        db.add_history("https://github.com", Some("GitHub"))
            .unwrap();
        db.add_history("https://wikipedia.org", Some("Wikipedia"))
            .unwrap();

        // Get recent history
        let history = db.get_recent_history(2).unwrap();
        assert_eq!(history.len(), 2);

        // Most recent first (Wikipedia)
        assert_eq!(history[0].url, "https://wikipedia.org");
        assert_eq!(history[0].title, Some("Wikipedia".to_string()));
        assert_eq!(history[1].url, "https://github.com");
    }

    #[test]
    fn test_search_history() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).unwrap();

        db.add_history("https://example.com", Some("Example Domain"))
            .unwrap();
        db.add_history("https://github.com", Some("GitHub"))
            .unwrap();
        db.add_history("https://wikipedia.org", Some("Wikipedia"))
            .unwrap();

        // Search by URL
        let results = db.search_history("github", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "https://github.com");

        // Search by title
        let results = db.search_history("Example", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "https://example.com");

        // Search that matches multiple
        let results = db.search_history("https", 10).unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_clear_history() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).unwrap();

        db.add_history("https://example.com", Some("Example"))
            .unwrap();
        db.add_history("https://github.com", Some("GitHub"))
            .unwrap();

        // Verify entries exist
        let history = db.get_recent_history(10).unwrap();
        assert_eq!(history.len(), 2);

        // Clear history
        db.clear_history().unwrap();

        // Verify empty
        let history = db.get_recent_history(10).unwrap();
        assert_eq!(history.len(), 0);
    }

    // ========== Bookmark Tests ==========

    #[test]
    fn test_add_bookmark() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).unwrap();

        let id = db
            .add_bookmark("https://example.com", Some("Example"), Some("Work"))
            .unwrap();
        assert!(id > 0);
    }

    #[test]
    fn test_add_bookmark_default_folder() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).unwrap();

        db.add_bookmark("https://example.com", Some("Example"), None)
            .unwrap();

        let bookmarks = db.get_bookmarks().unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].folder, "Unsorted");
    }

    #[test]
    fn test_get_bookmarks() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).unwrap();

        db.add_bookmark("https://example.com", Some("Example"), Some("Work"))
            .unwrap();
        db.add_bookmark("https://github.com", Some("GitHub"), Some("Dev"))
            .unwrap();

        let bookmarks = db.get_bookmarks().unwrap();
        assert_eq!(bookmarks.len(), 2);
    }

    #[test]
    fn test_get_bookmarks_by_folder() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).unwrap();

        db.add_bookmark("https://example.com", Some("Example"), Some("Work"))
            .unwrap();
        db.add_bookmark("https://github.com", Some("GitHub"), Some("Dev"))
            .unwrap();
        db.add_bookmark(
            "https://stackoverflow.com",
            Some("StackOverflow"),
            Some("Dev"),
        )
        .unwrap();

        let dev_bookmarks = db.get_bookmarks_by_folder("Dev").unwrap();
        assert_eq!(dev_bookmarks.len(), 2);

        let work_bookmarks = db.get_bookmarks_by_folder("Work").unwrap();
        assert_eq!(work_bookmarks.len(), 1);
        assert_eq!(work_bookmarks[0].url, "https://example.com");
    }

    #[test]
    fn test_remove_bookmark() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).unwrap();

        db.add_bookmark("https://example.com", Some("Example"), None)
            .unwrap();
        db.add_bookmark("https://github.com", Some("GitHub"), None)
            .unwrap();

        // Verify 2 bookmarks
        let bookmarks = db.get_bookmarks().unwrap();
        assert_eq!(bookmarks.len(), 2);

        // Remove one
        db.remove_bookmark("https://example.com").unwrap();

        // Verify 1 bookmark remains
        let bookmarks = db.get_bookmarks().unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].url, "https://github.com");
    }

    #[test]
    fn test_bookmark_unique_url() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).unwrap();

        // First bookmark succeeds
        db.add_bookmark("https://example.com", Some("Example 1"), None)
            .unwrap();

        // Duplicate URL fails
        let result = db.add_bookmark("https://example.com", Some("Example 2"), None);
        assert!(result.is_err());
    }

    // ========== Settings Tests ==========

    #[test]
    fn test_set_and_get_setting() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).unwrap();

        db.set_setting("theme", "dark").unwrap();

        let value = db.get_setting("theme").unwrap();
        assert_eq!(value, Some("dark".to_string()));
    }

    #[test]
    fn test_get_nonexistent_setting() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).unwrap();

        let value = db.get_setting("nonexistent").unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn test_update_setting() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).unwrap();

        // Set initial value
        db.set_setting("theme", "light").unwrap();
        assert_eq!(db.get_setting("theme").unwrap(), Some("light".to_string()));

        // Update value
        db.set_setting("theme", "dark").unwrap();
        assert_eq!(db.get_setting("theme").unwrap(), Some("dark".to_string()));
    }

    #[test]
    fn test_remove_setting() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).unwrap();

        db.set_setting("theme", "dark").unwrap();

        // Verify setting exists
        assert_eq!(db.get_setting("theme").unwrap(), Some("dark".to_string()));

        // Remove setting
        db.remove_setting("theme").unwrap();

        // Verify setting removed
        assert_eq!(db.get_setting("theme").unwrap(), None);
    }

    // ========== Performance Tests ==========

    #[test]
    fn test_large_history_insert() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).unwrap();

        // Insert 1000 entries
        for i in 0..1000 {
            db.add_history(
                &format!("https://example{}.com", i),
                Some(&format!("Example {}", i)),
            )
            .unwrap();
        }

        // Verify count
        let history = db.get_recent_history(1000).unwrap();
        assert_eq!(history.len(), 1000);
    }

    #[test]
    fn test_concurrent_reads() {
        use std::thread;

        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();

        {
            let db = Database::new(&temp_path).unwrap();
            db.add_history("https://example.com", Some("Example"))
                .unwrap();
        }

        // Multiple threads reading
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let path = temp_path.clone();
                thread::spawn(move || {
                    let db = Database::new(&path).unwrap();
                    let history = db.get_recent_history(10).unwrap();
                    assert_eq!(history.len(), 1);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
