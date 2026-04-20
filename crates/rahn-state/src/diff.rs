// SPDX-License-Identifier: Apache-2.0

//! Semantic diff between networks (charter §16.E).
//!
//! A diff reports object-level changes — added/removed nodes and links,
//! and node metadata keys that changed — never a byte or text diff.

use rahn_core::Network;

/// Semantic difference from network `from` to network `to`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Diff {
    /// Nodes present in `to` but not in `from`, sorted.
    pub added_nodes: Vec<String>,
    /// Nodes present in `from` but not in `to`, sorted.
    pub removed_nodes: Vec<String>,
    /// `(node, key)` pairs whose metadata value changed, sorted.
    pub changed_node_metadata: Vec<(String, String)>,
    /// Links `(a, b)` present in `to` but not in `from`, sorted.
    pub added_links: Vec<(String, String)>,
    /// Links `(a, b)` present in `from` but not in `to`, sorted.
    pub removed_links: Vec<(String, String)>,
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
            None => d.added_nodes.push(node.id.clone()),
            Some(old) => {
                for (k, v) in &node.metadata {
                    if old.metadata.get(k) != Some(v) {
                        d.changed_node_metadata.push((node.id.clone(), k.clone()));
                    }
                }
                for k in old.metadata.keys() {
                    if !node.metadata.contains_key(k) {
                        d.changed_node_metadata.push((node.id.clone(), k.clone()));
                    }
                }
            }
        }
    }
    for node in from.iter_nodes() {
        if to.node(&node.id).is_none() {
            d.removed_nodes.push(node.id.clone());
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

    fn build(ids: &[(&str, &[(&str, &str)])], links: &[(&str, &str)]) -> Network {
        let mut net = Network::empty();
        for (id, md) in ids {
            let mut m = Metadata::new();
            for (k, v) in *md {
                m.insert(k.to_string(), v.to_string());
            }
            net.add_node(id, m).unwrap();
        }
        for (a, b) in links {
            net.add_link(a, b).unwrap();
        }
        net
    }

    #[test]
    fn detects_additions_and_removals() {
        let base = build(&[("a", &[]), ("b", &[])], &[("a", "b")]);
        let next = build(&[("a", &[]), ("c", &[])], &[]);

        let d = diff(&base, &next);
        assert_eq!(d.added_nodes, vec!["c".to_string()]);
        assert_eq!(d.removed_nodes, vec!["b".to_string()]);
        assert_eq!(d.removed_links, vec![("a".to_string(), "b".to_string())]);
        assert!(d.added_links.is_empty());
    }

    #[test]
    fn detects_metadata_changes() {
        let base = build(&[("a", &[("role", "x")])], &[]);
        let next = build(&[("a", &[("role", "y")])], &[]);
        let d = diff(&base, &next);
        assert_eq!(d.changed_node_metadata, vec![("a".to_string(), "role".to_string())]);
    }

    #[test]
    fn identical_networks_diff_empty() {
        let n = build(&[("a", &[]), ("b", &[])], &[("a", "b")]);
        assert!(diff(&n, &n).is_empty());
    }
}
