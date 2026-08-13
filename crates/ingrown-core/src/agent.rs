use anyhow::{anyhow, Result};
use ingrown_api::capability::{Capability, CapabilityResult, CapabilitySpec};
use ingrown_api::ExecutionContext;
use serde_json::Value;
use std::sync::Arc;

use crate::registry::CapabilityRegistry;
use crate::validation;

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

    /// Register a capability. Fails if the name is already taken.
    pub fn register_capability(&self, cap: Arc<dyn Capability>) -> Result<()> {
        self.registry.register(cap)
    }

    pub fn list_capability_specs(&self) -> Vec<CapabilitySpec> {
        self.registry.list_specs()
    }

    pub fn lookup_capability(&self, name: &str) -> Option<Arc<dyn Capability>> {
        self.registry.get(name)
    }

    /// Execute a capability by name.
    ///
    /// The execution path is:
    ///   1. resolve the capability (or fail hard if unknown)
    ///   2. validate `input` against the capability's `input_schema`;
    ///      invalid input is returned as a structured `CapabilityResult::error`
    ///   3. construct an `ExecutionContext` and pass it to the capability
    pub async fn execute_capability(&self, name: &str, input: Value) -> Result<CapabilityResult> {
        let cap = self
            .lookup_capability(name)
            .ok_or_else(|| anyhow!("capability '{}' not found", name))?;

        if let Err(message) = validation::validate_input(cap.spec(), &input) {
            return Ok(CapabilityResult::error(message));
        }

        let ctx = ExecutionContext::new();
        cap.execute(input, &ctx).await
    }
}

impl Default for Agent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CounterCapability, EchoCapability};
    use serde_json::json;

    #[tokio::test]
    async fn execute_passes_context_to_capability() {
        // Snoop capability that records the context it received.
        struct Snoop {
            spec: CapabilitySpec,
            received: Arc<std::sync::Mutex<Option<String>>>,
        }

        impl Snoop {
            fn new(received: Arc<std::sync::Mutex<Option<String>>>) -> Self {
                let spec = CapabilitySpec {
                    name: "snoop".into(),
                    description: "records context".into(),
                    input_schema: json!({ "type": "object" }),
                    metadata: ingrown_api::capability::CapabilityMetadata {
                        category: "test".into(),
                        keywords: vec![],
                        application: "test".into(),
                        estimated_token_cost: 0,
                        risk: "low".into(),
                    },
                };
                Self { spec, received }
            }
        }

        #[async_trait::async_trait]
        impl Capability for Snoop {
            fn spec(&self) -> &CapabilitySpec {
                &self.spec
            }
            async fn execute(
                &self,
                _input: Value,
                context: &ExecutionContext,
            ) -> Result<CapabilityResult> {
                *self.received.lock().unwrap() = Some(context.execution_id.clone());
                Ok(CapabilityResult::ok(Value::Null))
            }
        }

        let received = Arc::new(std::sync::Mutex::new(None));
        let agent = Agent::new();
        agent.register_capability(Arc::new(Snoop::new(received.clone()))).unwrap();

        agent.execute_capability("snoop", json!({})).await.unwrap();

        let id = received.lock().unwrap().clone().expect("capability never ran");
        assert!(id.starts_with("exec-"), "unexpected execution id: {id}");
    }

    #[tokio::test]
    async fn invalid_input_yields_structured_failure_not_error() {
        let agent = Agent::new();
        agent.register_capability(Arc::new(EchoCapability::new())).unwrap();

        let res = agent
            .execute_capability("echo", json!({ "message": 42 }))
            .await
            .unwrap();

        assert!(!res.success);
        let err = res.error.expect("failed result should carry an error");
        assert!(err.contains("message"), "unexpected error: {err}");
    }

    #[tokio::test]
    async fn unknown_capability_fails_hard() {
        let agent = Agent::new();
        let err = agent.execute_capability("missing", json!({})).await.unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[tokio::test]
    async fn registration_rejects_duplicates() {
        let agent = Agent::new();
        agent.register_capability(Arc::new(EchoCapability::new())).unwrap();
        let err = agent.register_capability(Arc::new(EchoCapability::new())).unwrap_err();
        assert!(err.to_string().contains("already registered"));
    }

    #[tokio::test]
    async fn counter_state_persists_across_executions() {
        let agent = Agent::new();
        agent.register_capability(Arc::new(CounterCapability::new())).unwrap();

        let r1 = agent.execute_capability("counter.increment", json!({ "key": "hits" })).await.unwrap();
        let r2 = agent.execute_capability("counter.increment", json!({ "key": "hits", "amount": 4 })).await.unwrap();

        assert_eq!(r1.value["value"], json!(1));
        assert_eq!(r2.value["value"], json!(5));
    }
}
