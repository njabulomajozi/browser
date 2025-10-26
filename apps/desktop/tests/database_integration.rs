//! Integration tests for Database layer
//!
//! Tests end-to-end database workflows including:
//! - Concurrent database access from multiple threads
//! - History operations (add, query, limits)
//! - Database persistence across connections
//! - Error handling for invalid operations

use std::thread;
use storage::Database;
use tempfile::NamedTempFile;

#[test]
fn test_history_persistence_across_connections() {
    // Arrange: Create database, add history
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path();

    {
        let db = Database::new(db_path).unwrap();
        db.add_history("https://example.com", Some("Example Domain"))
            .unwrap();
        db.add_history("https://github.com", Some("GitHub"))
            .unwrap();
        db.add_history("https://google.com", Some("Google"))
            .unwrap();
    } // Database closed

    // Act: Open new connection
    {
        let db = Database::new(db_path).unwrap();

        // Assert: History should persist
        let history = db.get_recent_history(10).unwrap();
        assert_eq!(history.len(), 3);

        assert_eq!(history[0].url, "https://google.com");
        assert_eq!(history[0].title, Some("Google".to_string()));

        assert_eq!(history[1].url, "https://github.com");
        assert_eq!(history[1].title, Some("GitHub".to_string()));

        assert_eq!(history[2].url, "https://example.com");
        assert_eq!(history[2].title, Some("Example Domain".to_string()));
    }
}

#[test]
fn test_concurrent_database_writes() {
    // Arrange
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_path_buf();

    // Act: Multiple threads writing to database
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let db_path = db_path.clone();
            thread::spawn(move || {
                let db = Database::new(&db_path).unwrap();
                for j in 0..5 {
                    db.add_history(
                        &format!("https://thread{}-entry{}.com", i, j),
                        Some(&format!("Thread {} Entry {}", i, j)),
                    )
                    .unwrap();
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Assert: All entries should be present (10 threads * 5 entries = 50)
    let db = Database::new(&db_path).unwrap();
    let history = db.get_recent_history(100).unwrap();
    assert_eq!(history.len(), 50);

    // Verify all entries unique
    let urls: Vec<_> = history.iter().map(|e| &e.url).collect();
    let unique_urls: std::collections::HashSet<_> = urls.iter().collect();
    assert_eq!(unique_urls.len(), 50); // All URLs should be unique
}

#[test]
fn test_history_limit_enforcement() {
    // Arrange
    let temp_file = NamedTempFile::new().unwrap();
    let db = Database::new(temp_file.path()).unwrap();

    // Act: Add 20 history entries
    for i in 0..20 {
        db.add_history(
            &format!("https://example{}.com", i),
            Some(&format!("Example {}", i)),
        )
        .unwrap();
    }

    // Assert: get_history(5) returns only 5 most recent
    let history = db.get_recent_history(5).unwrap();
    assert_eq!(history.len(), 5);

    // Verify newest entries returned (example19, example18, ..., example15)
    assert_eq!(history[0].url, "https://example19.com");
    assert_eq!(history[1].url, "https://example18.com");
    assert_eq!(history[2].url, "https://example17.com");
    assert_eq!(history[3].url, "https://example16.com");
    assert_eq!(history[4].url, "https://example15.com");

    // Assert: get_history(30) returns all 20 entries
    let history = db.get_recent_history(30).unwrap();
    assert_eq!(history.len(), 20);
}

#[test]
fn test_history_chronological_order() {
    // Arrange
    let temp_file = NamedTempFile::new().unwrap();
    let db = Database::new(temp_file.path()).unwrap();

    // Act: Add entries with slight delay to ensure timestamp ordering
    for i in 0..5 {
        db.add_history(
            &format!("https://example{}.com", i),
            Some(&format!("Example {}", i)),
        )
        .unwrap();
        thread::sleep(std::time::Duration::from_millis(10));
    }

    // Assert: History should be in reverse chronological order (newest first)
    let history = db.get_recent_history(10).unwrap();
    assert_eq!(history.len(), 5);

    for i in 0..5 {
        let expected_url = format!("https://example{}.com", 4 - i); // Reverse order
        assert_eq!(history[i].url, expected_url);
    }
}

#[test]
fn test_concurrent_reads_and_writes() {
    // Arrange
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_path_buf();

    // Pre-populate database
    {
        let db = Database::new(&db_path).unwrap();
        for i in 0..10 {
            db.add_history(
                &format!("https://initial{}.com", i),
                Some(&format!("Initial {}", i)),
            )
            .unwrap();
        }
    }

    // Act: Concurrent readers and writers
    let write_handles: Vec<_> = (0..5)
        .map(|i| {
            let db_path = db_path.clone();
            thread::spawn(move || {
                let db = Database::new(&db_path).unwrap();
                for j in 0..5 {
                    db.add_history(
                        &format!("https://write{}-{}.com", i, j),
                        Some(&format!("Write {} {}", i, j)),
                    )
                    .unwrap();
                }
            })
        })
        .collect();

    let read_handles: Vec<_> = (0..5)
        .map(|_| {
            let db_path = db_path.clone();
            thread::spawn(move || {
                let db = Database::new(&db_path).unwrap();
                for _ in 0..10 {
                    let history = db.get_recent_history(20).unwrap();
                    assert!(history.len() >= 10); // At least initial entries
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in write_handles.into_iter().chain(read_handles) {
        handle.join().unwrap();
    }

    // Assert: Final count should be 10 initial + 25 written = 35
    let db = Database::new(&db_path).unwrap();
    let history = db.get_recent_history(100).unwrap();
    assert_eq!(history.len(), 35);
}

#[test]
fn test_history_without_title() {
    // Arrange
    let temp_file = NamedTempFile::new().unwrap();
    let db = Database::new(temp_file.path()).unwrap();

    // Act: Add history entry without title
    db.add_history("https://example.com", None).unwrap();

    // Assert: Entry should exist with None title
    let history = db.get_recent_history(10).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].url, "https://example.com");
    assert_eq!(history[0].title, None);
}

#[test]
fn test_database_creation_and_schema() {
    // Arrange & Act
    let temp_file = NamedTempFile::new().unwrap();
    let db = Database::new(temp_file.path()).unwrap();

    // Assert: Can perform operations (schema initialized)
    db.add_history("https://example.com", Some("Example"))
        .unwrap();
    let history = db.get_recent_history(10).unwrap();
    assert_eq!(history.len(), 1);

    // Verify timestamp is set
    assert!(history[0].visit_time.timestamp() > 0);
}

#[test]
fn test_multiple_sequential_connections() {
    // Arrange
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path();

    // Act & Assert: Open and close database multiple times
    for i in 0..10 {
        let db = Database::new(db_path).unwrap();
        db.add_history(
            &format!("https://example{}.com", i),
            Some(&format!("Example {}", i)),
        )
        .unwrap();

        let history = db.get_recent_history(100).unwrap();
        assert_eq!(history.len(), i + 1);
    }

    // Final verification
    let db = Database::new(db_path).unwrap();
    let history = db.get_recent_history(100).unwrap();
    assert_eq!(history.len(), 10);
}

#[test]
fn test_empty_database() {
    // Arrange
    let temp_file = NamedTempFile::new().unwrap();
    let db = Database::new(temp_file.path()).unwrap();

    // Act & Assert: Empty database returns empty history
    let history = db.get_recent_history(10).unwrap();
    assert_eq!(history.len(), 0);
}

#[test]
fn test_database_error_handling() {
    use std::path::Path;

    // Arrange & Act: Try to create database at invalid path
    let result = Database::new(Path::new("/invalid/path/that/does/not/exist/db.sqlite"));

    // Assert: Should return error
    assert!(result.is_err());
}

#[test]
fn test_stress_test_many_entries() {
    // Arrange
    let temp_file = NamedTempFile::new().unwrap();
    let db = Database::new(temp_file.path()).unwrap();

    // Act: Add 1000 entries
    for i in 0..1000 {
        db.add_history(
            &format!("https://example{}.com", i),
            Some(&format!("Example {}", i)),
        )
        .unwrap();
    }

    // Assert: All entries present
    let history = db.get_recent_history(2000).unwrap();
    assert_eq!(history.len(), 1000);

    // Verify newest entry first
    assert_eq!(history[0].url, "https://example999.com");
    assert_eq!(history[999].url, "https://example0.com");

    // Verify limited query works
    let limited = db.get_recent_history(10).unwrap();
    assert_eq!(limited.len(), 10);
    assert_eq!(limited[0].url, "https://example999.com");
}
