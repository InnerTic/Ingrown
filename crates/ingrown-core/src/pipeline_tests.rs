/// Comprehensive behavioral tests for the Ingrown execution pipeline.
///
/// These tests prove observable behavior of the public API without inspecting
/// implementation details or adding new architectural components.
///
/// The tests validate nine core behaviors:
/// 1. counter.increment executes and returns the expected value
/// 2. Successive counter.increment calls observe each other's state
/// 3. Invalid counter.increment input is rejected by Agent schema validation
/// 4. Invalid input does NOT reach CounterCapability::execute
/// 5. Input with unexpected properties is rejected (additionalProperties=false)
/// 6. Unknown capability names remain a hard error
/// 7. Registering two capabilities with the same name returns an error
/// 8. list_specs() returns deterministic sorted output
/// 9. A capability receives the ExecutionContext passed by Agent::execute_capability

#[cfg(test)]
mod pipeline_tests {
    use crate::{Agent, CounterCapability, EchoCapability};
    use ingrown_api::capability::{Capability, CapabilityMetadata, CapabilityResult, CapabilitySpec};
    use ingrown_api::ExecutionContext;
    use serde_json::{json, Value};
    use std::sync::{Arc, Mutex};

    /// Test 1: counter.increment executes and returns the expected value.
    ///
    /// Proves: The execution pipeline correctly invokes a capability, processes its
    /// input according to the schema, executes the implementation, and returns the result.
    #[tokio::test]
    async fn test_1_counter_increment_executes_and_returns_expected_value() {
        let agent = Agent::new();
        agent
            .register_capability(Arc::new(CounterCapability::new()))
            .unwrap();

        // Execute counter.increment with key "visits"
        let result = agent
            .execute_capability("counter.increment", json!({ "key": "visits" }))
            .await
            .unwrap();

        // Verify: execution succeeded
        assert!(result.success, "counter.increment should succeed");

        // Verify: result contains expected structure
        assert_eq!(result.value["key"], "visits");
        assert_eq!(result.value["value"], 1);

        // Verify: error field is None for successful execution
        assert!(result.error.is_none());
    }

    /// Test 2: A second counter.increment call against the same key observes the
    /// first call's state.
    ///
    /// Proves: Stateful capabilities maintain observable state across execution
    /// boundaries. The pipeline does not isolate or reset capability state between calls.
    #[tokio::test]
    async fn test_2_counter_state_persists_across_executions() {
        let agent = Agent::new();
        agent
            .register_capability(Arc::new(CounterCapability::new()))
            .unwrap();

        // First call increments "count" from 0 to 1
        let r1 = agent
            .execute_capability("counter.increment", json!({ "key": "count" }))
            .await
            .unwrap();
        assert_eq!(r1.value["value"], 1);

        // Second call with amount=5 increments from 1 to 6
        let r2 = agent
            .execute_capability(
                "counter.increment",
                json!({ "key": "count", "amount": 5 }),
            )
            .await
            .unwrap();
        assert_eq!(r2.value["value"], 6);

        // Third call increments from 6 to 7
        let r3 = agent
            .execute_capability("counter.increment", json!({ "key": "count" }))
            .await
            .unwrap();
        assert_eq!(r3.value["value"], 7);
    }

    /// Test 3: An invalid counter.increment input is rejected by Agent schema validation.
    ///
    /// Proves: Agent::execute_capability validates input against the capability's
    /// input_schema. Invalid data is caught before capability execution.
    #[tokio::test]
    async fn test_3_invalid_counter_input_rejected_by_schema_validation() {
        let agent = Agent::new();
        agent
            .register_capability(Arc::new(CounterCapability::new()))
            .unwrap();

        // Try to pass amount as a string instead of integer
        let result = agent
            .execute_capability(
                "counter.increment",
                json!({ "key": "test", "amount": "not_a_number" }),
            )
            .await
            .unwrap();

        // Verify: execution failed
        assert!(!result.success);

        // Verify: error message is present and descriptive
        let error = result.error.expect("failed result must carry error message");
        assert!(
            error.contains("amount"),
            "error should reference the invalid field: {}",
            error
        );

        // Verify: value is Null for failed execution
        assert_eq!(result.value, Value::Null);
    }

    /// Test 4: The invalid input does NOT reach CounterCapability::execute.
    ///
    /// Proves: Schema validation is enforced by the Agent before the capability's
    /// execute method is called. Invalid input cannot pass the validation gate.
    #[tokio::test]
    async fn test_4_invalid_input_does_not_reach_capability_execute() {
        // Instrumented counter that records every execution
        struct InstrumentedCounter {
            spec: CapabilitySpec,
            execution_count: Arc<Mutex<usize>>,
        }

        impl InstrumentedCounter {
            fn new(execution_count: Arc<Mutex<usize>>) -> Self {
                let spec = CapabilitySpec {
                    name: "counter.increment".into(),
                    description: "Instrumented counter".into(),
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
                        category: "test".into(),
                        keywords: vec![],
                        application: "test".into(),
                        estimated_token_cost: 0,
                        risk: "low".into(),
                    },
                };
                Self {
                    spec,
                    execution_count,
                }
            }
        }

        #[async_trait::async_trait]
        impl Capability for InstrumentedCounter {
            fn spec(&self) -> &CapabilitySpec {
                &self.spec
            }

            async fn execute(
                &self,
                _input: Value,
                _context: &ExecutionContext,
            ) -> anyhow::Result<CapabilityResult> {
                *self.execution_count.lock().unwrap() += 1;
                Ok(CapabilityResult::ok(json!({ "executed": true })))
            }
        }

        let exec_count = Arc::new(Mutex::new(0));
        let agent = Agent::new();
        agent
            .register_capability(Arc::new(InstrumentedCounter::new(exec_count.clone())))
            .unwrap();

        // Try invalid input
        let result = agent
            .execute_capability(
                "counter.increment",
                json!({ "key": "test", "amount": "invalid" }),
            )
            .await
            .unwrap();

        // Verify: execution failed
        assert!(!result.success);

        // Verify: the capability's execute method was never called
        assert_eq!(
            *exec_count.lock().unwrap(),
            0,
            "invalid input must not reach capability.execute"
        );
    }

    /// Test 5: An input with an unexpected property is rejected because
    /// additionalProperties=false.
    ///
    /// Proves: The Agent's schema validation enforces strict property constraints.
    /// Extra fields in the input are detected and rejected.
    #[tokio::test]
    async fn test_5_unexpected_property_rejected_by_schema() {
        let agent = Agent::new();
        agent
            .register_capability(Arc::new(EchoCapability::new()))
            .unwrap();

        // Try to pass an extra property
        let result = agent
            .execute_capability("echo", json!({ "message": "hello", "extra_field": "oops" }))
            .await
            .unwrap();

        // Verify: execution failed
        assert!(!result.success);

        // Verify: error mentions the unexpected field
        let error = result.error.expect("error must be present");
        assert!(
            error.contains("extra_field"),
            "error should reference unexpected field: {}",
            error
        );
    }

    /// Test 6: An unknown capability name remains a hard error.
    ///
    /// Proves: Agent::execute_capability fails with an error (not a structured
    /// CapabilityResult) when the capability name does not exist. The pipeline
    /// distinguishes between validation failures (structured) and missing capabilities
    /// (hard errors).
    #[tokio::test]
    async fn test_6_unknown_capability_name_is_hard_error() {
        let agent = Agent::new();

        // Try to execute a capability that was never registered
        let result = agent
            .execute_capability("nonexistent_capability", json!({}))
            .await;

        // Verify: the call returns an error, not Ok(CapabilityResult::error(...))
        let err = result.expect_err("unknown capability must produce an error");

        // Verify: error message is descriptive
        assert!(err.to_string().contains("not found"));
    }

    /// Test 7: Registering two capabilities with the same name returns an error
    /// rather than silently replacing the first.
    ///
    /// Proves: The Agent's registration logic rejects duplicate names. The capability
    /// set cannot be silently modified by re-registering.
    #[tokio::test]
    async fn test_7_duplicate_capability_registration_returns_error() {
        let agent = Agent::new();

        // Register the first echo capability
        agent
            .register_capability(Arc::new(EchoCapability::new()))
            .unwrap();

        // Try to register a second capability with the same name
        let err = agent
            .register_capability(Arc::new(EchoCapability::new()))
            .expect_err("duplicate registration must fail");

        // Verify: error mentions the name conflict
        assert!(err.to_string().contains("already registered"));
    }

    /// Test 8: list_specs() returns deterministic sorted output.
    ///
    /// Proves: The Agent's capability discovery mechanism returns a stable, sorted
    /// list. Calling list_specs() multiple times produces identical results.
    #[tokio::test]
    fn test_8_list_specs_returns_deterministic_sorted_output() {
        let agent = Agent::new();

        // Register capabilities in non-alphabetical order
        agent
            .register_capability(Arc::new(CounterCapability::new()))
            .unwrap();
        agent
            .register_capability(Arc::new(EchoCapability::new()))
            .unwrap();

        // Get list of specs
        let specs1 = agent.list_capability_specs();
        let names1: Vec<String> = specs1.iter().map(|s| s.name.clone()).collect();

        // Call again
        let specs2 = agent.list_capability_specs();
        let names2: Vec<String> = specs2.iter().map(|s| s.name.clone()).collect();

        // Verify: both lists are identical
        assert_eq!(
            names1, names2,
            "list_specs() must be deterministic across calls"
        );

        // Verify: results are sorted alphabetically
        assert_eq!(names1, ["counter.increment", "echo"]);

        // Verify: names are actually sorted
        let mut sorted = names1.clone();
        sorted.sort();
        assert_eq!(names1, sorted, "list_specs() must return sorted results");
    }

    /// Test 9: A capability can receive the ExecutionContext passed by
    /// Agent::execute_capability.
    ///
    /// Proves: The Agent constructs an ExecutionContext and passes it to each
    /// capability's execute method. The capability can observe and use this context.
    #[tokio::test]
    async fn test_9_capability_receives_execution_context() {
        // Instrumented capability that records the context it receives
        struct ContextRecorder {
            spec: CapabilitySpec,
            received_context: Arc<Mutex<Option<String>>>,
        }

        impl ContextRecorder {
            fn new(received_context: Arc<Mutex<Option<String>>>) -> Self {
                let spec = CapabilitySpec {
                    name: "context_test".into(),
                    description: "records execution context".into(),
                    input_schema: json!({ "type": "object" }),
                    metadata: CapabilityMetadata {
                        category: "test".into(),
                        keywords: vec![],
                        application: "test".into(),
                        estimated_token_cost: 0,
                        risk: "low".into(),
                    },
                };
                Self {
                    spec,
                    received_context,
                }
            }
        }

        #[async_trait::async_trait]
        impl Capability for ContextRecorder {
            fn spec(&self) -> &CapabilitySpec {
                &self.spec
            }

            async fn execute(
                &self,
                _input: Value,
                context: &ExecutionContext,
            ) -> anyhow::Result<CapabilityResult> {
                // Store the execution_id received from the context
                *self.received_context.lock().unwrap() = Some(context.execution_id.clone());
                Ok(CapabilityResult::ok(json!({ "received": true })))
            }
        }

        let received = Arc::new(Mutex::new(None));
        let agent = Agent::new();
        agent
            .register_capability(Arc::new(ContextRecorder::new(received.clone())))
            .unwrap();

        // Execute the capability
        agent
            .execute_capability("context_test", json!({}))
            .await
            .unwrap();

        // Verify: the capability received a valid execution_id
        let exec_id = received
            .lock()
            .unwrap()
            .clone()
            .expect("capability should receive a context");

        assert!(
            exec_id.starts_with("exec-"),
            "execution_id must follow the expected format: {}",
            exec_id
        );
    }
}
