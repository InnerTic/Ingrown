use ingrown_core::{Agent, EchoCapability};
use serde_json::json;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Create the agent
    let agent = Agent::new();

    // 2. Register EchoCapability
    let echo = Arc::new(EchoCapability::new());
    agent.register_capability(echo.clone());

    // 3. List its CapabilitySpec
    let specs = agent.list_capability_specs();
    println!("Registered capabilities:");
    for s in specs {
        println!("- {}: {}", s.name, s.description);
    }

    // 4. Execute it with {"message":"hello"}
    let input = json!({ "message": "hello" });
    let res = agent.execute_capability("echo", input).await?;

    // 5. Print the result.
    println!("Execution result: success={} value={}", res.success, res.value);

    Ok(())
}
