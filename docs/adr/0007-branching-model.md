<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# ADR 0007: Branching and Merge Model

- Status: Accepted
- Date: 2026-04-15
- Deciders: project founding (charter §10, §16.F–G)

## Context

Network evolution branches (experiments, staged changes, parallel proposals). Merging must be network-aware: a merge is not a textual operation, and semantic conflicts must be detected and explained, not silently resolved.

## Decision

1. **Branches are references.** `rahn branch <name>` creates a named pointer to a state ID (ADR 0004). Branching is O(1); no data is copied.
2. **Diff is semantic.** `rahn diff A B` reports object-level changes (± nodes, ± links, changed attributes), derived from state comparison, not text.
3. **Merge is three-way over state semantics.** Base (common ancestor) + branch A + branch B → candidate merged state, then the normal verification gate (ADR 0006) applies.
4. **v0.1 merge scope: non-conflicting topology changes only.** Disjoint additions/removals of nodes and links merge. If both branches touch the same objects, or if merged constraints could be jointly infeasible, the merge **fails closed** with an explanation. No automatic conflict resolution, no textual fallback.
5. **Fail-closed principle**: a merge the system cannot *prove* safe is rejected. False rejections are acceptable; false accepts are not. (Tension recorded; revisited in Stage 2 with better conflict analysis — RQ3, H4.)

## Consequences

- The lattice/ancestor machinery (common-ancestor computation over the commit graph) must be correct and tested.
- Semantic conflict explanation becomes a differentiator (constraint-level conflict reporting in Stage 2).
- Merges inherit verification: a merged candidate must pass the constitution before commit, same as any transition.

## Alternatives considered

- **Textual merge of serialized state**: nonsensical over canonical binary; cannot see semantics; rejected outright.
- **CRDT-style automatic convergence**: appealing for distribution later, but silently resolving constraint conflicts violates the constitution model; revisit only for provably commutative, constraint-free object classes (Stage 6+).
- **No branching in v0.1**: branching is cheap given references and forces the identity/ancestor machinery to exist early — where it belongs.

## Implementation amendment (2026-04-20, v0.1)

In the v0.1 implementation, branch refs point to **commit records** (which
immutably name their state and parents) rather than directly to state ids.
A branch therefore still references a state — transitively through the
commit — and gains a walkable history for free. The common-ancestor
machinery operates over the commit DAG (`rahn-state::history`). Merges are
fail-closed with content-aware conflict detection (identical effects on
both sides are permitted; anything else is rejected).

## References

- docs/adr/0004-storage-model.md; docs/adr/0006-verification-model.md; docs/spec/transitions.md
