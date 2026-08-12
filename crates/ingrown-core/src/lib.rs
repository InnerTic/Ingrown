pub mod registry;

use anyhow::Result;
use ingrown_api::capability::{Capability, CapabilityResult, CapabilitySpec, CapabilityMetadata};
use ingrown_api::ExecutionContext;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::registry::CapabilityRegistry;

/// Minimal Agent runtime.
#[derive(Clone)]
pub struct Agent {
    registry: CapabilityRegistry,
}

impl Agent {
    pub fn new() -> Self {
        Self {
            registry: CapabilityRegistry::new(),
        }
    }

    pub fn register_capability(&self, cap: Arc<dyn Capability>) {
        self.registry.register(cap);
    }

    pub fn list_capability_specs(&self) -> Vec<CapabilitySpec> {
        self.registry.list_specs()
    }

    pub fn lookup_capability(&self, name: &str) -> Option<Arc<dyn Capability>> {
        self.registry.get(name)
    }

    pub async fn execute_capability(&self, name: &str, input: Value) -> Result<CapabilityResult> {
        let cap = self
            .lookup_capability(name)
            .ok_or_else(|| anyhow::anyhow!("capability not found"))?;
        // create a fresh execution context for now
        let _ctx = ExecutionContext::new();
        let res = cap.execute(input).await?;
        Ok(res)
    }
}

/// Simple echo capability used for demonstration.
pub struct EchoCapability {
    spec: CapabilitySpec,
}

impl EchoCapability {
    pub fn new() -> Self {
        let spec = CapabilitySpec {
            name: "echo".into(),
            description: "Echoes the provided message".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "message": { "type": "string" } },
                "required": ["message"]
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

#[async_trait::async_trait]
impl Capability for EchoCapability {
    fn spec(&self) -> &CapabilitySpec {
        &self.spec
    }

    async fn execute(&self, input: Value) -> anyhow::Result<CapabilityResult> {
        let message = input
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing or invalid 'message' field"))?;
        let out = serde_json::json!({ "message": message });
        Ok(CapabilityResult::ok(out))
    }
}

/// Minimal in-memory Memory stub (not used by demo but provided).
pub struct MemoryStub {
    map: Mutex<HashMap<String, Value>>,
}

impl MemoryStub {
    pub fn new() -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl ingrown_api::Memory for MemoryStub {
    async fn store(&mut self, key: String, value: Value) -> anyhow::Result<()> {
        let mut m = self.map.lock().unwrap();
        m.insert(key, value);
        Ok(())
    }

    async fn retrieve(&self, key: &str) -> anyhow::Result<Option<Value>> {
        let m = self.map.lock().unwrap();
        Ok(m.get(key).cloned())
    }

    async fn list(&self) -> anyhow::Result<Vec<ingrown_api::memory::MemoryItem>> {
        let m = self.map.lock().unwrap();
        let mut v = Vec::new();
        for (k, val) in m.iter() {
            v.push(ingrown_api::memory::MemoryItem {
                key: k.clone(),
                value: val.clone(),
            });
        }
        Ok(v)
    }
}

/// Minimal ModelProvider stub (not used by demo but provided).
pub struct ModelProviderStub;

#[async_trait::async_trait]
impl ingrown_api::ModelProvider for ModelProviderStub {
    async fn chat(
        &self,
        _messages: Vec<ingrown_api::provider::ChatMessage>,
        _available_tools: Vec<CapabilitySpec>,
    ) -> anyhow::Result<ingrown_api::provider::ChatResponse> {
        Ok(ingrown_api::provider::ChatResponse {
            id: "stub".into(),
            message: "".into(),
            tool_calls: Vec::new(),
        })
    }
}
