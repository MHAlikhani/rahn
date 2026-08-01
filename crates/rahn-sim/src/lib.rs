// SPDX-License-Identifier: Apache-2.0

//! Simulation-only execution planning (ADR 0008) and the execution
//! backend abstraction (ADR 0016).
//!
//! `plan` derives the explicit sequence of actions that would realize a
//! target state from a current state. This crate NEVER touches the host,
//! the network, or any device: it produces an inspectable plan and nothing
//! else. Real backends (Stage 3+) must implement plan execution behind a
//! separate interface; representation stays free of backend concepts.

use std::fmt;

use rahn_core::{Endpoint, Network};
use rahn_state::diff;

pub mod backend;

/// One concrete action of an execution plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    CreateNode { id: String },
    RemoveNode { id: String },
    CreateInterface { node: String, name: String },
    RemoveInterface { node: String, name: String },
    CreateLink { a: Endpoint, b: Endpoint },
    RemoveLink { a: Endpoint, b: Endpoint },
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::CreateNode { id } => write!(f, "create node {id}"),
            Action::RemoveNode { id } => write!(f, "remove node {id}"),
            Action::CreateInterface { node, name } => write!(f, "create interface {node}/{name}"),
            Action::RemoveInterface { node, name } => write!(f, "remove interface {node}/{name}"),
            Action::CreateLink { a, b } => write!(f, "create link {a} <-> {b}"),
            Action::RemoveLink { a, b } => write!(f, "remove link {a} <-> {b}"),
        }
    }
}

/// An inspectable, simulation-only execution plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlan {
    /// Actions in dependency-safe order: removals first (links before
    /// interfaces before nodes), then additions (nodes before interfaces
    /// before links).
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
        actions.push(Action::RemoveLink {
            a: a.clone(),
            b: b.clone(),
        });
    }
    // Removing a node subsumes removing its interfaces; emit explicit
    // interface removals only for nodes that survive.
    for (node, name) in &d.removed_interfaces {
        if !d.removed_nodes.contains(node) {
            actions.push(Action::RemoveInterface {
                node: node.clone(),
                name: name.clone(),
            });
        }
    }
    for id in &d.removed_nodes {
        actions.push(Action::RemoveNode { id: id.clone() });
    }
    for id in &d.added_nodes {
        actions.push(Action::CreateNode { id: id.clone() });
    }
    for (node, name) in &d.added_interfaces {
        actions.push(Action::CreateInterface {
            node: node.clone(),
            name: name.clone(),
        });
    }
    for (a, b) in &d.added_links {
        actions.push(Action::CreateLink {
            a: a.clone(),
            b: b.clone(),
        });
    }
    ExecutionPlan { actions }
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
    fn plan_order_is_dependency_safe() {
        // Replace a-b with c-d.
        let current = net(
            &["a", "b"],
            &[("a", "eth0"), ("b", "eth0")],
            &[("a", "eth0", "b", "eth0")],
        );
        let target = net(
            &["a", "c", "d"],
            &[("a", "eth0"), ("c", "eth0"), ("d", "eth0")],
            &[("c", "eth0", "d", "eth0")],
        );
        let plan = plan(&current, &target);
        assert_eq!(
            plan.actions,
            vec![
                Action::RemoveLink {
                    a: Endpoint::new("a", "eth0").unwrap(),
                    b: Endpoint::new("b", "eth0").unwrap()
                },
                Action::RemoveNode { id: "b".into() },
                Action::CreateNode { id: "c".into() },
                Action::CreateNode { id: "d".into() },
                Action::CreateInterface {
                    node: "c".into(),
                    name: "eth0".into()
                },
                Action::CreateInterface {
                    node: "d".into(),
                    name: "eth0".into()
                },
                Action::CreateLink {
                    a: Endpoint::new("c", "eth0").unwrap(),
                    b: Endpoint::new("d", "eth0").unwrap()
                },
            ]
        );
        // Removals precede creations.
        let first_create = plan
            .actions
            .iter()
            .position(|a| matches!(a, Action::CreateNode { .. }))
            .unwrap();
        assert!(plan.actions[..first_create].iter().all(|a| matches!(
            a,
            Action::RemoveLink { .. } | Action::RemoveInterface { .. } | Action::RemoveNode { .. }
        )));
    }

    #[test]
    fn identical_states_yield_empty_plan() {
        let n = net(&["a"], &[("a", "eth0")], &[]);
        assert!(plan(&n, &n).is_empty());
    }

    #[test]
    fn plan_display_is_inspectable() {
        let current = Network::empty();
        let target = net(
            &["a", "b"],
            &[("a", "eth0"), ("b", "eth0")],
            &[("a", "eth0", "b", "eth0")],
        );
        let text = plan(&current, &target).to_string();
        assert!(text.contains("Execution plan:"));
        assert!(text.contains("create node a"));
        assert!(text.contains("create interface a/eth0"));
        assert!(text.contains("create link a/eth0 <-> b/eth0"));
        assert!(text.contains("simulation only"));
    }
}
