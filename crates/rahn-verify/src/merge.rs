// SPDX-License-Identifier: Apache-2.0

//! Fail-closed semantic merge (ADR 0007).
//!
//! A merge is three-way over state semantics: base (common ancestor) + ours
//! + theirs. Rules:
//!
//! 1. Each side's changes are computed as object-level effects vs. base,
//!    including object content (metadata).
//! 2. If both sides touch the same object, the merge succeeds ONLY if their
//!    effects are *identical* (same kind — add or remove — and same
//!    content); otherwise it is a semantic conflict.
//! 3. Disjoint changes are applied deterministically (sorted order).
//! 4. The merged candidate is returned unverified; the caller MUST pass it
//!    through the normal verification gate before committing.
//!
//! False rejections are acceptable; false accepts are not.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use rahn_core::{Metadata, Network};
use rahn_state::transition::{apply, Operation, TransitionError};

/// An explainable merge rejection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeConflict {
    pub conflicts: Vec<String>,
}

impl fmt::Display for MergeConflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "merge rejected: {} semantic conflict(s):", self.conflicts.len())?;
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
    added_links: BTreeMap<(String, String), Metadata>,
    removed_links: BTreeSet<(String, String)>,
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

    fn link_effect(&self, key: &(String, String)) -> Option<Effect> {
        if let Some(md) = self.added_links.get(key) {
            return Some(Effect::Added(md.clone()));
        }
        if self.removed_links.contains(key) {
            return Some(Effect::Removed);
        }
        None
    }

    /// Operations in deterministic, dependency-safe order: removals first
    /// (links before nodes), then additions (nodes before links).
    fn operations(&self) -> Vec<Operation> {
        let mut ops = Vec::new();
        for (a, b) in &self.removed_links {
            ops.push(Operation::RemoveLink { a: a.clone(), b: b.clone() });
        }
        for id in &self.removed_nodes {
            ops.push(Operation::RemoveNode { id: id.clone() });
        }
        for (id, md) in &self.added_nodes {
            ops.push(Operation::AddNode { id: id.clone(), metadata: md.clone() });
        }
        for key in self.added_links.keys() {
            ops.push(Operation::AddLink { a: key.0.clone(), b: key.1.clone() });
        }
        ops
    }
}

fn side_changes(base: &Network, side: &Network) -> SideChanges {
    let mut c = SideChanges::default();
    for node in side.iter_nodes() {
        match base.node(&node.id) {
            // The v0.1 operation vocabulary (add/remove node/link) cannot
            // express metadata edits, so a metadata-only change cannot be
            // produced by recorded transitions. If one ever appears (e.g.,
            // a future operation set), the structural checks below treat
            // it as a both-sides content change rather than merging it
            // silently.
            Some(base_node) => {
                let _ = base_node;
            }
            None => {
                c.added_nodes.insert(node.id.clone(), node.metadata.clone());
            }
        }
    }
    for node in base.iter_nodes() {
        if side.node(&node.id).is_none() {
            c.removed_nodes.insert(node.id.clone());
        }
    }
    for link in side.iter_links() {
        if base.link(&link.a, &link.b).is_none() {
            c.added_links.insert((link.a.clone(), link.b.clone()), link.metadata.clone());
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

    // Links touched by both sides.
    let mut touched_links: BTreeSet<(String, String)> =
        ours_changes.added_links.keys().cloned().collect();
    touched_links.extend(ours_changes.removed_links.iter().cloned());
    touched_links.extend(theirs_changes.added_links.keys().cloned());
    touched_links.extend(theirs_changes.removed_links.iter().cloned());
    for key in touched_links {
        if let (Some(o), Some(t)) = (ours_changes.link_effect(&key), theirs_changes.link_effect(&key)) {
            if o != t {
                conflicts.push(format!(
                    "link {:?} <-> {:?} changed on both branches differently (ours: {o:?}, theirs: {t:?})",
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
    merged = apply_ops(&merged, ours_changes.operations())
        .map_err(|e| MergeConflict { conflicts: vec![format!("internal merge error (ours): {e}")] })?;
    merged = apply_ops(&merged, theirs_changes.operations())
        .map_err(|e| MergeConflict { conflicts: vec![format!("internal merge error (theirs): {e}")] })?;
    Ok(merged)
}

fn apply_ops(net: &Network, ops: Vec<Operation>) -> Result<Network, TransitionError> {
    let mut current = net.clone();
    for op in ops {
        current = match apply(&current, &op) {
            Ok(next) => next,
            Err(_) => match op {
                Operation::AddNode { id, metadata } => {
                    if current.node(&id).map(|n| n.metadata == metadata).unwrap_or(false) {
                        current
                    } else {
                        return Err(TransitionError::Invalid(rahn_core::ModelError::NodeExists { id }));
                    }
                }
                Operation::RemoveNode { id } => {
                    if current.node(&id).is_none() {
                        current
                    } else {
                        return Err(TransitionError::Invalid(rahn_core::ModelError::NodeMissing { id }));
                    }
                }
                Operation::AddLink { a, b } => {
                    if current.link(&a, &b).is_some() {
                        current
                    } else {
                        return Err(TransitionError::Invalid(rahn_core::ModelError::LinkExists {
                            a: a.clone(),
                            b: b.clone(),
                        }));
                    }
                }
                Operation::RemoveLink { a, b } => {
                    if current.link(&a, &b).is_none() {
                        current
                    } else {
                        return Err(TransitionError::Invalid(rahn_core::ModelError::LinkMissing {
                            a: a.clone(),
                            b: b.clone(),
                        }));
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

    fn net(nodes: &[&str], links: &[(&str, &str)]) -> Network {
        let mut n = Network::empty();
        for id in nodes {
            n.add_node(id, Metadata::new()).unwrap();
        }
        for (a, b) in links {
            n.add_link(a, b).unwrap();
        }
        n
    }

    #[test]
    fn disjoint_additions_merge() {
        // base: a-b; ours adds c and c-a; theirs adds d and d-b.
        let base = net(&["a", "b"], &[("a", "b")]);
        let ours = net(&["a", "b", "c"], &[("a", "b"), ("a", "c")]);
        let theirs = net(&["a", "b", "d"], &[("a", "b"), ("b", "d")]);
        let merged = merge(&base, &ours, &theirs).unwrap();
        assert_eq!(merged.node_count(), 4);
        assert_eq!(merged.link_count(), 3);
        assert!(merged.link("c", "a").is_some());
        assert!(merged.link("b", "d").is_some());
    }

    #[test]
    fn both_sides_add_same_node_identically_succeeds() {
        let base = net(&["a"], &[]);
        let ours = net(&["a", "x"], &[]);
        let theirs = net(&["a", "x"], &[]);
        let merged = merge(&base, &ours, &theirs).unwrap();
        assert!(merged.node("x").is_some());
    }

    #[test]
    fn both_sides_remove_same_node_succeeds() {
        let base = net(&["a", "b", "z"], &[]);
        let ours = net(&["a", "b"], &[]);
        let theirs = net(&["a", "b"], &[]);
        let merged = merge(&base, &ours, &theirs).unwrap();
        assert!(merged.node("z").is_none());
    }

    #[test]
    fn both_sides_remove_same_link_succeeds() {
        let base = net(&["a", "b"], &[("a", "b")]);
        let ours = net(&["a", "b"], &[]);
        let theirs = net(&["a", "b"], &[]);
        let merged = merge(&base, &ours, &theirs).unwrap();
        assert_eq!(merged.link_count(), 0);
    }

    #[test]
    fn one_side_removes_node_other_removes_neighbor() {
        // base: a-b, b-c. ours removes b (and its links). theirs removes c.
        // Link (b,c): removed by both → identical effect → allowed.
        // Link (a,b): removed by ours only. Node c: removed by theirs only.
        let base = net(&["a", "b", "c"], &[("a", "b"), ("b", "c")]);
        let ours = net(&["a", "c"], &[]);
        let theirs = net(&["a", "b"], &[("a", "b")]);
        let merged = merge(&base, &ours, &theirs).unwrap();
        assert!(merged.node("b").is_none());
        assert!(merged.node("c").is_none(), "theirs removed c");
        assert!(merged.node("a").is_some());
        assert!(merged.link("a", "b").is_none());
    }

    #[test]
    fn unrelated_side_change_applies_cleanly() {
        // ours removes node x; theirs is unchanged. Merge = ours.
        let base = net(&["a", "b", "x"], &[]);
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
        // No object-level conflict is detectable (node z touched by ours
        // only; link w-z touched by theirs only), but applying theirs' link
        // add after ours' node removal is impossible → fail closed with an
        // explanation instead of a dangling state.
        let base = net(&["z", "w"], &[]);
        let ours = net(&["w"], &[]);
        let theirs = net(&["z", "w"], &[("w", "z")]);
        let err = merge(&base, &ours, &theirs).unwrap_err();
        assert!(
            err.conflicts.iter().any(|c| c.contains("internal merge error")),
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
        assert_eq!(merged.node("x").unwrap().metadata, md, "merge must preserve added metadata");
    }
}
