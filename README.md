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

**v0.2.0-alpha — Stage 2: the network graph model.**

An experimental, deterministic state engine for evolving network topologies: interfaces and interface-addressed links, canonical serialization (format v2), content-addressed state identity, local persistence, explicit transitions, semantic diff, branches, fail-closed semantic merge, deterministic path discovery, isolation constraints, a deterministic invariant engine, verification-gated commits, and simulation-only execution. **No real execution exists** — `rahn apply` prints an execution plan and touches nothing.

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
- [docs/whitepaper/RAHN-Whitepaper.md](docs/whitepaper/RAHN-Whitepaper.md) — white paper (outline; evolves with the architecture)
- [ROADMAP.md](ROADMAP.md) — research roadmap toward v1.0

## Implementation status

The primary implementation language is **Rust**. Rust is an implementation choice, not the identity of the architecture.

- **Implemented (v0.2.0-alpha):** deterministic interface-based network graph (nodes, interfaces, interface-endpoint links), canonical serialization (format v2), content-addressed state identity, local content-addressed persistence, explicit transitions, semantic diff, branches with checkout, fail-closed semantic merge, deterministic graph queries (shortest path with lexicographic tie-break, reachability, neighbors, components), connectivity and isolation constraints, deterministic invariant engine, verification-gated commits, and simulation-only execution planning. Scaling evidence: [docs/research/stages/v0.2.md](docs/research/stages/v0.2.md).
- **Not implemented (do not assume otherwise):** real execution of any kind — no Linux namespaces, no eBPF/XDP, no P4; no observability; no causal memory; no distributed state; no AI; no DSL. `rahn apply` prints an execution plan and touches nothing.
- **Next (Stage 3 / v0.3):** isolated Linux execution — network namespaces, with explicit opt-in and host-mutation safety tests (see [ROADMAP.md](ROADMAP.md)).
- **Deferred within Stage 2's scope:** typed addressing on interfaces, service objects, constraint-level merge-conflict analysis.

## Project

RAHN was created and is initially led by Mohammad Hossein Alikhani ([MHAlikhani](https://github.com/MHAlikhani)). Governance is designed for the project to grow beyond its founder — see [GOVERNANCE.md](GOVERNANCE.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Security-relevant reporting: [SECURITY.md](SECURITY.md). Governance: [GOVERNANCE.md](GOVERNANCE.md).

## License

RAHN source code is licensed under the [Apache License 2.0](LICENSE).

RAHN documentation and research materials are licensed under [CC BY 4.0](LICENSES/CC-BY-4.0.txt) unless otherwise stated.

Third-party components remain under their respective licenses. See [docs/licensing.md](docs/licensing.md) for the full policy and [docs/third-party.md](docs/third-party.md) for the third-party inventory.
