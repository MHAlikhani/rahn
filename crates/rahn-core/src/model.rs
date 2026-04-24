// SPDX-License-Identifier: Apache-2.0

//! The core object model: [`Node`], [`Link`], [`Network`], and [`State`].
//!
//! Invariants enforced at construction (docs/spec/objects.md):
//! - identifiers are non-empty, at most 64 characters, ASCII alphanumeric
//!   plus `-`, `_`, `.`;
//! - links are undirected and stored with normalized (lexicographically
//!   ordered) endpoints, so `A-B` and `B-A` are the same link;
//! - self-loops are prohibited;
//! - links MUST reference existing nodes;
//! - duplicate links cannot be represented (map key by normalized endpoints).
//!
//! No floating point, no timestamps, no ambient state: the model contains
//! only what canonical serialization can represent deterministically.

use std::collections::BTreeMap;
use std::fmt;

use crate::error::ModelError;

/// Maximum length of a node or link endpoint identifier.
pub const MAX_ID_LEN: usize = 64;

/// Free-form key/value metadata attached to objects.
///
/// Keys and values are bounded, UTF-8 strings; ordering is the map's own
/// total order so canonical serialization needs no extra rules here.
pub type Metadata = BTreeMap<String, String>;

/// Validate an object identifier.
pub fn validate_id(id: &str) -> Result<(), ModelError> {
    if id.is_empty() {
        return Err(ModelError::InvalidId {
            id: id.to_owned(),
            reason: "must not be empty",
        });
    }
    if id.len() > MAX_ID_LEN {
        return Err(ModelError::InvalidId {
            id: id.to_owned(),
            reason: "exceeds 64 characters",
        });
    }
    for ch in id.chars() {
        let ok = ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.';
        if !ok {
            return Err(ModelError::InvalidId {
                id: id.to_owned(),
                reason: "only ASCII alphanumeric, '-', '_', '.' are allowed",
            });
        }
    }
    Ok(())
}

/// A named network node with metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub id: String,
    pub metadata: Metadata,
}

/// An undirected link between two nodes, endpoints normalized so `a <= b`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    /// Lexicographically smaller endpoint.
    pub a: String,
    /// Lexicographically larger endpoint.
    pub b: String,
    pub metadata: Metadata,
}

impl Link {
    /// Construct a normalized link. Rejects self-loops.
    pub fn new(a: impl Into<String>, b: impl Into<String>) -> Result<Self, ModelError> {
        let (a, b) = (a.into(), b.into());
        validate_id(&a)?;
        validate_id(&b)?;
        if a == b {
            return Err(ModelError::SelfLoop { node: a });
        }
        if a <= b {
            Ok(Link {
                a,
                b,
                metadata: Metadata::new(),
            })
        } else {
            Ok(Link {
                a: b,
                b: a,
                metadata: Metadata::new(),
            })
        }
    }

    /// Normalized endpoint key used for map ordering.
    pub fn key(&self) -> (String, String) {
        (self.a.clone(), self.b.clone())
    }
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} <-> {}", self.a, self.b)
    }
}

/// A deterministic network: nodes and links with total ordering everywhere.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Network {
    nodes: BTreeMap<String, Node>,
    links: BTreeMap<(String, String), Link>,
}

impl Network {
    pub fn empty() -> Self {
        Self::default()
    }

    /// Add a node. Fails if the identifier is invalid or already present.
    pub fn add_node(&mut self, id: &str, metadata: Metadata) -> Result<(), ModelError> {
        validate_id(id)?;
        if self.nodes.contains_key(id) {
            return Err(ModelError::NodeExists { id: id.to_owned() });
        }
        self.nodes.insert(
            id.to_owned(),
            Node {
                id: id.to_owned(),
                metadata,
            },
        );
        Ok(())
    }

    /// Remove a node. Fails while any link references it.
    pub fn remove_node(&mut self, id: &str) -> Result<(), ModelError> {
        if !self.nodes.contains_key(id) {
            return Err(ModelError::NodeMissing { id: id.to_owned() });
        }
        let attached: Vec<(String, String)> = self
            .links
            .keys()
            .filter(|(a, b)| a == id || b == id)
            .cloned()
            .collect();
        if !attached.is_empty() {
            return Err(ModelError::NodeInUse {
                id: id.to_owned(),
                links: attached,
            });
        }
        self.nodes.remove(id);
        Ok(())
    }

    /// Add a link between two existing nodes. Rejects self-loops and duplicates.
    pub fn add_link(&mut self, a: &str, b: &str) -> Result<(), ModelError> {
        if !self.nodes.contains_key(a) {
            return Err(ModelError::NodeMissing { id: a.to_owned() });
        }
        if !self.nodes.contains_key(b) {
            return Err(ModelError::NodeMissing { id: b.to_owned() });
        }
        let link = Link::new(a, b)?;
        let key = link.key();
        if self.links.contains_key(&key) {
            return Err(ModelError::LinkExists { a: key.0, b: key.1 });
        }
        self.links.insert(key, link);
        Ok(())
    }

    /// Remove a link. Fails if the link does not exist.
    pub fn remove_link(&mut self, a: &str, b: &str) -> Result<(), ModelError> {
        if !self.nodes.contains_key(a) {
            return Err(ModelError::NodeMissing { id: a.to_owned() });
        }
        if !self.nodes.contains_key(b) {
            return Err(ModelError::NodeMissing { id: b.to_owned() });
        }
        let (x, y) = if a <= b {
            (a.to_owned(), b.to_owned())
        } else {
            (b.to_owned(), a.to_owned())
        };
        self.links
            .remove(&(x.clone(), y.clone()))
            .ok_or(ModelError::LinkMissing { a: x, b: y })?;
        Ok(())
    }

    /// Insert an already-constructed (normalized) link.
    ///
    /// Used by deserialization. Enforces every structural invariant: the
    /// link must be normalized (`a < b`), both endpoints must exist, and no
    /// duplicate link may pre-exist.
    pub fn add_existing_link(&mut self, link: Link) -> Result<(), ModelError> {
        if link.a >= link.b {
            return Err(ModelError::InvalidId {
                id: format!("{}>{}", link.b, link.a),
                reason: "link endpoints must be normalized (a < b)",
            });
        }
        if !self.nodes.contains_key(&link.a) {
            return Err(ModelError::NodeMissing { id: link.a.clone() });
        }
        if !self.nodes.contains_key(&link.b) {
            return Err(ModelError::NodeMissing { id: link.b.clone() });
        }
        let key = link.key();
        if self.links.contains_key(&key) {
            return Err(ModelError::LinkExists { a: key.0, b: key.1 });
        }
        self.links.insert(key, link);
        Ok(())
    }

    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// Look up a link regardless of endpoint argument order.
    pub fn link(&self, a: &str, b: &str) -> Option<&Link> {
        let key = if a <= b {
            (a.to_owned(), b.to_owned())
        } else {
            (b.to_owned(), a.to_owned())
        };
        self.links.get(&key)
    }

    pub fn iter_nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    pub fn iter_links(&self) -> impl Iterator<Item = &Link> {
        self.links.values()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn link_count(&self) -> usize {
        self.links.len()
    }
}

/// A RAHN state: the versioned container around a network.
///
/// v0.1 carries only the topology component of the full conceptual model
/// (`State_t` in ARCHITECTURE.md); additional components (policy, intent,
/// observations) are deferred and MUST be added through the ADR process.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct State {
    pub network: Network,
}

impl State {
    pub fn empty() -> Self {
        Self::default()
    }
}

/// Machine-readable verification summary stored in commit records.
///
/// The full, human-explainable report lives in `rahn-verify`; this is the
/// durable, canonical summary (ADR 0005: commits record verification).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationSummary {
    pub passed: bool,
    /// Identifiers of invariants that failed, in evaluation order.
    pub failed_invariants: Vec<String>,
}

impl VerificationSummary {
    pub fn passed() -> Self {
        Self {
            passed: true,
            failed_invariants: Vec::new(),
        }
    }

    pub fn failed(ids: Vec<String>) -> Self {
        Self {
            passed: false,
            failed_invariants: ids,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_are_normalized_undirected() {
        let mut net = Network::empty();
        net.add_node("b", Metadata::new()).unwrap();
        net.add_node("a", Metadata::new()).unwrap();
        net.add_link("b", "a").unwrap();
        assert!(net.link("a", "b").is_some());
        assert!(net.link("b", "a").is_some());
        assert_eq!(net.link_count(), 1);
        // Reversed re-add is a duplicate.
        assert_eq!(
            net.add_link("a", "b"),
            Err(ModelError::LinkExists {
                a: "a".into(),
                b: "b".into()
            })
        );
    }

    #[test]
    fn self_loops_are_prohibited() {
        assert!(Link::new("x", "x").is_err());
    }

    #[test]
    fn identifiers_are_strict() {
        assert!(validate_id("web-01").is_ok());
        assert!(validate_id("").is_err());
        assert!(validate_id("has space").is_err());
        assert!(validate_id("emoji😀").is_err());
        assert!(validate_id(&"x".repeat(65)).is_err());
    }

    #[test]
    fn nodes_cannot_be_removed_while_linked() {
        let mut net = Network::empty();
        for id in ["a", "b"] {
            net.add_node(id, Metadata::new()).unwrap();
        }
        net.add_link("a", "b").unwrap();
        let err = net.remove_node("a").unwrap_err();
        assert!(matches!(err, ModelError::NodeInUse { .. }));
        net.remove_link("b", "a").unwrap();
        net.remove_node("a").unwrap();
    }
}
