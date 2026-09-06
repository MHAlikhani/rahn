<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Design

Status: living document. Defines the design principles and how they are applied.

## Principles

The project follows nineteen architectural principles (charter §19), applied as follows:

1. **Determinism first.** Every operation is a pure function of its inputs where possible; all randomness, iteration order, and timestamps are either eliminated or explicitly recorded as inputs. Canonical serialization (see [docs/adr/0003-canonical-serialization.md](docs/adr/0003-canonical-serialization.md)) is the enforcement point.
2. **Explicit state transitions.** No function mutates a state in place; operations return new states.
3. **Immutable historical state.** Committed states are content-addressed and never rewritten.
4. **Strong typing.** Invalid states are hard to represent: link endpoints must reference existing nodes at the type/validation boundary, not deep in the engine.
5. **No hidden global mutable state.** All mutable context is passed explicitly.
6. **No unnecessary runtime magic.** No reflection-driven behavior, no dynamic dispatch where static suffices.
7. **No AI dependency in the core.** AI is an optional reasoning layer consuming state, proposing candidates that pass through the same verifier as any other transition.
8. **No network side effects by default.** Simulation is the default; the isolated namespace backend is opt-in (ADR 0012).
9. **Verify before execute.** Enforced structurally: the execution engine cannot be reached without a verification result.
10. **Simple primitives over frameworks.**
11–13. **Separation** of control plane from execution plane, representation from execution, observation from decision.
14. **Explainable failures.** Errors carry the violated invariant, the offending objects, and the transition that produced them.
15. **Inspectable transitions.** Every committed transition is queryable (`rahn log`, `rahn inspect`).
16–17. **No premature** distributed-systems complexity or optimization.
18. **Long-term API compatibility** over short-term convenience.
19. **Experimental features behind clear boundaries** (crate and feature-gate boundaries).

## Philosophy

> Make the network understandable before trying to make it autonomous.
>
> Observe before acting. Verify before executing. Record before forgetting.
>
> Intelligence may propose. The architecture must verify.
>
> The system should remain useful without AI.

## Implementation rules

Small modules; no giant files; no speculative abstractions, premature plugins, or dynamic behavior where static behavior suffices; explicit errors; strong types; documented invariants; deterministic behavior; tests close to semantics; public API separated from implementation details.

If a design seems wrong, **document the issue first** — do not silently rewrite the architecture because implementation is inconvenient, and do not silently change core assumptions: record an ADR, update ARCHITECTURE.md and the white paper, and document the reason.

## Priority order

When goals conflict:

```
correctness > clarity > architecture > reproducibility > security
> testability > extensibility > performance > ecosystem > popularity
```
