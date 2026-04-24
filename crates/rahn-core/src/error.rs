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
    SelfLoop { node: String },
    /// The referenced node does not exist in the network.
    NodeMissing { id: String },
    /// A node with this identifier already exists.
    NodeExists { id: String },
    /// A link between these endpoints already exists (in either direction).
    LinkExists { a: String, b: String },
    /// The referenced link does not exist.
    LinkMissing { a: String, b: String },
    /// A node cannot be removed while links reference it.
    NodeInUse {
        id: String,
        links: Vec<(String, String)>,
    },
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelError::InvalidId { id, reason } => {
                write!(f, "invalid identifier {id:?}: {reason}")
            }
            ModelError::SelfLoop { node } => {
                write!(
                    f,
                    "invalid topology: self-loop on node {node:?} is prohibited"
                )
            }
            ModelError::NodeMissing { id } => {
                write!(f, "node {id:?} does not exist")
            }
            ModelError::NodeExists { id } => {
                write!(f, "node {id:?} already exists")
            }
            ModelError::LinkExists { a, b } => {
                write!(f, "link {a:?} <-> {b:?} already exists")
            }
            ModelError::LinkMissing { a, b } => {
                write!(f, "link {a:?} <-> {b:?} does not exist")
            }
            ModelError::NodeInUse { id, links } => {
                write!(
                    f,
                    "node {id:?} cannot be removed while {} link(s) reference it: {:?}",
                    links.len(),
                    links
                )
            }
        }
    }
}

impl std::error::Error for ModelError {}
