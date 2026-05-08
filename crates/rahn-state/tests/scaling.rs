// SPDX-License-Identifier: Apache-2.0

//! Scaling benchmark (Stage 2 experiment E2, docs/research/benchmark-methodology.md).
//!
//! Run with: `cargo test -p rahn-state --release --ignored -- --nocapture`
//!
//! Deterministic inputs (seeded xorshift, fixed seeds). Reports medians
//! over repeated runs. Machine details MUST be recorded alongside results
//! (see docs/research/stages/v0.2.md).

use rahn_core::{Endpoint, Metadata, Network, State};
use rahn_state::canonical::{canonical_bytes, parse_canonical};
use rahn_state::diff::diff;
use rahn_state::graph::shortest_path;
use rahn_state::identity::StateId;
use rahn_verify::verify;
use std::time::Instant;

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed.max(1))
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }
}

/// Ring topology with cross-links: `n` nodes, one interface each, `n`
/// ring links. Cheap to construct deterministically at any scale.
fn ring(n: usize) -> Network {
    let mut net = Network::empty();
    for i in 0..n {
        net.add_node(&format!("n{i}"), Metadata::new()).unwrap();
        net.add_interface(&format!("n{i}"), "eth0").unwrap();
    }
    for i in 0..n {
        let a = format!("n{i}");
        let b = format!("n{}", (i + 1) % n);
        if i + 1 == n && n == 1 {
            break; // single node: no self-loop
        }
        net.add_link(
            Endpoint::new(&a, "eth0").unwrap(),
            Endpoint::new(&b, "eth0").unwrap(),
        )
        .unwrap();
    }
    net
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn time_ms<F: FnMut()>(reps: usize, mut f: F) -> f64 {
    let mut samples = Vec::new();
    for _ in 0..reps {
        let start = Instant::now();
        f();
        samples.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    median(samples)
}

#[test]
#[ignore]
fn scaling_benchmark() {
    let mut rng = Rng::new(42);
    let _ = rng.next_u64(); // warm the PRNG
    println!(
        "{:>8} {:>12} {:>12} {:>10} {:>10} {:>10} {:>12}",
        "nodes", "build(ms)", "serialize", "hash", "diff", "verify", "path(hops)"
    );
    for &n in &[10usize, 100, 1_000, 10_000, 100_000] {
        let build = time_ms(3, || {
            let _ = ring(n);
        });
        let net = ring(n);
        let state = State {
            network: net.clone(),
        };
        let ser = time_ms(3, || {
            let _ = canonical_bytes(&state);
        });
        let bytes = canonical_bytes(&state);
        let hash = time_ms(5, || {
            let _ = StateId::of(&state);
        });
        // Diff against a mutated copy: sever the ring in the middle third
        // (remove incident links, then the nodes — removals are ordered).
        let drop_from = n / 3;
        let drop_to = (n * 2) / 3;
        let _ = &mut rng;
        // Built directly (not via per-op transitions): per-op application
        // clones the whole network, which is O(n^2) for n removals — that
        // cost is itself a recorded finding (see stage report).
        let mut mutated = net.clone();
        for i in (drop_from - 1)..drop_to {
            let a = Endpoint::new(&format!("n{i}"), "eth0").unwrap();
            let b = Endpoint::new(&format!("n{}", (i + 1) % n), "eth0").unwrap();
            mutated.remove_link(a, b).unwrap();
        }
        for i in drop_from..drop_to {
            mutated.remove_node(&format!("n{i}")).unwrap();
        }
        let diff_ms = time_ms(3, || {
            let _ = diff(&net, &mutated);
        });
        let constitution = rahn_verify::Constitution::default();
        let verify_ms = time_ms(3, || {
            let _ = verify(&state, &constitution);
        });
        // Path from node 0 to the far end of the ring (worst case: n/2 hops).
        let path_target = format!("n{}", n / 2);
        let path_ms = time_ms(3, || {
            let _ = shortest_path(&net, "n0", &path_target).unwrap();
        });
        // Round-trip sanity at every scale.
        let parsed = parse_canonical(&bytes).unwrap();
        assert_eq!(parsed, state);
        println!(
            "{:>8} {:>12.3} {:>12.3} {:>10.3} {:>10.3} {:>10.3} {:>12.3}",
            n, build, ser, hash, diff_ms, verify_ms, path_ms
        );
    }
}
