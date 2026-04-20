// SPDX-License-Identifier: Apache-2.0

//! Core object model for RAHN: nodes, links, networks, and states.
//!
//! This crate defines *representation only*. Serialization, identity, and
//! transitions live in `rahn-state`; persistence in `rahn-store`; verification
//! in `rahn-verify`. The model MUST remain free of execution-backend concepts
//! (ADR 0008).

pub mod error;
pub mod model;

pub use error::ModelError;
pub use model::{Link, Metadata, Network, State, VerificationSummary};
