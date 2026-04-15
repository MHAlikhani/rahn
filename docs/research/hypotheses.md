<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Hypotheses

Status: living research document (Stage 0). Each hypothesis is falsifiable and names what would refute it. Do not state hypotheses as facts in any RAHN material.

## H1 — Deterministic network state is practical
A useful fraction of network meaning (topology, policy, identity, intent, constraints) can be captured in a deterministic, content-addressed state object whose identity is stable across runs and platforms.
**Refuted if:** canonicalization proves brittle across versions, or identity churns under benign changes (e.g., reordering, timestamps).

## H2 — Transitions beat configuration as the core primitive
Modeling every change as an explicit, verifiable transition yields better safety and explainability than declarative-config reconciliation, at acceptable ergonomic cost.
**Refuted if:** transition overhead dominates workflows or expressiveness gaps force users around the model.

## H3 — Constitutions are expressible and checkable
Real-world network invariants (isolation, encryption, redundancy, control-plane reachability) can be expressed as deterministic predicates and checked fast enough to gate every transition.
**Refuted if:** representative invariants require undecidable or computationally intractable checks on realistic topologies.

## H4 — Semantic merge conflicts are detectable in useful classes
For non-conflicting topology changes, semantic merge succeeds; for constraint-level conflicts (e.g., infeasible combined requirements), conflicts are detected and explained rather than silently merged.
**Refuted if:** merge either misses conflicts found later by verification, or rejects so many merges that branching becomes unusable.

## H5 — Causal memory adds diagnostic value beyond telemetry
Linking observations to state transitions yields root-cause explanations measurably better than telemetry-only analysis on recorded incident scenarios.
**Refuted if:** attribution precision/recall does not beat a telemetry-only baseline on a common benchmark set.

## H6 — Replay is feasible from immutable history
`rahn replay --at T` can reconstruct state (and useful observations) for past points with acceptable fidelity and cost, from content-addressed history.
**Refuted if:** reconstruction cost grows unacceptably or fidelity is too low to answer incident questions.

## H7 — Verify-before-execute prevents the important failures
A verify gate with a constitution catches the class of state errors that historically cause outages (isolation violations, unreachable control plane, single failure domains) before execution.
**Refuted if:** post-hoc incident analysis shows most harmful states would have passed verification.

## H8 — Simulation-first execution is sufficient until Stage 3
Simulation-only execution (plan generation + inspection, no OS effects) satisfies v0.1–v0.2 needs without real networking.
**Refuted if:** validation of the model requires real behavior earlier than Stage 3.

## H9 — Determinism does not preclude usefulness
A fully deterministic core (no AI, no dynamic behavior) supports real operator workflows.
**Refuted if:** practical use demands nondeterministic helpers inside the core.
