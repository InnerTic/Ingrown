//! Core capability abstraction.
//!
//! Separates the capability specification (what the agent needs to know)
//! from the capability implementation (what can actually execute it).

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::context::ExecutionContext;

/// Metadata about a capability.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CapabilityMetadata {
    /// Category for organizational purposes (e.g., "modeling", "scripting")
    pub category: String,
    /// Keywords for discovery
    pub keywords: Vec<String>,
    /// Application domain (e.g., "blender", "freecad", "general")
    pub application: String,
    /// Rough estimate of tokens this capability might consume
    pub estimated_token_cost: u32,
    /// Risk level (e.g., "low", "medium", "high")
    pub risk: String,
}

/// Static description of what a capability can do.
///
/// This is what the agent uses to decide whether to call a capability,
/// without needing to execute it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CapabilitySpec {
    /// Unique identifier (e.g., "echo", "blender.mesh.create")
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// JSON schema describing input parameters
    pub input_schema: Value,
    /// Metadata for filtering and risk assessment
    pub metadata: CapabilityMetadata,
}

/// Result from executing a capability.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CapabilityResult {
    /// Whether execution succeeded
    pub success: bool,
    /// The result value
    pub value: Value,
    /// Optional error message
    pub error: Option<String>,
}

impl CapabilityResult {
    /// Create a successful result.
    pub fn ok(value: Value) -> Self {
        Self {
            success: true,
            value,
            error: None,
        }
    }

    /// Create a failed result.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            value: Value::Null,
            error: Some(message.into()),
        }
    }
}

/// A capability that can be executed by an agent.
///
/// All capabilities (native, MCP, Python, external processes, etc.)
/// must implement this trait.
#[async_trait::async_trait]
pub trait Capability: Send + Sync {
    /// Get the specification of this capability.
    /// This is called frequently for discovery and planning.
    fn spec(&self) -> &CapabilitySpec;

    /// Execute this capability with the given input.
    /// The input is a JSON object matching the input_schema.
    ///
    /// `context` carries execution identity (id, parent chain) and is a future
    /// home for tracing, cancellation, and metadata. Capabilities receive it by
    /// reference and must not mutate orchestration state through it.
    async fn execute(&self, input: Value, context: &ExecutionContext) -> anyhow::Result<CapabilityResult>;
}
