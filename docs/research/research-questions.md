<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Research Questions

Status: living research document (Stage 0). Each question names its evidence source and the roadmap stage where it should be addressed.

## RQ1 — Fundamental primitives
Which of the candidate primitives (Node, Link, Path, Flow, Service, Identity, Capability, Policy, Intent, Constraint, Invariant, Observation, Event, Transition, State, State Commit, Branch, Merge, Causal Relation, Execution Plan, Verification Result) are fundamental, and which are derivable from a smaller core?
*Evidence: implementation experience in v0.1–v0.2; try eliminating each primitive and see what breaks.*

## RQ2 — State identity
What canonical serialization and hashing scheme gives stable, deterministic state identity across implementations, versions, and platforms — while allowing the state model to evolve without invalidating all history?
*Evidence: v0.1 implementation + adversarial review. Related ADR: [adr/0002](../adr/0002-state-identity.md).*

## RQ3 — Semantic merge completeness
Can network-aware merge detect all meaningful semantic conflicts, or is completeness impossible (à la Rice)? What practical conflict classes can be detected soundly?
*Evidence: v0.1–v0.2 merge implementation + formal analysis. Stage 2 milestone.*

## RQ4 — Constitution expressiveness
What invariant language is expressive enough for real network constitutions yet decidable and deterministic? Where is the line between checkable invariants and infeasible constraint solving?
*Evidence: v0.1 invariant engine → v0.2 constraint engine; formal analysis later.*

## RQ5 — Causal attribution fidelity
Can observations be linked to state transitions reliably enough that `rahn explain` output is trustworthy, given imperfect telemetry and concurrent changes?
*Evidence: Stage 4–5 experiments with fault injection; measure attribution precision/recall.*

## RQ6 — Replay fidelity
Can historical network behavior be reproduced from state + observations to useful fidelity? Which behaviors are reproducible from state alone vs. requiring captured observations?
*Evidence: Stage 5 replay experiments.*

## RQ7 — Distributed consistency requirements
What consistency model does replicated RAHN state actually need — who writes, who reads, how stale may views be, what merges across replicas? (Deliberately asked before choosing any consensus protocol.)
*Evidence: Stage 6 requirements analysis + workload studies.*

## RQ8 — Safe execution
Can execution plans be proven safe against a network model before deployment, and what assurance level is achievable in practice?
*Evidence: Stage 3 namespace prototype → Stage 7 backends; differential testing between planned and observed effects.*

## RQ9 — Explainable routing
Can the system explain why a path was selected, in terms a human operator can audit, deterministically?
*Evidence: Stage 2+ graph algorithms with explanation traces.*

## RQ10 — Failure forecasting
Can historical state + current observations predict likely failures better than telemetry-only baselines?
*Evidence: Stage 5+ with recorded failure scenarios; controlled comparison.*

## RQ11 — Neuro-symbolic control
Can neural models proposing candidate transitions add value while every proposal passes the same deterministic verifier — and can we measure that value honestly?
*Evidence: explicit experiments, only after Stages 2+; AI must never be load-bearing.*

## RQ12 — Model completeness
Which aspects of real network state resist the RAHN model (unmodelable or too dynamic), and how should modeled/real divergence be represented and reconciled?
*Evidence: Stage 3–4 observations vs. model comparisons.*
