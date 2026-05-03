// SPDX-License-Identifier: Apache-2.0

//! Semantic diff between networks (charter §16.E; ADR 0011).
//!
//! A diff reports object-level changes — added/removed nodes, interfaces,
//! and links — never a byte or text diff.

use rahn_core::{Endpoint, Network};

/// Semantic difference from network `from` to network `to`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Diff {
    /// Nodes present in `to` but not in `from`, sorted.
    pub added_nodes: Vec<String>,
    /// Nodes present in `from` but not in `to`, sorted.
    pub removed_nodes: Vec<String>,
    /// Interfaces `(node, name)` present in `to` but not in `from`, sorted.
    pub added_interfaces: Vec<(String, String)>,
    /// Interfaces `(node, name)` present in `from` but not in `to`, sorted.
    pub removed_interfaces: Vec<(String, String)>,
    /// Links present in `to` but not in `from`, sorted.
    pub added_links: Vec<(Endpoint, Endpoint)>,
    /// Links present in `from` but not in `to`, sorted.
    pub removed_links: Vec<(Endpoint, Endpoint)>,
}

impl Diff {
    pub fn is_empty(&self) -> bool {
        self == &Diff::default()
    }
}

/// Compute the semantic diff `from -> to`.
pub fn diff(from: &Network, to: &Network) -> Diff {
    let mut d = Diff::default();

    for node in to.iter_nodes() {
        match from.node(&node.id) {
            None => {
                d.added_nodes.push(node.id.clone());
                // An added node's interfaces are part of the change: they
                // must be reported (and re-created by diff-derived
                // operations) for links referencing them to exist.
                for ifc in node.iter_interfaces() {
                    d.added_interfaces.push((node.id.clone(), ifc.name.clone()));
                }
            }
            Some(old) => {
                for ifc in node.iter_interfaces() {
                    if old.interface(&ifc.name).is_none() {
                        d.added_interfaces.push((node.id.clone(), ifc.name.clone()));
                    }
                }
                for ifc in old.iter_interfaces() {
                    if node.interface(&ifc.name).is_none() {
                        d.removed_interfaces
                            .push((node.id.clone(), ifc.name.clone()));
                    }
                }
            }
        }
    }
    for node in from.iter_nodes() {
        if to.node(&node.id).is_none() {
            d.removed_nodes.push(node.id.clone());
            for ifc in node.iter_interfaces() {
                d.removed_interfaces
                    .push((node.id.clone(), ifc.name.clone()));
            }
        }
    }
    for link in to.iter_links() {
        if from.link(&link.a, &link.b).is_none() {
            d.added_links.push((link.a.clone(), link.b.clone()));
        }
    }
    for link in from.iter_links() {
        if to.link(&link.a, &link.b).is_none() {
            d.removed_links.push((link.a.clone(), link.b.clone()));
        }
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;
    use rahn_core::Metadata;

    fn build(
        nodes: &[&str],
        ifaces: &[(&str, &str)],
        links: &[(&str, &str, &str, &str)],
    ) -> Network {
        let mut net = Network::empty();
        for id in nodes {
            net.add_node(id, Metadata::new()).unwrap();
        }
        for (n, i) in ifaces {
            net.add_interface(n, i).unwrap();
        }
        for (an, ai, bn, bi) in links {
            net.add_link(
                Endpoint::new(an, ai).unwrap(),
                Endpoint::new(bn, bi).unwrap(),
            )
            .unwrap();
        }
        net
    }

    #[test]
    fn detects_additions_and_removals() {
        let base = build(
            &["a", "b"],
            &[("a", "eth0"), ("b", "eth0")],
            &[("a", "eth0", "b", "eth0")],
        );
        let next = build(&["a", "c"], &[("a", "eth0")], &[]);

        let d = diff(&base, &next);
        assert_eq!(d.added_nodes, vec!["c".to_string()]);
        assert_eq!(d.removed_nodes, vec!["b".to_string()]);
        assert_eq!(
            d.removed_interfaces,
            vec![("b".to_string(), "eth0".to_string())]
        );
        assert_eq!(
            d.removed_links,
            vec![(
                Endpoint::new("a", "eth0").unwrap(),
                Endpoint::new("b", "eth0").unwrap()
            )]
        );
        assert!(d.added_links.is_empty());
    }

    #[test]
    fn detects_interface_changes() {
        let base = build(&["a"], &[("a", "eth0")], &[]);
        let next = build(&["a"], &[("a", "eth0"), ("a", "eth1")], &[]);
        let d = diff(&base, &next);
        assert_eq!(
            d.added_interfaces,
            vec![("a".to_string(), "eth1".to_string())]
        );
        assert!(d.added_nodes.is_empty());
    }

    #[test]
    fn identical_networks_diff_empty() {
        let n = build(
            &["a", "b"],
            &[("a", "eth0"), ("b", "eth0")],
            &[("a", "eth0", "b", "eth0")],
        );
        assert!(diff(&n, &n).is_empty());
    }
}
