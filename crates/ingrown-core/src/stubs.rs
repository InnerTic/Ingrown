use ingrown_api::capability::CapabilitySpec;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;

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

impl Default for MemoryStub {
    fn default() -> Self {
        Self::new()
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
