// SPDX-License-Identifier: Apache-2.0

//! Execution backend abstraction (ADR 0016).
//!
//! A backend turns a (current, target) state pair into an ordered list of
//! [`BackendStep`]s. `plan()` is a pure function: identical inputs produce
//! identical steps. The `simulation` backend returns only `Describe`
//! steps and never touches anything; real backends return `Invoke` steps
//! and are gated behind explicit opt-in at the CLI boundary.

use std::fmt;

use crate::Action;
use rahn_core::Network;

/// What a backend can realize. Unsupported capability requirements fail
/// explicitly — never silently partially executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Capabilities {
    /// Can realize link-level topology (nodes, interfaces, links).
    pub link_topology: bool,
    /// Can realize addresses (Future — Stage 2-scope deferral).
    pub addressing: bool,
    /// Can realize traffic control (Future).
    pub traffic_control: bool,
}

/// Why a backend refused a plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendError {
    UnsupportedCapability {
        backend: &'static str,
        capability: &'static str,
    },
    Plan(String),
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackendError::UnsupportedCapability {
                backend,
                capability,
            } => {
                write!(f, "backend {backend:?} does not support capability {capability:?}; refusing explicitly")
            }
            BackendError::Plan(e) => write!(f, "backend plan error: {e}"),
        }
    }
}

impl std::error::Error for BackendError {}

/// One step of a backend plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendStep {
    /// A model-level action (simulation; also the dry-run view of real
    /// backends).
    Describe(Action),
    /// A concrete host invocation, e.g. `ip netns add rahn-web`.
    Invoke { program: String, args: Vec<String> },
}

impl fmt::Display for BackendStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackendStep::Describe(a) => write!(f, "{a}"),
            BackendStep::Invoke { program, args } => {
                write!(f, "{program} {}", args.join(" "))
            }
        }
    }
}

/// The backend contract (ADR 0016).
pub trait ExecutionBackend {
    fn name(&self) -> &'static str;
    fn capabilities(&self) -> Capabilities;
    /// Pure, deterministic mapping from (current, target) to steps.
    fn plan(&self, current: &Network, target: &Network) -> Result<Vec<BackendStep>, BackendError>;
}

/// The default backend: model-level description only, no host effects.
pub struct SimulationBackend;

impl ExecutionBackend for SimulationBackend {
    fn name(&self) -> &'static str {
        "simulation"
    }

    fn capabilities(&self) -> Capabilities {
        // The model can *describe* everything the state model can express;
        // it realizes nothing.
        Capabilities {
            link_topology: true,
            addressing: true,
            traffic_control: true,
        }
    }

    fn plan(&self, current: &Network, target: &Network) -> Result<Vec<BackendStep>, BackendError> {
        Ok(crate::plan(current, target)
            .actions
            .into_iter()
            .map(BackendStep::Describe)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rahn_core::{Endpoint, Metadata};

    fn net(nodes: &[&str], links: &[(&str, &str, &str, &str)]) -> Network {
        let mut n = Network::empty();
        for id in nodes {
            n.add_node(id, Metadata::new()).unwrap();
            n.add_interface(id, "eth0").unwrap();
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
    fn simulation_backend_describes_only() {
        let empty = Network::empty();
        let topo = net(&["a", "b"], &[("a", "eth0", "b", "eth0")]);
        let b = SimulationBackend;
        assert_eq!(b.name(), "simulation");
        let steps = b.plan(&empty, &topo).unwrap();
        assert_eq!(steps.len(), 5); // 2 CreateNode + 2 CreateInterface + 1 CreateLink
        assert!(steps.iter().all(|s| matches!(s, BackendStep::Describe(_))));
        // Deterministic.
        assert_eq!(b.plan(&empty, &topo).unwrap(), steps);
    }
}
