// SPDX-License-Identifier: Apache-2.0

//! Core object model for RAHN: nodes, interfaces, links, networks, states.
//!
//! This crate defines *representation only*. Serialization, identity, and
//! transitions live in `rahn-state`; persistence in `rahn-store`; verification
//! in `rahn-verify`. The model MUST remain free of execution-backend concepts
//! (ADR 0008).
//!
//! Since ADR 0011 (canonical format v2), links connect **interfaces**;
//! interfaces are owned by nodes.

pub mod error;
pub mod model;

pub use error::ModelError;
pub use model::{
    validate_id, Endpoint, Interface, Link, Metadata, Network, State, VerificationSummary,
};
