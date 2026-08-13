use anyhow::Result;
use ingrown_api::capability::{
    Capability, CapabilityMetadata, CapabilityResult, CapabilitySpec,
};
use ingrown_api::ExecutionContext;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Mutex;

/// Stateful counter capability.
///
/// Proves that the execution path carries meaningful state and side effects:
/// successive executions against the same key observe each other's results.
pub struct CounterCapability {
    spec: CapabilitySpec,
    state: Mutex<HashMap<String, i64>>,
}

impl CounterCapability {
    pub fn new() -> Self {
        let spec = CapabilitySpec {
            name: "counter.increment".into(),
            description: "Increments a named counter and returns its new value".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "key": { "type": "string" },
                    "amount": { "type": "integer", "default": 1 }
                },
                "required": ["key"],
                "additionalProperties": false
            }),
            metadata: CapabilityMetadata {
                category: "state".into(),
                keywords: vec!["counter".into(), "increment".into(), "state".into()],
                application: "general".into(),
                estimated_token_cost: 1,
                risk: "low".into(),
            },
        };
        Self {
            spec,
            state: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for CounterCapability {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Capability for CounterCapability {
    fn spec(&self) -> &CapabilitySpec {
        &self.spec
    }

    async fn execute(&self, input: Value, _context: &ExecutionContext) -> Result<CapabilityResult> {
        let key = input.get("key").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        let amount = input.get("amount").and_then(|v| v.as_i64()).unwrap_or(1);

        let mut state = self.state.lock().unwrap();
        let entry = state.entry(key.clone()).or_insert(0);
        *entry += amount;

        Ok(CapabilityResult::ok(json!({ "key": key, "value": *entry })))
    }
}
