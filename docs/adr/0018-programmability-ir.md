<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# ADR 0018: Public API, SDK Facade, and the v0.9 IR

- Status: Accepted
- Date: 2026-08-19

## Context

Stage 9 (charter): stabilize API/SDK/IR/execution-plan/verification interfaces; investigate the RAHN IR; the DSL is investigated only once IR semantics are mature. Gate: external code must interact with RAHN through a coherent, documented, stable programmatic model.

## Alternatives

1. **Design a new IR language/AST now:** rejected — the IR already exists implicitly; inventing a second representation before it earns its keep is speculative abstraction (charter §33).
2. **Expose every crate path as public API:** rejected — implementation details become compatibility burdens.
3. **Curated SDK facade + declared IR (chosen).**

## Decision

1. **The v0.9 IR is the existing pair:** (a) the `Operation` vocabulary (`rahn-state::transition::Operation` — add/remove node/interface/link, the only way committed state changes) and (b) the canonical byte encoding of states and records (ADR 0003, format v2). Together they are the interchange representation: every artifact (commits, sync payloads, CI reports, future DSL output) compiles down to or is derived from them. A future DSL compiles *to* Operations — nothing else.
2. **SDK facade — new crate `rahn-sdk`:** a thin, documented, curated re-export layer grouping the stable surface by concern (model, history, verification, observations, causality, sync, execution, CI). It adds no logic; it declares what external code may rely on. Anything not reachable through `rahn-sdk` is internal and may change without notice.
3. **Versioning policy (pre-1.0):** breaking changes to the SDK surface may occur in MINOR releases (0.x semantics) but MUST be recorded in CHANGELOG with migration notes; patch releases never break. The IR byte format can only change via format-version bump + ADR (existing rule).
4. **Documentation:** SDK items carry doc examples that run as doctests — examples are tests.
5. **DSL:** explicitly deferred until a workload demonstrates Operations are insufficient as an authoring surface (reconsideration condition in ADR 0001 terms).

## Consequences

- External projects can build against `rahn-sdk` alone; internal crates remain free below it.
- The facade is a contract surface: API-compatibility tests guard it.
- No network/auth changes; CI (ADR 0017) and sync (ADR 0015) surfaces are consumed as-is.

## Reconsideration conditions

- A DSL workload appears; SDK facade costs outweigh curation (flatten into crates).
