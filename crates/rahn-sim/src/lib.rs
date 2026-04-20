// SPDX-License-Identifier: Apache-2.0

//! Simulation-only execution planning (ADR 0008).
//!
//! `plan` derives the explicit sequence of actions that would realize a
//! target state from a current state. This crate NEVER touches the host,
//! the network, or any device: it produces an inspectable plan and nothing
//! else. Real backends (Stage 3+) must implement plan execution behind a
//! separate interface; representation stays free of backend concepts.

use std::fmt;

use rahn_core::Network;
use rahn_state::diff;

/// One concrete action of an execution plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    CreateNode { id: String },
    RemoveNode { id: String },
    CreateLink { a: String, b: String },
    RemoveLink { a: String, b: String },
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::CreateNode { id } => write!(f, "create node {id}"),
            Action::RemoveNode { id } => write!(f, "remove node {id}"),
            Action::CreateLink { a, b } => write!(f, "create link {a} <-> {b}"),
            Action::RemoveLink { a, b } => write!(f, "remove link {a} <-> {b}"),
        }
    }
}

/// An inspectable, simulation-only execution plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlan {
    /// Actions in dependency-safe order: removals first (links before
    /// nodes), then additions (nodes before links).
    pub actions: Vec<Action>,
}

impl ExecutionPlan {
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }
}

impl fmt::Display for ExecutionPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.actions.is_empty() {
            return write!(f, "Execution plan: (no changes)");
        }
        writeln!(f, "Execution plan:")?;
        for (i, action) in self.actions.iter().enumerate() {
            writeln!(f, "  {}. {action}", i + 1)?;
        }
        writeln!(
            f,
            "(simulation only — no action is taken against any system)"
        )
    }
}

/// Derive the execution plan from `current` to `target`.
pub fn plan(current: &Network, target: &Network) -> ExecutionPlan {
    let d = diff(current, target);
    let mut actions = Vec::new();
    for (a, b) in &d.removed_links {
        actions.push(Action::RemoveLink { a: a.clone(), b: b.clone() });
    }
    for id in &d.removed_nodes {
        actions.push(Action::RemoveNode { id: id.clone() });
    }
    for id in &d.added_nodes {
        actions.push(Action::CreateNode { id: id.clone() });
    }
    for (a, b) in &d.added_links {
        actions.push(Action::CreateLink { a: a.clone(), b: b.clone() });
    }
    ExecutionPlan { actions }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rahn_core::{Metadata, Network};

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
    fn plan_order_is_dependency_safe() {
        // Replace a-b with c-d.
        let current = net(&["a", "b"], &[("a", "b")]);
        let target = net(&["a", "c", "d"], &[("c", "d")]);
        let plan = plan(&current, &target);
        assert_eq!(
            plan.actions,
            vec![
                Action::RemoveLink { a: "a".into(), b: "b".into() },
                Action::RemoveNode { id: "b".into() },
                Action::CreateNode { id: "c".into() },
                Action::CreateNode { id: "d".into() },
                Action::CreateLink { a: "c".into(), b: "d".into() },
            ]
        );
        // Removals precede creations.
        let first_create = plan.actions.iter().position(|a| matches!(a, Action::CreateNode { .. })).unwrap();
        assert!(plan.actions[..first_create].iter().all(|a| matches!(a, Action::RemoveLink { .. } | Action::RemoveNode { .. })));
    }

    #[test]
    fn identical_states_yield_empty_plan() {
        let n = net(&["a"], &[]);
        assert!(plan(&n, &n).is_empty());
    }

    #[test]
    fn plan_display_is_inspectable() {
        let current = Network::empty();
        let target = net(&["a", "b"], &[("a", "b")]);
        let text = plan(&current, &target).to_string();
        assert!(text.contains("Execution plan:"));
        assert!(text.contains("create node a"));
        assert!(text.contains("simulation only"));
    }
}
