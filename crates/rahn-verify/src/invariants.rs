// SPDX-License-Identifier: Apache-2.0

//! The deterministic invariant engine (ADR 0006).
//!
//! Evaluation rules:
//! - fixed evaluation order (structural invariants first, then constitution
//!   requirements in file order);
//! - no randomness, no time, no I/O;
//! - every invariant yields a pass/fail with machine-readable evidence —
//!   never a bare boolean;
//! - "verified" means "all encoded invariants hold" and nothing more
//!   (docs/research/limitations.md).

use std::collections::BTreeMap;
use std::fmt;

use rahn_core::{Network, State};
use rahn_state::StateId;

use crate::constitution::Constitution;

/// Human-stable invariant identifiers (recorded in commit summaries).
pub const INV_REFERENTIAL_INTEGRITY: &str = "referential-integrity";
pub const INV_LINK_ENDPOINTS: &str = "link-endpoints-exist";
pub const INV_NO_DUPLICATE_LINKS: &str = "no-duplicate-links";
pub const INV_NO_SELF_LOOPS: &str = "no-self-loops";
pub const INV_CONNECTIVITY: &str = "named-connectivity";

/// Result of checking one invariant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvariantReport {
    pub id: String,
    pub passed: bool,
    /// Machine-readable evidence: empty when passed; a concrete description
    /// of the violation otherwise.
    pub evidence: String,
}

impl fmt::Display for InvariantReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.passed {
            write!(f, "PASS {}", self.id)
        } else {
            write!(f, "FAIL {} — {}", self.id, self.evidence)
        }
    }
}

/// Full verification report over one state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationReport {
    pub state_id: rahn_state::StateId,
    pub reports: Vec<InvariantReport>,
}

impl VerificationReport {
    pub fn passed(&self) -> bool {
        self.reports.iter().all(|r| r.passed)
    }

    /// IDs of failed invariants, in evaluation order.
    pub fn failed_ids(&self) -> Vec<String> {
        self.reports
            .iter()
            .filter(|r| !r.passed)
            .map(|r| r.id.clone())
            .collect()
    }

    /// Compact summary for durable storage in commit records.
    pub fn summary(&self) -> rahn_core::VerificationSummary {
        if self.passed() {
            rahn_core::VerificationSummary::passed()
        } else {
            rahn_core::VerificationSummary::failed(self.failed_ids())
        }
    }
}

impl fmt::Display for VerificationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "state {}", self.state_id.as_hex())?;
        for r in &self.reports {
            writeln!(f, "  {r}")?;
        }
        if self.passed() {
            writeln!(f, "verification PASSED ({} invariants)", self.reports.len())
        } else {
            writeln!(f, "verification FAILED ({} of {} invariants)", self.failed_ids().len(), self.reports.len())
        }
    }
}

/// Verify a state against its structural invariants and the constitution.
///
/// Structural invariants are checked even if the constitution is empty:
/// they are the floor of validity, not configurable policy.
pub fn verify(state: &State, constitution: &Constitution) -> VerificationReport {
    let net = &state.network;
    let mut reports = vec![
        check_referential_integrity(net),
        check_link_endpoints(net),
        check_no_duplicate_links(net),
        check_no_self_loops(net),
    ];
    reports.extend(
        constitution
            .connectivity_requirements
            .iter()
            .map(|(a, b)| check_connectivity(net, a, b)),
    );
    VerificationReport { state_id: StateId::of(state), reports }
}

fn pass(id: &str) -> InvariantReport {
    InvariantReport { id: id.to_owned(), passed: true, evidence: String::new() }
}

fn fail(id: &str, evidence: String) -> InvariantReport {
    InvariantReport { id: id.to_owned(), passed: false, evidence }
}

/// Every link's endpoints must exist as nodes (with metadata and links in
/// the same structure, this is enforced by construction; the check exists
/// to verify the guarantee, not to establish it).
fn check_referential_integrity(net: &Network) -> InvariantReport {
    for link in net.iter_links() {
        if net.node(&link.a).is_none() {
            return fail(INV_REFERENTIAL_INTEGRITY, format!("link endpoint {:?} missing", link.a));
        }
        if net.node(&link.b).is_none() {
            return fail(INV_REFERENTIAL_INTEGRITY, format!("link endpoint {:?} missing", link.b));
        }
    }
    pass(INV_REFERENTIAL_INTEGRITY)
}

fn check_link_endpoints(net: &Network) -> InvariantReport {
    for link in net.iter_links() {
        if link.a >= link.b {
            return fail(
                INV_LINK_ENDPOINTS,
                format!("link {link} endpoints are not normalized"),
            );
        }
    }
    pass(INV_LINK_ENDPOINTS)
}

fn check_no_duplicate_links(net: &Network) -> InvariantReport {
    // Links are keyed by normalized endpoints, so duplicates are
    // unrepresentable; re-verify the guarantee explicitly.
    let mut seen = std::collections::BTreeSet::new();
    for link in net.iter_links() {
        let key = (link.a.clone(), link.b.clone());
        if !seen.insert(key.clone()) {
            return fail(INV_NO_DUPLICATE_LINKS, format!("duplicate link {key:?}"));
        }
    }
    pass(INV_NO_DUPLICATE_LINKS)
}

fn check_no_self_loops(net: &Network) -> InvariantReport {
    for link in net.iter_links() {
        if link.a == link.b {
            return fail(INV_NO_SELF_LOOPS, format!("self-loop on {:?}", link.a));
        }
    }
    pass(INV_NO_SELF_LOOPS)
}

/// Breadth-first connectivity check over undirected links.
fn check_connectivity(net: &Network, from: &str, to: &str) -> InvariantReport {
    let id = format!("{INV_CONNECTIVITY}:{from}:{to}");
    if net.node(from).is_none() || net.node(to).is_none() {
        return fail(&id, format!("endpoint missing: {from:?} or {to:?} not in network"));
    }
    if from == to {
        return pass(&id);
    }
    let mut adjacency: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for link in net.iter_links() {
        adjacency.entry(link.a.as_str()).or_default().push(link.b.as_str());
        adjacency.entry(link.b.as_str()).or_default().push(link.a.as_str());
    }
    let mut visited = std::collections::BTreeSet::new();
    let mut queue = std::collections::VecDeque::new();
    visited.insert(from);
    queue.push_back(from);
    while let Some(current) = queue.pop_front() {
        if current == to {
            return pass(&id);
        }
        for next in adjacency.get(current).into_iter().flatten() {
            if visited.insert(next) {
                queue.push_back(next);
            }
        }
    }
    fail(&id, format!("no path between {from:?} and {to:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rahn_core::{Metadata, Network};

    fn net(build: impl FnOnce(&mut Network)) -> State {
        let mut n = Network::empty();
        build(&mut n);
        State { network: n }
    }

    fn empty_constitution() -> Constitution {
        Constitution::default()
    }

    #[test]
    fn valid_state_passes_all() {
        let s = net(|n| {
            n.add_node("a", Metadata::new()).unwrap();
            n.add_node("b", Metadata::new()).unwrap();
            n.add_link("a", "b").unwrap();
        });
        let report = verify(&s, &empty_constitution());
        assert!(report.passed(), "{report}");
    }

    #[test]
    fn connectivity_requirement_enforced() {
        let s = net(|n| {
            n.add_node("a", Metadata::new()).unwrap();
            n.add_node("b", Metadata::new()).unwrap();
            n.add_node("c", Metadata::new()).unwrap();
            n.add_link("a", "b").unwrap();
        });
        let mut c = empty_constitution();
        c.connectivity_requirements.push(("a".into(), "c".into()));
        let report = verify(&s, &c);
        assert!(!report.passed());
        assert_eq!(report.failed_ids(), vec!["named-connectivity:a:c".to_string()]);
        assert!(report.reports.iter().any(|r| r.evidence.contains("no path")));
    }

    #[test]
    fn connectivity_passes_through_intermediate_nodes() {
        let s = net(|n| {
            for id in ["a", "b", "c"] {
                n.add_node(id, Metadata::new()).unwrap();
            }
            n.add_link("a", "b").unwrap();
            n.add_link("b", "c").unwrap();
        });
        let mut c = empty_constitution();
        c.connectivity_requirements.push(("a".into(), "c".into()));
        assert!(verify(&s, &c).passed());
    }

    #[test]
    fn connectivity_requirement_with_missing_node_fails() {
        let s = net(|n| {
            n.add_node("a", Metadata::new()).unwrap();
        });
        let mut c = empty_constitution();
        c.connectivity_requirements.push(("a".into(), "ghost".into()));
        let report = verify(&s, &c);
        assert!(!report.passed());
        assert!(report.reports.iter().any(|r| r.evidence.contains("missing")));
    }

    #[test]
    fn determinism_same_input_same_report() {
        let s = net(|n| {
            n.add_node("a", Metadata::new()).unwrap();
            n.add_node("b", Metadata::new()).unwrap();
        });
        let mut c = empty_constitution();
        c.connectivity_requirements.push(("a".into(), "b".into()));
        let r1 = verify(&s, &c);
        let r2 = verify(&s, &c);
        assert_eq!(r1, r2);
    }

    #[test]
    fn structural_floor_always_checked() {
        // Empty constitution still checks structural invariants.
        let s = State::empty();
        let report = verify(&s, &empty_constitution());
        assert_eq!(report.reports.len(), 4);
        assert!(report.passed());
    }
}
