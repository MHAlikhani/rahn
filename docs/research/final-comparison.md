<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN — Final Prior-Art Comparison (v1.0)

Status: final review at v1.0.0. Builds on the per-system survey in [prior-art.md](prior-art.md); scoped to the sources consulted there plus repository evidence through Stage 9. **No claim below extends beyond that scope.**

## What RAHN is

A stateful execution architecture for evolving networks: deterministic content-addressed network state (interface-based topology), explicit recorded transitions, a constitution of invariants gating every commit, semantic diff/branch/fail-closed merge, deterministic graph queries, simulation-first execution with an opt-in isolated Linux namespace backend, deterministic observation records with provenance, asserted status-labeled causal edges, peer sync with byte-identical convergence, CI verification, and a curated SDK over a declared IR.

## What RAHN is not

Not a router, VPN, SDN controller, monitoring dashboard, packet analyzer, IaC tool, "Git for networks", LLM wrapper, or production platform. No causal inference, no AI, no addressing/traffic control yet, no distributed strong consistency.

## Comparison at v1.0 (validated capabilities only)

| System | RAHN's distinction (evidence: [stages/](stages/)) |
|---|---|
| Git | Same content-addressed mechanics; RAHN's merge is semantic and fail-closed over network constraints, and commits carry verification summaries (ADR 0005/0007). |
| Terraform/IaC | RAHN versions the reconciled state itself with identity + provenance; plan/apply is one lifecycle stage, not the model. |
| SDN controllers | State-semantics-first; execution is a backend behind ADR 0016, simulation by default. |
| IBN (RFC 9315) | State (not intent) is the versioned object; intent appears as the lifecycle entry, assurance as verification + observations. |
| Batfish / HSA / NetKAT | Those verify configs/data planes; RAHN verifies candidate states in a versioned lifecycle — complementary layers. |
| Digital twins / emulators | RAHN's namespace backend (ADR 0012) uses the same OS primitives but execution is gated, recorded, and backend-neutral (ADR 0016). |
| Observability (Prom/OTel) | RAHN observations are state-associated with provenance (`state_ref` per record, ADR 0013); telemetry is not transition-linked. |
| Event sourcing | RAHN keeps both content-addressed states and their generating transitions, with network semantics (constitutions, semantic merge). |
| Raft/CRDTs | RAHN derives requirements first (RQ7): peer sync + fail-closed merge convergence with byte-identical merge commits; no leader, no silent conflict resolution (ADR 0015). |

## Uniqueness statement (scoped)

The combination and emphasis — a full lifecycle of content-addressed identity, branching, semantic fail-closed merging, constitution-gated verification, simulation-first execution, deterministic provenance-bearing observations, asserted causal edges, and peer sync around *network state itself* — was not identified in the surveyed sources. Individual elements have close relatives as documented in prior-art.md. This remains bounded by the survey's scope.

## Validated vs experimental vs future (at v1.0)

- **Implemented + test-enforced:** state identity, canonical serialization, transitions, diff, branch/merge, invariants/constitution, graph queries, backend abstraction, observation records, causal edges, peer sync, CI surface, SDK/IR.
- **Measured (single-machine):** scaling (10–100k objects), observation ingestion, causal-graph ops, sync transport ([stages/](stages/)).
- **Experimental (CI-validated, minimal scenario):** namespace execution plan/observation differential (E6 partial).
- **Proposed/Future:** addressing, traffic control, eBPF/XDP backends, log replication, signed transitions, DSL, causal inference, distributed strong consistency.
