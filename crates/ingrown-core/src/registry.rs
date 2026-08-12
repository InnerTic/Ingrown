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

    pub fn register(&self, cap: Arc<dyn Capability>) {
        let name = cap.spec().name.clone();
        self.caps.lock().unwrap().insert(name, cap);
    }

    pub fn list_specs(&self) -> Vec<CapabilitySpec> {
        let caps = self.caps.lock().unwrap();
        caps.values().map(|c| c.spec().clone()).collect()
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Capability>> {
        self.caps.lock().unwrap().get(name).cloned()
    }
}
