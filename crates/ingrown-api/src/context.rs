//! Execution context for capabilities.
//!
//! Minimal context passed to capabilities during execution.
//! May be expanded with request tracing, rate limiting, etc.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_EXECUTION_ID: AtomicU64 = AtomicU64::new(1);

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
        let id = NEXT_EXECUTION_ID.fetch_add(1, Ordering::SeqCst);
        Self {
            execution_id: format!("exec-{}", id),
            parent_context: None,
        }
    }

    /// Create a child context.
    pub fn child(&self) -> Self {
        let id = NEXT_EXECUTION_ID.fetch_add(1, Ordering::SeqCst);
        Self {
            execution_id: format!("exec-{}", id),
            parent_context: Some(Box::new(self.clone())),
        }
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}
