// SPDX-License-Identifier: Apache-2.0

//! Causal memory (ADR 0014): explicit, status-labeled causal edges
//! between immutable anchors (observations, commits), stored as an
//! append-only DAG. No inference: every edge is an asserted relation
//! whose status labels its epistemic strength.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;

/// Why a causal edge or query was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CausalError {
    InvalidAnchor { anchor: String },
    DanglingAnchor { anchor: String },
    InvalidStatus { status: String },
    VerifiedRequiresCommits,
    Cycle,
    MalformedRecord { offset: usize, message: String },
    Io(String),
}

impl fmt::Display for CausalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CausalError::InvalidAnchor { anchor } => write!(
                f,
                "invalid anchor {anchor:?} (expected obs:<seq> or commit:<64-hex>)"
            ),
            CausalError::DanglingAnchor { anchor } => {
                write!(f, "anchor {anchor} does not exist")
            }
            CausalError::InvalidStatus { status } => write!(
                f,
                "invalid status {status:?} (expected temporal-correlation|hypothesis|verified)"
            ),
            CausalError::VerifiedRequiresCommits => {
                write!(f, "verified edges require Commit anchors on both ends")
            }
            CausalError::Cycle => {
                write!(f, "edge rejected: it would close a cycle in the causal DAG")
            }
            CausalError::MalformedRecord { offset, message } => {
                write!(f, "malformed causal record at byte {offset}: {message}")
            }
            CausalError::Io(e) => write!(f, "io error: {e}"),
        }
    }
}

impl std::error::Error for CausalError {}

/// An immutable endpoint of a causal edge.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Anchor {
    /// Positional reference into the observation log.
    Observation(u64),
    /// Content-addressed commit reference.
    Commit([u8; 32]),
}

impl Anchor {
    pub fn parse(s: &str) -> Result<Self, CausalError> {
        let bad = || CausalError::InvalidAnchor {
            anchor: s.to_owned(),
        };
        let (kind, rest) = s.split_once(':').ok_or_else(bad)?;
        match kind {
            "obs" => rest
                .parse::<u64>()
                .map(Anchor::Observation)
                .map_err(|_| bad()),
            "commit" => match crate::identity::StateId::from_hex(rest) {
                Some(id) => Ok(Anchor::Commit(id.as_bytes())),
                None => Err(bad()),
            },
            _ => Err(bad()),
        }
    }

    fn tag(&self) -> u64 {
        match self {
            Anchor::Observation(_) => 1,
            Anchor::Commit(_) => 2,
        }
    }

    fn write(&self, w: &mut crate::canonical::Writer) {
        w.u64(self.tag());
        match self {
            Anchor::Observation(seq) => w.u64(*seq),
            Anchor::Commit(id) => w.bytes(id),
        }
    }

    fn read(r: &mut crate::canonical::Reader) -> Result<Self, CausalError> {
        let err = |r: &crate::canonical::Reader, m: &str| CausalError::MalformedRecord {
            offset: r.pos,
            message: m.to_owned(),
        };
        match r.u64().map_err(|e| err(r, &e.message))? {
            1 => Ok(Anchor::Observation(
                r.u64().map_err(|e| err(r, &e.message))?,
            )),
            2 => {
                let b = r.bytes().map_err(|e| err(r, &e.message))?;
                if b.len() != 32 {
                    return Err(err(r, "commit anchor must be 32 bytes"));
                }
                let mut id = [0u8; 32];
                id.copy_from_slice(b);
                Ok(Anchor::Commit(id))
            }
            t => Err(err(r, &format!("unknown anchor tag {t}"))),
        }
    }
}

impl fmt::Display for Anchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Anchor::Observation(seq) => write!(f, "obs:{seq}"),
            Anchor::Commit(id) => {
                write!(
                    f,
                    "commit:{}",
                    crate::identity::StateId::from_bytes(*id).as_hex()
                )
            }
        }
    }
}

/// Epistemic status of a causal edge (ADR 0014).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    /// Ordering/adjacency asserted; no causal claim.
    TemporalCorrelation,
    /// Suspected causal link.
    Hypothesis,
    /// Structurally checkable provenance: Commit-anchored only.
    Verified,
}

impl Status {
    pub fn parse(s: &str) -> Result<Self, CausalError> {
        match s {
            "temporal-correlation" => Ok(Status::TemporalCorrelation),
            "hypothesis" => Ok(Status::Hypothesis),
            "verified" => Ok(Status::Verified),
            other => Err(CausalError::InvalidStatus {
                status: other.to_owned(),
            }),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Status::TemporalCorrelation => "temporal-correlation",
            Status::Hypothesis => "hypothesis",
            Status::Verified => "verified",
        }
    }
}

/// An asserted causal relation between two anchors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CausalEdge {
    /// Ingest position (count of prior edges).
    pub seq: u64,
    pub from: Anchor,
    pub to: Anchor,
    pub status: Status,
    /// Bounded free text (≤256 bytes).
    pub note: String,
}

impl CausalEdge {
    const MAX_NOTE_BYTES: usize = 256;

    pub fn new(
        seq: u64,
        from: Anchor,
        to: Anchor,
        status: Status,
        note: String,
    ) -> Result<Self, CausalError> {
        if from == to {
            return Err(CausalError::Cycle);
        }
        if note.len() > Self::MAX_NOTE_BYTES {
            return Err(CausalError::MalformedRecord {
                offset: 0,
                message: format!("note exceeds {} bytes", Self::MAX_NOTE_BYTES),
            });
        }
        if status == Status::Verified
            && (!matches!(from, Anchor::Commit(_)) || !matches!(to, Anchor::Commit(_)))
        {
            return Err(CausalError::VerifiedRequiresCommits);
        }
        Ok(CausalEdge {
            seq,
            from,
            to,
            status,
            note,
        })
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut w = crate::canonical::Writer::new();
        w.u64(self.seq);
        self.from.write(&mut w);
        self.to.write(&mut w);
        w.u64(match self.status {
            Status::TemporalCorrelation => 1,
            Status::Hypothesis => 2,
            Status::Verified => 3,
        });
        w.string(&self.note);
        w.buf
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, CausalError> {
        let mut r = crate::canonical::Reader { buf: bytes, pos: 0 };
        let err = |r: &crate::canonical::Reader, m: &str| CausalError::MalformedRecord {
            offset: r.pos,
            message: m.to_owned(),
        };
        let seq = r.u64().map_err(|e| err(&r, &e.message))?;
        let from = Anchor::read(&mut r)?;
        let to = Anchor::read(&mut r)?;
        let status = match r.u64().map_err(|e| err(&r, &e.message))? {
            1 => Status::TemporalCorrelation,
            2 => Status::Hypothesis,
            3 => Status::Verified,
            t => return Err(err(&r, &format!("unknown status tag {t}"))),
        };
        let note = r.string().map_err(|e| err(&r, &e.message))?;
        if r.pos != bytes.len() {
            return Err(err(&r, "trailing bytes"));
        }
        CausalEdge::new(seq, from, to, status, note)
    }
}

/// The causal DAG: validated edge set with acyclicity and anchor checks.
pub struct CausalGraph {
    edges: Vec<CausalEdge>,
    /// adjacency: anchor -> anchors it points to
    adj: BTreeMap<Anchor, BTreeSet<Anchor>>,
    /// reverse adjacency: anchor -> anchors pointing to it
    radj: BTreeMap<Anchor, BTreeSet<Anchor>>,
}

impl CausalGraph {
    pub fn new(edges: Vec<CausalEdge>) -> Self {
        let mut g = CausalGraph {
            edges: Vec::new(),
            adj: BTreeMap::new(),
            radj: BTreeMap::new(),
        };
        for e in edges {
            // Stored edges were validated at creation; re-assert DAG shape.
            g.link(e.from.clone(), e.to.clone());
            g.edges.push(e);
        }
        g
    }

    pub fn edges(&self) -> &[CausalEdge] {
        &self.edges
    }

    /// Validate an insertion against existing anchors and DAG shape.
    pub fn check_insert(
        &self,
        from: &Anchor,
        to: &Anchor,
        status: Status,
        anchor_exists: impl Fn(&Anchor) -> bool,
    ) -> Result<(), CausalError> {
        let probe = CausalEdge::new(0, from.clone(), to.clone(), status, String::new())?;
        for a in [&probe.from, &probe.to] {
            if !anchor_exists(a) {
                return Err(CausalError::DanglingAnchor {
                    anchor: a.to_string(),
                });
            }
        }
        // Cycle check: can `to` already reach `from`? (Deterministic DFS.)
        let mut seen = BTreeSet::new();
        let mut queue = VecDeque::new();
        seen.insert(to.clone());
        queue.push_back(to.clone());
        while let Some(cur) = queue.pop_front() {
            if &cur == from {
                return Err(CausalError::Cycle);
            }
            for next in self.adj.get(&cur).into_iter().flatten() {
                if seen.insert(next.clone()) {
                    queue.push_back(next.clone());
                }
            }
        }
        Ok(())
    }

    /// Record adjacency in both directions (forward + reverse) so
    /// incident queries are O(component), not O(V·E).
    fn link(&mut self, from: Anchor, to: Anchor) {
        self.adj.entry(from.clone()).or_default().insert(to.clone());
        self.radj.entry(to).or_default().insert(from);
    }

    /// Insert a validated edge (caller owns persistence).
    pub fn insert(&mut self, edge: CausalEdge) {
        self.link(edge.from.clone(), edge.to.clone());
        self.edges.push(edge);
    }

    /// The connected component around `anchor` (undirected closure),
    /// including the anchor itself. This is the v0.5 "incident" query.
    pub fn incident_around(&self, anchor: &Anchor) -> BTreeSet<Anchor> {
        let mut seen = BTreeSet::new();
        seen.insert(anchor.clone());
        let mut queue = VecDeque::new();
        queue.push_back(anchor.clone());
        while let Some(cur) = queue.pop_front() {
            // Forward and reverse neighbors (reverse map keeps this
            // O(component) rather than O(V·E)).
            for next in self.adj.get(&cur).into_iter().flatten() {
                if seen.insert(next.clone()) {
                    queue.push_back(next.clone());
                }
            }
            for next in self.radj.get(&cur).into_iter().flatten() {
                if seen.insert(next.clone()) {
                    queue.push_back(next.clone());
                }
            }
        }
        seen
    }
}

/// Validate an observation-anchored edge target exists in the log.
pub fn anchor_exists(
    anchor: &Anchor,
    obs_count: u64,
    commit_exists: impl Fn(&[u8; 32]) -> bool,
) -> bool {
    match anchor {
        Anchor::Observation(seq) => *seq < obs_count,
        Anchor::Commit(id) => commit_exists(id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obs(n: u64) -> Anchor {
        Anchor::Observation(n)
    }

    #[test]
    fn edge_round_trip() {
        let e = CausalEdge::new(
            0,
            obs(1),
            obs(2),
            Status::Hypothesis,
            "link down first".into(),
        )
        .unwrap();
        assert_eq!(CausalEdge::parse(&e.canonical_bytes()).unwrap(), e);
        let commit = Anchor::Commit([7u8; 32]);
        let commit2 = Anchor::Commit([8u8; 32]);
        let e2 =
            CausalEdge::new(1, commit, commit2, Status::Verified, "transition".into()).unwrap();
        assert_eq!(CausalEdge::parse(&e2.canonical_bytes()).unwrap(), e2);
    }

    #[test]
    fn verified_requires_commit_anchors() {
        assert_eq!(
            CausalEdge::new(0, obs(1), obs(2), Status::Verified, String::new()),
            Err(CausalError::VerifiedRequiresCommits)
        );
        // Commit -> observation is also rejected for verified.
        assert_eq!(
            CausalEdge::new(
                0,
                Anchor::Commit([1; 32]),
                obs(2),
                Status::Verified,
                String::new()
            ),
            Err(CausalError::VerifiedRequiresCommits)
        );
    }

    #[test]
    fn self_edge_and_long_note_rejected() {
        assert_eq!(
            CausalEdge::new(0, obs(1), obs(1), Status::Hypothesis, String::new()),
            Err(CausalError::Cycle)
        );
        assert!(CausalEdge::new(0, obs(1), obs(2), Status::Hypothesis, "x".repeat(257)).is_err());
    }

    #[test]
    fn status_and_anchor_parsing() {
        assert!(Status::parse("nope").is_err());
        assert!(Anchor::parse("obs:x").is_err());
        assert!(Anchor::parse("commit:zz").is_err());
        assert_eq!(Anchor::parse("obs:5").unwrap(), obs(5));
    }

    #[test]
    fn dag_cycle_detection() {
        let mut g = CausalGraph::new(vec![]);
        let exists = |_: &Anchor| true;
        g.insert(CausalEdge::new(0, obs(1), obs(2), Status::Hypothesis, String::new()).unwrap());
        g.insert(CausalEdge::new(1, obs(2), obs(3), Status::Hypothesis, String::new()).unwrap());
        // 3 -> 1 would close the cycle 1→2→3→1.
        assert_eq!(
            g.check_insert(&obs(3), &obs(1), Status::Hypothesis, exists),
            Err(CausalError::Cycle)
        );
        // 1 -> 3 is fine (no path 3→1 exists).
        assert!(g
            .check_insert(&obs(1), &obs(3), Status::Hypothesis, exists)
            .is_ok());
    }

    #[test]
    fn dangling_anchor_rejected() {
        let g = CausalGraph::new(vec![]);
        let exists = |a: &Anchor| matches!(a, Anchor::Observation(1));
        assert_eq!(
            g.check_insert(&obs(1), &obs(2), Status::Hypothesis, exists),
            Err(CausalError::DanglingAnchor {
                anchor: "obs:2".into()
            })
        );
    }

    #[test]
    fn incident_is_connected_component() {
        let mut g = CausalGraph::new(vec![]);
        for (f, t) in [(1u64, 2u64), (2, 3), (9, 10)] {
            g.insert(
                CausalEdge::new(f, obs(f), obs(t), Status::Hypothesis, String::new()).unwrap(),
            );
        }
        let comp = g.incident_around(&obs(2));
        assert_eq!(comp, BTreeSet::from([obs(1), obs(2), obs(3)]));
        assert_eq!(
            g.incident_around(&obs(10)),
            BTreeSet::from([obs(9), obs(10)])
        );
    }
}
