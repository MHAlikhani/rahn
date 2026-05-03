// SPDX-License-Identifier: Apache-2.0

//! Fail-closed semantic merge (ADR 0007; interface-aware per ADR 0011).
//!
//! A merge is three-way over state semantics: base (common ancestor) + ours
//! + theirs. Rules:
//!
//! 1. Each side's changes are computed as object-level effects vs. base
//!    (nodes, interfaces, links — links carry endpoint identity).
//! 2. If both sides touch the same object, the merge succeeds ONLY if their
//!    effects are *identical*; otherwise it is a semantic conflict.
//! 3. Disjoint changes are applied deterministically (sorted order).
//! 4. The merged candidate is returned unverified; the caller MUST pass it
//!    through the normal verification gate before committing.
//!
//! False rejections are acceptable; false accepts are not.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use rahn_core::{Endpoint, Metadata, Network};
use rahn_state::transition::{apply, Operation, TransitionError};

/// An explainable merge rejection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeConflict {
    pub conflicts: Vec<String>,
}

impl fmt::Display for MergeConflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "merge rejected: {} semantic conflict(s):",
            self.conflicts.len()
        )?;
        for c in &self.conflicts {
            writeln!(f, "  - {c}")?;
        }
        Ok(())
    }
}

impl std::error::Error for MergeConflict {}

/// The exact effect one side had on an object, relative to base.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Effect {
    Added(Metadata),
    Removed,
}

/// Object-level changes for one side of the merge, with content.
#[derive(Debug, Default)]
struct SideChanges {
    added_nodes: BTreeMap<String, Metadata>,
    removed_nodes: BTreeSet<String>,
    added_interfaces: BTreeMap<(String, String), Metadata>,
    removed_interfaces: BTreeSet<(String, String)>,
    added_links: BTreeMap<(Endpoint, Endpoint), Metadata>,
    removed_links: BTreeSet<(Endpoint, Endpoint)>,
}

impl SideChanges {
    fn node_effect(&self, id: &str) -> Option<Effect> {
        if let Some(md) = self.added_nodes.get(id) {
            return Some(Effect::Added(md.clone()));
        }
        if self.removed_nodes.contains(id) {
            return Some(Effect::Removed);
        }
        None
    }

    fn interface_effect(&self, key: &(String, String)) -> Option<Effect> {
        if let Some(md) = self.added_interfaces.get(key) {
            return Some(Effect::Added(md.clone()));
        }
        if self.removed_interfaces.contains(key) {
            return Some(Effect::Removed);
        }
        None
    }

    fn link_effect(&self, key: &(Endpoint, Endpoint)) -> Option<Effect> {
        if let Some(md) = self.added_links.get(key) {
            return Some(Effect::Added(md.clone()));
        }
        if self.removed_links.contains(key) {
            return Some(Effect::Removed);
        }
        None
    }

    /// Operations in deterministic, dependency-safe order: removals first
    /// (links before interfaces before nodes), then additions (nodes before
    /// interfaces before links).
    fn operations(&self) -> Vec<Operation> {
        let mut ops = Vec::new();
        for key in &self.removed_links {
            ops.push(Operation::RemoveLink {
                a: key.0.clone(),
                b: key.1.clone(),
            });
        }
        for key in &self.removed_interfaces {
            ops.push(Operation::RemoveInterface {
                node: key.0.clone(),
                name: key.1.clone(),
            });
        }
        for id in &self.removed_nodes {
            ops.push(Operation::RemoveNode { id: id.clone() });
        }
        for (id, md) in &self.added_nodes {
            ops.push(Operation::AddNode {
                id: id.clone(),
                metadata: md.clone(),
            });
        }
        for key in self.added_interfaces.keys() {
            ops.push(Operation::AddInterface {
                node: key.0.clone(),
                name: key.1.clone(),
            });
        }
        for key in self.added_links.keys() {
            ops.push(Operation::AddLink {
                a: key.0.clone(),
                b: key.1.clone(),
            });
        }
        ops
    }
}

fn side_changes(base: &Network, side: &Network) -> SideChanges {
    let mut c = SideChanges::default();
    for node in side.iter_nodes() {
        if base.node(&node.id).is_none() {
            c.added_nodes.insert(node.id.clone(), node.metadata.clone());
            // An added node's interfaces must be materialized explicitly:
            // operations are the only mechanism merge uses to build state,
            // and links referencing those interfaces need them to exist.
            for ifc in node.iter_interfaces() {
                c.added_interfaces
                    .insert((node.id.clone(), ifc.name.clone()), ifc.metadata.clone());
            }
        }
    }
    for node in base.iter_nodes() {
        if side.node(&node.id).is_none() {
            c.removed_nodes.insert(node.id.clone());
        }
    }
    // Interfaces are compared only on nodes that exist on both sides
    // (added nodes contribute all their interfaces implicitly).
    for node in side.iter_nodes() {
        if let Some(base_node) = base.node(&node.id) {
            for ifc in node.iter_interfaces() {
                if base_node.interface(&ifc.name).is_none() {
                    c.added_interfaces
                        .insert((node.id.clone(), ifc.name.clone()), ifc.metadata.clone());
                }
            }
            for ifc in base_node.iter_interfaces() {
                if node.interface(&ifc.name).is_none() {
                    c.removed_interfaces
                        .insert((node.id.clone(), ifc.name.clone()));
                }
            }
        }
    }
    for link in side.iter_links() {
        if base.link(&link.a, &link.b).is_none() {
            c.added_links
                .insert((link.a.clone(), link.b.clone()), link.metadata.clone());
        }
    }
    for link in base.iter_links() {
        if side.link(&link.a, &link.b).is_none() {
            c.removed_links.insert((link.a.clone(), link.b.clone()));
        }
    }
    c
}

/// Three-way semantic merge. Returns the merged candidate state, which the
/// caller MUST verify before committing (ADR 0005/0006).
pub fn merge(base: &Network, ours: &Network, theirs: &Network) -> Result<Network, MergeConflict> {
    let ours_changes = side_changes(base, ours);
    let theirs_changes = side_changes(base, theirs);

    let mut conflicts: Vec<String> = Vec::new();

    // Nodes touched by both sides.
    let mut touched_nodes: BTreeSet<&String> = ours_changes.added_nodes.keys().collect();
    touched_nodes.extend(ours_changes.removed_nodes.iter());
    touched_nodes.extend(theirs_changes.added_nodes.keys());
    touched_nodes.extend(theirs_changes.removed_nodes.iter());
    for id in touched_nodes {
        if let (Some(o), Some(t)) = (ours_changes.node_effect(id), theirs_changes.node_effect(id)) {
            if o != t {
                conflicts.push(format!(
                    "node {id:?} changed on both branches differently (ours: {o:?}, theirs: {t:?})"
                ));
            }
        }
    }

    // Interfaces touched by both sides.
    let mut touched_ifaces: BTreeSet<(String, String)> =
        ours_changes.added_interfaces.keys().cloned().collect();
    touched_ifaces.extend(ours_changes.removed_interfaces.iter().cloned());
    touched_ifaces.extend(theirs_changes.added_interfaces.keys().cloned());
    touched_ifaces.extend(theirs_changes.removed_interfaces.iter().cloned());
    for key in touched_ifaces {
        if let (Some(o), Some(t)) = (
            ours_changes.interface_effect(&key),
            theirs_changes.interface_effect(&key),
        ) {
            if o != t {
                conflicts.push(format!(
                    "interface {}/{} changed on both branches differently (ours: {o:?}, theirs: {t:?})",
                    key.0, key.1
                ));
            }
        }
    }

    // Links touched by both sides.
    let mut touched_links: BTreeSet<(Endpoint, Endpoint)> =
        ours_changes.added_links.keys().cloned().collect();
    touched_links.extend(ours_changes.removed_links.iter().cloned());
    touched_links.extend(theirs_changes.added_links.keys().cloned());
    touched_links.extend(theirs_changes.removed_links.iter().cloned());
    for key in touched_links {
        if let (Some(o), Some(t)) = (
            ours_changes.link_effect(&key),
            theirs_changes.link_effect(&key),
        ) {
            if o != t {
                conflicts.push(format!(
                    "link {} <-> {} changed on both branches differently (ours: {o:?}, theirs: {t:?})",
                    key.0, key.1
                ));
            }
        }
    }

    if !conflicts.is_empty() {
        return Err(MergeConflict { conflicts });
    }

    // Disjoint (or identical) effects: apply ours, then theirs. An
    // identical double application fails benignly (add of an existing
    // object, remove of an absent one) and is tolerated only when the
    // effect is provably the same one the conflict check already approved.
    let mut merged = base.clone();
    merged = apply_ops(&merged, ours_changes.operations()).map_err(|e| MergeConflict {
        conflicts: vec![format!("internal merge error (ours): {e}")],
    })?;
    merged = apply_ops(&merged, theirs_changes.operations()).map_err(|e| MergeConflict {
        conflicts: vec![format!("internal merge error (theirs): {e}")],
    })?;
    Ok(merged)
}

fn apply_ops(net: &Network, ops: Vec<Operation>) -> Result<Network, TransitionError> {
    let mut current = net.clone();
    for op in ops {
        current = match apply(&current, &op) {
            Ok(next) => next,
            Err(_) => match op {
                Operation::AddNode { id, metadata } => {
                    if current
                        .node(&id)
                        .map(|n| n.metadata == metadata)
                        .unwrap_or(false)
                    {
                        current
                    } else {
                        return Err(TransitionError::Invalid(
                            rahn_core::ModelError::NodeExists { id },
                        ));
                    }
                }
                Operation::RemoveNode { id } => {
                    if current.node(&id).is_none() {
                        current
                    } else {
                        return Err(TransitionError::Invalid(
                            rahn_core::ModelError::NodeMissing { id },
                        ));
                    }
                }
                Operation::AddInterface { node, name } => {
                    if current
                        .node(&node)
                        .and_then(|n| n.interface(&name))
                        .map(|i| i.metadata.is_empty())
                        .unwrap_or(false)
                    {
                        current
                    } else {
                        return Err(TransitionError::Invalid(
                            rahn_core::ModelError::InterfaceExists { node, name },
                        ));
                    }
                }
                Operation::RemoveInterface { node, name } => {
                    if current
                        .node(&node)
                        .map(|n| n.interface(&name).is_none())
                        .unwrap_or(true)
                    {
                        current
                    } else {
                        return Err(TransitionError::Invalid(
                            rahn_core::ModelError::InterfaceMissing { node, name },
                        ));
                    }
                }
                Operation::AddLink { a, b } => {
                    if current.link(&a, &b).is_some() {
                        current
                    } else {
                        return Err(TransitionError::Invalid(
                            rahn_core::ModelError::LinkExists {
                                a: a.to_string(),
                                b: b.to_string(),
                            },
                        ));
                    }
                }
                Operation::RemoveLink { a, b } => {
                    if current.link(&a, &b).is_none() {
                        current
                    } else {
                        return Err(TransitionError::Invalid(
                            rahn_core::ModelError::LinkMissing {
                                a: a.to_string(),
                                b: b.to_string(),
                            },
                        ));
                    }
                }
            },
        };
    }
    Ok(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rahn_core::Metadata;

    fn net(nodes: &[&str], ifaces: &[(&str, &str)], links: &[(&str, &str, &str, &str)]) -> Network {
        let mut n = Network::empty();
        for id in nodes {
            n.add_node(id, Metadata::new()).unwrap();
        }
        for (node, iface) in ifaces {
            n.add_interface(node, iface).unwrap();
        }
        for (an, ai, bn, bi) in links {
            n.add_link(
                Endpoint::new(an, ai).unwrap(),
                Endpoint::new(bn, bi).unwrap(),
            )
            .unwrap();
        }
        n
    }

    #[test]
    fn disjoint_additions_merge() {
        // base: a-b; ours adds c and c-a; theirs adds d and d-b.
        let base = net(
            &["a", "b"],
            &[("a", "eth0"), ("b", "eth0")],
            &[("a", "eth0", "b", "eth0")],
        );
        let ours = net(
            &["a", "b", "c"],
            &[("a", "eth0"), ("b", "eth0"), ("c", "eth0")],
            &[("a", "eth0", "b", "eth0"), ("a", "eth0", "c", "eth0")],
        );
        let theirs = net(
            &["a", "b", "d"],
            &[("a", "eth0"), ("b", "eth0"), ("d", "eth0")],
            &[("a", "eth0", "b", "eth0"), ("b", "eth0", "d", "eth0")],
        );
        let merged = merge(&base, &ours, &theirs).unwrap();
        assert_eq!(merged.node_count(), 4);
        assert_eq!(merged.link_count(), 3);
        assert!(merged
            .link(
                &Endpoint::new("a", "eth0").unwrap(),
                &Endpoint::new("c", "eth0").unwrap()
            )
            .is_some());
        assert!(merged
            .link(
                &Endpoint::new("b", "eth0").unwrap(),
                &Endpoint::new("d", "eth0").unwrap()
            )
            .is_some());
    }

    #[test]
    fn disjoint_interface_additions_merge() {
        let base = net(&["a"], &[("a", "eth0")], &[]);
        let ours = net(&["a"], &[("a", "eth0"), ("a", "eth1")], &[]);
        let theirs = net(&["a"], &[("a", "eth0"), ("a", "eth2")], &[]);
        let merged = merge(&base, &ours, &theirs).unwrap();
        assert_eq!(merged.node("a").unwrap().interface_count(), 3);
    }

    #[test]
    fn both_sides_add_same_interface_identically_succeeds() {
        let base = net(&["a"], &[("a", "eth0")], &[]);
        let ours = net(&["a"], &[("a", "eth0"), ("a", "eth1")], &[]);
        let theirs = ours.clone();
        let merged = merge(&base, &ours, &theirs).unwrap();
        assert!(merged.node("a").unwrap().interface("eth1").is_some());
    }

    #[test]
    fn both_sides_remove_same_node_succeeds() {
        let base = net(&["a", "b", "z"], &[], &[]);
        let ours = net(&["a", "b"], &[], &[]);
        let theirs = net(&["a", "b"], &[], &[]);
        let merged = merge(&base, &ours, &theirs).unwrap();
        assert!(merged.node("z").is_none());
    }

    #[test]
    fn one_side_removes_node_other_removes_neighbor() {
        // base: a-b, b-c. ours removes b (and its links). theirs removes c.
        let base = net(
            &["a", "b", "c"],
            &[("a", "eth0"), ("b", "eth0"), ("b", "eth1"), ("c", "eth0")],
            &[("a", "eth0", "b", "eth0"), ("b", "eth1", "c", "eth0")],
        );
        let ours = net(&["a", "c"], &[("a", "eth0"), ("c", "eth0")], &[]);
        let theirs = net(
            &["a", "b"],
            &[("a", "eth0"), ("b", "eth0")],
            &[("a", "eth0", "b", "eth0")],
        );
        let merged = merge(&base, &ours, &theirs).unwrap();
        assert!(merged.node("b").is_none());
        assert!(merged.node("c").is_none(), "theirs removed c");
        assert!(merged.node("a").is_some());
    }

    #[test]
    fn unrelated_side_change_applies_cleanly() {
        // ours removes node x; theirs is unchanged. Merge = ours.
        let base = net(&["a", "b", "x"], &[], &[]);
        let ours = {
            let mut n = base.clone();
            n.remove_node("x").unwrap();
            n
        };
        let theirs = base.clone();
        let merged = merge(&base, &ours, &theirs).unwrap();
        assert!(merged.node("x").is_none());
        assert!(merged.node("a").is_some());
    }

    #[test]
    fn link_added_by_one_side_to_node_removed_by_other_fails_closed() {
        // base: z, w. ours removes z. theirs adds link w-z.
        let base = net(&["z", "w"], &[("z", "eth0"), ("w", "eth0")], &[]);
        let ours = net(&["w"], &[("w", "eth0")], &[]);
        let theirs = net(
            &["z", "w"],
            &[("z", "eth0"), ("w", "eth0")],
            &[("w", "eth0", "z", "eth0")],
        );
        let err = merge(&base, &ours, &theirs).unwrap_err();
        assert!(
            err.conflicts
                .iter()
                .any(|c| c.contains("internal merge error")),
            "{err}"
        );
    }

    #[test]
    fn added_node_content_is_preserved() {
        let mut base_net = Network::empty();
        base_net.add_node("a", Metadata::new()).unwrap();
        // ours adds x with metadata; theirs is unchanged.
        let mut ours = Network::empty();
        ours.add_node("a", Metadata::new()).unwrap();
        let mut md = Metadata::new();
        md.insert("role".to_owned(), "api".to_owned());
        ours.add_node("x", md.clone()).unwrap();
        let theirs = base_net.clone();

        let merged = merge(&base_net, &ours, &theirs).unwrap();
        assert_eq!(
            merged.node("x").unwrap().metadata,
            md,
            "merge must preserve added metadata"
        );
    }
}
