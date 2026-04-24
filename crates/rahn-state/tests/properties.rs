// SPDX-License-Identifier: Apache-2.0

//! Property-based tests over randomly generated networks (docs/testing.md).
//!
//! Generators use a seeded xorshift PRNG: fully deterministic across runs
//! and platforms (no floating point, no system randomness), so failures
//! reproduce from the recorded seed. Each test prints its seed on failure.

use rahn_core::{Link, Metadata, Network, State};
use rahn_state::canonical::{canonical_bytes, parse_canonical};
use rahn_state::diff::diff;
use rahn_state::identity::StateId;
use rahn_state::transition::{apply, Operation};
use rahn_verify::merge;

/// Deterministic xorshift64* PRNG.
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

    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

/// Generate a random node id.
fn node_name(rng: &mut Rng, i: usize) -> String {
    format!("n{}-{}", i, rng.below(1_000_000))
}

/// Generate a random network with `node_count` nodes and a random subset
/// of possible links. Metadata-free: the v0.1 operation vocabulary cannot
/// express metadata edits, so reconstruction properties are stated over
/// topology (metadata round-trip is covered by the fixed-vector tests).
fn random_network(rng: &mut Rng, node_count: usize, link_prob: u64) -> Network {
    let mut net = Network::empty();
    let mut names = Vec::with_capacity(node_count);
    for i in 0..node_count {
        let name = node_name(rng, i);
        net.add_node(&name, Metadata::new()).unwrap();
        names.push(name);
    }
    for i in 0..node_count {
        for j in (i + 1)..node_count {
            if rng.next_u64() % 100 < link_prob {
                net.add_link(&names[i], &names[j]).unwrap();
            }
        }
    }
    net
}

/// Property 1: canonical round-trip is exact for every generated state,
/// and identity is stable under re-serialization.
#[test]
fn property_canonical_round_trip_and_identity() {
    for seed in 1..=200u64 {
        let mut rng = Rng::new(seed);
        let count = 1 + rng.below(12);
        let link_prob = rng.below(100) as u64;
        let net = random_network(&mut rng, count, link_prob);
        let state = State { network: net };

        let bytes = canonical_bytes(&state);
        let parsed =
            parse_canonical(&bytes).unwrap_or_else(|e| panic!("seed {seed}: parse failed: {e}"));
        assert_eq!(parsed, state, "seed {seed}: round-trip mismatch");
        assert_eq!(
            canonical_bytes(&parsed),
            bytes,
            "seed {seed}: canonicalization not idempotent"
        );
        assert_eq!(
            StateId::of(&parsed),
            StateId::of(&state),
            "seed {seed}: identity churn"
        );
    }
}

/// Property 2: the diff of `from -> to`, converted to operations, exactly
/// reconstructs `to` when applied to `from`.
#[test]
fn property_diff_operations_reconstruct_target() {
    for seed in 1..=200u64 {
        let mut rng = Rng::new(seed);
        let mut base_rng = Rng::new(seed * 7 + 1);
        let base = random_network(&mut base_rng, 1 + rng.below(8), 40);
        let target = {
            // Mutate the base with random valid transitions.
            let mut current = base.clone();
            let mutations = 1 + rng.below(6);
            for _ in 0..mutations {
                current = mutate(&current, &mut rng);
            }
            current
        };
        let d = diff(&base, &target);
        let ops = diff_to_ops(&base, &target, &d);
        let reconstructed = rahn_state::transition::apply_all(&base, &ops)
            .unwrap_or_else(|e| panic!("seed {seed}: ops rejected: {e}"));
        assert_eq!(
            reconstructed, target,
            "seed {seed}: diff-derived operations did not reconstruct target"
        );
    }
}

fn mutate(net: &Network, rng: &mut Rng) -> Network {
    let nodes: Vec<String> = net.iter_nodes().map(|n| n.id.clone()).collect();
    let choice = rng.below(4);
    match choice {
        0 => {
            let idx = rng.below(1000);
            let name = node_name(rng, idx);
            apply(
                net,
                &Operation::AddNode {
                    id: name,
                    metadata: Metadata::new(),
                },
            )
            .unwrap_or_else(|_| net.clone())
        }
        1 => {
            // Remove a node with no links (pick any removable one).
            let removable: Vec<&String> = nodes
                .iter()
                .filter(|id| !net.iter_links().any(|l| &l.a == *id || &l.b == *id))
                .collect();
            if removable.is_empty() {
                return net.clone();
            }
            let victim = removable[rng.below(removable.len())].clone();
            apply(net, &Operation::RemoveNode { id: victim }).unwrap_or_else(|_| net.clone())
        }
        2 => {
            if nodes.len() < 2 {
                return net.clone();
            }
            let a = &nodes[rng.below(nodes.len())];
            let b = &nodes[rng.below(nodes.len())];
            apply(
                net,
                &Operation::AddLink {
                    a: a.clone(),
                    b: b.clone(),
                },
            )
            .unwrap_or_else(|_| net.clone())
        }
        _ => {
            let links: Vec<(String, String)> = net
                .iter_links()
                .map(|l: &Link| (l.a.clone(), l.b.clone()))
                .collect();
            if links.is_empty() {
                return net.clone();
            }
            let (a, b) = links[rng.below(links.len())].clone();
            apply(net, &Operation::RemoveLink { a, b }).unwrap_or_else(|_| net.clone())
        }
    }
}

fn diff_to_ops(from: &Network, to: &Network, d: &rahn_state::Diff) -> Vec<Operation> {
    let _ = (from, to);
    let mut ops = Vec::new();
    for (a, b) in &d.removed_links {
        ops.push(Operation::RemoveLink {
            a: a.clone(),
            b: b.clone(),
        });
    }
    for id in &d.removed_nodes {
        ops.push(Operation::RemoveNode { id: id.clone() });
    }
    for id in &d.added_nodes {
        // Metadata-free generator: additions carry empty metadata.
        ops.push(Operation::AddNode {
            id: id.clone(),
            metadata: Metadata::new(),
        });
    }
    for (a, b) in &d.added_links {
        ops.push(Operation::AddLink {
            a: a.clone(),
            b: b.clone(),
        });
    }
    ops
}

/// Property 3: merging two branches that remove disjoint node sets yields
/// the union regardless of side order (merge commutativity for disjoint
/// effects), and the result satisfies all structural invariants.
#[test]
fn property_merge_disjoint_removals_is_commutative_and_valid() {
    for seed in 1..=100u64 {
        let mut rng = Rng::new(seed);
        let base = random_network(&mut rng, 8, 30);

        // Each side removes a disjoint, non-empty subset of link-free nodes.
        let removable: Vec<String> = base
            .iter_nodes()
            .filter(|n| !base.iter_links().any(|l| l.a == n.id || l.b == n.id))
            .map(|n| n.id.clone())
            .collect();
        if removable.len() < 2 {
            continue;
        }
        let half = 1 + rng.below(removable.len() / 2);
        let ours_remove: Vec<String> = removable.iter().take(half).cloned().collect();
        let theirs_remove: Vec<String> = removable
            .iter()
            .skip(half)
            .take(1 + rng.below(removable.len() - half))
            .cloned()
            .collect();

        let ours = remove_nodes(&base, &ours_remove);
        let theirs = remove_nodes(&base, &theirs_remove);

        let ab = merge(&base, &ours, &theirs)
            .unwrap_or_else(|e| panic!("seed {seed}: ours-then-theirs rejected: {e}"));
        let ba = merge(&base, &theirs, &ours)
            .unwrap_or_else(|e| panic!("seed {seed}: theirs-then-ours rejected: {e}"));
        assert_eq!(
            ab, ba,
            "seed {seed}: merge is order-dependent for disjoint effects"
        );

        // Every removed node is gone; everything else survives.
        for id in ours_remove.iter().chain(theirs_remove.iter()) {
            assert!(ab.node(id).is_none(), "seed {seed}: {id} should be removed");
        }
        assert_eq!(
            ab.node_count(),
            base.node_count() - ours_remove.len() - theirs_remove.len()
        );
    }
}

fn remove_nodes(net: &Network, ids: &[String]) -> Network {
    let mut current = net.clone();
    for id in ids {
        current = apply(&current, &Operation::RemoveNode { id: id.clone() })
            .unwrap_or_else(|e| panic!("remove {id}: {e}"));
    }
    current
}
