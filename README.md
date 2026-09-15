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
[![Release](https://img.shields.io/badge/release-v1.0.0-informational)](https://github.com/MHAlikhani/rahn/releases)
[![CI](https://github.com/MHAlikhani/rahn/actions/workflows/ci.yml/badge.svg)](https://github.com/MHAlikhani/rahn/actions/workflows/ci.yml)

*راه — rah: a path, a way of reaching somewhere.*

</div>

---

## The problem: a network's state has no home

Ask a network engineer three questions:

- **What exactly is in the network right now** — every node, interface, and link, not what a config file *wants* to be there?
- **What changed since the last incident** — precisely, not "someone edited something on Tuesday"?
- **Can I prove this change is safe before applying it** — with a check that runs in CI, not a review checklist?

Today, no tool answers these directly. Configuration management pushes *desired* config and hopes devices comply. Monitoring observes *symptoms* after the fact. The actual state — the thing that breaks — lives in device memory, spreadsheets, and people's heads. When something fails, the answer to "what changed?" is reconstructed by hand from ticket threads and terminal history.

The deeper issue: **network state is never treated as data**. You cannot hold it, hash it, diff it, branch it, or hand it to a machine for verification — the way you can with source code or a database.

## The approach: state as a value

RAHN's proposal is that network state should be a **first-class computational object**, like a commit in version control — but built for the specific semantics of networks, not reused from them:

1. **Model it exactly.** State is a typed object graph — nodes, interfaces, links — serialized canonically, so the same state is byte-identical everywhere.
2. **Give it an identity.** A state's id is the SHA-256 of its canonical bytes. Two states with the same id are the same state — checkable, shareable, undeniable.
3. **Change it only on purpose.** State never mutates. Changes are explicit transition operations; every commit records exactly what was applied.
4. **Prove it before it happens.** A *constitution* of invariants gates every commit; simulation precedes any real execution; real execution is opt-in and isolated.
5. **Remember why.** Observations and causal relationships are recorded next to the states they explain — with honest labels, not guesses.

If you know Git: RAHN borrows the content-addressed discipline, then rebuilds change control for a domain where state has topology semantics and changes have physical consequences. [RAHN vs Git →](https://mhalikhani.github.io/rahn/rahn-vs-git/)

## Who this is for

- **Network engineers** who want "what changed?" to be a command, not an investigation — and who are willing to be explicit in exchange for being able to prove things.
- **Platform/automation engineers** building the next generation of network tooling, who need a verifiable state substrate rather than another templating layer.
- **Researchers** in verification, formal methods, and distributed systems: RAHN is a concrete, runnable artifact with open research questions (measurement studies included) and a documented [research program](docs/research/research-questions.md).
- **Rust developers** who enjoy small, dependency-light systems code with architecture-level test discipline.

RAHN is a **research project at v1.0 architectural stability** — not a production controller. It is the right tool today for learning, research, prototyping, and contributing; it is not the right tool for running a production network.

## Concrete use cases

Each of these works with the v1.0 implementation today, simulation-first:

**1. Network CI — block bad changes before they merge.**
Declare the intended topology, write the rules it must satisfy (`require-connectivity web db`, `prohibit-connectivity hr finance`), and let `rahn test` gate the change in CI with an exit-code contract. Verified means "all encoded invariants hold" — nothing more, and RAHN never claims otherwise.

```console
$ rahn test
state 3892a88a…4c0a5
PASS    referential-integrity
PASS    link-endpoints-exist
PASS    no-duplicate-links
PASS    no-self-loops
PASS    require-connectivity(web,db)
test PASSED (5 invariants)
```

**2. Incident forensics — "what changed?" as a diff, not an investigation.**
When something breaks, diff the state before and after the incident window and read the exact transitions that were applied. `rahn explain` traces causal chains between observations and commits — with epistemic labels (`temporal-correlation`, `hypothesis`, `verified`) so you always know how strong a claim is.

**3. Safe experimentation — branch the network, not the config.**
Branch `incident-42`, apply a proposed fix in simulation, and compare outcomes. The simulation backend renders an execution plan and touches nothing. Merging back is semantic and fail-closed: conflicting changes are explained and rejected, never silently resolved.

**4. Verifiable replicas — sync state you can check.**
Peers synchronize over content-addressed history; every replica independently verifies what it receives by re-hashing. Divergence fails closed. No trust in the sender required.

**5. A runnable research artifact.**
Measure verification latency across scaling regimes, study merge conflict rates, test how expressive the invariant language is against real-world invariants — the experiments are specified, several are open, and the methodology is written down before the numbers exist.

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

Every state change is verified before it can be committed; `rahn apply` renders an execution plan and touches nothing. See [examples/cli-walkthrough.md](examples/cli-walkthrough.md) for the full session, [examples/network-ci.yml](examples/network-ci.yml) to gate changes in CI, and `bash examples/demo/demo.sh` for a runnable end-to-end demo ([examples/demo/README.md](examples/demo/README.md)).

For external code, build against [`rahn-sdk`](crates/rahn-sdk) — the curated, doctest-covered public API.

## What RAHN models

| | |
|---|---|
| **State** | Deterministic, content-addressed network topology — nodes, interfaces, links — with canonical serialization and stable identity across machines and time |
| **Transitions** | A minimal vocabulary of pure, total operations — the only way committed state changes; every commit records exactly what was applied |
| **Constitution** | Architectural invariants (`require-connectivity`, `prohibit-connectivity`, structural integrity) that gate every candidate state — on every branch |
| **Evolution** | Semantic diff, branches with checkout, fail-closed three-way merge that produces byte-identical merge commits on all replicas |
| **Memory** | Provenance-bearing observation records and asserted, status-labeled causal edges — `temporal-correlation`, `hypothesis`, `verified` |
| **Execution** | Dependency-safe execution plans; simulation by default; opt-in isolated Linux namespace backend |

Determinism is architectural: the same command sequence produces byte-identical states, identities, and logs — on any machine, at any time.

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

## How it was built — v1.0.0, stage by stage

| Stage | Release | Delivered |
|---|---|---|
| 0–1 | `rahn-v0.1.0-alpha` | State engine: canonical serialization, content-addressed identity, transitions, persistence |
| — | `rahn-v0.1.0-alpha.2` | Hardening: checkout, property tests, CI, licensing |
| 2 | `rahn-v0.2.0-alpha` | Network graph: interfaces, path discovery, isolation constraints, scaling evidence to 100k objects |
| 3 | `rahn-v0.3.0-alpha` | Isolated Linux execution: namespaces + veth, explicit opt-in, structural host-safety |
| 4 | `rahn-v0.4.0-alpha` | Observability: provenance-bearing observation records |
| 5 | `rahn-v0.5.0-alpha` | Causal memory: status-labeled edges, DAG enforcement — asserted, never inferred |
| 6 | `rahn-v0.6.0-alpha` | Distributed state: peer sync with byte-identical convergence, fail-closed divergence |
| 7 | `rahn-v0.7.0-alpha` | Execution backends: `ExecutionBackend` trait, capability negotiation |
| 8 | `rahn-v0.8.0-alpha` | Network CI: `rahn test` with an exit-code contract |
| 9 | `rahn-v0.9.0-alpha` | Programmability: `rahn-sdk`, declared IR — DSL deliberately deferred |
| 10 | **`rahn-v1.0.0`** | **Stable architecture: semver, frozen IR format, stable extension model** |

## Limitations and non-goals

RAHN does not yet implement addressing, traffic control, causal inference, authentication, or cross-replica strong consistency. Verification covers only encoded invariants; conflicts fail closed rather than auto-resolving; all measurements are single-machine. The full, honest list: [docs/research/limitations.md](docs/research/limitations.md).

RAHN is not a router, VPN, SDN controller, monitoring dashboard, packet analyzer, IaC tool, or "Git for networks" — and it adds no AI dependency: the system is complete without it.

## Documentation

| | |
|---|---|
| [ARCHITECTURE.md](ARCHITECTURE.md) | The architecture: goals, trade-offs, failure modes |
| [docs/concepts.md](docs/concepts.md) | Core concepts in depth · [concept pages](https://mhalikhani.github.io/rahn/concepts/) on the web |
| [docs/adr/](docs/adr/) | 19 Architecture Decision Records |
| [docs/spec/](docs/spec/) | Normative technical specification |
| [docs/research/](docs/research/) | Problem statement, prior art, research questions, experiments · [research overview](https://mhalikhani.github.io/rahn/research/) |
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
