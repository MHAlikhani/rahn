// SPDX-License-Identifier: Apache-2.0

//! Explicit state transitions (ADR 0005).
//!
//! A transition is a pure, total function:
//! `(Network, Operation) -> Result<Network, TransitionError>`.
//!
//! - No I/O, no clock, no randomness.
//! - Committed states are never mutated: every successful application
//!   returns a new [`Network`].
//! - Rejections are structured and explainable.

use rahn_core::{Metadata, Network};

/// The complete v0.1 operation vocabulary (ADR 0005). Deliberately minimal;
/// new operations require semantic specification and tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    AddNode { id: String, metadata: Metadata },
    RemoveNode { id: String },
    AddLink { a: String, b: String },
    RemoveLink { a: String, b: String },
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::AddNode { id, .. } => write!(f, "add_node {id}"),
            Operation::RemoveNode { id } => write!(f, "remove_node {id}"),
            Operation::AddLink { a, b } => write!(f, "add_link {a} {b}"),
            Operation::RemoveLink { a, b } => write!(f, "remove_link {a} {b}"),
        }
    }
}

/// Structured transition rejection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionError {
    /// The transition's target object violates model rules (duplicate,
    /// missing, invalid identifier, self-loop). The carried [`rahn_core::ModelError`]
    /// explains precisely.
    Invalid(rahn_core::ModelError),
}

impl std::fmt::Display for TransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransitionError::Invalid(e) => write!(f, "transition rejected: {e}"),
        }
    }
}

impl std::error::Error for TransitionError {}

impl From<rahn_core::ModelError> for TransitionError {
    fn from(e: rahn_core::ModelError) -> Self {
        TransitionError::Invalid(e)
    }
}

/// Apply an operation to a network, producing a new network or a structured
/// rejection. The input network is never mutated.
pub fn apply(network: &Network, op: &Operation) -> Result<Network, TransitionError> {
    let mut next = network.clone();
    match op {
        Operation::AddNode { id, metadata } => next.add_node(id, metadata.clone())?,
        Operation::RemoveNode { id } => next.remove_node(id)?,
        Operation::AddLink { a, b } => next.add_link(a, b)?,
        Operation::RemoveLink { a, b } => next.remove_link(a, b)?,
    }
    Ok(next)
}

/// Apply a sequence of operations in order, stopping at the first rejection.
pub fn apply_all(network: &Network, ops: &[Operation]) -> Result<Network, TransitionError> {
    let mut current = network.clone();
    for op in ops {
        current = apply(&current, op)?;
    }
    Ok(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rahn_core::ModelError;

    fn net() -> Network {
        let mut n = Network::empty();
        n.add_node("a", Metadata::new()).unwrap();
        n.add_node("b", Metadata::new()).unwrap();
        n
    }

    #[test]
    fn add_and_remove_node() {
        let n = net();
        let n2 = apply(&n, &Operation::AddNode { id: "c".into(), metadata: Metadata::new() }).unwrap();
        assert_eq!(n2.node_count(), 3);
        // Original untouched (purity).
        assert_eq!(n.node_count(), 2);
        let n3 = apply(&n2, &Operation::RemoveNode { id: "c".into() }).unwrap();
        assert_eq!(n3.node_count(), 2);
    }

    #[test]
    fn duplicate_add_is_rejected() {
        let n = net();
        assert!(matches!(
            apply(&n, &Operation::AddNode { id: "a".into(), metadata: Metadata::new() }),
            Err(TransitionError::Invalid(ModelError::NodeExists { .. }))
        ));
    }

    #[test]
    fn remove_missing_node_is_rejected() {
        let n = net();
        assert!(matches!(
            apply(&n, &Operation::RemoveNode { id: "zz".into() }),
            Err(TransitionError::Invalid(ModelError::NodeMissing { .. }))
        ));
    }

    #[test]
    fn link_lifecycle() {
        let n = net();
        let n2 = apply(&n, &Operation::AddLink { a: "a".into(), b: "b".into() }).unwrap();
        assert_eq!(n2.link_count(), 1);
        // Duplicate (even reversed) rejected.
        assert!(apply(&n2, &Operation::AddLink { a: "b".into(), b: "a".into() }).is_err());
        let n3 = apply(&n2, &Operation::RemoveLink { a: "b".into(), b: "a".into() }).unwrap();
        assert_eq!(n3.link_count(), 0);
    }

    #[test]
    fn link_to_missing_node_is_rejected() {
        let n = net();
        assert!(apply(&n, &Operation::AddLink { a: "a".into(), b: "nope".into() }).is_err());
    }

    #[test]
    fn remove_node_with_links_is_rejected() {
        let n = net();
        let n2 = apply(&n, &Operation::AddLink { a: "a".into(), b: "b".into() }).unwrap();
        assert!(apply(&n2, &Operation::RemoveNode { id: "a".into() }).is_err());
        // Order matters: removing the link first succeeds.
        let n3 = apply(&n2, &Operation::RemoveLink { a: "a".into(), b: "b".into() }).unwrap();
        assert!(apply(&n3, &Operation::RemoveNode { id: "a".into() }).is_ok());
    }

    #[test]
    fn apply_all_stops_at_first_rejection() {
        let n = net();
        let ops = vec![
            Operation::AddNode { id: "c".into(), metadata: Metadata::new() },
            Operation::AddNode { id: "c".into(), metadata: Metadata::new() }, // duplicate
            Operation::AddNode { id: "d".into(), metadata: Metadata::new() },
        ];
        assert!(apply_all(&n, &ops).is_err());
    }
}
