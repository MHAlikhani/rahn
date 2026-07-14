// SPDX-License-Identifier: Apache-2.0

//! Distributed-state failure scenarios (ADR 0015): divergence, partition,
//! recovery, conflicts, corruption. All deterministic (in-memory stores,
//! fixed command sequences; no sockets, no threads).

use rahn_core::State;
use rahn_dist::{offer, sync_pair, BranchOutcome, ReplicaId, SyncError};
use rahn_state::transition::Operation;
use rahn_store::Store;

fn temp_repo(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rahn-dist-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn open(tag: &str) -> Store {
    Store::init(temp_repo(tag).join(".rahn")).unwrap()
}

/// Commit explicit operations against the branch's parent state
/// (index-free: distributed tests reason about committed history only).
fn commit_ops(store: &Store, name: &str, ops: &[Operation]) -> rahn_state::commit::CommitId {
    let parent = store.get_branch("main").unwrap();
    let parent_state = match parent {
        Some(id) => {
            let rec = store.get_commit(id).unwrap().unwrap();
            store.get_state(rec.state_id).unwrap().unwrap()
        }
        None => State::empty(),
    };
    let network = rahn_state::transition::apply_all(&parent_state.network, ops)
        .expect("test operations must be valid transitions");
    let new_state = State { network };
    let report = rahn_verify::verify(&new_state, &rahn_verify::Constitution::default());
    assert!(report.passed());
    let state_id = store.put_state(&new_state).unwrap();
    let record = rahn_state::commit::CommitRecord {
        state_id,
        parents: parent.into_iter().collect(),
        operations: ops.to_vec(),
        message: name.into(),
        verification: rahn_core::VerificationSummary::passed(),
    };
    let id = store.put_commit(&record).unwrap();
    store.set_branch("main", id).unwrap();
    store.save_index(&new_state).unwrap();
    id
}

fn add_node_commit(store: &Store, name: &str, node: &str) -> rahn_state::commit::CommitId {
    commit_ops(
        store,
        name,
        &[Operation::AddNode {
            id: node.into(),
            metadata: Default::default(),
        }],
    )
}

#[test]
fn identical_histories_are_up_to_date() {
    let a = open("same-a");
    let b = open("same-b");
    add_node_commit(&a, "n1", "x");
    add_node_commit(&b, "n1", "x"); // same deterministic commit
    let rep = sync_pair(
        &a,
        &ReplicaId::new("a").unwrap(),
        &b,
        &ReplicaId::new("b").unwrap(),
    )
    .unwrap();
    assert_eq!(rep.outcomes.get("main"), Some(&BranchOutcome::UpToDate));
}

#[test]
fn strictly_behind_replica_adopts_peer_tip() {
    let ahead = open("ahead");
    let behind = open("behind");
    add_node_commit(&ahead, "n1", "x");
    // `behind` is empty: no branch. Sync adopts ahead's history.
    let rep = sync_pair(
        &ahead,
        &ReplicaId::new("a").unwrap(),
        &behind,
        &ReplicaId::new("b").unwrap(),
    )
    .unwrap();
    assert_eq!(rep.outcomes.get("main"), Some(&BranchOutcome::Fetched));
    assert_eq!(offer(&ahead).unwrap(), offer(&behind).unwrap());
    // Convergence is content-identical.
    assert_eq!(
        offer(&ahead).unwrap().get("main"),
        offer(&behind).unwrap().get("main")
    );

    // Reverse direction: strictly-behind peer adopts (we're ahead).
    add_node_commit(&ahead, "n2", "y");
    let rep = sync_pair(
        &behind,
        &ReplicaId::new("b").unwrap(),
        &ahead,
        &ReplicaId::new("a").unwrap(),
    )
    .unwrap();
    assert_eq!(rep.outcomes.get("main"), Some(&BranchOutcome::Fetched));
    assert_eq!(offer(&ahead).unwrap(), offer(&behind).unwrap());
}

#[test]
fn divergent_disjoint_histories_merge_identically() {
    let a = open("div-a");
    let b = open("div-b");
    add_node_commit(&a, "base", "x");
    // b adopts base (fetch).
    sync_pair(
        &a,
        &ReplicaId::new("a").unwrap(),
        &b,
        &ReplicaId::new("b").unwrap(),
    )
    .unwrap();
    // Partition: both write independently (disjoint nodes).
    add_node_commit(&a, "a adds y", "y");
    add_node_commit(&b, "b adds z", "z");
    // Reconnect and sync — from both directions.
    let rep_ab = sync_pair(
        &a,
        &ReplicaId::new("a").unwrap(),
        &b,
        &ReplicaId::new("b").unwrap(),
    )
    .unwrap();
    assert_eq!(
        rep_ab.outcomes.get("main"),
        Some(&BranchOutcome::Merged(offer(&a).unwrap()["main"]))
    );
    let rep_ba = sync_pair(
        &b,
        &ReplicaId::new("b").unwrap(),
        &a,
        &ReplicaId::new("a").unwrap(),
    )
    .unwrap();
    assert_eq!(rep_ba.outcomes.get("main"), Some(&BranchOutcome::UpToDate));
    // Byte-identical converged history.
    assert_eq!(offer(&a).unwrap(), offer(&b).unwrap());
}

#[test]
fn conflicting_divergence_fails_closed_and_persists() {
    let a = open("conf-a");
    let b = open("conf-b");
    // Base: node x with an interface.
    commit_ops(
        &a,
        "base",
        &[
            Operation::AddNode {
                id: "x".into(),
                metadata: Default::default(),
            },
            Operation::AddInterface {
                node: "x".into(),
                name: "eth0".into(),
            },
        ],
    );
    sync_pair(
        &a,
        &ReplicaId::new("a").unwrap(),
        &b,
        &ReplicaId::new("b").unwrap(),
    )
    .unwrap();
    // True fail-closed conflict: A removes x; B (independent) adds node z
    // and links it to x — B's link references the node A removed. No
    // object-level conflict exists, but the merged application is
    // impossible: the merge must fail closed (ADR 0015).
    commit_ops(
        &a,
        "a removes x",
        &[Operation::RemoveNode { id: "x".into() }],
    );
    commit_ops(
        &b,
        "b adds z linked to x",
        &[
            Operation::AddNode {
                id: "z".into(),
                metadata: Default::default(),
            },
            Operation::AddInterface {
                node: "z".into(),
                name: "eth0".into(),
            },
            Operation::AddLink {
                a: rahn_core::Endpoint::new("z", "eth0").unwrap(),
                b: rahn_core::Endpoint::new("x", "eth0").unwrap(),
            },
        ],
    );

    let err = sync_pair(
        &a,
        &ReplicaId::new("a").unwrap(),
        &b,
        &ReplicaId::new("b").unwrap(),
    );
    match err {
        Err(SyncError::Conflict(c)) => assert!(!c.conflicts.is_empty()),
        other => panic!("expected conflict, got {other:?}"),
    }
    // Fail-closed persistence: tips remain divergent, each replica keeps
    // its own history, and re-sync produces the same conflict.
    assert_ne!(offer(&a).unwrap()["main"], offer(&b).unwrap()["main"]);
    assert!(matches!(
        sync_pair(
            &a,
            &ReplicaId::new("a").unwrap(),
            &b,
            &ReplicaId::new("b").unwrap()
        ),
        Err(SyncError::Conflict(_))
    ));
}

#[test]
fn restart_recovery_reopens_and_syncs() {
    let b_root = temp_repo("rec-b");
    let a = open("rec-a");
    let b = Store::init(b_root.join(".rahn")).unwrap();
    add_node_commit(&a, "n1", "x");
    sync_pair(
        &a,
        &ReplicaId::new("a").unwrap(),
        &b,
        &ReplicaId::new("b").unwrap(),
    )
    .unwrap();
    // "Restart" b: drop all in-memory handles and reopen from disk only.
    drop(b);
    let b2 = Store::open(b_root.join(".rahn")).unwrap();
    add_node_commit(&a, "n2", "y");
    let rep = sync_pair(
        &a,
        &ReplicaId::new("a").unwrap(),
        &b2,
        &ReplicaId::new("b").unwrap(),
    )
    .unwrap();
    assert_eq!(rep.outcomes.get("main"), Some(&BranchOutcome::Fetched));
    assert_eq!(offer(&a).unwrap(), offer(&b2).unwrap());
}

#[test]
fn corrupted_replication_data_is_refused() {
    let a = open("cor-a");
    let b = open("cor-b");
    add_node_commit(&a, "n1", "x");
    // Corrupt a's state object so fetch verification fails loudly.
    let mut corrupted_dir = None;
    for e in std::fs::read_dir(a.root().join("objects"))
        .unwrap()
        .flatten()
    {
        if e.path().is_dir() {
            for f in std::fs::read_dir(e.path()).unwrap().flatten() {
                let bytes = std::fs::read(f.path()).unwrap();
                let mut bad = bytes.clone();
                let last = bad.len() - 1;
                bad[last] ^= 0xFF;
                std::fs::write(f.path(), &bad).unwrap();
                corrupted_dir = Some(());
            }
        }
    }
    assert!(corrupted_dir.is_some());
    let err = sync_pair(
        &a,
        &ReplicaId::new("a").unwrap(),
        &b,
        &ReplicaId::new("b").unwrap(),
    );
    assert!(err.is_err(), "corrupted objects must fail loudly");
}

#[test]
fn sync_is_idempotent_under_repetition_and_order() {
    // Repeated sync (duplicated messages) and reversed roles converge to
    // the same state — the deterministic-message-exchange property.
    let a = open("idem-a");
    let b = open("idem-b");
    add_node_commit(&a, "base", "x");
    sync_pair(
        &a,
        &ReplicaId::new("a").unwrap(),
        &b,
        &ReplicaId::new("b").unwrap(),
    )
    .unwrap();
    add_node_commit(&a, "a1", "y");
    add_node_commit(&b, "b1", "z");
    let id = |s: &str| ReplicaId::new(s).unwrap();
    let r1 = sync_pair(&a, &id("a"), &b, &id("b")).unwrap();
    assert!(matches!(
        r1.outcomes.get("main"),
        Some(BranchOutcome::Merged(_))
    ));
    // Duplicated messages are idempotent: the repeat sees convergence.
    let r2 = sync_pair(&a, &id("a"), &b, &id("b")).unwrap();
    assert_eq!(r2.outcomes.get("main"), Some(&BranchOutcome::UpToDate));
    // Reversed direction likewise.
    let r3 = sync_pair(&b, &id("b"), &a, &id("a")).unwrap();
    assert_eq!(r3.outcomes.get("main"), Some(&BranchOutcome::UpToDate));
    // Converged offers are byte-identical.
    assert_eq!(offer(&a).unwrap(), offer(&b).unwrap());
}
