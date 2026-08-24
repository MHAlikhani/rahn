// SPDX-License-Identifier: Apache-2.0

//! # RAHN SDK
//!
//! Curated public API for RAHN (ADR 0018): a stable, documented facade
//! over the core crates. Everything external code needs is reachable
//! here; anything not re-exported is internal and may change without
//! notice (pre-1.0 policy: breaking SDK changes land in MINOR releases
//! and are recorded in the CHANGELOG).
//!
//! The **IR** (ADR 0018) is the [`transition::Operation`] vocabulary plus
//! the canonical byte encoding (ADR 0003, format v2): every artifact
//! compiles down to Operations or is derived from their states.
//!
//! ## Build, verify, and persist a network
//! ```
//! use rahn_sdk::prelude::*;
//!
//! let mut net = Network::empty();
//! net.add_node("web", Metadata::new()).unwrap();
//! net.add_interface("web", "eth0").unwrap();
//! net.add_node("db", Metadata::new()).unwrap();
//! net.add_interface("db", "eth0").unwrap();
//! net.add_link(Endpoint::new("web", "eth0").unwrap(), Endpoint::new("db", "eth0").unwrap()).unwrap();
//! let state = State { network: net };
//!
//! // Verify against a constitution (verify before execute).
//! let report = verify(&state, &Constitution::default());
//! assert!(report.passed());
//!
//! // Deterministic identity (ADR 0002/0003).
//! let id = StateId::of(&state);
//! assert_eq!(StateId::of(&state), id);
//!
//! // Canonical round-trip is exact.
//! let parsed = parse_canonical(&canonical_bytes(&state)).unwrap();
//! assert_eq!(parsed, state);
//! ```
//!
//! ## Apply transitions (the IR)
//! ```
//! use rahn_sdk::prelude::*;
//!
//! let ops = vec![
//!     Operation::AddNode { id: "a".into(), metadata: Metadata::new() },
//!     Operation::AddNode { id: "b".into(), metadata: Metadata::new() },
//! ];
//! let empty = State::empty();
//! let next = apply_all(&empty.network, &ops).unwrap();
//! assert_eq!(next.node_count(), 2);
//! ```
//!
//! ## Plan execution for a backend (ADR 0016)
//! ```
//! use rahn_sdk::prelude::*;
//!
//! let from = State::empty();
//! let mut to = State::empty();
//! to.network.add_node("a", Metadata::new()).unwrap();
//! let backend = SimulationBackend;
//! let steps = backend.plan(&from.network, &to.network).unwrap();
//! assert!(steps.iter().all(|s| matches!(s, BackendStep::Describe(_))));
//! ```

pub mod prelude {
    // Model (rahn-core).
    pub use rahn_core::{
        validate_id, Endpoint, Interface, Link, Metadata, Network, Node, State, VerificationSummary,
    };
    // IR + identity + transitions + diff + history (rahn-state).
    pub use rahn_state::{
        apply, apply_all, canonical_bytes, diff, parse_canonical, transition::Operation,
        CanonicalError, CommitId, CommitRecord, Diff, Operation as IrOperation, StateId,
        CANONICAL_FORMAT_VERSION,
    };
    // Graph queries.
    pub use rahn_state::{components, neighbors, reachable_from, shortest_path};
    // Observations + causality.
    pub use rahn_state::{
        order_key, Anchor, CausalEdge, CausalGraph, Observation, Status as CausalStatus, Subject,
        Value,
    };
    // Verification.
    pub use rahn_verify::{verify, Constitution, InvariantReport, VerificationReport};
    // Execution (ADR 0016).
    pub use rahn_sim::{
        backend::{BackendError, BackendStep, Capabilities, ExecutionBackend},
        plan, Action, ExecutionPlan, SimulationBackend,
    };
    // Persistence.
    pub use rahn_store::Store;
}

/// Compile-time surface guards: the facade must keep exposing the types
/// external code is promised (ADR 0018). Failure to compile here is an
/// SDK-contract break.
#[cfg(test)]
mod api_surface {
    use crate::prelude::*;

    #[test]
    fn model_types_are_reachable() {
        let mut net = Network::empty();
        net.add_node("n", Metadata::new()).unwrap();
        net.add_interface("n", "eth0").unwrap();
        let e = Endpoint::new("n", "eth0").unwrap();
        let _link: Option<&Link> = net.link(&e, &Endpoint::new("n", "eth1").unwrap());
        let _state: State = State { network: net };
    }

    #[test]
    fn ir_round_trip() {
        let ops = vec![
            Operation::AddNode {
                id: "a".into(),
                metadata: Metadata::new(),
            },
            Operation::AddInterface {
                node: "a".into(),
                name: "eth0".into(),
            },
        ];
        let next = apply_all(&State::empty().network, &ops).unwrap();
        let state = State { network: next };
        let bytes = canonical_bytes(&state);
        assert_eq!(parse_canonical(&bytes).unwrap(), state);
        assert_eq!(
            StateId::of(&parse_canonical(&bytes).unwrap()),
            StateId::of(&state)
        );
    }

    #[test]
    fn verification_and_planning_are_reachable() {
        let state = State::empty();
        let report: VerificationReport = verify(&state, &Constitution::default());
        assert!(report.passed());
        let steps = SimulationBackend
            .plan(&state.network, &state.network)
            .unwrap();
        assert!(steps.is_empty());
    }

    #[test]
    fn persistence_round_trip() {
        let dir = std::env::temp_dir().join(format!(
            "rahn-sdk-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let store = Store::init(dir.join(".rahn")).unwrap();
        let state = State::empty();
        let id = store.put_state(&state).unwrap();
        assert_eq!(store.get_state(id).unwrap().unwrap(), state);
        std::fs::remove_dir_all(&dir).ok();
    }
}
