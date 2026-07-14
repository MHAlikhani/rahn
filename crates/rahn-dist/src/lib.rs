// SPDX-License-Identifier: Apache-2.0

//! Peer synchronization over content-addressed history (ADR 0015).
//!
//! Model: every replica is authoritative for its own history; sync
//! exchanges immutable records (commits, states) and branch-tip offers;
//! divergence converges only through the fail-closed semantic merge,
//! producing a **byte-identical merge commit on every replica**. No
//! linearizability or strong consistency across replicas is claimed.
//! This module is a deterministic engine: message exchange is simulated
//! in-memory and all outcomes are reproducible.

use std::collections::{BTreeMap, BTreeSet};

use rahn_core::State;
use rahn_state::commit::{CommitId, CommitRecord};
use rahn_state::history::common_ancestor;
use rahn_store::{Store, StoreError};

/// Self-asserted replica identity (v0.6 trust boundary; ADR 0015).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReplicaId(pub String);

impl ReplicaId {
    pub fn new(id: &str) -> Result<Self, SyncError> {
        rahn_core::validate_id(id).map_err(|e| SyncError::InvalidReplicaId(e.to_string()))?;
        Ok(ReplicaId(id.to_owned()))
    }
}

/// What one replica advertises: branch name -> tip commit id.
pub type Offer = BTreeMap<String, CommitId>;

#[derive(Debug)]
pub enum SyncError {
    Store(StoreError),
    InvalidReplicaId(String),
    /// Fail-closed divergence: the merge cannot be proven safe.
    Conflict(rahn_verify::MergeConflict),
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncError::Store(e) => write!(f, "store error: {e}"),
            SyncError::InvalidReplicaId(e) => write!(f, "invalid replica id: {e}"),
            SyncError::Conflict(c) => write!(f, "sync divergence unresolved: {c}"),
        }
    }
}

impl std::error::Error for SyncError {}

impl From<StoreError> for SyncError {
    fn from(e: StoreError) -> Self {
        SyncError::Store(e)
    }
}

/// Per-branch synchronization outcome (ADR 0015).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchOutcome {
    UpToDate,
    /// Adopted the peer's tip (we had no divergence).
    Fetched,
    /// Divergence resolved by merge; carries the identical merge commit
    /// created on both replicas.
    Merged(CommitId),
    /// Fail-closed: tips remain divergent; conflict is explainable.
    Conflict(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SyncReport {
    pub outcomes: BTreeMap<String, BranchOutcome>,
}

impl SyncReport {
    pub fn converged(&self) -> bool {
        self.outcomes
            .values()
            .all(|o| !matches!(o, BranchOutcome::Conflict(_)))
    }
}

/// Advertise branch tips of a store.
pub fn offer(store: &Store) -> Result<Offer, SyncError> {
    let mut out = Offer::new();
    for b in store.list_branches()? {
        if let Some(c) = store.get_branch(&b)? {
            out.insert(b, c);
        }
    }
    Ok(out)
}

/// Fetch all objects reachable from `tips` that are missing locally,
/// copying from `remote`. Only complete ancestries are adopted.
fn fetch_missing(
    local: &Store,
    remote: &Store,
    tips: impl IntoIterator<Item = CommitId>,
) -> Result<(), SyncError> {
    let mut queue: Vec<CommitId> = tips.into_iter().collect();
    let mut seen = BTreeSet::new();
    while let Some(id) = queue.pop() {
        if !seen.insert(id) {
            continue;
        }
        if local.get_commit(id)?.is_some() {
            continue;
        }
        let rec: CommitRecord =
            remote
                .get_commit(id)?
                .ok_or(SyncError::Store(StoreError::NotFound(format!(
                    "remote is missing advertised commit {}",
                    id.as_hex()
                ))))?;
        // Fetch the state object too (records reference it).
        if local.get_state(rec.state_id)?.is_none() {
            let state: State =
                remote
                    .get_state(rec.state_id)?
                    .ok_or(SyncError::Store(StoreError::NotFound(format!(
                        "remote is missing state {}",
                        rec.state_id.as_hex()
                    ))))?;
            local.put_state(&state)?;
        }
        local.put_commit(&rec)?;
        queue.extend(rec.parents);
    }
    Ok(())
}

/// Two-way synchronization of two repositories (deterministic).
///
/// For every branch name known to either side:
/// - adopt the peer tip if we have none or are strictly behind
///   (peer tip's ancestry contains ours);
/// - if diverged (common ancestor exists, tips differ, neither is an
///   ancestor of the other): attempt the fail-closed semantic merge over
///   the base; on success both replicas create the same merge commit
///   (parents ordered lexicographically by commit id, message
///   `sync merge`); on conflict the branch stays divergent and the
///   conflict is reported.
pub fn sync_pair(
    a: &Store,
    ra: &ReplicaId,
    b: &Store,
    rb: &ReplicaId,
) -> Result<SyncReport, SyncError> {
    let _ = (ra, rb); // identities recorded in future protocol logging
    let offer_a = offer(a)?;
    let offer_b = offer(b)?;
    let mut report = SyncReport::default();

    let mut names: BTreeSet<String> = offer_a.keys().cloned().collect();
    names.extend(offer_b.keys().cloned());
    let names: Vec<String> = names.into_iter().collect();

    for name in names {
        let ours = a.get_branch(&name)?;
        let theirs = b.get_branch(&name)?;
        let outcome = match (ours, theirs) {
            (None, None) => BranchOutcome::UpToDate,
            (None, Some(t)) => {
                fetch_missing(a, b, [t])?;
                b.set_branch(&name, t)?;
                a.set_branch(&name, t)?;
                BranchOutcome::Fetched
            }
            (Some(t), None) => {
                fetch_missing(b, a, [t])?;
                b.set_branch(&name, t)?;
                BranchOutcome::Fetched
            }
            (Some(ours), Some(theirs)) => {
                if ours == theirs {
                    BranchOutcome::UpToDate
                } else {
                    fetch_missing(a, b, [theirs])?;
                    fetch_missing(b, a, [ours])?;
                    let parents_of = |id: CommitId| -> Option<Vec<CommitId>> {
                        a.get_commit(id).ok().flatten().map(|r| r.parents)
                    };
                    let ancestor = common_ancestor(ours, theirs, parents_of);
                    // ancestor == theirs  ⇒ ours already contains theirs (we
                    // are ahead).  ancestor == ours ⇒ theirs contains ours
                    // (we are strictly behind: adopt theirs).
                    let (we_are_ahead, we_are_behind) = match ancestor {
                        Some(base) if base == theirs => (true, false),
                        Some(base) if base == ours => (false, true),
                        _ => (false, false),
                    };
                    if we_are_behind {
                        a.set_branch(&name, theirs)?;
                        BranchOutcome::Fetched
                    } else if we_are_ahead {
                        // Peer is strictly behind: two-way sync pushes our
                        // history to them (fetch + adopt).
                        fetch_missing(b, a, [ours])?;
                        b.set_branch(&name, ours)?;
                        BranchOutcome::Fetched
                    } else {
                        // True divergence: fail-closed semantic merge.
                        let base = match ancestor {
                            Some(base) => state_of(a, base)?,
                            None => State::empty(),
                        };
                        let ours_state = state_of(a, ours)?;
                        let theirs_state = state_of(a, theirs)?;
                        let merged_network = rahn_verify::merge(
                            &base.network,
                            &ours_state.network,
                            &theirs_state.network,
                        )
                        .map_err(|c| {
                            report
                                .outcomes
                                .insert(name.clone(), BranchOutcome::Conflict(c.conflicts.clone()));
                            SyncError::Conflict(c)
                        })?;
                        let merged = State {
                            network: merged_network,
                        };
                        // Constitution must accept the merged candidate.
                        let constitution_text = a.load_constitution_text()?;
                        let constitution = rahn_verify::Constitution::parse(&constitution_text)
                            .map_err(|e| SyncError::Store(StoreError::InvalidRef(e.to_string())))?;
                        let v = rahn_verify::verify(&merged, &constitution);
                        if !v.passed() {
                            let conf = vec![format!(
                                "merged state rejected by verification: {}",
                                v.failed_ids().join(", ")
                            )];
                            report
                                .outcomes
                                .insert(name.clone(), BranchOutcome::Conflict(conf.clone()));
                            return Err(SyncError::Conflict(rahn_verify::MergeConflict {
                                conflicts: conf,
                            }));
                        }
                        // Deterministic merge commit on BOTH replicas:
                        // parents ordered lexicographically by commit id.
                        let mut parents = vec![ours, theirs];
                        parents.sort();
                        let state_id = a.put_state(&merged)?;
                        let record = CommitRecord {
                            state_id,
                            parents: parents.clone(),
                            operations: Vec::new(),
                            message: "sync merge".into(),
                            verification: rahn_core::VerificationSummary::passed(),
                        };
                        let merge_id = a.put_commit(&record)?;
                        // b must hold the same objects (it may not have the
                        // merged state yet).
                        if b.get_state(state_id)?.is_none() {
                            b.put_state(&merged)?;
                        }
                        if b.get_commit(merge_id)?.is_none() {
                            b.put_commit(&record)?;
                        }
                        a.set_branch(&name, merge_id)?;
                        b.set_branch(&name, merge_id)?;
                        BranchOutcome::Merged(merge_id)
                    }
                }
            }
        };
        report.outcomes.entry(name).or_insert(outcome);
    }
    Ok(report)
}

fn state_of(store: &Store, id: CommitId) -> Result<State, SyncError> {
    let rec = store
        .get_commit(id)?
        .ok_or(SyncError::Store(StoreError::NotFound(format!(
            "commit {} missing",
            id.as_hex()
        ))))?;
    store
        .get_state(rec.state_id)?
        .ok_or(SyncError::Store(StoreError::NotFound(format!(
            "state {} missing",
            rec.state_id.as_hex()
        ))))
}
