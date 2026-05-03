// SPDX-License-Identifier: Apache-2.0

//! The core object model: [`Node`], [`Interface`], [`Link`], [`Network`],
//! and [`State`].
//!
//! Invariants enforced at construction (docs/spec/objects.md, ADR 0011):
//! - identifiers are non-empty, at most 64 characters, ASCII alphanumeric
//!   plus `-`, `_`, `.` (identical rules for node and interface names);
//! - interfaces are owned by nodes and uniquely named within them;
//! - links are undirected, connect *interfaces*, and are stored with
//!   normalized (lexicographically ordered) endpoints, so `a/eth0-b/eth0`
//!   and `b/eth0-a/eth0` are the same link;
//! - links between two interfaces of the same node are prohibited;
//! - links MUST reference existing interfaces;
//! - duplicate links cannot be represented (map key by endpoint pair).
//!
//! No floating point, no timestamps, no ambient state: the model contains
//! only what canonical serialization can represent deterministically.

use std::collections::BTreeMap;
use std::fmt;

use crate::error::ModelError;

/// Maximum length of a node or interface identifier.
pub const MAX_ID_LEN: usize = 64;

/// Free-form key/value metadata attached to objects.
///
/// Keys and values are bounded, UTF-8 strings; ordering is the map's own
/// total order so canonical serialization needs no extra rules here.
pub type Metadata = BTreeMap<String, String>;

/// Validate an object identifier (nodes and interface names).
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

/// A named network interface owned by a node (ADR 0011).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interface {
    pub name: String,
    pub metadata: Metadata,
}

impl Interface {
    pub fn new(name: impl Into<String>) -> Result<Self, ModelError> {
        let name = name.into();
        validate_id(&name)?;
        Ok(Interface {
            name,
            metadata: Metadata::new(),
        })
    }
}

/// An attachment point: `(node, interface)`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Endpoint {
    pub node: String,
    pub iface: String,
}

impl Endpoint {
    pub fn new(node: &str, iface: &str) -> Result<Self, ModelError> {
        validate_id(node)?;
        validate_id(iface)?;
        Ok(Endpoint {
            node: node.to_owned(),
            iface: iface.to_owned(),
        })
    }

    pub fn parse(s: &str) -> Result<Self, ModelError> {
        let (node, iface) = s.split_once('/').ok_or_else(|| ModelError::InvalidId {
            id: s.to_owned(),
            reason: "endpoints must be node/interface (e.g. web/eth0)",
        })?;
        Endpoint::new(node, iface)
    }

    pub fn as_key(&self) -> (String, String) {
        (self.node.clone(), self.iface.clone())
    }
}

impl fmt::Display for Endpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.node, self.iface)
    }
}

/// An undirected link between two interfaces, endpoints normalized so
/// `a <= b` (lexicographic over `(node, iface)`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    /// Lexicographically smaller endpoint.
    pub a: Endpoint,
    /// Lexicographically larger endpoint.
    pub b: Endpoint,
    pub metadata: Metadata,
}

impl Link {
    /// Construct a normalized link. Rejects self-loops and same-node loops.
    pub fn new(a: Endpoint, b: Endpoint) -> Result<Self, ModelError> {
        if a == b {
            return Err(ModelError::SelfLoop {
                endpoint: a.to_string(),
            });
        }
        if a.node == b.node {
            return Err(ModelError::SameNodeLoop { node: a.node });
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
    pub fn key(&self) -> (Endpoint, Endpoint) {
        (self.a.clone(), self.b.clone())
    }
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} <-> {}", self.a, self.b)
    }
}

/// A named network node with metadata and owned interfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub id: String,
    pub metadata: Metadata,
    interfaces: BTreeMap<String, Interface>,
}

impl Node {
    fn new(id: String, metadata: Metadata) -> Self {
        Node {
            id,
            metadata,
            interfaces: BTreeMap::new(),
        }
    }

    pub fn iter_interfaces(&self) -> impl Iterator<Item = &Interface> {
        self.interfaces.values()
    }

    pub fn interface(&self, name: &str) -> Option<&Interface> {
        self.interfaces.get(name)
    }

    pub fn interface_count(&self) -> usize {
        self.interfaces.len()
    }
}

/// A deterministic network: nodes (with interfaces) and links, with total
/// ordering everywhere.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Network {
    nodes: BTreeMap<String, Node>,
    links: BTreeMap<(Endpoint, Endpoint), Link>,
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
        self.nodes
            .insert(id.to_owned(), Node::new(id.to_owned(), metadata));
        Ok(())
    }

    /// Remove a node. Fails while any link references its interfaces.
    pub fn remove_node(&mut self, id: &str) -> Result<(), ModelError> {
        if !self.nodes.contains_key(id) {
            return Err(ModelError::NodeMissing { id: id.to_owned() });
        }
        let attached: Vec<(String, String)> = self
            .links
            .keys()
            .filter(|(a, b)| a.node == id || b.node == id)
            .map(|(a, b)| (a.to_string(), b.to_string()))
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

    /// Add an interface to a node.
    pub fn add_interface(&mut self, node: &str, name: &str) -> Result<(), ModelError> {
        let ifc = Interface::new(name)?;
        let n = self
            .nodes
            .get_mut(node)
            .ok_or_else(|| ModelError::NodeMissing {
                id: node.to_owned(),
            })?;
        if n.interfaces.contains_key(&ifc.name) {
            return Err(ModelError::InterfaceExists {
                node: node.to_owned(),
                name: ifc.name,
            });
        }
        n.interfaces.insert(ifc.name.clone(), ifc);
        Ok(())
    }

    /// Remove an interface. Fails while any link references it.
    pub fn remove_interface(&mut self, node: &str, name: &str) -> Result<(), ModelError> {
        if !self.nodes.contains_key(node) {
            return Err(ModelError::NodeMissing {
                id: node.to_owned(),
            });
        }
        let attached: Vec<(String, String)> = self
            .links
            .keys()
            .filter(|(a, b)| {
                (a.node == node && a.iface == name) || (b.node == node && b.iface == name)
            })
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();
        if !attached.is_empty() {
            return Err(ModelError::InterfaceInUse {
                node: node.to_owned(),
                name: name.to_owned(),
                links: attached,
            });
        }
        let n = self.nodes.get_mut(node).expect("checked above");
        n.interfaces
            .remove(name)
            .ok_or_else(|| ModelError::InterfaceMissing {
                node: node.to_owned(),
                name: name.to_owned(),
            })?;
        Ok(())
    }

    /// Add a link between two existing interfaces. Rejects self-loops,
    /// same-node loops, and duplicates.
    pub fn add_link(&mut self, a: Endpoint, b: Endpoint) -> Result<(), ModelError> {
        self.ensure_endpoint(&a)?;
        self.ensure_endpoint(&b)?;
        let link = Link::new(a, b)?;
        let key = link.key();
        if self.links.contains_key(&key) {
            return Err(ModelError::LinkExists {
                a: key.0.to_string(),
                b: key.1.to_string(),
            });
        }
        self.links.insert(key, link);
        Ok(())
    }

    /// Remove a link. Accepts endpoints in either direction.
    pub fn remove_link(&mut self, a: Endpoint, b: Endpoint) -> Result<(), ModelError> {
        self.ensure_endpoint(&a)?;
        self.ensure_endpoint(&b)?;
        let key = if a <= b { (a, b) } else { (b, a) };
        self.links
            .remove(&key)
            .ok_or_else(|| ModelError::LinkMissing {
                a: key.0.to_string(),
                b: key.1.to_string(),
            })?;
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
        self.ensure_endpoint(&link.a)?;
        self.ensure_endpoint(&link.b)?;
        let key = link.key();
        if self.links.contains_key(&key) {
            return Err(ModelError::LinkExists {
                a: key.0.to_string(),
                b: key.1.to_string(),
            });
        }
        self.links.insert(key, link);
        Ok(())
    }

    fn ensure_endpoint(&self, e: &Endpoint) -> Result<(), ModelError> {
        let n = self
            .nodes
            .get(&e.node)
            .ok_or_else(|| ModelError::NodeMissing { id: e.node.clone() })?;
        if n.interfaces.contains_key(&e.iface) {
            Ok(())
        } else {
            Err(ModelError::InterfaceMissing {
                node: e.node.clone(),
                name: e.iface.clone(),
            })
        }
    }

    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// Look up a link regardless of endpoint argument order.
    pub fn link(&self, a: &Endpoint, b: &Endpoint) -> Option<&Link> {
        let key = if a <= b {
            (a.clone(), b.clone())
        } else {
            (b.clone(), a.clone())
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

    pub fn interface_count(&self) -> usize {
        self.nodes.values().map(|n| n.interface_count()).sum()
    }
}

/// A RAHN state: the versioned container around a network.
///
/// v0.2 carries the topology component (nodes, interfaces, links) plus
/// metadata; policy, intent, and observations are deferred and MUST be
/// added through the ADR process (docs/spec/state.md).
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
        net.add_interface("a", "eth0").unwrap();
        net.add_interface("b", "eth0").unwrap();
        let b_e = Endpoint::new("b", "eth0").unwrap();
        let a_e = Endpoint::new("a", "eth0").unwrap();
        net.add_link(b_e.clone(), a_e.clone()).unwrap();
        assert!(net.link(&a_e, &b_e).is_some());
        assert!(net.link(&b_e, &a_e).is_some());
        assert_eq!(net.link_count(), 1);
        // Reversed re-add is a duplicate.
        assert!(net.add_link(a_e, b_e).is_err());
    }

    #[test]
    fn self_loops_and_same_node_loops_are_prohibited() {
        let e = Endpoint::new("x", "eth0").unwrap();
        assert!(matches!(
            Link::new(e.clone(), e.clone()),
            Err(ModelError::SelfLoop { .. })
        ));
        let e2 = Endpoint::new("x", "eth1").unwrap();
        assert!(matches!(
            Link::new(e, e2),
            Err(ModelError::SameNodeLoop { node }) if node == "x"
        ));
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
    fn endpoint_parse_requires_slash() {
        assert!(Endpoint::parse("web/eth0").is_ok());
        assert!(Endpoint::parse("web").is_err());
        assert!(Endpoint::parse("web/eth 0").is_err());
    }

    #[test]
    fn interfaces_are_node_scoped() {
        let mut net = Network::empty();
        net.add_node("a", Metadata::new()).unwrap();
        net.add_node("b", Metadata::new()).unwrap();
        net.add_interface("a", "eth0").unwrap();
        // Same interface name on another node is fine.
        net.add_interface("b", "eth0").unwrap();
        // Duplicate on the same node is not.
        assert!(net.add_interface("a", "eth0").is_err());
        assert!(net.add_interface("ghost", "eth0").is_err());
        // Interface cannot be removed while linked.
        net.add_link(
            Endpoint::new("a", "eth0").unwrap(),
            Endpoint::new("b", "eth0").unwrap(),
        )
        .unwrap();
        assert!(net.remove_interface("a", "eth0").is_err());
        // Node cannot be removed while any of its interfaces is linked.
        assert!(net.remove_node("a").is_err());
        net.remove_link(
            Endpoint::new("b", "eth0").unwrap(),
            Endpoint::new("a", "eth0").unwrap(),
        )
        .unwrap();
        net.remove_interface("a", "eth0").unwrap();
        assert_eq!(net.interface_count(), 1);
    }
}
