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

use std::fmt;

use rahn_core::Network;
use rahn_state::StateId;

use crate::constitution::Constitution;

/// Human-stable invariant identifiers (recorded in commit summaries).
pub const INV_REFERENTIAL_INTEGRITY: &str = "referential-integrity";
pub const INV_LINK_ENDPOINTS: &str = "link-endpoints-exist";
pub const INV_NO_DUPLICATE_LINKS: &str = "no-duplicate-links";
pub const INV_NO_SELF_LOOPS: &str = "no-self-loops";
pub const INV_CONNECTIVITY: &str = "named-connectivity";
pub const INV_NO_CONNECTIVITY: &str = "prohibited-connectivity";

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
    pub state_id: StateId,
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
            writeln!(
                f,
                "verification FAILED ({} of {} invariants)",
                self.failed_ids().len(),
                self.reports.len()
            )
        }
    }
}

/// Verify a state against its structural invariants and the constitution.
///
/// Structural invariants are checked even if the constitution is empty:
/// they are the floor of validity, not configurable policy.
pub fn verify(state: &rahn_core::State, constitution: &Constitution) -> VerificationReport {
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
    reports.extend(
        constitution
            .prohibited_connectivity
            .iter()
            .map(|(a, b)| check_prohibited_connectivity(net, a, b)),
    );
    VerificationReport {
        state_id: StateId::of(state),
        reports,
    }
}

fn pass(id: &str) -> InvariantReport {
    InvariantReport {
        id: id.to_owned(),
        passed: true,
        evidence: String::new(),
    }
}

fn fail(id: &str, evidence: String) -> InvariantReport {
    InvariantReport {
        id: id.to_owned(),
        passed: false,
        evidence,
    }
}

/// Every link's endpoints must exist as interfaces (with the model's
/// construction rules, this is enforced by construction; the check exists
/// to verify the guarantee, not to establish it).
fn check_referential_integrity(net: &Network) -> InvariantReport {
    for link in net.iter_links() {
        let a_ok = net
            .node(&link.a.node)
            .map(|n| n.interface(&link.a.iface).is_some())
            .unwrap_or(false);
        if !a_ok {
            return fail(
                INV_REFERENTIAL_INTEGRITY,
                format!("link endpoint {} missing", link.a),
            );
        }
        let b_ok = net
            .node(&link.b.node)
            .map(|n| n.interface(&link.b.iface).is_some())
            .unwrap_or(false);
        if !b_ok {
            return fail(
                INV_REFERENTIAL_INTEGRITY,
                format!("link endpoint {} missing", link.b),
            );
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
            return fail(INV_NO_SELF_LOOPS, format!("self-loop on {}", link.a));
        }
        if link.a.node == link.b.node {
            return fail(
                INV_NO_SELF_LOOPS,
                format!("same-node loop on {:?}", link.a.node),
            );
        }
    }
    pass(INV_NO_SELF_LOOPS)
}

/// `require-connectivity <a> <b>`: a path must exist between the two nodes
/// (via the node graph induced by interface links).
fn check_connectivity(net: &Network, from: &str, to: &str) -> InvariantReport {
    let id = format!("{INV_CONNECTIVITY}:{from}:{to}");
    match rahn_state::graph::shortest_path(net, from, to) {
        Err(rahn_state::graph::PathError::EndpointMissing) => fail(
            &id,
            format!("endpoint missing: {from:?} or {to:?} not in network"),
        ),
        Ok(None) => fail(&id, format!("no path between {from:?} and {to:?}")),
        Ok(Some(_)) => pass(&id),
    }
}

/// `prohibit-connectivity <a> <b>`: no path may exist between the two
/// nodes (isolation requirements, e.g. "database must not be reachable
/// from the public segment").
fn check_prohibited_connectivity(net: &Network, from: &str, to: &str) -> InvariantReport {
    let id = format!("{INV_NO_CONNECTIVITY}:{from}:{to}");
    match rahn_state::graph::shortest_path(net, from, to) {
        Err(rahn_state::graph::PathError::EndpointMissing) => fail(
            &id,
            format!("endpoint missing: {from:?} or {to:?} not in network"),
        ),
        Ok(None) => pass(&id),
        Ok(Some(path)) => fail(
            &id,
            format!("prohibited path exists: {}", path.join(" -> ")),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rahn_core::{Endpoint, Metadata};

    fn net(
        nodes: &[&str],
        ifaces: &[(&str, &str)],
        links: &[(&str, &str, &str, &str)],
    ) -> rahn_core::State {
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
        rahn_core::State { network: n }
    }

    fn empty_constitution() -> Constitution {
        Constitution::default()
    }

    #[test]
    fn valid_state_passes_all() {
        let s = net(
            &["a", "b"],
            &[("a", "eth0"), ("b", "eth0")],
            &[("a", "eth0", "b", "eth0")],
        );
        let report = verify(&s, &empty_constitution());
        assert!(report.passed(), "{report}");
    }

    #[test]
    fn connectivity_requirement_enforced() {
        let s = net(
            &["a", "b", "c"],
            &[("a", "eth0"), ("b", "eth0"), ("c", "eth0")],
            &[("a", "eth0", "b", "eth0")],
        );
        let mut c = empty_constitution();
        c.connectivity_requirements.push(("a".into(), "c".into()));
        let report = verify(&s, &c);
        assert!(!report.passed());
        assert_eq!(
            report.failed_ids(),
            vec!["named-connectivity:a:c".to_string()]
        );
        assert!(report
            .reports
            .iter()
            .any(|r| r.evidence.contains("no path")));
    }

    #[test]
    fn connectivity_passes_through_intermediate_nodes() {
        let s = net(
            &["a", "b", "c"],
            &[("a", "eth0"), ("b", "eth0"), ("b", "eth1"), ("c", "eth0")],
            &[("a", "eth0", "b", "eth0"), ("b", "eth1", "c", "eth0")],
        );
        let mut c = empty_constitution();
        c.connectivity_requirements.push(("a".into(), "c".into()));
        assert!(verify(&s, &c).passed());
    }

    #[test]
    fn connectivity_requirement_with_missing_node_fails() {
        let s = net(&["a"], &[("a", "eth0")], &[]);
        let mut c = empty_constitution();
        c.connectivity_requirements
            .push(("a".into(), "ghost".into()));
        let report = verify(&s, &c);
        assert!(!report.passed());
        assert!(report
            .reports
            .iter()
            .any(|r| r.evidence.contains("missing")));
    }

    #[test]
    fn prohibited_connectivity_enforced() {
        let s = net(
            &["public", "db"],
            &[("public", "eth0"), ("db", "eth0")],
            &[("public", "eth0", "db", "eth0")],
        );
        let mut c = empty_constitution();
        c.prohibited_connectivity
            .push(("public".into(), "db".into()));
        let report = verify(&s, &c);
        assert!(!report.passed());
        assert_eq!(
            report.failed_ids(),
            vec!["prohibited-connectivity:public:db".to_string()]
        );
        assert!(report
            .reports
            .iter()
            .any(|r| r.evidence.contains("prohibited path exists")));
    }

    #[test]
    fn prohibited_connectivity_passes_when_isolated() {
        let s = net(
            &["public", "db"],
            &[("public", "eth0"), ("db", "eth0")],
            &[],
        );
        let mut c = empty_constitution();
        c.prohibited_connectivity
            .push(("public".into(), "db".into()));
        assert!(verify(&s, &c).passed());
    }

    #[test]
    fn determinism_same_input_same_report() {
        let s = net(&["a", "b"], &[("a", "eth0"), ("b", "eth0")], &[]);
        let mut c = empty_constitution();
        c.connectivity_requirements.push(("a".into(), "b".into()));
        let r1 = verify(&s, &c);
        let r2 = verify(&s, &c);
        assert_eq!(r1, r2);
    }

    #[test]
    fn structural_floor_always_checked() {
        // Empty constitution still checks structural invariants.
        let s = rahn_core::State::empty();
        let report = verify(&s, &empty_constitution());
        assert_eq!(report.reports.len(), 4);
        assert!(report.passed());
    }
}
