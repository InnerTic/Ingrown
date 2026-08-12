//! Core capability abstractions for Ingrown.
//!
//! This module defines the foundational interfaces that all capabilities
//! (whether native Rust, MCP, Python, or external) must implement.
//! The goal is to make the agent agnostic to the underlying implementation.

pub mod capability;
pub mod context;
pub mod memory;
pub mod provider;

pub use capability::{Capability, CapabilityResult, CapabilitySpec};
pub use context::ExecutionContext;
pub use memory::Memory;
pub use provider::ModelProvider;
