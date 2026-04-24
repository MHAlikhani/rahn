// SPDX-License-Identifier: Apache-2.0

//! History graph helpers: common-ancestor computation over the commit DAG.
//!
//! The commit graph is the backbone of branching and merging (ADR 0007);
//! ancestor computation is deterministic (BFS over parent lists in fixed
//! order) and has no side effects.

use crate::commit::CommitId;

/// Find the best common ancestor of two commits.
///
/// `parents_of` fetches a commit's parent list. Returns `None` if the
/// histories share no ancestor (including root commits with no parents).
///
/// Determinism: the "best" ancestor is the first commit reached by
/// breadth-first traversal from `a` (parents in stored order) that is also
/// reachable from `b`. Ties are broken by traversal order, which is fully
/// determined by the stored parent lists.
pub fn common_ancestor(
    a: CommitId,
    b: CommitId,
    parents_of: impl Fn(CommitId) -> Option<Vec<CommitId>>,
) -> Option<CommitId> {
    if a == b {
        return Some(a);
    }
    // Reachability set from b.
    let mut b_reachable = std::collections::BTreeSet::new();
    let mut queue = std::collections::VecDeque::new();
    b_reachable.insert(b);
    queue.push_back(b);
    while let Some(id) = queue.pop_front() {
        for p in parents_of(id)?.into_iter() {
            if b_reachable.insert(p) {
                queue.push_back(p);
            }
        }
    }
    // BFS from a; first ancestor also reachable from b wins.
    let mut seen = std::collections::BTreeSet::new();
    seen.insert(a);
    queue.clear();
    queue.push_back(a);
    while let Some(id) = queue.pop_front() {
        if b_reachable.contains(&id) {
            return Some(id);
        }
        for p in parents_of(id)?.into_iter() {
            if seen.insert(p) {
                queue.push_back(p);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn graph(edges: &[(u8, u8)]) -> BTreeMap<CommitId, Vec<CommitId>> {
        let mut g: BTreeMap<CommitId, Vec<CommitId>> = BTreeMap::new();
        let node = |n: u8| CommitId([n; 32]);
        for (child, parent) in edges {
            g.entry(node(*child)).or_default().push(node(*parent));
        }
        g
    }

    fn parents_of(
        g: &BTreeMap<CommitId, Vec<CommitId>>,
    ) -> impl Fn(CommitId) -> Option<Vec<CommitId>> + '_ {
        move |id: CommitId| Some(g.get(&id).cloned().unwrap_or_default())
    }

    fn id(n: u8) -> CommitId {
        CommitId([n; 32])
    }

    #[test]
    fn linear_history() {
        // 3 -> 2 -> 1
        let g = graph(&[(2, 1), (3, 2)]);
        assert_eq!(common_ancestor(id(3), id(2), parents_of(&g)), Some(id(2)));
        assert_eq!(common_ancestor(id(3), id(1), parents_of(&g)), Some(id(1)));
    }

    #[test]
    fn branched_history() {
        // 5 -> 3 -> 1, 4 -> 2 -> 1
        let g = graph(&[(5, 3), (3, 1), (4, 2), (2, 1)]);
        assert_eq!(common_ancestor(id(5), id(4), parents_of(&g)), Some(id(1)));
    }

    #[test]
    fn same_commit() {
        let g = graph(&[]);
        assert_eq!(common_ancestor(id(7), id(7), parents_of(&g)), Some(id(7)));
    }

    #[test]
    fn disjoint_histories() {
        // 2 -> 1 and 9 -> 8: no common ancestor.
        let g = graph(&[(2, 1), (9, 8)]);
        assert_eq!(common_ancestor(id(2), id(9), parents_of(&g)), None);
    }

    #[test]
    fn diamond() {
        // 4 -> 3 -> 1, 4 -> 2 -> 1 (merge-style parents), 3 and 2 merge at 4.
        let mut g = graph(&[(3, 1), (2, 1)]);
        g.entry(id(4)).or_default().extend([id(3), id(2)]);
        assert_eq!(common_ancestor(id(4), id(3), parents_of(&g)), Some(id(3)));
    }
}
