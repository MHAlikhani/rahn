<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Architecture

Status: living document. Reviewed whenever architectural code or core assumptions change.
White paper counterpart: [docs/whitepaper/RAHN-Whitepaper.md](docs/whitepaper/RAHN-Whitepaper.md).
Decisions are recorded in [docs/adr/](docs/adr/); this document summarizes, it does not decide.

## 1. Problem

Modern networking is fragmented across layers and tools — routing protocols, SDN controllers, intent-based networking, cloud networking, observability systems, digital twins, packet processing (eBPF/XDP/P4), VPNs/overlays, policy engines, IaC, telemetry, incident tooling, simulation, and automation. Each solves part of the problem, but the network remains represented as **fragmented views**:

```
configuration + topology + runtime state + telemetry + policy
+ identity + routing + incidents + historical information
```

No single coherent model lets all of these participate in one evolving state. Consequences: changes are hard to verify before execution; history is lost or non-causal; incidents are diagnosed from telemetry alone ("what happened") without a link to state evolution ("what changed, and which transition caused it"); and semantic conflicts between constraints are discovered in production, not at design time.

## 2. Assumptions

1. Network state can be represented as a deterministic, serializable, hashable data structure — at least for the aspects RAHN models (topology, policy, identity, intent, constraints, observations, history references).
2. A meaningful subset of network correctness is expressible as **invariants** — predicates that must hold across all valid states.
3. Most real network changes can be modeled as **explicit transitions** between states, rather than side-effecting scripts.
4. Observation (telemetry, health, events) can be associated with states and transitions with sufficient fidelity to be useful, even if the association is imperfect.
5. Determinism is achievable and worth paying for; simulation fidelity is achievable without touching a real network.
6. The first deployments will be local, single-writer, and simulation-only; distributed operation arrives later and must not contaminate early state semantics.

Assumptions are revisited as evidence accumulates; an invalidated assumption triggers an ADR.

## 3. Design goals

- **Determinism first.** Same input state + same transition → same output state, same identity, same verification result.
- **Explicit state transitions** as the primary primitive — not configuration.
- **Immutable historical state** where practical; states are content-addressed and never mutated in place.
- **Verify before execute.** A candidate state that violates the constitution is rejected before any execution.
- **No network side effects by default.** The safest mode is read-only / simulated.
- **No AI in the core.** Intelligence may propose; the architecture must verify. RAHN remains fully useful without AI.
- **Explainability.** Failures, diffs, verification results, and (eventually) path decisions must be explainable, not just Boolean.
- **Separation of planes.** Representation from execution; control plane from execution plane; observation from decision.

## 4. Non-goals

Not a router, VPN, SDN controller, monitoring dashboard, packet analyzer, generic simulator, IaC tool, or configuration version store. Not a "ChatGPT for networking". No blockchain. No premature distributed consensus, cloud/Kubernetes integration, DSL, custom transport protocol, GUI, or AI dependency. (Full list in the project charter, §3 and §17.)

## 5. Alternatives considered

- **Git-like configuration version control.** Rejected as the core model: textual diff/merge cannot express semantic conflicts (e.g., branch A adds `latency <= 10ms`, branch B requires post-quantum encryption, and no available path satisfies both). Version control concepts are *inspired by* Git but applied to semantic state, not text.
- **SDN controller / intent-based networking platform.** Rejected as the identity of the project: these assume execution-first designs with weak or no state history, causality, or verification of evolution. RAHN may interoperate with them.
- **Digital twin / simulator.** Rejected as the core: simulation is one lifecycle stage in RAHN (candidate → verify → execute → observe), not the whole model.
- **Infrastructure-as-code + CI.** Rejected: IaC versions *declarations*, not reconciled evolving state with provenance and causality.
- **Event sourcing / event-sourced databases as the state model.** Closest existing-pattern relative; RAHN adopts projection-from-history ideas but centers *network state* (topology, policy, causality) rather than application entities. See [docs/research/prior-art.md](docs/research/prior-art.md).

## 6. Chosen design

### Core lifecycle

```
Intent → State → Constraints → Candidate Transition → Verification
       → Execution → Real Network → Observation → Causal Memory → New State
```

### State model

State is a first-class value, eventually shaped roughly as:

```
State_t = {
    topology, policy, identity, intent,
    observations, constraints, history_reference
}
```

Every state is inspectable, serializable, comparable, hashable, reproducible, verifiable, versionable, and replayable. State identity is derived from **canonical serialization + content hashing** (scheme internal in v0.1; see [docs/adr/0002-state-identity.md](docs/adr/0002-state-identity.md) and [docs/adr/0003-canonical-serialization.md](docs/adr/0003-canonical-serialization.md)). The model is deliberately not frozen; it is an evolving research artifact.

### Transitions

```
State A + Intent/Operation + Constraints + Preconditions → Candidate State B
Verify(B) ? Execute(B) : Reject(B)
```

The architecture makes unsafe transitions difficult to *express*: operations (add/remove node/link in v0.1) always produce a new state; nothing mutates a live state.

### Constitution

A collection of architectural invariants (e.g., "databases must never be publicly reachable", "control-plane connectivity must be preserved") checked against every candidate state. A violating candidate is rejected. Distinct from ordinary configuration directives.

### Causal memory

A causal graph connecting observations to state transitions, enabling "which transition introduced this change?" reasoning — not merely a metrics graph.

### Evolution, branching, merging

Network evolution is a graph of states (S0 → S1 → S2 …), branchable. Branches are lightweight references to states. Merging is **network-aware**: semantic conflicts (contradictory constraints, infeasible combinations) are detected and explained; v0.1 fails on conflict rather than auto-resolving.

### Implementation shape (v0.3)

Rust workspace with small crates: `rahn-core`, `rahn-state`, `rahn-store`, `rahn-verify`, `rahn-cli`, `rahn-sim`, `rahn-exec`. Persistence: filesystem + canonical serialized objects (format v2). The topology model is interface-based (ADR 0011): links connect `(node, interface)` endpoints. Deterministic graph queries (shortest path with lexicographic tie-break, reachability, components) and isolation constraints (`prohibit-connectivity`) are implemented. Execution: simulation-first — `apply` prints the exact iproute2 commands; real execution is opt-in (`--execute --yes-i-know`) against isolated Linux network namespaces + veth (ADR 0012), with host-side operations restricted to `rahn-*` namespaces and CI validation. No addressing yet, so connectivity is link-existence only. (See [docs/adr/0004-storage-model.md](docs/adr/0004-storage-model.md), [docs/adr/0008-execution-boundary.md](docs/adr/0008-execution-boundary.md).)

## 7. Trade-offs

- **Determinism over performance.** Canonical serialization and content hashing cost CPU; accepted because identity, replay, and verification depend on it.
- **Explicitness over convenience.** Every change is a named, recorded transition; accepted because implicit changes destroy explainability.
- **Conservatism over automation.** Merges fail closed on semantic conflict; accepted because wrong automatic merges are worse than manual resolution.
- **Immutability over mutation.** Storage grows with history; accepted because rollback, replay, and provenance require it. Pruning strategies come later, deliberately.
- **Simulation-first over execution-first.** v0.1 touches no real network; accepted to protect the state model from execution concerns until it is stable.
- **No AI in core over richer UX.** Accepted for determinism, security, reproducibility, and formal reasonability.

## 8. Failure modes

- **Corrupted state store.** Content-addressed storage makes corruption detectable (hash mismatch); the system must fail loudly, refuse to load, and never silently "repair". Tests cover this.
- **Invalid transitions / malicious state input.** Parsing is total and strict; malformed input produces explicit errors, never partial states.
- **Incompleteness of the state model.** Real networks contain state RAHN does not yet model (e.g., ARP caches, TCAM state). The model is explicitly partial; observation-based reconciliation arrives in later stages. Risk: divergence between modeled and real state — mitigated by never executing against real networks before the model can perceive it.
- **Semantic merge misses.** A merge could pass syntactic checks while violating a cross-branch semantic constraint. v0.1 mitigates by failing closed on any conflict class it cannot prove safe.
- **Verification gaps.** An invariant engine proves only what it encodes; "verified" never means "safe" beyond the encoded constitution. Documented wherever claimed.
- **History bloat.** Immutable history grows unboundedly; addressed by design (content dedup) before pruning.

## 9. Future evolution

The staged roadmap ([ROADMAP.md](ROADMAP.md)) extends the same core: richer topology and services (0.2), Linux-namespace execution prototype (0.3), observations and provenance (0.4), causal memory and replay (0.5), distributed state with an *evidence-driven* consistency model (0.6), execution backends behind a stable abstraction (0.7), network CI (0.8), programmability/IR/DSL evaluation (0.9), and a 1.0 only when state semantics, execution trustworthiness, and the security model are mature. Each architectural change follows the ADR → ARCHITECTURE.md → white paper update sequence.
