<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Limitations

Status: living research document (updated at v0.5.0-alpha / Stage 5). Honest limitations of the architecture and of the current implementation state. This document must never shrink by hiding a limitation — only by resolving it (with evidence) or narrowing a claim.

## Limitations of the current project state

- **Implementation is alpha-grade.** The v0.5 system (interface-based state engine, canonical format v2, graph queries, isolation constraints, namespace execution backend per ADR 0012, observations per ADR 0013, causal memory per ADR 0014) is implemented and test-enforced ([testing.md](../testing.md)), but no production use, external review, or real deployment exists; the Linux execution path is validated by CI only.
- **Causality is asserted, never inferred.** Edge statuses are honest labels; there is no automatic causal analysis, no AI reasoning, and no distributed causal memory.
- **Evidence is partial.** Determinism claims are test-enforced; the only measurements are the Stage 2 single-machine scaling baselines ([stages/v0.2.md](stages/v0.2.md)). Hypotheses ([hypotheses.md](hypotheses.md)) are largely untested; the full experiment program (E2 completion, E3–E7) has not run.
- **No novelty claims are warranted yet.** Prior-art analysis ([prior-art.md](prior-art.md)) is a working survey and must be deepened before the white paper asserts any novelty.
- **Known performance debt (recorded, not yet addressed):** repeated operation application currently clones the whole network, producing roughly O(m·n) cost for m batched operations on an n-object network. Acceptable at measured scales (see [stages/v0.2.md](stages/v0.2.md)); a copy-on-write or batched-apply design would require its own ADR. Do not benchmark future changes against this as-is without noting the debt.

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
