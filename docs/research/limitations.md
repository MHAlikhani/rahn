<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Limitations

Status: living research document (Stage 0). Honest limitations of the architecture and of the current (documentation-only) state of the project. This document must never shrink by hiding a limitation — only by resolving it (with evidence) or narrowing a claim.

## Limitations of the current project state

- **No implementation exists yet.** Everything in this repository is design and research documentation. No behavior is proven, implemented, or measured. All claims herein are *proposed*, not demonstrated.
- **No evidence yet.** Hypotheses ([hypotheses.md](hypotheses.md)) are untested; no experiment has run; no benchmark exists.
- **No novelty claims are warranted yet.** Prior-art analysis ([prior-art.md](prior-art.md)) is preliminary and must be deepened before the white paper asserts any novelty.

## Architectural limitations (inherent or accepted trade-offs)

1. **Partial model of reality.** RAHN models a chosen subset of network meaning. Real networks contain state that is dynamic, hidden, or vendor-specific (TCAM contents, kernel caches, firmware behavior). Modeled state can diverge from real state; RAHN must never execute against a real network it cannot observe (hence simulation-first).
2. **Verification is only as good as its encoding.** "Verified" means "all encoded invariants hold" — nothing more. Unencoded failure modes pass verification.
3. **Merge is incomplete in principle.** Semantic conflict detection cannot be complete in general; RAHN compensates by failing closed, which means some mergeable states will be rejected.
4. **Determinism costs.** Canonical serialization, content hashing, and explicit transitions cost CPU and storage; immutable history grows unboundedly until pruning is designed.
5. **Explainability is bounded by instrumentation.** `rahn explain` can only trace what was recorded; gaps in observation yield gaps in causal chains, and confidence in attribution will be probabilistic at best.
6. **Single-writer in early stages.** Stages 0–5 assume local, single-writer state; distributed operation (Stage 6) is unsolved by design until its requirements are derived.
7. **Ergonomics tax.** Explicitness (transitions, verification, recorded provenance) is more work than editing configs. If this tax proves too high, the project fails in practice even if the architecture is sound.
8. **Rust is an implementation choice with consequences.** Compile-time strictness slows exploration; the architecture must not become Rust-shaped rather than problem-shaped.

## Known unknowns

- The right consistency model for distributed RAHN state (RQ7).
- Whether causal attribution fidelity (RQ5) can cross the trustworthiness threshold.
- Whether the constitution language can stay decidable while remaining useful (RQ4).
