<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN: A Stateful Execution Architecture for Evolving Networks

**White Paper v0.1 — Structured Outline**
Status: research document under development. Date: 2026-04-15.
Versioned independently of the codebase (white paper policy): revisions correspond to meaningful architectural changes, not commits. Version history at the end of this document.

**Claim-status legend (mandatory usage):** every substantive statement in future drafts MUST be marked as one of:
- **[Proven]** — established with evidence in this repository (formal argument + reproduction)
- **[Implemented]** — exists in code with tests
- **[Experimental]** — measured in an experiment; methodology and data available
- **[Proposed]** — future concept. **Proposed capabilities must never be presented as implemented.**

**Licensing statement (mandatory):** RAHN software is licensed under Apache License 2.0. RAHN documentation and research materials (including this white paper) are licensed under CC BY 4.0 unless otherwise stated. The paper's CC BY 4.0 license does not change the software license.

---

## Abstract
(To be written last.) One page: the problem of fragmented network state; the thesis that network state should be a first-class, versioned, causal, verifiable computational object; the architecture (lifecycle, constitution, causal memory); current status — honest.

## 1. Introduction
What RAHN is; what it is not (not "Git for networks", not an SDN controller, not a VPN…); project posture: research first, implementation second, ecosystem third.

## 2. Problem Statement
Fragmentation of network representation across configuration, topology, runtime state, telemetry, policy, identity, incidents, and history. Expand from [docs/research/problem-statement.md](../research/problem-statement.md).

## 3. Motivation
Cost of unverified change; loss of causality; semantic conflicts discovered in production; explainability gap.

## 4. Existing Networking Models
Config management/IaC, SDN, IBN, observability, digital twins, verification tooling — what each models well and what each leaves out. [Proven/Implemented] claims only about those systems, with citations.

## 5. Related Work / Prior Art
Condensed from [docs/research/prior-art.md](../research/prior-art.md). **Rule: no novelty claim without a completed prior-art check.** Explicitly separate existing work, related work, RAHN-specific ideas, and unresolved questions.

## 6. Design Goals
Determinism first; explicit transitions; immutable history; verify before execute; no network side effects by default; no AI in the core; explainability; separation of planes.

## 7. Non-Goals
Charter §3/§17 list, stated as architecture, not marketing.

## 8. Core Concepts
State, transition, constitution, causal memory, branching/merge, replay — from [docs/concepts.md](../concepts.md), with formal definitions as they stabilize.

## 9. Network State Model
State_t = {topology, policy, identity, intent, observations, constraints, history_reference}; canonical serialization; content-addressed identity. Mark each element [Implemented]/[Proposed].

## 10. State Transition Model
Pure, total, recorded transitions (ADR 0005); operation vocabulary; rejection semantics.

## 11. Network Constitution
Invariants as first-class data; v0.1 invariant vocabulary; expressiveness limits (RQ4).

## 12. Verification Model
Deterministic checking; structured, explainable results; the honesty rule ("verified" = encoded invariants hold).

## 13. Network Evolution
Evolution graph S0→S1→…; branches as references; provenance.

## 14. Branching and Semantic Merge
Three-way semantic merge; fail-closed; semantic-conflict detection and explanation; completeness limits (RQ3).

## 15. Observation Model
Observation↔transition linkage; separation of observation from decision. [Proposed until Stage 4.]

## 16. Causal Memory
Causal graph over observations and transitions; transition attribution. [Proposed until Stage 5; experimental claims only with E4 data.]

## 17. Replay and Time Travel
Reconstruction from immutable history; fidelity questions (RQ6, E5).

## 18. Execution Architecture
Execution plans; backend abstraction; simulation-first boundary (ADR 0008); namespace prototype (Stage 3).

## 19. Security Model
Read-only/simulated defaults; untrusted state input; hash-verified integrity; future: identity, signed transitions, capabilities. Threat model reference: [docs/spec/security.md](../spec/security.md).

## 20. Distributed Operation
Deliberately unsolved until Stage 6; requirements-first methodology (RQ7).

## 21. Programmability
API/SDK/IR/policy language; DSL evaluation only after IR understanding (Stage 9+). [Proposed.]

## 22. AI / Neuro-Symbolic Extensions
Neural proposal vs. symbolic verification; AI optional and never load-bearing (RQ11). [Proposed.]

## 23. Implementation Architecture
Rust workspace (rahn-core/state/store/verify/cli/sim); language-boundary policy; dependency discipline.

## 24. Prototype
v0.1 scope and its success criteria (charter §15, §34).

## 25. Evaluation Methodology
From [docs/research/experiment-plan.md](../research/experiment-plan.md) and [benchmark-methodology.md](../research/benchmark-methodology.md).

## 26. Experimental Results
Empty in v0.1 of the paper — will contain E1–E6 results or will state that none exist. **Never fabricate or pad this section.**

## 27. Limitations
From [docs/research/limitations.md](../research/limitations.md) — mandatory section; the paper must ship with it.

## 28. Open Research Questions
From [docs/research/research-questions.md](../research/research-questions.md).

## 29. Future Work
Roadmap-ordered; no promises beyond the roadmap's own honesty rules.

## 30. Conclusion
Network state as a first-class computational object; what has been demonstrated vs. what remains.

## 31. References
Only verifiable citations. **Inventing references is prohibited.** Every citation added must have been actually consulted.

---

## Version history

| Version | Date | Change | Trigger |
|---|---|---|---|
| v0.1 | 2026-04-15 | Initial structured outline; no claims of implemented or proven behavior | Project bootstrap (Stage 0) |

Revision policy: a white paper revision corresponds to a meaningful architectural change, not every commit. When a major architectural assumption changes: create/update the ADR → update ARCHITECTURE.md → update this paper → document the reason, here and in the CHANGELOG.
