<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN: A Stateful Execution Architecture for Evolving Networks

**White Paper v1.0 — Stable Release Document**
Date: 2026-09-06. Repository: https://github.com/MHAlikhani/rahn
Versioned independently of the codebase; revisions correspond to meaningful architectural changes. Version history at the end.

**Licensing:** RAHN software is licensed under Apache License 2.0. RAHN documentation and research materials (including this paper) are licensed under CC BY 4.0 unless otherwise stated. The paper's license does not change the software license.

**Claim-status legend (mandatory).** Every substantive statement below is marked:

- **[Implemented]** — exists in the v0.1 code with tests (commit `bb38634` lineage; see docs/testing.md).
- **[Experimental]** — measured in a documented experiment; methodology and data available.
- **[Proposed]** — designed, not yet built.
- **[Future]** — research direction, no design yet.

First measurement-kind evidence exists as of the v0.3 revision of this paper (Stage 2 scaling benchmarks, §27); it is scoped single-machine baseline measurement, not a performance guarantee. Determinism claims remain test-kind evidence (fixed and property-based tests). Sections state their evidence class explicitly and will remain honest as evidence accumulates.

---

## 1. Abstract

Modern network operation fragments a network across configuration, topology, telemetry, policy, and incident views, none of which carry a shared notion of state, identity, or causality. This paper presents RAHN, an architecture that treats *the network's evolving state itself* as a first-class computational object: deterministic and content-addressed, mutated only by explicit transitions, gated by a constitution of invariants before any execution, branchable and semantically mergeable, and recorded as an immutable history intended to support future causal reasoning and replay. We describe the conceptual model, its v0.1 implementation in Rust (canonical serialization, content-addressed identity, a content-addressed store, transitions, semantic diff, fail-closed semantic merge, a deterministic invariant engine, verification-gated commits, and simulation-only execution planning), the testing strategy that backs the implementation claims, the open research questions (semantic merge completeness, causal attribution fidelity, replay fidelity, distributed consistency), and the explicit limitations. RAHN is positioned as architectural research: it is not a production network controller, and this paper does not claim performance results or novelty beyond a scoped statement in §6.

## 2. Introduction

RAHN's thesis: *a network should not be treated merely as a collection of devices and configurations, but as an evolving computational system* with state, history, intent, constraints, invariants, causality, observations, executable transitions, verification, and controlled evolution.

Two sentences the project uses to position itself:

> RAHN is a stateful execution architecture for evolving networks.
> RAHN gives networks state, memory, causality, and verifiable evolution.

Project posture: **architectural research first, systems implementation second, ecosystem third.** The name is inspired by the Persian concept of *rah* (راه) — a path. RAHN is not "Git for networks" (§6.1) and not an SDN controller, VPN, dashboard, simulator, or IaC tool. The founder and lead author is Mohammad Hossein Alikhani; the project is designed to grow beyond any individual.

## 3. Problem Statement

A typical network environment involves routing protocols, SDN controllers, intent systems, cloud networking, observability stacks, packet-processing frameworks (eBPF/XDP/P4), overlays, policy engines, IaC, telemetry, and incident tooling. Each solves part of the problem; together they leave the network represented as **fragmented views**:

```
configuration + topology + runtime state + telemetry + policy
+ identity + routing + incidents + history
```

Consequences that RAHN targets:

1. **Changes are not verified as states.** Verification, where it exists, applies to configs or plans, not to a candidate future state with identity.
2. **History is lost or non-causal.** Telemetry answers "what happened?" but has no principled link to "which state transition introduced it?".
3. **Semantic conflicts surface in production.** Constraint combinations (latency budgets vs. encryption requirements vs. available paths) are checked nowhere before deployment.
4. **Rollback and explanation are ad hoc.** Without state provenance, "why is the network like this?" requires archaeology.

The underlying question: *can network state become a first-class computational object — versioned, reasoned about, verified, replayed, branched, simulated, evolved, and safely executed?*

## 4. Motivation

- Verification before execution is standard in software CI; networking largely lacks an equivalent anchored in state semantics.
- Immutable, content-addressed history is the foundation for explanation and replay; mutable configuration + logs is not.
- The cost of wrong network changes justifies explicitness: an architecture that makes unsafe transitions hard to *express* is more valuable than one that makes them easy to execute and easy to roll back.
- Determinism is a prerequisite for reproducibility, testing, and formal reasoning; networking tools rarely offer it.

## 5. Existing Network Abstractions

A compressed view of the abstractions RAHN is situated among (full survey in docs/research/prior-art.md):

| Abstraction | What it versions | What it lacks for RAHN's purposes |
|---|---|---|
| Configuration management / IaC | declarations | reconciled-state history, semantic merge, causality |
| SDN controllers | operational view | immutable identity-bearing states, evolution graph |
| Intent-based networking | intents | state history, provenance, semantic branch/merge |
| Batfish-class verification | config analysis results | integration into a versioned lifecycle |
| Digital twins / emulators | runnable labs | invariants, constitution, causal linkage |
| Observability | measurements | linkage to state transitions (provenance) |
| Event sourcing | application events | network semantics (topology, invariants, semantic merge) |

## 6. Related Work

The survey distinguishes **existing work**, **related work**, **RAHN-specific ideas (provisional)**, and **unresolved questions**; see docs/research/prior-art.md for per-system detail (problem, abstraction, strengths, limitations, overlap, differences, integration, conflicts). Key relatives:

1. **Git** — content-addressed history, branches, three-way merge. RAHN borrows the mechanics; the semantic layer (invariant-gated states, network-aware merge, planned causality) is the difference.
2. **Event sourcing** — replayable audit; RAHN keeps both states and transitions content-addressed and adds network semantics.
3. **Terraform's plan/apply** — the shape resembles RAHN's candidate→verify→execute, without history, branches, or a constitution.
4. **Batfish and formal network verification (HSA, VeriFlow, NetKAT, Minesweeper)** — rigorous analysis of configurations/data planes; the most important prior-art family for RAHN's verification research, operating on different inputs (configs vs. versioned candidate states).
5. **IBN (RFC 9315's analysis)** — intent assurance overlaps RAHN's verification+observation stages; RAHN anchors on state instead of intent.

**Scoped novelty statement.** The *combination and emphasis* — a full lifecycle of content-addressed identity, branching, semantic fail-closed merging, constitution-gated verification, and simulation-first execution around network state — was not identified in the surveyed sources. Individual elements have close relatives as documented. This statement is bounded by the survey's scope and must be re-validated against a systematic literature review before publication-level novelty claims. **[Proposed — pending deeper survey]**

## 7. Design Goals

1. Determinism first: same inputs → same states, ids, verdicts. **[Implemented, test-enforced]**
2. Explicit state transitions as the only way state changes. **[Implemented]**
3. Immutable, content-addressed history. **[Implemented]**
4. Verify before execute, structurally enforced. **[Implemented at the commit/merge boundary and at the execution gate]**
5. No network side effects by default; safest mode is read-only/simulated. **[Implemented]**
6. No AI in the core; the system is complete without it. **[Held by design]**
7. Explainability of failures, diffs, and verification results. **[Implemented for v0.1 scope]**
8. Separation of planes: representation ≠ execution; control ≠ execution; observation ≠ decision. **[Implemented for representation/execution; observation pending]**

## 8. Non-Goals

Not a router, VPN, SDN controller, monitoring dashboard, packet analyzer, generic simulator, IaC tool, or configuration version store; not "Git for networks"; not an LLM wrapper. No blockchain. No AI dependency. No premature distributed consensus, DSL, custom transport, GUI, or cloud integration. The complete list with rationale is in the project charter and ARCHITECTURE.md §4.

## 9. RAHN Conceptual Model

The lifecycle:

```
Intent → State → Constraints → Candidate Transition → Verification
       → Execution → Real Network → Observation → Causal Memory → New State
```

Intent shapes candidate changes; constraints and history feed verification; verification gates execution; execution (eventually) affects the real network; observation feeds a causal graph anchored in transitions; the loop repeats. In v0.1 the cycle is realized up to simulation: intent → candidate → verification → commit → (planned) execution. **[Implemented up to simulation; execution/observation/causal stages Proposed/Future]**

## 10. Network State

A state is a first-class value: inspectable, serializable, comparable, hashable, reproducible, verifiable, versionable, replayable. Conceptual shape:

```
State_t = { topology, policy, identity, intent,
            observations, constraints, history_reference }
```

v0.2 implements the **topology + metadata** component with an interface-based model (nodes own named interfaces; links connect `node/interface` endpoints; ADR 0011) inside a versioned container. **[Implemented for topology]**; the remaining components are **[Proposed]** with staged admission through the ADR process. The model is deliberately not frozen; it is an evolving research artifact whose evolution rules are specified (docs/spec/state.md).

## 11. State Identity **[Implemented]**

Identity = SHA-256 over the state's **canonical serialization** (format-version tagged). Canonicalization rules (ADR 0003): total ordering everywhere; no ambient information; no floating point; strict total parsing (unknown versions, truncation, trailing bytes, unsorted content, dangling endpoints are hard errors). The hash scheme is an internal v0.1 detail; the version tag is the evolution seam. Evidence: property tests over 200 seeded random networks assert round-trip exactness and identity stability; an end-to-end test asserts identical histories in separate runs produce identical commit ids.

## 12. State Transitions **[Implemented]**

A transition is a pure, total function `(State A, Operation) → candidate B | structured rejection` (ADR 0005). The v0.2 vocabulary: `add_node`, `remove_node`, `add_interface`, `remove_interface`, `add_link`, `remove_link` (endpoints are interfaces; ADR 0011). No I/O, clock, or randomness inside transitions; committed states are never mutated; rejections are structured and explainable. Commit records (content-addressed) capture state id, parents, the operations applied, the message, and the verification summary — making history a first-class artifact and future replay structurally feasible.

## 13. Network Constitution

The constitution is a named, versioned collection of invariants that must hold across all valid states (e.g., "databases must never be publicly reachable", "control-plane connectivity must be preserved"). A candidate state violating any invariant is rejected **before** any durable write or execution. **[Implemented as data + engine for the v0.1 vocabulary; expressiveness research open — RQ4]**

v0.2 requirement vocabulary: structural invariants (referential integrity, link endpoints, duplicate links, self-loops incl. same-node loops) always checked; declared `require-connectivity a b` and `prohibit-connectivity a b` (isolation) requirements checked per candidate over the interface-induced node graph. Important observed semantics: the constitution gates **branch states too** — a change that breaks a requirement cannot be committed on any branch (test-enforced).

## 14. Verification **[Implemented for v0.1 scope]**

Deterministic checking with fixed evaluation order, no environment dependence, and per-invariant reports carrying machine-readable evidence — never a bare boolean. The honesty rule is normative: **"verified" means "all encoded invariants hold" and nothing more.** Commits carry a verification summary; the merge path re-verifies merged candidates as defense-in-depth.

## 15. Branching and Evolution **[Implemented for v0.1 scope]**

Evolution is a commit DAG; branches are refs to commit records (which immutably name states and parents). Diff is semantic (object-level). Merge is three-way over state semantics (ADR 0007): each side's object-level effects (with content) are computed against the common ancestor; both sides touching an object merges only when their effects are provably identical; anything else — including ambiguous identical additions and application-order hazards — fails closed with an explanation. Merged candidates pass the normal verification gate. Property tests assert merge commutativity for disjoint effects over random networks.

## 16. Observation **[Implemented — v0.4; ingestion measured]**

Observation is a first-class record (ADR 0013): `{seq, time_ns, subject, metric, value}` with exact integer values (counter/gauge/event), **caller-supplied timestamps** (no implicit clocks — determinism holds end-to-end, test-enforced), positional sequence numbers, and `(time_ns, seq)` total ordering. Records are validated at ingest against the current HEAD state and carry that state id as provenance; unknown subjects are rejected. Storage is an append-only, versioned-framed log with loud corruption refusal. The CLI exposes `rahn observe` and machine-readable `rahn observations` output.

**Scope:** measurement *records* only — no aggregation, no units system, no clock authority beyond the caller, and no causal semantics (§17 remains future). Ingestion is measured (§27): read/parse of 10⁵ records costs ~68 ms, but single-record append is ~144 µs (per-append file open+flush), recorded as known performance debt.

## 17. Causal Memory **[Implemented — v0.5, as asserted structure; inference remains Future]**

Causal memory is implemented as an explicit, append-only edge set over immutable anchors — `Observation(seq)` records (ADR 0013) and content-addressed `Commit` ids — with strict epistemic statuses: **temporal-correlation** (ordering only), **hypothesis** (suspected link), and **verified**, the last structurally restricted to edges whose *both* anchors are Commits (the only class with mechanically checkable provenance). The edge set is a DAG (cycle-closing insertions are rejected), anchors must exist at creation, and the system never generates edges automatically: every relation is an explicit assertion whose status labels its epistemic strength. Incidents are query-time connected components (`rahn explain`), with output carrying statuses verbatim.

**Honest boundary:** RAHN records and queries *asserted* structure; it discovers nothing. Attribution fidelity (RQ5) — how trustworthy hypothesis chains are against real telemetry — remains the open research question and requires the addressing/traffic deferrals to be lifted before it can be exercised meaningfully. Graph-operation cost is measured (§27).

## 18. Replay **[Partially available; full replay Future]**

Committed states are immutable and content-addressed, so *state* reconstruction at any historical point is already possible via recorded history (`rahn inspect`, `rahn log`). Full replay — reconstructing observations and behavior as of a past time — is **[Future]** (RQ6, E5) and must not be claimed before it is measured.

## 19. Execution Architecture **[Implemented for simulation; backends Proposed]**

Execution is separated from representation through an explicit, inspectable **execution plan** (dependency-safe ordering: removals before additions). The default remains **simulation**: `rahn apply` prints the exact command sequence and performs no I/O. The first real backend (v0.3, ADR 0012) maps plans deterministically to iproute2 commands: nodes become network namespaces (`rahn-<node>`), interfaces become dummy interfaces, links become veth pairs with deterministic IFNAMSIZ-safe names. Since v0.7, backends sit behind an explicit `ExecutionBackend` trait (ADR 0016) with capability negotiation; the namespace backend's semantics are unchanged. Real execution requires `--execute --yes-i-know`, Linux, and root; host-side operations are structurally restricted to `rahn-*` namespaces (test-enforced); `rahn destroy --yes-i-know` is the recovery path. **[Implemented; validated on Linux CI for a 2-node scenario]** No addressing exists, so connectivity is link-existence only. **[Future]**

## 20. Security

v0.1 posture **[Implemented]**: no network I/O; no privileged operations; default mode read-only/simulated; all loaded state treated as untrusted input (strict total parsing; hash verification precedes use; corruption refused loudly). Future concerns **[Proposed/Future]**: identity, authentication, capability-based execution, signed transitions, provenance verification, replay-attack resistance, privilege separation. The threat model lives in docs/spec/security.md and the security policy in SECURITY.md.

## 21. Distributed Operation **[Implemented — v0.6, as peer sync; strong consistency explicitly not claimed]**

The requirements derivation (RQ7) concluded that RAHN's writes are low-frequency, human-scale, and merge-aware — so the v0.6 model is **peer synchronization over content-addressed history** (ADR 0015), not leader consensus: every replica is authoritative for its own history; synchronization exchanges only immutable hash-verified records (branch-tip offers + fetched commits/states with complete ancestries); divergence converges exclusively through the fail-closed semantic merge, producing **byte-identical merge commits on every replica** (deterministic: parents ordered lexicographically by commit id); conflicts fail closed and persist. Under partition, local writes remain available and divergence is represented rather than prevented.

**Explicitly not claimed:** linearizability, serializability, or any cross-replica strong consistency; per-replica read-your-writes and monotonic history are the only read guarantees. **Deliberate non-concepts:** leader election and epochs (nothing for an epoch to order without a single writer); CRDT convergence (constraint conflicts require explanation, not silent resolution). Replica identity is self-asserted (documented trust boundary; integrity via content hashes; replay is idempotent); signed transitions remain Future. Sync transport is measured (§27) with per-object cost recorded as known debt.

## 22. Programmability **[Partially Implemented — v0.9]**

The IR is declared (ADR 0018): the transition `Operation` vocabulary plus the canonical byte encoding — the representation every artifact compiles to or derives from; a future DSL compiles *to* Operations and is deliberately deferred until authoring evidence demands it. The public surface is formalized as the **`rahn-sdk` facade**: a curated, documented, doctest-covered API with compile-time surface guards; anything outside it is internal. Pre-1.0 policy: SDK-breaking changes land in MINOR releases with migration notes. Language bindings beyond Rust remain [Future].

## 23. AI / Neuro-Symbolic Extensions **[Future]**

AI may propose (diagnosis, ranking, candidate transitions); the deterministic verifier disposes. AI is optional by charter, excluded from the core, and must never bypass verification (RQ9); the system must remain useful without it (RQ10). No AI component exists.

## 24. Prototype Architecture **[Implemented]**

Rust workspace (v0.3.0-alpha): `rahn-core` (object model: nodes, interfaces, interface-endpoint links), `rahn-state` (canonicalization v2, identity, transitions, semantic diff, graph queries, commits, history), `rahn-store` (content-addressed store, refs, index), `rahn-verify` (constitution incl. isolation requirements, invariants, fail-closed merge), `rahn-sim` (plans), `rahn-exec` (Linux namespace backend, ADR 0012), `rahn-cli` (the `rahn` binary). Single third-party dependency (`sha2`, Apache-2.0 OR MIT), recorded in docs/third-party.md. Repository layout: `.rahn/{objects,refs,HEAD,index,constitution}`.

## 25. v0.1 Implementation **[Implemented]**

The v0.1 scope followed the charter's strict list: state model, identity, local persistence, four transitions, semantic diff, branches + checkout, conservative merge, invariant engine, deterministic verification, simulation-only execution, small CLI, extensive tests. Testing strategy: docs/testing.md — 81 tests including adversarial canonical-parsing cases, corrupted-object refusal, end-to-end CLI flows, seeded property tests, and a cross-run determinism test. Known v0.1 limitations: no CLI-side divergent histories beyond checkout+merge flows; no metadata-edit operations (diffs reject them); merge conflict detection is object-level, not constraint-level.

## 26. Experimental Methodology

No experiments have been run yet; the methodology is pre-registered in docs/research/experiment-plan.md (E1 determinism — partially executed as tests; E2 verification cost; E3 merge conflict detection; E4 causal attribution; E5 replay fidelity; E6 plan/observation differential testing) and docs/research/benchmark-methodology.md (environment, baselines, tail metrics, no hand-picked numbers, negative results published).

## 27. Results

The first measurement-kind evidence exists as of Stage 2 (v0.2.0-alpha): scaling timings for state construction, canonical serialization, identity hashing, diff, verification, and shortest-path on ring topologies of 10 to 100 000 nodes, recorded with environment and methodology in [docs/research/stages/v0.2.md](../research/stages/v0.2.md). Headline scoped observations: serialization and identity hashing remain in the low milliseconds at 10⁵ objects; verification of the structural invariant floor over 10⁵ links costs ~100 ms (single machine, release build). These are baseline measurements of one workload on one machine — not performance claims, and not yet reproducible cross-machine.

Stage 6 added **sync measurements** (E-sync): replicating a 10 000-commit history between fresh repositories costs ~80.9 s (~3–8 ms/object — per-object file open/rename plus double hashing; transport-shaped, not algorithm-shaped); `offer` is sub-millisecond at all scales; smaller sizes: 10 commits ≈ 21 ms, 100 ≈ 145 ms, 1 000 ≈ 2 981 ms. Single machine, medians, methodology in [docs/research/stages/v0.6.md](../research/stages/v0.6.md). [Measured]

Stage 5 added **causal-graph measurements** (E-causal): on a 10 000-edge linear chain (deepest DFS case), graph build costs 18.5 ms, a full-chain cycle check 4.3 ms, and an incident query 5.7 ms — the last after fixing an O(V·E) first implementation (2 701.9 ms) found by this benchmark and replaced with a reverse adjacency index. Methodology in [docs/research/stages/v0.5.md](../research/stages/v0.5.md). [Measured]

Stage 4 added **observation ingestion measurements** (E-obs): encoding+appending 10⁵ records costs ~25.8 s (~3 878 rec/s; ~144 µs single-append median, dominated by per-append file open+flush — recorded as debt); full read+parse of 10⁵ records costs ~68 ms; storage ≈156.5 B/record. Single machine, medians, methodology in [docs/research/stages/v0.4.md](../research/stages/v0.4.md). [Measured]

Stage 3 added the first **plan/observation differential evidence** (E6, partial): on Linux CI, the real-execution test instantiates a 2-node/1-link topology in network namespaces and asserts the observed namespace state (`ip netns list`, `ip -n rahn-a link show`) contains the planned objects, then asserts complete teardown. No plan/observation mismatches were observed in this scenario. Scoped strictly: one scenario, link-level connectivity, no addressing or traffic. [Experimental — CI-validated]

Previously this section read: **None — no benchmark or controlled experiment had been conducted.** What existed was test evidence (docs/testing.md): deterministic identity across runs and platforms, canonical round-trip exactness, corrupted-state refusal, fail-closed merge behavior, verification gating. Test evidence demonstrates *behavior under the tested conditions*; it is not a performance or effectiveness result. This section will report experiment outcomes with full methodology or state that none exist — it will never be padded.

## 28. Limitations

1. **Partial model of reality.** Real networks contain state RAHN does not model; divergence between modeled and real state is expected — which is why real execution is isolated to fresh namespaces, is opt-in, and stays limited to what the model perceives (link-level; no addressing yet).
2. **Verification completeness.** "Verified" covers only encoded invariants; unencoded failure modes pass.
3. **Merge incompleteness.** Semantic conflict detection cannot be complete; fail-closed trades false rejections for no false accepts; the false-rejection rate is unmeasured.
4. **Determinism costs** CPU and storage; immutable history grows without pruning (deferred deliberately).
5. **Causal explainability is bounded by instrumentation** — and does not exist yet at all.
6. **Single-writer, local-only** through Stage 5.
7. **Ergonomics tax** for explicitness; if it proves too high in practice, the project fails regardless of architectural soundness.
8. **Limited measurement evidence.** The only measurements are the Stage 2 single-machine scaling baselines (§27); no cross-machine, no real-network, and no comparative measurements exist.
9. **Transition-application cost (known performance debt, recorded).** Repeated operation application currently clones the whole network — roughly O(m·n) for m batched operations on an n-object network. (b) observation ingestion is dominated by per-append file open+flush (~144 µs); (c) distributed sync transport costs ~3–8 ms/object (per-object file operations + double hashing) — a packfile/batched-transfer design is the natural follow-up. All are acceptable at measured scales (§27); copy-on-write, batched-apply, buffered-writer, or packed-transfer designs would require their own ADRs and have deliberately not been attempted.

Full list with reasoning: docs/research/limitations.md.

## 29. Research Questions

The live list, each with motivation, hypothesis, method, measurable result, and limitations (docs/research/research-questions.md): RQ1 deterministic content-addressed state (partially answered, [Implemented] for topology); RQ2 verification before execution ([Implemented] for v0.1 vocabulary; cost unmeasured); RQ3 semantic branch/merge ([Implemented] fail-closed; E3 pending); RQ4 constitution expressiveness (open); RQ5 causal attribution fidelity (open); RQ6 incident reconstruction (open); RQ7 distributed consistency requirements (deliberately deferred); RQ8 multi-backend execution (open); RQ9 AI as advisor not authority (deferred); RQ10 usefulness without AI (holding by design).

## 30. Future Work

Staged per ROADMAP.md: Stage 2 richer model (interfaces, addressing, paths, services, stronger constraints, graph algorithms); Stage 3 Linux-namespace execution prototype; Stage 4 observation + provenance; Stage 5 causal memory and `rahn explain`; Stage 6 distributed state (requirements-first); Stage 7 backend abstraction; Stage 8 network CI; Stage 9 programmability; 1.0 gated on stability, security credibility, and reproducibility. Publication path only if genuinely novel, reproducible findings emerge (most plausibly from E4/E5).

## 31. Conclusion

RAHN's v0.1 demonstrates — at test-grade, not measurement-grade — that the core primitive is buildable: network state as a deterministic, content-addressed, invariant-gated, branchable object with recorded, verifiable evolution and simulation-first execution. What remains open is exactly what makes the project research rather than engineering: whether semantic merge can be made usefully complete, whether causal attribution can be made trustworthy, whether replay fidelity suffices for incident work, and what consistency model distributed network state actually needs. The architecture is designed so that each of these can be answered with evidence, and corrected when the evidence disagrees.

## 32. References

Primary citations are added as the survey deepens (docs/research/prior-art.md carries the per-section source notes). Currently citable primary sources:

1. IETF RFC 9315, *An Analysis of Intent-Based Networking* (intent decomposition and assurance analysis).
2. Apache License 2.0; Creative Commons Attribution 4.0 International (project licensing).
3. The RAHN repository itself: ADRs 0001–0019, docs/spec/, docs/research/, docs/testing.md, docs/reproducibility.md.

**Fabrication rule: no reference appears in this paper that the project has not actually consulted.**

---

## Version history

| Version | Date | Change | Trigger |
|---|---|---|---|
| v0.1 | 2026-04-15 | Structured outline; no claims | Project bootstrap (Stage 0) |
| v0.2 | 2026-04-24 | Full initial draft; claim-status markers throughout; §27 states explicitly that no measurements exist; scoped novelty statement | Post-v0.1 hardening milestone (Stage 1); first implementation exists, test-grade evidence available |
| v0.3 | 2026-05-08 | Interface-based state model (ADR 0011) reflected in §10/§12/§13; graph queries and isolation constraints in §13; first scaling measurements in §27 (single-machine, methodology recorded) | Stage 2 (v0.2.0-alpha) |
| v0.4 | 2026-05-30 | §19 updated: namespace execution backend implemented (ADR 0012), simulation remains default; §24 adds rahn-exec; §27 gains first E6 differential evidence (CI, 2-node scenario); §28 limitation 1 reworded | Stage 3 (v0.3.0-alpha) |
| v0.5 | 2026-06-13 | §16 upgraded from [Future] to [Implemented]: deterministic observation model (ADR 0013) with ingestion measurements in §27; §17 causal memory remains [Future] with the observation log as its designated input | Stage 4 (v0.4.0-alpha) |
| v0.6 | 2026-06-26 | §17 upgraded to [Implemented, asserted structure]: causal edges with epistemic statuses (ADR 0014), DAG enforcement, incident queries; E-causal measurements in §27 (incl. the O(V·E)→O(component) fix found by benchmark) | Stage 5 (v0.5.0-alpha) |
| v0.7 | 2026-07-19 | §21 upgraded from [Future] to [Implemented, peer sync]: consistency model per RQ7 (ADR 0015), byte-identical convergence, fail-closed divergence; E-sync measurements in §27; explicit non-claims recorded | Stage 6 (v0.6.0-alpha) |
| v0.8 | 2026-08-06 | §19: backends formalized behind the ExecutionBackend trait (ADR 0016) with capability negotiation; simulation made a backend rather than a special case | Stage 7 (v0.7.0-alpha) |
| v0.9 | 2026-08-15 | CI surface documented (ADR 0017): `rahn test` with exit-code contract and deterministic TSV; example workflow; scope limited to committed states | Stage 8 (v0.8.0-alpha) |
| v1.0 | 2026-08-28 | §22 upgraded to [Partially Implemented]: IR declared (ADR 0018), `rahn-sdk` facade with surface guards; DSL deferred | Stage 9 (v0.9.0-alpha) |
| v1.1 | 2026-09-02 | **v1.0.0 stable release document.** Final structure per charter §51: all sections present, evidence-linked to measured stages (§27: scaling, observation ingestion, causal-graph ops, sync transport, E6 partial); final-comparison and v1.0 review referenced; compatibility commitment (ADR 0019) recorded; limitations carried forward explicitly | Stage 10 (v1.0.0) |
