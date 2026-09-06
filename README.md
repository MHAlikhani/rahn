<!-- SPDX-License-Identifier: CC-BY-4.0 -->

<div align="center">

# RAHN

**A stateful execution architecture for evolving networks.**

*"RAHN gives networks state, memory, causality, and verifiable evolution."*

<p>
  <img src="docs/assets/rahn-hero.png" alt="RAHN — a stateful execution architecture for evolving networks" width="720" />
</p>

[![License: Apache-2.0](https://img.shields.io/badge/source-Apache--2.0-blue)](LICENSE)
[![Docs: CC BY 4.0](https://img.shields.io/badge/docs-CC_BY_4.0-lightgrey)](LICENSES/CC-BY-4.0.txt)
[![Release](https://img.shields.io/badge/release-v1.0.0-informational)](https://github.com/MHAlikhani/rahn/releases/tag/v1.0.0)
[![CI](https://github.com/MHAlikhani/rahn/actions/workflows/ci.yml/badge.svg)](https://github.com/MHAlikhani/rahn/actions/workflows/ci.yml)

*راه — rah: a path, a way of reaching somewhere.*

</div>

---

## The research question

> Can network state become a first-class computational object — versioned, reasoned about, verified, replayed, branched, simulated, evolved, and safely executed?

RAHN answers the first half of that question with a working v1.0 architecture. Networks are modeled not as devices and configurations but as an **evolving computational system**: every change is an explicit transition, every state has a deterministic identity, and every commit passes a constitution of invariants before it can exist.

**RAHN is an architectural research project** — architecture first, systems implementation second, ecosystem third. The architecture, the normative specification, the research record, and the Rust implementation are all in this repository.

## What RAHN models

| | |
|---|---|
| **State** | Deterministic, content-addressed network topology — nodes, interfaces, links — with canonical serialization and stable identity across machines and time |
| **Transitions** | A minimal vocabulary of pure, total operations — the only way committed state changes; every commit records exactly what was applied |
| **Constitution** | Architectural invariants (`require-connectivity`, `prohibit-connectivity`, structural integrity) that gate every candidate state — on every branch |
| **Evolution** | Semantic diff, branches with checkout, fail-closed three-way merge that produces byte-identical merge commits on all replicas |
| **Memory** | Provenance-bearing observation records and asserted, status-labeled causal edges — `temporal-correlation`, `hypothesis`, `verified` |
| **Execution** | Dependency-safe execution plans; simulation by default; opt-in isolated Linux namespace backend |

## The lifecycle

```
        intent
          │
          ▼
   ┌─────────────┐     constraints     ┌──────────────┐
   │    state     │───────────────────▶│ verification │
   │  (content-   │                    └──────┬───────┘
   │   addressed) │                           │ pass
   └──────▲──────┘                           ▼
          │                            execution plan
     causal memory                          │
          │                                 ▼
   ┌──────┴───────┐                    observation
   │  transitions │◀───────────────────────┘
   └──────────────┘
```

Determinism is architectural: the same command sequence produces byte-identical states, identities, and logs — on any machine, at any time.

## v1.0.0 — what was built, stage by stage

| Stage | Release | Delivered |
|---|---|---|
| 0–1 | `v0.1.0-alpha` | State engine: canonical serialization, content-addressed identity, transitions, persistence |
| — | `v0.1.0-alpha.2` | Hardening: checkout, property tests, CI, licensing |
| 2 | `v0.2.0-alpha` | Network graph: interfaces, path discovery, isolation constraints, scaling evidence to 100k objects |
| 3 | `v0.3.0-alpha` | Isolated Linux execution: namespaces + veth, explicit opt-in, structural host-safety |
| 4 | `v0.4.0-alpha` | Observability: provenance-bearing observation records |
| 5 | `v0.5.0-alpha` | Causal memory: status-labeled edges, DAG enforcement — asserted, never inferred |
| 6 | `v0.6.0-alpha` | Distributed state: peer sync with byte-identical convergence, fail-closed divergence |
| 7 | `v0.7.0-alpha` | Execution backends: `ExecutionBackend` trait, capability negotiation |
| 8 | `v0.8.0-alpha` | Network CI: `rahn test` with an exit-code contract |
| 9 | `v0.9.0-alpha` | Programmability: `rahn-sdk`, declared IR — DSL deliberately deferred |
| 10 | **`v1.0.0`** | **Stable architecture: semver, frozen IR format, stable extension model** |

**Stable v1.0 architecture — not a production network controller.**

## Limitations and non-goals

RAHN does not yet implement addressing, traffic control, causal inference, authentication, or cross-replica strong consistency. Verification covers only encoded invariants; conflicts fail closed rather than auto-resolving; all measurements are single-machine. The full, honest list: [docs/research/limitations.md](docs/research/limitations.md).

RAHN is not a router, VPN, SDN controller, monitoring dashboard, packet analyzer, IaC tool, or "Git for networks" — and it adds no AI dependency: the system is complete without it.

## Quick start

```console
$ cargo install --path crates/rahn-cli   # or: cargo build --release
$ rahn init
$ rahn node add web
$ rahn node add db
$ rahn interface add web eth0
$ rahn interface add db eth0
$ rahn link add web/eth0 db/eth0
$ rahn test
state 3892a88a…4c0a5
PASS    referential-integrity
PASS    link-endpoints-exist
PASS    no-duplicate-links
PASS    no-self-loops
test PASSED (4 invariants)
$ rahn commit -m "web — db topology"
```

Every state change is verified before it can be committed; `rahn apply` renders an execution plan and touches nothing. See [examples/cli-walkthrough.md](examples/cli-walkthrough.md) for the full session, and [examples/network-ci.yml](examples/network-ci.yml) to gate changes in CI. For a runnable version of the same workflow, run `bash examples/demo/demo.sh` — see [examples/demo/README.md](examples/demo/README.md).

For external code, build against [`rahn-sdk`](crates/rahn-sdk) — the curated, doctest-covered public API.

## Documentation

| | |
|---|---|
| [ARCHITECTURE.md](ARCHITECTURE.md) | The architecture: goals, trade-offs, failure modes |
| [docs/concepts.md](docs/concepts.md) | Core concepts in depth |
| [docs/adr/](docs/adr/) | 19 Architecture Decision Records |
| [docs/spec/](docs/spec/) | Normative technical specification |
| [docs/research/](docs/research/) | Problem statement, prior art, research questions, experiments |
| [docs/whitepaper/RAHN-Whitepaper.md](docs/whitepaper/RAHN-Whitepaper.md) | Technical white paper (claim-status-marked) |
| [ROADMAP.md](ROADMAP.md) | Completed stages and post-v1.0 extensions |
| [CONTRIBUTOR_PATH.md](CONTRIBUTOR_PATH.md) | How a contributor grows, from a first issue to maintainer |
| [DEVELOPMENT.md](DEVELOPMENT.md) | Workspace layout, commands, and pull-request checks |
| [docs/community/](docs/community/) | Discussion categories, curated first issues, and planned issues |
| [AGENTS.md](AGENTS.md) | Instructions for coding agents; [llms.txt](llms.txt) indexes these docs for AI tools |

## Post-v1.0 direction

Stable, independently scoped extensions — each requiring its own ADR, specification, and research evidence: addressing, traffic control, signed transitions and authenticated replicas, replicated logs, further dataplane backends, and a DSL if the IR ever proves insufficient as an authoring surface. Nothing is scheduled; nothing is promised.

## Project

RAHN was created and is initially led by Mohammad Hossein Alikhani ([MHAlikhani](https://github.com/MHAlikhani)); governance is designed for the project to grow beyond its founder — [GOVERNANCE.md](GOVERNANCE.md).

Contributions follow [CONTRIBUTING.md](CONTRIBUTING.md) (RFC path for architectural changes); security reports per [SECURITY.md](SECURITY.md). To cite RAHN in research, use [CITATION.cff](CITATION.cff).

## License

Source code: [Apache License 2.0](LICENSE) · Documentation and research materials: [CC BY 4.0](LICENSES/CC-BY-4.0.txt)
