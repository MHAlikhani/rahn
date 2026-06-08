// SPDX-License-Identifier: Apache-2.0

//! Observation ingestion benchmark (Stage 4, docs/research/stages/v0.4.md).
//!
//! Run: `cargo test -p rahn-state --release --test obs_scaling -- --ignored --nocapture`
//!
//! Deterministic workload (no randomness, no wall clock for data): N
//! records over a 100-node ring topology; measures encode+append, full
//! log read+parse, filtered query, and storage size.

use rahn_core::{Endpoint, Metadata, Network};
use rahn_state::obs::log::ObsLog;
use rahn_state::obs::{Observation, Subject, Value};
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
fn observation_ingestion_benchmark() {
    let root = std::env::temp_dir().join(format!(
        "rahn-obs-bench-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();

    // 100-node ring as association target.
    let mut net = Network::empty();
    for i in 0..100 {
        net.add_node(&format!("n{i}"), Metadata::new()).unwrap();
        net.add_interface(&format!("n{i}"), "eth0").unwrap();
    }
    for i in 0..100 {
        net.add_link(
            Endpoint::new(&format!("n{i}"), "eth0").unwrap(),
            Endpoint::new(&format!("n{}", (i + 1) % 100), "eth0").unwrap(),
        )
        .unwrap();
    }
    let state_ref = "ab".repeat(32);
    let n = 100_000usize;

    // Build records (not timed).
    let records: Vec<Observation> = (0..n)
        .map(|i| {
            let node = format!("n{}", i % 100);
            let subject = if i % 3 == 0 {
                Subject::Interface {
                    node: node.clone(),
                    iface: "eth0".into(),
                }
            } else {
                Subject::Node(node)
            };
            let value = match i % 3 {
                0 => Value::Gauge(i as i64 % 1000),
                1 => Value::Counter(i as u64),
                _ => Value::Event(format!("event-{i}")),
            };
            Observation::new(
                i as u64,
                1_000_000_000 + i as u64,
                state_ref.clone(),
                subject,
                "latency_ns",
                value,
            )
            .unwrap()
        })
        .collect();

    // Ingestion (encode + append).
    let ingest = median(
        (0..3)
            .map(|_| {
                std::fs::remove_file(ObsLog::path(&root)).unwrap();
                drop(ObsLog::open(&root).unwrap());
                ms(|| {
                    let log = ObsLog::open(&root).unwrap();
                    for (i, r) in records.iter().enumerate() {
                        let mut r = r.clone();
                        r.seq = i as u64;
                        log.append(&r).unwrap();
                    }
                })
            })
            .collect(),
    );
    // Re-open the fully populated log for read benchmarks.
    let log = ObsLog::open(&root).unwrap();

    let read = median(
        (0..5)
            .map(|_| {
                ms(|| {
                    log.read_all().unwrap();
                })
            })
            .collect(),
    );
    let size = std::fs::metadata(ObsLog::path(&root)).unwrap().len();

    println!("{n} records over a 100-node ring (medians):");
    println!(
        "  ingest (encode+append): {ingest:.1} ms ({:.0} rec/s)",
        n as f64 / (ingest / 1000.0)
    );
    println!("  full read+parse:        {read:.1} ms");
    println!(
        "  log size:               {} bytes ({:.1} B/record)",
        size,
        size as f64 / n as f64
    );

    // Per-record latency (median of isolated single appends).
    let mut single = Vec::new();
    for r in records.iter().take(50) {
        let s = Instant::now();
        log.append(r).unwrap();
        single.push(s.elapsed().as_secs_f64() * 1e6);
    }
    single.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!(
        "  single-append latency:  {:.1} µs (median)",
        single[single.len() / 2]
    );

    std::fs::remove_dir_all(&root).ok();
}
