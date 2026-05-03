// SPDX-License-Identifier: Apache-2.0

//! Deterministic graph queries over the network topology (Stage 2).
//!
//! All queries operate on the **node graph induced by interface links**:
//! two nodes are adjacent when any of their interfaces is linked. This
//! keeps constitution semantics node-level (ADR 0011, decision 5) while
//! the physical topology is interface-level.
//!
//! Determinism rules: traversal order is fully determined by the network's
//! total order — neighbors are visited in sorted order, and shortest-path
//! ties are broken lexicographically (BFS over sorted adjacency). No
//! randomness, no environment dependence.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use rahn_core::Network;

/// Why a path query could not be answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathError {
    /// `from` or `to` does not exist in the network.
    EndpointMissing,
}

/// Sorted node-to-node adjacency induced by interface links.
pub fn adjacency(net: &Network) -> BTreeMap<&str, BTreeSet<&str>> {
    let mut adj: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for node in net.iter_nodes() {
        adj.entry(node.id.as_str()).or_default();
    }
    for link in net.iter_links() {
        adj.entry(link.a.node.as_str())
            .or_default()
            .insert(link.b.node.as_str());
        adj.entry(link.b.node.as_str())
            .or_default()
            .insert(link.a.node.as_str());
    }
    adj
}

/// Nodes directly adjacent to `node` (sorted). `Ok(None)` if the node does
/// not exist.
pub fn neighbors<'a>(net: &'a Network, node: &str) -> Option<BTreeSet<&'a str>> {
    net.node(node)?;
    let mut out = BTreeSet::new();
    for link in net.iter_links() {
        if link.a.node == node {
            out.insert(link.b.node.as_str());
        } else if link.b.node == node {
            out.insert(link.a.node.as_str());
        }
    }
    Some(out)
}

/// All nodes reachable from `start`, including `start` itself (sorted).
/// `Ok(None)` if the start node does not exist.
pub fn reachable_from<'a>(net: &'a Network, start: &'a str) -> Option<BTreeSet<&'a str>> {
    net.node(start)?;
    let adj = adjacency(net);
    let mut visited = BTreeSet::new();
    visited.insert(start);
    let mut queue = VecDeque::new();
    queue.push_back(start);
    while let Some(current) = queue.pop_front() {
        for next in adj.get(current).into_iter().flatten() {
            if visited.insert(next) {
                queue.push_back(next);
            }
        }
    }
    Some(visited)
}

/// Shortest path (fewest links) between two nodes.
///
/// Determinism: BFS over sorted adjacency; among equal-length shortest
/// paths, the lexicographically smallest node sequence is returned.
/// Returns `Ok(Some(path))` (including both endpoints), `Ok(None)` when no
/// path exists, or `Err(PathError::EndpointMissing)` when an endpoint does not exist.
pub fn shortest_path(
    net: &Network,
    from: &str,
    to: &str,
) -> Result<Option<Vec<String>>, PathError> {
    if net.node(from).is_none() || net.node(to).is_none() {
        return Err(PathError::EndpointMissing);
    }
    if from == to {
        return Ok(Some(vec![from.to_owned()]));
    }
    let adj = adjacency(net);
    // BFS with sorted frontier expansion: the first time a node is reached
    // determines its canonical shortest path (predecessor chosen by BFS
    // order, which is lexicographic among equal-length options).
    let mut prev: BTreeMap<&str, &str> = BTreeMap::new();
    let mut visited: BTreeSet<&str> = BTreeSet::new();
    visited.insert(from);
    let mut queue = VecDeque::new();
    queue.push_back(from);
    while let Some(current) = queue.pop_front() {
        for next in adj.get(current).into_iter().flatten() {
            if visited.insert(next) {
                prev.insert(next, current);
                if *next == to {
                    // Reconstruct.
                    let mut path = vec![to.to_owned()];
                    let mut cursor = to;
                    while let Some(&p) = prev.get(cursor) {
                        path.push(p.to_owned());
                        cursor = p;
                    }
                    path.reverse();
                    return Ok(Some(path));
                }
                queue.push_back(next);
            }
        }
    }
    Ok(None)
}

/// Connected components of the node graph, each sorted; components are
/// returned sorted by their smallest member.
pub fn components(net: &Network) -> Vec<Vec<String>> {
    let adj = adjacency(net);
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut out = Vec::new();
    for node in net.iter_nodes() {
        let id = node.id.as_str();
        if seen.contains(id) {
            continue;
        }
        let mut component = Vec::new();
        let mut queue = VecDeque::new();
        seen.insert(id);
        queue.push_back(id);
        while let Some(current) = queue.pop_front() {
            component.push(current.to_owned());
            for next in adj.get(current).into_iter().flatten() {
                if seen.insert(next) {
                    queue.push_back(next);
                }
            }
        }
        component.sort();
        out.push(component);
    }
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use rahn_core::{Endpoint, Metadata};

    /// line: a - b - c ; star: d-e, d-f
    fn net() -> Network {
        let mut n = Network::empty();
        for id in ["a", "b", "c", "d", "e", "f"] {
            n.add_node(id, Metadata::new()).unwrap();
        }
        for (node, iface) in [
            ("a", "eth0"),
            ("b", "eth0"),
            ("b", "eth1"),
            ("c", "eth0"),
            ("d", "eth0"),
            ("d", "eth1"),
            ("e", "eth0"),
            ("f", "eth0"),
        ] {
            n.add_interface(node, iface).unwrap();
        }
        for (an, ai, bn, bi) in [
            ("a", "eth0", "b", "eth0"),
            ("b", "eth1", "c", "eth0"),
            ("d", "eth0", "e", "eth0"),
            ("d", "eth1", "f", "eth0"),
        ] {
            n.add_link(
                Endpoint::new(an, ai).unwrap(),
                Endpoint::new(bn, bi).unwrap(),
            )
            .unwrap();
        }
        n
    }

    #[test]
    fn neighbors_are_symmetric_and_sorted() {
        let n = net();
        let nb = neighbors(&n, "b").unwrap();
        assert_eq!(nb.len(), 2);
        let v: Vec<&str> = nb.into_iter().collect();
        assert_eq!(v, vec!["a", "c"]);
        assert!(neighbors(&n, "ghost").is_none());
    }

    #[test]
    fn shortest_path_prefers_fewest_hops() {
        let n = net();
        let p = shortest_path(&n, "a", "c").unwrap().unwrap();
        assert_eq!(p, vec!["a", "b", "c"]);
        assert_eq!(shortest_path(&n, "a", "a").unwrap().unwrap(), vec!["a"]);
        assert_eq!(shortest_path(&n, "a", "e").unwrap(), None);
        assert!(shortest_path(&n, "a", "ghost").is_err());
    }

    #[test]
    fn shortest_path_ties_break_lexicographically() {
        // diamond: s - x, s - y, x - t, y - t
        let mut n = Network::empty();
        for id in ["s", "x", "y", "t"] {
            n.add_node(id, Metadata::new()).unwrap();
        }
        for (node, iface) in [
            ("s", "e0"),
            ("s", "e1"),
            ("x", "e0"),
            ("y", "e0"),
            ("t", "e0"),
            ("t", "e1"),
        ] {
            n.add_interface(node, iface).unwrap();
        }
        for (an, ai, bn, bi) in [
            ("s", "e0", "x", "e0"),
            ("s", "e1", "y", "e0"),
            ("x", "e0", "t", "e0"),
            ("y", "e0", "t", "e1"),
        ] {
            n.add_link(
                Endpoint::new(an, ai).unwrap(),
                Endpoint::new(bn, bi).unwrap(),
            )
            .unwrap();
        }
        let p = shortest_path(&n, "s", "t").unwrap().unwrap();
        assert_eq!(
            p,
            vec!["s", "x", "t"],
            "equal-length ties must pick the lexicographically smaller route"
        );
    }

    #[test]
    fn components_separate_disjoint_parts() {
        let n = net();
        let comps = components(&n);
        assert_eq!(comps.len(), 2);
        assert_eq!(comps[0], vec!["a", "b", "c"]);
        assert_eq!(comps[1], vec!["d", "e", "f"]);
    }

    #[test]
    fn reachable_from_spans_component() {
        let n = net();
        let r = reachable_from(&n, "e").unwrap();
        assert_eq!(r.len(), 3);
        assert!(reachable_from(&n, "ghost").is_none());
    }
}
