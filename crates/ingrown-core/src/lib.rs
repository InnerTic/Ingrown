//! Ingrown core: orchestration machinery on top of the `ingrown-api` contracts.
//!
//! `ingrown-api` defines the contracts; this crate owns the agent runtime:
//! capability registration, schema validation, and execution.

pub mod agent;
pub mod counter;
pub mod echo;
pub mod registry;
pub mod stubs;

mod validation;

pub use agent::Agent;
pub use counter::CounterCapability;
pub use echo::EchoCapability;
pub use registry::CapabilityRegistry;
pub use stubs::{MemoryStub, ModelProviderStub};
