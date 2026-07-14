// SPDX-License-Identifier: Apache-2.0

//! Sync benchmark (Stage 6, docs/research/stages/v0.6.md).
//! Run: `cargo test -p rahn-dist --release --test sync_scaling -- --ignored --nocapture`

use rahn_core::State;
use rahn_dist::{offer, sync_pair, ReplicaId};
use rahn_state::transition::Operation;
use rahn_store::Store;
use std::time::Instant;

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn ms(f: impl FnOnce()) -> f64 {
    let s = Instant::now();
    f();
    s.elapsed().as_secs_f64() * 1000.0
}

fn temp(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!(
        "rahn-sync-bench-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Build a store with `n` sequential node-add commits (ring-free).
fn build(tag: &str, n: usize) -> Store {
    let store = Store::init(temp(tag).join(".rahn")).unwrap();
    for i in 0..n {
        let parent = store.get_branch("main").unwrap();
        let parent_state = match parent {
            Some(id) => {
                let rec = store.get_commit(id).unwrap().unwrap();
                store.get_state(rec.state_id).unwrap().unwrap()
            }
            None => State::empty(),
        };
        let ops = vec![Operation::AddNode {
            id: format!("n{i}"),
            metadata: Default::default(),
        }];
        let network = rahn_state::transition::apply_all(&parent_state.network, &ops).unwrap();
        let new_state = State { network };
        let state_id = store.put_state(&new_state).unwrap();
        let rec = rahn_state::commit::CommitRecord {
            state_id,
            parents: parent.into_iter().collect(),
            operations: ops,
            message: format!("c{i}"),
            verification: rahn_core::VerificationSummary::passed(),
        };
        let id = store.put_commit(&rec).unwrap();
        store.set_branch("main", id).unwrap();
        store.save_index(&new_state).unwrap();
    }
    store
}

fn two_repos(tag: &str) -> (Store, Store) {
    let a = build(&format!("{tag}-a"), 0);
    let b = Store::init(temp(&format!("{tag}-b")).join(".rahn")).unwrap();
    (a, b)
}

#[test]
#[ignore]
fn sync_scaling_benchmark() {
    let id_a = ReplicaId::new("a").unwrap();
    let id_b = ReplicaId::new("b").unwrap();
    println!(
        "{:>8} {:>14} {:>14} {:>12}",
        "commits", "offer(ms)", "sync(ms)", "log-objs"
    );
    for &n in &[10usize, 100, 1_000] {
        // Fresh pair per size; a builds n commits, b empty.
        let root = temp(&format!("s{n}"));
        let a = build(&format!("s{n}"), n);
        let b = Store::init(root.join("b/.rahn")).unwrap();
        let o = median((0..3).map(|_| ms(|| { offer(&a).unwrap(); })).collect());
        let sync = ms(|| { sync_pair(&a, &id_a, &b, &id_b).unwrap(); });
        let objects = {
            let mut count = 0;
            for d in std::fs::read_dir(a.root().join("objects"))
                .unwrap()
                .flatten()
            {
                count += std::fs::read_dir(d.path()).unwrap().count();
            }
            count
        };
        // Verify convergence.
        assert_eq!(offer(&a).unwrap(), offer(&b).unwrap());
        println!("{n:>8} {o:>14.3} {sync:>14.3} {objects:>12}");
    }
}
