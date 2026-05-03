// SPDX-License-Identifier: Apache-2.0

//! Structured errors for the core object model.
//!
//! Every error carries the information needed to explain the failure
//! (DESIGN.md principle 14); errors are never bare strings at the API
//! boundary.

use std::fmt;

/// Errors produced when constructing or validating core objects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelError {
    /// An identifier violates the identifier rules (see [`crate::model`]).
    InvalidId { id: String, reason: &'static str },
    /// A link whose endpoints are identical (self-loop).
    SelfLoop { endpoint: String },
    /// A link between two interfaces of the same node (topological loop).
    SameNodeLoop { node: String },
    /// The referenced node does not exist in the network.
    NodeMissing { id: String },
    /// A node with this identifier already exists.
    NodeExists { id: String },
    /// The referenced interface does not exist on the node.
    InterfaceMissing { node: String, name: String },
    /// An interface with this name already exists on the node.
    InterfaceExists { node: String, name: String },
    /// The referenced link does not exist.
    LinkMissing { a: String, b: String },
    /// A link between these endpoints already exists (in either direction).
    LinkExists { a: String, b: String },
    /// A node cannot be removed while links reference it.
    NodeInUse {
        id: String,
        links: Vec<(String, String)>,
    },
    /// An interface cannot be removed while links reference it.
    InterfaceInUse {
        node: String,
        name: String,
        links: Vec<(String, String)>,
    },
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelError::InvalidId { id, reason } => {
                write!(f, "invalid identifier {id:?}: {reason}")
            }
            ModelError::SelfLoop { endpoint } => {
                write!(
                    f,
                    "invalid topology: self-loop on endpoint {endpoint:?} is prohibited"
                )
            }
            ModelError::SameNodeLoop { node } => {
                write!(
                    f,
                    "invalid topology: link between two interfaces of node {node:?} is prohibited"
                )
            }
            ModelError::NodeMissing { id } => {
                write!(f, "node {id:?} does not exist")
            }
            ModelError::NodeExists { id } => {
                write!(f, "node {id:?} already exists")
            }
            ModelError::InterfaceMissing { node, name } => {
                write!(f, "interface {name:?} does not exist on node {node:?}")
            }
            ModelError::InterfaceExists { node, name } => {
                write!(f, "interface {name:?} already exists on node {node:?}")
            }
            ModelError::LinkMissing { a, b } => {
                write!(f, "link {a:?} <-> {b:?} does not exist")
            }
            ModelError::LinkExists { a, b } => {
                write!(f, "link {a:?} <-> {b:?} already exists")
            }
            ModelError::NodeInUse { id, links } => {
                write!(
                    f,
                    "node {id:?} cannot be removed while {} link(s) reference it: {:?}",
                    links.len(),
                    links
                )
            }
            ModelError::InterfaceInUse { node, name, links } => {
                write!(
                    f,
                    "interface {name:?} on node {node:?} cannot be removed while {} link(s) reference it: {:?}",
                    links.len(),
                    links
                )
            }
        }
    }
}

impl std::error::Error for ModelError {}
