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

## Stage 3 — RAHN 0.3: Linux execution prototype *(implemented in v0.3.0-alpha; see docs/research/stages/v0.3.md)*

First point where RAHN touches real networking: isolated topologies inside Linux network namespaces (veth, netlink, routing tables, nftables, traffic control).

Success: RAHN can create and manipulate isolated Linux network topologies. **Achieved for the link-level model** (namespaces, veth, host-safety tests, CI-validated); addressing and traffic control remain future work.

## Stage 4 — RAHN 0.4: observability *(implemented in v0.4.0-alpha; see docs/research/stages/v0.4.md)*

Network observations, events, health, latency, packet statistics, transition provenance. **Achieved:** observations are associated with states by validated provenance (state id per record). Causal interpretation is Stage 5's causal-memory model (asserted edges, no inference).

## Stage 5 — RAHN 0.5: causal memory *(implemented in v0.5.0-alpha; see docs/research/stages/v0.5.md)*

**Achieved:** causal edges between observations and commits with strict epistemic statuses (temporal-correlation / hypothesis / verified-with-Commits-only); DAG enforcement; `rahn relate` / `rahn explain`; incidents as connected components. Historical replay and narrative `explain <incident>` remain future work within/beyond this stage.

## Stage 6 — RAHN 0.6: distributed state *(implemented in v0.6.0-alpha; see docs/research/stages/v0.6.md)*

Peer sync over content-addressed history (ADR 0015): every replica authoritative for its own history; convergence only via fail-closed semantic merge producing byte-identical merge commits; conflicts persist, never auto-resolved. No leader/epoch (deliberate non-concept); no linearizability claims.

## Stage 7 — RAHN 0.7: execution backends *(implemented in v0.7.0-alpha; see docs/research/stages/v0.7.md)*

Adapters for Linux, namespaces, eBPF, XDP, selected programmable dataplanes. **Achieved:** ExecutionBackend trait (ADR 0016) with simulation (default, Describe-only) and linux-ns backends; capability negotiation with explicit refusal; no dataplane code yet.

## Stage 8 — RAHN 0.8: network CI / verification ecosystem *(implemented in v0.8.0-alpha; see docs/research/stages/v0.8.md)*

`rahn test`, `rahn verify`, `rahn simulate`, `rahn replay`; CI integration (GitHub Actions, GitLab, local). **Achieved:** `rahn test [ref]` — deterministic TSV verification with exit-code contract (ADR 0017); example CI workflow. Reachability-class checks grow with addressing.

Success: network changes participate in CI and verification workflows.

## Stage 9 — RAHN 0.9: programmability *(implemented in v0.9.0-alpha; see docs/research/stages/v0.9.md)*

RAHN API, SDK, formalized IR, policy language, stronger execution planning; begin evaluating DSL design.

## Stage 10 — RAHN 1.0 *(released as v1.0.0; see docs/research/v1.0-review.md)*

The final gate was evaluated in docs/research/v1.0-review.md against the charter requirements; verdict: met, with limitations carried forward explicitly (no addressing/tc, recorded performance debts, CI-scoped execution validation, no signed transitions, single-machine measurements, unverified binary reproducibility).

Success: RAHN provides a coherent, documented abstraction for representing and safely evolving network state.

## Release honesty

The current release is **`v1.0.0`** — the first stable release. Releases so far: `v0.1.0-alpha.1`, `v0.1.0-alpha.2` (hardening), `v0.2.0-alpha` (network graph), `v0.3.0-alpha` (isolated Linux execution), `v0.4.0-alpha` (observability), `v0.5.0-alpha` (causal memory), `v0.6.0-alpha` (distributed state), `v0.7.0-alpha` (backend abstraction), `v0.8.0-alpha` (CI verification), `v0.9.0-alpha` (programmability), `v1.0.0` (stable architecture). No release may be presented as a production network controller or a replacement for SDN, IBN, network OSes, or digital twins. Post-1.0 development continues inside the stable extension model (ADR 0019): addressing, traffic control, dataplane backends, signed transitions, and log replication are candidate directions requiring their own ADRs. The white paper matures with the evidence.
