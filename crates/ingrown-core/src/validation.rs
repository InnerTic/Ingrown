use ingrown_api::capability::CapabilitySpec;
use serde_json::Value;

/// Validate `input` against the capability's declared `input_schema`.
///
/// Ingrown (the orchestrator) is the authority on whether arguments are valid,
/// not the individual capability implementations. Returns a human-readable
/// description of the first validation failure.
pub(crate) fn validate_input(spec: &CapabilitySpec, input: &Value) -> Result<(), String> {
    let validator = jsonschema::validator_for(&spec.input_schema)
        .map_err(|e| format!("invalid input_schema for '{}': {}", spec.name, e))?;
    validator
        .validate(input)
        .map_err(|e| format!("'{}': {}", e.instance_path(), e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ingrown_api::capability::{CapabilityMetadata, CapabilitySpec};
    use serde_json::json;

    fn spec() -> CapabilitySpec {
        CapabilitySpec {
            name: "test".into(),
            description: "test".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "message": { "type": "string" },
                    "count": { "type": "integer" }
                },
                "required": ["message"],
                "additionalProperties": false
            }),
            metadata: CapabilityMetadata {
                category: "test".into(),
                keywords: vec![],
                application: "test".into(),
                estimated_token_cost: 0,
                risk: "low".into(),
            },
        }
    }

    #[test]
    fn accepts_valid_input() {
        assert!(validate_input(&spec(), &json!({ "message": "hi" })).is_ok());
        assert!(validate_input(&spec(), &json!({ "message": "hi", "count": 3 })).is_ok());
    }

    #[test]
    fn rejects_missing_required_field() {
        let err = validate_input(&spec(), &json!({})).unwrap_err();
        assert!(err.contains("message"), "unexpected error: {err}");
    }

    #[test]
    fn rejects_wrong_type() {
        let err = validate_input(&spec(), &json!({ "message": 42 })).unwrap_err();
        assert!(err.contains("message"), "unexpected error: {err}");
    }

    #[test]
    fn rejects_unknown_properties() {
        let err = validate_input(&spec(), &json!({ "message": "hi", "extra": true })).unwrap_err();
        assert!(err.contains("extra"), "unexpected error: {err}");
    }
}
