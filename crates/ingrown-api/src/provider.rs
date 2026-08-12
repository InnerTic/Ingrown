//! Model provider abstraction.
//!
//! Interface for interacting with language models.
//! Future implementations will support different LLM backends.

use serde::{Deserialize, Serialize};

/// A message in the conversation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Role: "system", "user", or "assistant"
    pub role: String,
    /// Message content
    pub content: String,
}

/// A tool call made by the model.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique identifier for this call
    pub id: String,
    /// Name of the capability/tool
    pub name: String,
    /// Arguments as JSON
    pub arguments: serde_json::Value,
}

/// Response from the model.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Unique identifier for this response
    pub id: String,
    /// The assistant's message
    pub message: String,
    /// Any tool calls requested by the model
    pub tool_calls: Vec<ToolCall>,
}

/// Interface for language model providers.
#[async_trait::async_trait]
pub trait ModelProvider: Send + Sync {
    /// Send messages to the model and get a response.
    /// The model may request tool calls in its response.
    async fn chat(
        &self,
        messages: Vec<ChatMessage>,
        available_tools: Vec<crate::capability::CapabilitySpec>,
    ) -> anyhow::Result<ChatResponse>;
}
