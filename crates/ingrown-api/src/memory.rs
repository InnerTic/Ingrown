//! Agent memory abstraction.
//!
//! Minimal interface for storing and retrieving information.
//! Future implementations may include embedding-based search, persistence, etc.

use serde_json::Value;

/// A single memory item.
#[derive(Clone, Debug)]
pub struct MemoryItem {
    pub key: String,
    pub value: Value,
}

/// Agent memory interface.
///
/// Deliberately minimal. Implementations may be in-memory, persistent, semantic, etc.
#[async_trait::async_trait]
pub trait Memory: Send + Sync {
    /// Store a value in memory.
    async fn store(&mut self, key: String, value: Value) -> anyhow::Result<()>;

    /// Retrieve a value from memory.
    async fn retrieve(&self, key: &str) -> anyhow::Result<Option<Value>>;

    /// List all memory items (for debugging/inspection).
    async fn list(&self) -> anyhow::Result<Vec<MemoryItem>>;
}
