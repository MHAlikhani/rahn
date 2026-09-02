# RAHN

**RAHN** is a stateful execution architecture for evolving networks.

> RAHN gives networks state, memory, causality, and verifiable evolution.

The name is inspired by the Persian concept of *راه / rah* — a path, a way of reaching somewhere. The project is designed to remain globally usable and independent of any individual's identity.

## What RAHN is

RAHN models a network not as a collection of devices and configurations, but as an **evolving computational system** with:

- state
- history
- intent
- constraints and invariants
- causality
- observations
- executable transitions
- verification
- controlled evolution

The central research question:

> Can network state become a first-class computational object — versioned, reasoned about, verified, replayed, branched, simulated, evolved, and safely executed?

RAHN is developed as an **architectural research project first**, a systems implementation second, and an ecosystem third.

## What RAHN is not

RAHN is not a router, a VPN, a mesh VPN, an SDN controller clone, a monitoring dashboard, a packet analyzer, a generic simulator, an infrastructure-as-code tool, an LLM wrapper, or a Git-like configuration store. It may eventually integrate with many of those ecosystems, but it must not be reduced to any of them.

It is **not** "Git for networks". Git inspires certain concepts (immutable history, branching, diffing), but RAHN's target abstraction is deeper: *versioned + causal + executable + verifiable network state*.

## Status

**v1.0.0 — Stable architecture release.**

An experimental, deterministic state engine for evolving network topologies: interfaces and interface-addressed links, canonical serialization (format v2), content-addressed state identity, local persistence, explicit transitions, semantic diff, branches, fail-closed semantic merge, deterministic path discovery, isolation constraints, a deterministic invariant engine, verification-gated commits, and simulation-first execution. **By default `rahn apply` prints the exact commands and touches nothing**; real execution is explicit opt-in (`--execute --yes-i-know`): isolated Linux network namespaces + veth links via iproute2, with structural host-safety guarantees (ADR 0012) — Linux + root only, CI-validated.

```
cargo build --release
cargo test
```

Per the project charter, documentation is part of the architecture, not a secondary artifact:

- [ARCHITECTURE.md](ARCHITECTURE.md) — the architecture, its goals, trade-offs, and failure modes
- [DESIGN.md](DESIGN.md) — design principles and how they shape the system
- [docs/concepts.md](docs/concepts.md) — core concepts and terminology in depth
- [docs/adr/](docs/adr/) — Architecture Decision Records
- [docs/spec/](docs/spec/) — normative technical specification (in progress)
- [docs/research/](docs/research/) — problem statement, prior art, research questions, hypotheses
- [docs/whitepaper/RAHN-Whitepaper.md](docs/whitepaper/RAHN-Whitepaper.md) — technical white paper (living draft with claim-status markers)
- [ROADMAP.md](ROADMAP.md) — research roadmap toward v1.0

## Implementation status

The primary implementation language is **Rust**. Rust is an implementation choice, not the identity of the architecture.

- **Implemented (v0.2):** deterministic interface-based network graph (nodes, interfaces, interface-endpoint links), canonical serialization (format v2), content-addressed state identity, local content-addressed persistence, explicit transitions, semantic diff, branches with checkout, fail-closed semantic merge, deterministic graph queries (shortest path with lexicographic tie-break, reachability, neighbors, components), connectivity and isolation constraints, deterministic invariant engine, and verification-gated commits. Scaling evidence: [docs/research/stages/v0.2.md](docs/research/stages/v0.2.md).
- **Implemented (v0.3, opt-in):** isolated Linux execution — network namespaces and veth links via iproute2, with structural host-safety guarantees (host-side operations limited to `rahn-*` namespaces), `--execute --yes-i-know` gating, and CI validation. The default remains simulation: `rahn apply` prints the exact commands and touches nothing.
- **Implemented (v0.4):** deterministic observations — structured `Observation` records (counter/gauge/event; exact integer values) with caller-supplied timestamps, positional sequence numbers, append-only storage, `(time_ns, seq)` ordering, and validated state provenance per record; `rahn observe` / `rahn observations`. Ingestion evidence: [docs/research/stages/v0.4.md](docs/research/stages/v0.4.md).
- **Implemented (v0.5):** causal memory — explicit causal edges between immutable anchors (observations, commits) with strict epistemic statuses (`temporal-correlation` / `hypothesis` / `verified` — the last only between commits); DAG enforcement; dangling-anchor rejection; `rahn relate` / `rahn explain` (incidents as connected components). **The system proves nothing about the world: edges are asserted, statuses are honest.** Evidence: [docs/research/stages/v0.5.md](docs/research/stages/v0.5.md).
- **Implemented (v0.6):** peer synchronization over content-addressed history — every replica authoritative for its own history; sync exchanges only immutable hash-verified records; divergence converges through the fail-closed semantic merge, producing **byte-identical merge commits on all replicas**; conflicts fail closed and persist (never auto-resolved). Per-replica read-your-writes only — **no linearizability or strong-consistency claims**. Evidence: [docs/research/stages/v0.6.md](docs/research/stages/v0.6.md).
- **Implemented (v0.7):** execution backend abstraction (ADR 0016) — `ExecutionBackend` trait with capability negotiation and explicit refusal of unsupported capabilities; `simulation` backend (default, description-only, cannot execute); `linux-ns` backend behind the same interface (ADR 0012 semantics unchanged); `rahn apply --backend <name>`. No dataplane code yet — eBPF/XDP are future trait implementations.
- **Implemented (v0.8):** `rahn test [ref]` — deterministic, exit-code-driven verification of any committed state for CI pipelines (ADR 0017); example GitHub Actions gate in [examples/network-ci.yml](examples/network-ci.yml).
- **Implemented (v0.9):** `rahn-sdk` — curated public API facade with doc-tested examples and compile-time API-surface guards; the IR is formally declared (ADR 0018): transition Operations + canonical byte encoding. DSL deliberately deferred.
- **Not implemented (do not assume otherwise):** eBPF/XDP, P4, addressing, traffic control, causal *inference*, automatic edge extraction, distributed consensus/leader election, authentication (replica identity is self-asserted), AI, DSL.
- **Stability (v1.0):** semver via `rahn-sdk` + CLI contracts; canonical format v2 frozen for 1.x; extension model stable (ADR 0019). Research continues within these extension points.
- **Deferred within Stage 2's scope:** typed addressing on interfaces, service objects, constraint-level merge-conflict analysis.

**Current limitations:** no addressing (namespaces have links but no IPs — connectivity is link-existence only), no traffic control, no automatic causal analysis (edges are human-asserted), no cross-replica strong consistency or authentication; execution requires Linux, root, and iproute2, and is CI-validated for a minimal scenario; alpha-grade software with no production use.

## Project

RAHN was created and is initially led by Mohammad Hossein Alikhani ([MHAlikhani](https://github.com/MHAlikhani)). Governance is designed for the project to grow beyond its founder — see [GOVERNANCE.md](GOVERNANCE.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Security-relevant reporting: [SECURITY.md](SECURITY.md). Governance: [GOVERNANCE.md](GOVERNANCE.md).

## License

RAHN source code is licensed under the [Apache License 2.0](LICENSE).

RAHN documentation and research materials are licensed under [CC BY 4.0](LICENSES/CC-BY-4.0.txt) unless otherwise stated.

Third-party components remain under their respective licenses. See [docs/licensing.md](docs/licensing.md) for the full policy and [docs/third-party.md](docs/third-party.md) for the third-party inventory.
