//! Execution context for capabilities.
//!
//! Minimal context passed to capabilities during execution.
//! May be expanded with request tracing, rate limiting, etc.

use serde::{Deserialize, Serialize};

/// Context in which a capability executes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// Unique identifier for this execution
    pub execution_id: String,
    /// Optional parent context for nested executions
    pub parent_context: Option<Box<ExecutionContext>>,
}

impl ExecutionContext {
    /// Create a new root execution context.
    pub fn new() -> Self {
        Self {
            execution_id: uuid::Uuid::new_v4().to_string(),
            parent_context: None,
        }
    }

    /// Create a child context.
    pub fn child(&self) -> Self {
        Self {
            execution_id: uuid::Uuid::new_v4().to_string(),
            parent_context: Some(Box::new(self.clone())),
        }
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}
