<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# ADR 0005: Transition Model — Explicit, Total, Recorded

- Status: Accepted
- Date: 2026-04-15
- Deciders: project founding (charter §7, §16.D)

## Context

The core primitive is the state transition, not configuration. The model must make unsafe transitions hard to express and every change inspectable.

## Decision

A transition is a **pure, explicit function**:

```
State A + Operation + Constraints + Preconditions → Result[State B | Rejection]
```

Rules:

1. **Totality**: every operation either produces a well-formed candidate state or a structured rejection — never a partially applied state, never in-place mutation.
2. **Purity**: candidate state B is a deterministic function of (A, operation, parameters). No I/O, no clock, no randomness inside transitions.
3. **Recorded**: committed transitions are stored with parent state ID, operation, parameters, and verification result — making history a first-class artifact and enabling replay.
4. **Gated**: execution (even simulation) of a transition requires a verification result (ADR 0006); the type system enforces that unverified candidates cannot reach the executor.
5. **v0.1 operation set** is intentionally minimal: `add_node`, `remove_node`, `add_link`, `remove_link`. New operations require semantic specification and tests, not ad hoc extension points.

## Consequences

- Replay and time-travel feasibility follow structurally (recorded pure transitions over immutable states).
- Undo is not a special mechanism; it is a transition (or a branch/rollback).
- Ergonomic cost for callers accepted and mitigated by a small operation vocabulary.

## Alternatives considered

- **Declarative desired-state reconciliation** (IaC-style): hides the transition, weakens causality and explainability; may later serve as *syntax* over explicit transitions.
- **Event log with implicit state** (pure event sourcing): replayable but makes "current state" a derived artifact and complicates diff/branch; RAHN keeps both explicit states *and* their generating transitions.
- **Mutable graph with journaling**: faster, but every mutator is a hole in the immutability invariant.

## References

- docs/adr/0006-verification-model.md; docs/adr/0008-execution-boundary.md; docs/spec/transitions.md
