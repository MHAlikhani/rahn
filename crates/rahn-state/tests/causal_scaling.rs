// SPDX-License-Identifier: Apache-2.0

//! Causal-graph benchmark (Stage 5, docs/research/stages/v0.5.md).
//! Run: `cargo test -p rahn-state --release --test causal_scaling -- --ignored --nocapture`

use rahn_state::causal::{Anchor, CausalEdge, CausalGraph, Status};
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

#[test]
#[ignore]
fn causal_graph_benchmark() {
    let n = 10_000usize;
    // Linear chain obs:0 -> obs:1 -> ... (worst case for cycle-check DFS depth).
    let edges: Vec<CausalEdge> = (0..n)
        .map(|i| {
            CausalEdge::new(
                i as u64,
                Anchor::Observation(i as u64),
                Anchor::Observation(i as u64 + 1),
                Status::Hypothesis,
                String::new(),
            )
            .unwrap()
        })
        .collect();

    let build = median(
        (0..5)
            .map(|_| {
                ms(|| {
                    CausalGraph::new(edges.clone());
                })
            })
            .collect(),
    );
    let g = CausalGraph::new(edges);
    let exists = |_: &Anchor| true;

    // Cycle-check on the chain tail (deepest possible DFS: to reaches from
    // through the whole chain).
    let check = median(
        (0..5)
            .map(|_| {
                ms(|| {
                    g.check_insert(
                        &Anchor::Observation(n as u64),
                        &Anchor::Observation(0),
                        Status::Hypothesis,
                        exists,
                    )
                    .unwrap_err();
                })
            })
            .collect(),
    );
    // Incident query at the chain end.
    let incident = median(
        (0..5)
            .map(|_| {
                ms(|| {
                    g.incident_around(&Anchor::Observation(n as u64));
                })
            })
            .collect(),
    );

    println!("{n}-edge linear chain:");
    println!("  graph build (from edges):     {build:.1} ms");
    println!("  cycle check (full-chain DFS): {check:.1} ms");
    println!("  incident (connected comp):    {incident:.1} ms");
}
