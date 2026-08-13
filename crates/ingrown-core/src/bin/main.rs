use ingrown_core::{Agent, CounterCapability, EchoCapability};
use serde_json::json;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Create the agent
    let agent = Agent::new();

    // 2. Register capabilities (duplicate registration would fail)
    agent.register_capability(Arc::new(EchoCapability::new()))?;
    agent.register_capability(Arc::new(CounterCapability::new()))?;

    // 3. List their CapabilitySpecs (deterministically sorted)
    let specs = agent.list_capability_specs();
    println!("Registered capabilities ({}):", specs.len());
    for s in &specs {
        println!("- {}: {}", s.name, s.description);
    }

    // 4. Execute with valid input
    let res = agent.execute_capability("echo", json!({ "message": "hello" })).await?;
    println!("\necho(\"hello\")          -> success={} value={}", res.success, res.value);

    // 5. Invalid input: schema validation is the agent's job, not the capability's
    let bad = agent.execute_capability("echo", json!({ "message": 42 })).await?;
    println!("echo(message: 42)    -> success={} error={:?}", bad.success, bad.error);

    // 6. Stateful capability: side effects persist across executions
    let c1 = agent.execute_capability("counter.increment", json!({ "key": "hits" })).await?;
    let c2 = agent.execute_capability("counter.increment", json!({ "key": "hits", "amount": 5 })).await?;
    println!("counter.increment(1) -> success={} value={}", c1.success, c1.value);
    println!("counter.increment(5) -> success={} value={}", c2.success, c2.value);

    Ok(())
}
