use anyhow::{anyhow, Result};
use ingrown_api::capability::{
    Capability, CapabilityMetadata, CapabilityResult, CapabilitySpec,
};
use ingrown_api::ExecutionContext;
use serde_json::{json, Value};

/// Simple echo capability used for demonstration.
pub struct EchoCapability {
    spec: CapabilitySpec,
}

impl EchoCapability {
    pub fn new() -> Self {
        let spec = CapabilitySpec {
            name: "echo".into(),
            description: "Echoes the provided message".into(),
            input_schema: json!({
                "type": "object",
                "properties": { "message": { "type": "string" } },
                "required": ["message"],
                "additionalProperties": false
            }),
            metadata: CapabilityMetadata {
                category: "general".into(),
                keywords: vec!["echo".into()],
                application: "general".into(),
                estimated_token_cost: 1,
                risk: "low".into(),
            },
        };
        Self { spec }
    }
}

impl Default for EchoCapability {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Capability for EchoCapability {
    fn spec(&self) -> &CapabilitySpec {
        &self.spec
    }

    async fn execute(&self, input: Value, _context: &ExecutionContext) -> Result<CapabilityResult> {
        let message = input
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("missing or invalid 'message' field"))?;
        Ok(CapabilityResult::ok(json!({ "message": message })))
    }
}
