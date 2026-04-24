<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN: A Stateful Execution Architecture for Evolving Networks

**White Paper v0.2 — Initial Draft (technical research document)**
Date: 2026-04-24. Repository: https://github.com/MHAlikhani/rahn
Versioned independently of the codebase; revisions correspond to meaningful architectural changes. Version history at the end.

**Licensing:** RAHN software is licensed under Apache License 2.0. RAHN documentation and research materials (including this paper) are licensed under CC BY 4.0 unless otherwise stated. The paper's license does not change the software license.

**Claim-status legend (mandatory).** Every substantive statement below is marked:

- **[Implemented]** — exists in the v0.1 code with tests (commit `bb38634` lineage; see docs/testing.md).
- **[Experimental]** — measured in a documented experiment; methodology and data available.
- **[Proposed]** — designed, not yet built.
- **[Future]** — research direction, no design yet.

**No experimental results are reported in this version: no benchmark or controlled experiment has been run.** The determinism evidence cited is of test-kind (fixed and property-based tests), not measurement-kind. Section 27 (Results) states this explicitly and will remain honest as evidence accumulates.

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
4. Verify before execute, structurally enforced. **[Implemented at the commit/merge boundary; execution is simulation-only]**
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

v0.1 implements the **topology + metadata** component (nodes, normalized undirected links, bounded metadata) inside a versioned container. **[Implemented for topology]**; the remaining components are **[Proposed]** with staged admission through the ADR process. The model is deliberately not frozen; it is an evolving research artifact whose evolution rules are specified (docs/spec/state.md).

## 11. State Identity **[Implemented]**

Identity = SHA-256 over the state's **canonical serialization** (format-version tagged). Canonicalization rules (ADR 0003): total ordering everywhere; no ambient information; no floating point; strict total parsing (unknown versions, truncation, trailing bytes, unsorted content, dangling endpoints are hard errors). The hash scheme is an internal v0.1 detail; the version tag is the evolution seam. Evidence: property tests over 200 seeded random networks assert round-trip exactness and identity stability; an end-to-end test asserts identical histories in separate runs produce identical commit ids.

## 12. State Transitions **[Implemented]**

A transition is a pure, total function `(State A, Operation) → candidate B | structured rejection` (ADR 0005). The v0.1 vocabulary: `add_node`, `remove_node`, `add_link`, `remove_link`. No I/O, clock, or randomness inside transitions; committed states are never mutated; rejections are structured and explainable. Commit records (content-addressed) capture state id, parents, the operations applied, the message, and the verification summary — making history a first-class artifact and future replay structurally feasible.

## 13. Network Constitution

The constitution is a named, versioned collection of invariants that must hold across all valid states (e.g., "databases must never be publicly reachable", "control-plane connectivity must be preserved"). A candidate state violating any invariant is rejected **before** any durable write or execution. **[Implemented as data + engine for the v0.1 vocabulary; expressiveness research open — RQ4]**

v0.1 requirement vocabulary: structural invariants (referential integrity, link endpoints, duplicate links, self-loops) always checked; declared `require-connectivity a b` requirements checked per candidate. Important observed semantics: the constitution gates **branch states too** — a change that breaks a requirement cannot be committed on any branch (test-enforced).

## 14. Verification **[Implemented for v0.1 scope]**

Deterministic checking with fixed evaluation order, no environment dependence, and per-invariant reports carrying machine-readable evidence — never a bare boolean. The honesty rule is normative: **"verified" means "all encoded invariants hold" and nothing more.** Commits carry a verification summary; the merge path re-verifies merged candidates as defense-in-depth.

## 15. Branching and Evolution **[Implemented for v0.1 scope]**

Evolution is a commit DAG; branches are refs to commit records (which immutably name states and parents). Diff is semantic (object-level). Merge is three-way over state semantics (ADR 0007): each side's object-level effects (with content) are computed against the common ancestor; both sides touching an object merges only when their effects are provably identical; anything else — including ambiguous identical additions and application-order hazards — fails closed with an explanation. Merged candidates pass the normal verification gate. Property tests assert merge commutativity for disjoint effects over random networks.

## 16. Observation **[Future — Stage 4]**

Planned: observations (health, latency, statistics, events) as first-class objects linked to the states/transitions they concern; observation pipeline separated from decision-making. No observation machinery exists in v0.1; this section describes design intent only.

## 17. Causal Memory **[Future — Stage 5, core research]**

Planned: a causal graph over observations and transitions enabling "which transition introduced this change?" reasoning, with explicit epistemic status per edge: **temporal correlation**, **causal hypothesis**, **verified causal relation**. This is the project's most important open research direction (RQ5). Nothing causal exists in v0.1; recorded transitions and their provenance are the designed substrate.

## 18. Replay **[Partially available; full replay Future]**

Committed states are immutable and content-addressed, so *state* reconstruction at any historical point is already possible via recorded history (`rahn inspect`, `rahn log`). Full replay — reconstructing observations and behavior as of a past time — is **[Future]** (RQ6, E5) and must not be claimed before it is measured.

## 19. Execution Architecture **[Implemented for simulation; backends Proposed]**

Execution is separated from representation through an explicit, inspectable **execution plan** (dependency-safe ordering: removals before additions). v0.1 is **simulation-only**: `rahn apply` prints the plan and touches nothing — enforced by architecture (no execution backend exists) and by tests. The first real backend target is isolated Linux network namespaces (Stage 3), with explicit opt-in execution, never silent host modification. **[Proposed]**

## 20. Security

v0.1 posture **[Implemented]**: no network I/O; no privileged operations; default mode read-only/simulated; all loaded state treated as untrusted input (strict total parsing; hash verification precedes use; corruption refused loudly). Future concerns **[Proposed/Future]**: identity, authentication, capability-based execution, signed transitions, provenance verification, replay-attack resistance, privilege separation. The threat model lives in docs/spec/security.md and the security policy in SECURITY.md.

## 21. Distributed Operation **[Future — Stage 6, deliberately unsolved]**

Replication, identity, consistency, synchronization, and conflict handling will be researched only after local state semantics are stable — and mechanism selection (consensus, CRDTs, or otherwise) follows a requirements derivation (RQ7), not familiarity. No distributed claims are made in this paper.

## 22. Programmability **[Future — Stage 9+]**

API, SDK, formalized IR, policy language, and DSL evaluation come after the IR is understood. The v0.1 public surface is the CLI and the crate APIs; neither is stable, and both say so.

## 23. AI / Neuro-Symbolic Extensions **[Future]**

AI may propose (diagnosis, ranking, candidate transitions); the deterministic verifier disposes. AI is optional by charter, excluded from the core, and must never bypass verification (RQ9); the system must remain useful without it (RQ10). No AI component exists.

## 24. Prototype Architecture **[Implemented]**

Rust workspace (v0.1.0-alpha.2): `rahn-core` (object model), `rahn-state` (canonicalization, identity, transitions, diff, commits, history), `rahn-store` (content-addressed store, refs, index), `rahn-verify` (constitution, invariants, merge), `rahn-sim` (plans), `rahn-cli` (the `rahn` binary). Single third-party dependency (`sha2`, Apache-2.0 OR MIT), recorded in docs/third-party.md. Repository layout: `.rahn/{objects,refs,HEAD,index,constitution}`.

## 25. v0.1 Implementation **[Implemented]**

The v0.1 scope followed the charter's strict list: state model, identity, local persistence, four transitions, semantic diff, branches + checkout, conservative merge, invariant engine, deterministic verification, simulation-only execution, small CLI, extensive tests. Testing strategy: docs/testing.md — 81 tests including adversarial canonical-parsing cases, corrupted-object refusal, end-to-end CLI flows, seeded property tests, and a cross-run determinism test. Known v0.1 limitations: no CLI-side divergent histories beyond checkout+merge flows; no metadata-edit operations (diffs reject them); merge conflict detection is object-level, not constraint-level.

## 26. Experimental Methodology

No experiments have been run yet; the methodology is pre-registered in docs/research/experiment-plan.md (E1 determinism — partially executed as tests; E2 verification cost; E3 merge conflict detection; E4 causal attribution; E5 replay fidelity; E6 plan/observation differential testing) and docs/research/benchmark-methodology.md (environment, baselines, tail metrics, no hand-picked numbers, negative results published).

## 27. Results

**None.** No benchmark or controlled experiment has been conducted. What exists is test evidence (docs/testing.md): deterministic identity across runs and platforms, canonical round-trip exactness, corrupted-state refusal, fail-closed merge behavior, verification gating. Test evidence demonstrates *behavior under the tested conditions*; it is not a performance or effectiveness result. This section will report experiment outcomes with full methodology or state that none exist — it will never be padded.

## 28. Limitations

1. **Partial model of reality.** Real networks contain state RAHN does not model; divergence between modeled and real state is expected and is why execution stays simulation-only until the model can perceive what it would change.
2. **Verification completeness.** "Verified" covers only encoded invariants; unencoded failure modes pass.
3. **Merge incompleteness.** Semantic conflict detection cannot be complete; fail-closed trades false rejections for no false accepts; the false-rejection rate is unmeasured.
4. **Determinism costs** CPU and storage; immutable history grows without pruning (deferred deliberately).
5. **Causal explainability is bounded by instrumentation** — and does not exist yet at all.
6. **Single-writer, local-only** through Stage 5.
7. **Ergonomics tax** for explicitness; if it proves too high in practice, the project fails regardless of architectural soundness.
8. **Test-kind evidence only**; no measurements yet (§27).

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
3. The RAHN repository itself: ADRs 0001–0010, docs/spec/, docs/research/, docs/testing.md, docs/reproducibility.md.

**Fabrication rule: no reference appears in this paper that the project has not actually consulted.**

---

## Version history

| Version | Date | Change | Trigger |
|---|---|---|---|
| v0.1 | 2026-04-15 | Structured outline; no claims | Project bootstrap (Stage 0) |
| v0.2 | 2026-04-24 | Full initial draft; claim-status markers throughout; §27 states explicitly that no measurements exist; scoped novelty statement | Post-v0.1 hardening milestone (Stage 1); first implementation exists, test-grade evidence available |
