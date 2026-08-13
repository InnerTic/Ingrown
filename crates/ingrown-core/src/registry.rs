use anyhow::{anyhow, Result};
use ingrown_api::capability::{Capability, CapabilitySpec};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Simple registry for capabilities.
#[derive(Clone, Default)]
pub struct CapabilityRegistry {
    caps: Arc<Mutex<HashMap<String, Arc<dyn Capability>>>>,
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self {
            caps: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a capability.
    ///
    /// Fails if a capability with the same name is already registered, so a
    /// caller can never silently change the agent's available capability set.
    pub fn register(&self, cap: Arc<dyn Capability>) -> Result<()> {
        let name = cap.spec().name.clone();
        let mut caps = self.caps.lock().unwrap();
        if caps.contains_key(&name) {
            return Err(anyhow!("capability '{}' is already registered", name));
        }
        caps.insert(name, cap);
        Ok(())
    }

    /// List all capability specs, deterministically sorted by name.
    pub fn list_specs(&self) -> Vec<CapabilitySpec> {
        let caps = self.caps.lock().unwrap();
        let mut specs: Vec<CapabilitySpec> = caps.values().map(|c| c.spec().clone()).collect();
        specs.sort_by(|a, b| a.name.cmp(&b.name));
        specs
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Capability>> {
        self.caps.lock().unwrap().get(name).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EchoCapability;

    #[test]
    fn register_rejects_duplicate_names() {
        let registry = CapabilityRegistry::new();
        registry.register(Arc::new(EchoCapability::new())).unwrap();

        let err = registry.register(Arc::new(EchoCapability::new())).unwrap_err();
        assert!(err.to_string().contains("already registered"));
    }

    #[test]
    fn list_specs_is_sorted_and_deterministic() {
        let registry = CapabilityRegistry::new();
        registry.register(Arc::new(crate::CounterCapability::new())).unwrap();
        registry.register(Arc::new(EchoCapability::new())).unwrap();

        let names: Vec<String> = registry.list_specs().iter().map(|s| s.name.clone()).collect();
        assert_eq!(names, ["counter.increment", "echo"]);

        let again: Vec<String> = registry.list_specs().iter().map(|s| s.name.clone()).collect();
        assert_eq!(again, names);
    }

    #[test]
    fn get_returns_none_for_unknown() {
        let registry = CapabilityRegistry::new();
        assert!(registry.get("nope").is_none());
    }
}
