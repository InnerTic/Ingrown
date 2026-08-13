//! End-to-end contract test: registration -> schema validation -> context
//! wiring -> execution -> side effects, exercised purely through the public
//! `ingrown_core` API.

use ingrown_core::{Agent, CounterCapability, EchoCapability};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn contract_registration_list_execute_roundtrip() {
    let agent = Agent::new();
    agent.register_capability(Arc::new(EchoCapability::new())).unwrap();
    agent.register_capability(Arc::new(CounterCapability::new())).unwrap();

    // Deterministic, sorted enumeration.
    let specs = agent.list_capability_specs();
    let names: Vec<String> = specs.iter().map(|s| s.name.clone()).collect();
    assert_eq!(names, ["counter.increment", "echo"]);

    // Every spec advertises an input_schema.
    for spec in &specs {
        assert_eq!(spec.input_schema["type"], "object");
        assert!(spec.input_schema["properties"].is_object());
    }

    // Valid execution.
    let res = agent.execute_capability("echo", json!({ "message": "hi" })).await.unwrap();
    assert!(res.success);
    assert_eq!(res.value, json!({ "message": "hi" }));

    // Invalid execution: structured failure, not a hard error.
    let res = agent.execute_capability("echo", json!({ "message": 42 })).await.unwrap();
    assert!(!res.success);
    assert!(res.error.is_some());

    // Unknown capability: hard error.
    let err = agent.execute_capability("does.not.exist", json!({})).await.unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn contract_counter_side_effects_persist() {
    let agent = Agent::new();
    agent.register_capability(Arc::new(CounterCapability::new())).unwrap();

    let a = agent.execute_capability("counter.increment", json!({ "key": "clicks" })).await.unwrap();
    assert_eq!(a.value["value"], json!(1));

    // Negative amounts are allowed, so the counter can also decrement.
    let b = agent.execute_capability("counter.increment", json!({ "key": "clicks", "amount": -3 })).await.unwrap();
    assert_eq!(b.value["value"], json!(-2));

    // Keys are namespaced independently.
    let c = agent.execute_capability("counter.increment", json!({ "key": "other" })).await.unwrap();
    assert_eq!(c.value["value"], json!(1));
    assert_eq!(a.value["key"], "clicks");
    assert_eq!(c.value["key"], "other");
}
