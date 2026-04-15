<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# ADR 0001: Project Scope — Architectural Research First

- Status: Accepted
- Date: 2026-04-15
- Deciders: project founding (charter §1–§4, §15–§17)

## Context

RAHN could be built as many things: a networking utility, an SDN controller, a VPN, a configuration version store, an AI wrapper. Each shape is achievable quickly and each would corrupt the core idea: networks as evolving, verifiable computational state.

## Decision

RAHN is developed as an **architectural research project first, a systems implementation second, and an ecosystem third.** v0.1 is the *smallest credible implementation of versioned, verifiable network state* — a conceptual validation, explicitly not a production network controller. v0.1 implements only: core state model (Node/Link/Network/metadata), state identity, local persistence, explicit transitions (add/remove node/link), semantic diff, branches, conservative merge, a minimal deterministic invariant engine, deterministic verification, simulation-only execution, a small CLI, and extensive tests.

Explicit v0.1 non-goals (charter §17): no packet forwarding, router configuration, eBPF/XDP/P4, cloud/Kubernetes integration, AI, distributed consensus, cluster mode, QUIC control plane, GUI/web/mobile, remote execution, auto-optimization/remediation, advanced path computation.

## Consequences

- The project cannot demonstrate "real networking" value until Stage 3; this is accepted.
- Scope creep is the primary risk; every feature proposal is gated by "does this strengthen verifiable, evolving network state?"
- Simulation-only execution means v0.1 cannot cause harm to real networks — a security and trust advantage.

## Alternatives considered

- **SDN-controller-first**: faster to demo, but execution-first designs accumulate weak state semantics that are costly to retrofit.
- **Config-store-first ("Git for networks")**: textual merge cannot express semantic conflicts; reduces the project to a UX over diffs.
- **Simulator-first**: simulation is a lifecycle stage, not the model.

## References

- ARCHITECTURE.md §6; ROADMAP.md Stages 0–1; docs/research/problem-statement.md
