<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Prior Art

Status: living research document (Stage 0). **Rule: never claim novelty without evidence.** This document will be expanded with real citations before the white paper draft claims anything.

This document distinguishes four categories:

1. **Existing work** — systems that already do a specific thing RAHN also wants.
2. **Related work** — systems addressing adjacent problems with transferable ideas.
3. **RAHN-specific ideas** — combinations/emphases we have not yet found elsewhere (provisional; each needs verification).
4. **Unresolved research questions** — where no adequate prior art has been identified yet.

## 1. Existing work

- **Version control (Git, Mercurial)** — immutable content-addressed history, branching, diffing, merging. RAHN borrows the *mechanics* but applies them to semantic network state, not text; network-aware merge is the differentiator.
- **Event sourcing / event-sourced stores** — immutable append-only history, deterministic replay, projections. Directly transferable pattern; RAHN applies it to network state with invariants and causality.
- **Infrastructure as Code (Terraform, Nornir, Batfish-adjacent tooling)** — declarative network/infra definitions, plan/apply cycles, drift detection. Terraform's plan/verify/apply resembles RAHN's candidate→verify→execute; but state is a flat reconciled resource graph with no causal memory, semantic branching, or constitution concept.
- **SDN controllers (OpenDaylight, ONOS, Faucet)** — centralized network control with global topology view. Related; typically execution-first, history-poor.
- **Intent-based networking (per IETF/ONF work on IBN, e.g., RFC 9315 analysis of IBN)** — intent as desired outcome with verification of intent fulfillment. Overlaps with RAHN's intent/verification stages; IBN typically lacks immutable state history, branching, and causal memory.
- **Network modeling/verification (Batfish, Cbf, Minesweeper, Header Space Analysis, VeriFlow, NetKAT)** — formal verification of network *configurations*/data-plane behavior. Strong prior art for RAHN's verification stage; RAHN's contribution would be integrating verification into a versioned, causal state lifecycle rather than into static configs.
- **Digital twins / simulators (GNS3, Containerlab, Mininet, Cisco CML)** — simulate networks. RAHN's simulation stage (v0.1–v0.3) will likely build on similar namespace/container techniques; these tools do not provide versioned semantic state, constitutions, or causal memory.
- **Time-series observability (Prometheus, InfluxDB) and event correlation systems** — answer "what happened"; generally lack linkage to state transitions.

## 2. Related work

- **Distributed versioned state** — CRDTs, version vectors, replicated logs, consensus (Raft/Paxos). Relevant to Stage 6; RAHN must derive its consistency requirements before adopting any of these.
- **Content-addressed storage** — Git object model, CAS systems, IPFS/Bittorrent-style addressing. The natural storage shape for immutable states.
- **Formal methods for systems** — TLA+/PlusCal, P language, model checking of protocols. Candidate methodology for verifying transition semantics.
- **Policy engines (OPA/Rego, Cedar, Kyverno)** — declarative policy evaluation. Candidate technology or pattern for RAHN's constitution engine; must remain deterministic.
- **QUIC, TLS, SCION, P4, eBPF/XDP** — protocol and dataplane ecosystems RAHN should reuse rather than replace.
- **Neuro-symbolic systems research** — neural proposal + symbolic verification pattern; matches RAHN's mandated AI posture.

## 3. RAHN-specific ideas (provisional — require verification against literature)

- **Network constitution** as a named, first-class, evolving collection of architectural invariants governing all valid states.
- **Semantic network merge**: branch merge over network state with constraint-level conflict detection and explanation (e.g., infeasible combined requirements), fail-closed.
- **Causal network memory**: a causal graph whose nodes/edges are linked to state transitions, enabling transition-attributed incident explanation.
- **Network time travel** as a first-class operation over content-addressed network state (not just logs).
- Treating *the network's evolving state itself* — rather than configs, intents, or packets — as the primary computational object across the whole lifecycle.

Each item must be searched against academic literature (SIGCOMM/NSDI/HotNets, formal-methods venues) and industry systems before any novelty claim. If found elsewhere, reclassify into §1/§2 with citation.

## 4. Unresolved research questions

See [research-questions.md](research-questions.md); the ones with no known adequate prior art:

- Can semantic network merge be made complete (all meaningful conflicts detected) or is it necessarily conservative/heuristic?
- What consistency model does distributed RAHN state actually need (Stage 6)?
- Can causal links between observations and transitions be established reliably enough to trust `rahn explain` output, given imperfect telemetry?
- Can execution plans be *proven* safe against a model before deployment, and with what assurance level?
