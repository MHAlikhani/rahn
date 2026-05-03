// SPDX-License-Identifier: Apache-2.0

//! Explicit state transitions (ADR 0005; vocabulary extended by ADR 0011).
//!
//! A transition is a pure, total function:
//! `(Network, Operation) -> Result<Network, TransitionError>`.
//!
//! - No I/O, no clock, no randomness.
//! - Committed states are never mutated: every successful application
//!   returns a new [`Network`].
//! - Rejections are structured and explainable.

use rahn_core::{Endpoint, Metadata, Network};

/// The complete v0.2 operation vocabulary (ADR 0005 + ADR 0011).
/// Deliberately minimal; new operations require semantic specification
/// and tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    AddNode { id: String, metadata: Metadata },
    RemoveNode { id: String },
    AddInterface { node: String, name: String },
    RemoveInterface { node: String, name: String },
    AddLink { a: Endpoint, b: Endpoint },
    RemoveLink { a: Endpoint, b: Endpoint },
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::AddNode { id, .. } => write!(f, "add_node {id}"),
            Operation::RemoveNode { id } => write!(f, "remove_node {id}"),
            Operation::AddInterface { node, name } => write!(f, "add_interface {node}/{name}"),
            Operation::RemoveInterface { node, name } => {
                write!(f, "remove_interface {node}/{name}")
            }
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
        Operation::AddInterface { node, name } => next.add_interface(node, name)?,
        Operation::RemoveInterface { node, name } => next.remove_interface(node, name)?,
        Operation::AddLink { a, b } => next.add_link(a.clone(), b.clone())?,
        Operation::RemoveLink { a, b } => next.remove_link(a.clone(), b.clone())?,
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

    fn net() -> Network {
        let mut n = Network::empty();
        n.add_node("a", Metadata::new()).unwrap();
        n.add_node("b", Metadata::new()).unwrap();
        n.add_interface("a", "eth0").unwrap();
        n.add_interface("b", "eth0").unwrap();
        n
    }

    #[test]
    fn add_and_remove_node() {
        let n = net();
        let n2 = apply(
            &n,
            &Operation::AddNode {
                id: "c".into(),
                metadata: Metadata::new(),
            },
        )
        .unwrap();
        assert_eq!(n2.node_count(), 3);
        // Original untouched (purity).
        assert_eq!(n.node_count(), 2);
        let n3 = apply(&n2, &Operation::RemoveNode { id: "c".into() }).unwrap();
        assert_eq!(n3.node_count(), 2);
    }

    #[test]
    fn interface_lifecycle() {
        let n = net();
        let n2 = apply(
            &n,
            &Operation::AddInterface {
                node: "a".into(),
                name: "eth1".into(),
            },
        )
        .unwrap();
        assert_eq!(n2.interface_count(), 3);
        // Duplicate rejected.
        assert!(apply(
            &n2,
            &Operation::AddInterface {
                node: "a".into(),
                name: "eth1".into()
            }
        )
        .is_err());
        // Interface on missing node rejected.
        assert!(apply(
            &n2,
            &Operation::AddInterface {
                node: "zz".into(),
                name: "eth0".into()
            }
        )
        .is_err());
        let n3 = apply(
            &n2,
            &Operation::RemoveInterface {
                node: "a".into(),
                name: "eth1".into(),
            },
        )
        .unwrap();
        assert_eq!(n3.interface_count(), 2);
    }

    #[test]
    fn link_lifecycle_with_interfaces() {
        let n = net();
        let a = Endpoint::new("a", "eth0").unwrap();
        let b = Endpoint::new("b", "eth0").unwrap();
        let n2 = apply(
            &n,
            &Operation::AddLink {
                a: a.clone(),
                b: b.clone(),
            },
        )
        .unwrap();
        assert_eq!(n2.link_count(), 1);
        // Duplicate (even reversed) rejected.
        assert!(apply(
            &n2,
            &Operation::AddLink {
                a: b.clone(),
                b: a.clone()
            }
        )
        .is_err());
        let n3 = apply(&n2, &Operation::RemoveLink { a: b, b: a }).unwrap();
        assert_eq!(n3.link_count(), 0);
    }

    #[test]
    fn link_to_missing_interface_is_rejected() {
        let n = net();
        assert!(apply(
            &n,
            &Operation::AddLink {
                a: Endpoint::new("a", "eth0").unwrap(),
                b: Endpoint::new("b", "eth1").unwrap(),
            }
        )
        .is_err());
    }

    #[test]
    fn remove_node_with_links_is_rejected() {
        let n = net();
        let n2 = apply(
            &n,
            &Operation::AddLink {
                a: Endpoint::new("a", "eth0").unwrap(),
                b: Endpoint::new("b", "eth0").unwrap(),
            },
        )
        .unwrap();
        assert!(apply(&n2, &Operation::RemoveNode { id: "a".into() }).is_err());
        // Order matters: removing the link first succeeds.
        let n3 = apply(
            &n2,
            &Operation::RemoveLink {
                a: Endpoint::new("a", "eth0").unwrap(),
                b: Endpoint::new("b", "eth0").unwrap(),
            },
        )
        .unwrap();
        assert!(apply(&n3, &Operation::RemoveNode { id: "a".into() }).is_ok());
    }

    #[test]
    fn remove_interface_referenced_by_link_is_rejected() {
        let n = net();
        let n2 = apply(
            &n,
            &Operation::AddLink {
                a: Endpoint::new("a", "eth0").unwrap(),
                b: Endpoint::new("b", "eth0").unwrap(),
            },
        )
        .unwrap();
        assert!(apply(
            &n2,
            &Operation::RemoveInterface {
                node: "a".into(),
                name: "eth0".into()
            }
        )
        .is_err());
    }

    #[test]
    fn apply_all_stops_at_first_rejection() {
        let n = net();
        let ops = vec![
            Operation::AddNode {
                id: "c".into(),
                metadata: Metadata::new(),
            },
            Operation::AddNode {
                id: "c".into(),
                metadata: Metadata::new(),
            }, // duplicate
            Operation::AddNode {
                id: "d".into(),
                metadata: Metadata::new(),
            },
        ];
        assert!(apply_all(&n, &ops).is_err());
    }
}
