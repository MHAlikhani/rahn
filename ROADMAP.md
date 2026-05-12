<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Roadmap

Status: a **research roadmap**, not a fixed promise. Stages are ordered by dependency: do not skip stages; do not build the ecosystem before proving the primitive.

Every architectural change follows: create/update ADR → update ARCHITECTURE.md → update white paper → document the reason.

## Stage 0 — Research / Foundation (current)

Prior-art analysis, terminology, architecture specification, invariants, threat model, state model, open questions, repository bootstrap. Deliverables: ARCHITECTURE.md, DESIGN.md, research corpus, ROADMAP.md, ADRs. No large implementation.

## Stage 1 — RAHN 0.1: smallest credible state engine

Deterministic network state; state identity; local persistence; explicit transitions; semantic diff; branches; conservative merges; basic constitution/invariant engine; deterministic verification; **simulation-only execution**; clean CLI; extensive tests.

Success: a developer can create a network, modify it, commit state, branch it, compare it, validate it, simulate a transition, and inspect history — without touching a production network.

## Stage 2 — RAHN 0.2: richer model *(implemented in v0.2.0-alpha; see docs/research/stages/v0.2.md)*

Interfaces as first-class objects (links connect `node/interface` endpoints); deterministic path discovery and graph queries; isolation constraints (`prohibit-connectivity`); fail-closed semantic merge extended to the interface model; scaling benchmarks to 100k objects.

Remaining for this stage: typed addressing on interfaces; service objects; stronger constraint engine (constraint-level merge-conflict analysis, E3).

## Stage 3 — RAHN 0.3: Linux execution prototype

First point where RAHN touches real networking: isolated topologies inside Linux network namespaces (veth, netlink, routing tables, nftables, traffic control).

Success: RAHN can create and manipulate isolated Linux network topologies.

## Stage 4 — RAHN 0.4: observability

Network observations, events, health, latency, packet statistics, transition provenance. Begin connecting state ↔ observation.

## Stage 5 — RAHN 0.5: causal memory

Causal events and edges; state-linked incidents; historical replay; incident reconstruction (`rahn explain`). Potential research milestone.

Success: RAHN can connect network observations and historical state transitions.

## Stage 6 — RAHN 0.6: distributed state

Replicated state, consistency model, node identity, synchronization, conflict handling. **Do not** adopt Raft (or any consensus) by familiarity — first identify the actual consistency requirements.

## Stage 7 — RAHN 0.7: execution backends

Adapters for Linux, namespaces, eBPF, XDP, selected programmable dataplanes. Abstract network state stays separated from backend execution.

## Stage 8 — RAHN 0.8: network CI / verification ecosystem

`rahn test`, `rahn verify`, `rahn simulate`, `rahn replay`; CI integration (GitHub Actions, GitLab, local). Goal: a network change is testable before deployment.

Success: network changes participate in CI and verification workflows.

## Stage 9 — RAHN 0.9: programmability

RAHN API, SDK, formalized IR, policy language, stronger execution planning; begin evaluating DSL design.

## Stage 10 — RAHN 1.0

Only when: state semantics are stable; APIs are documented; the execution model is trustworthy; the security model is credible; reproducibility is strong; tests are comprehensive; documentation is mature; ecosystem boundaries are clear.

Success: RAHN provides a coherent, documented abstraction for representing and safely evolving network state.

## Release honesty

The current release is **`v0.2.0-alpha`** (network graph foundation; interfaces, deterministic path discovery, isolation constraints, simulation-only execution). Releases so far: `v0.1.0-alpha.1` (*"an experimental state engine for evolving network topologies"*), `v0.1.0-alpha.2` (hardening), `v0.2.0-alpha`. No release may be presented as a production network controller or a replacement for SDN, IBN, network OSes, or digital twins. **Next: Stage 3 — isolated Linux execution (v0.3).** The white paper matures with the evidence.
