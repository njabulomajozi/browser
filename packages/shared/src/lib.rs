//! Shared types and utilities for the browser
//!
//! This crate contains common types, error definitions, and utilities
//! used across the browser workspace.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Unique identifier for browser tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TabId(pub usize);

impl fmt::Display for TabId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tab({})", self.0)
    }
}

/// Browser-wide error types
#[derive(Error, Debug)]
pub enum BrowserError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Rendering error: {0}")]
    Rendering(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("IPC error: {0}")]
    Ipc(String),
}

/// Result type for browser operations
pub type Result<T> = std::result::Result<T, BrowserError>;
