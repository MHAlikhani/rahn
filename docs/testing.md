<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Testing Strategy

Testing is part of the architecture (DESIGN.md principle 19; charter §27). This document describes the strategy, the current suite, and the rules for extending it.

## Principles

1. **Correctness over speed** (v0.1 priority). Tests may be slow; wrong results may not.
2. **Tests live close to semantics.** Unit tests sit next to the code whose semantics they pin; end-to-end tests encode user-visible flows.
3. **Determinism extends to tests.** No wall-clock, no network, no system randomness in test inputs. Property tests use a seeded xorshift PRNG so any failure reproduces from the printed seed.
4. **Every regression gets a test.** Edge cases discovered in review or incident analysis are added as named tests referencing the scenario.
5. **"Verified" is only as good as its tests.** The test suite is the evidence behind the [Implemented] claims in the white paper and specs.

## Current suite layout (v0.1)

| Location | What it pins |
|---|---|
| `crates/rahn-core/src/model.rs` (`#[cfg(test)]`) | Identifier rules, link normalization, self-loop prohibition, node-in-use protection |
| `crates/rahn-state/src/canonical.rs` | Canonical round-trip exactness and idempotence; rejection of unknown versions, trailing bytes, truncation, unsorted content, dangling endpoints, unnormalized endpoints |
| `crates/rahn-state/src/identity.rs` | Identity stability, hex round-trip, different-states-different-ids |
| `crates/rahn-state/src/transition.rs` | Purity (input untouched), totality, structured rejections, duplicate/missing-object rejections |
| `crates/rahn-state/src/commit.rs` | Commit record round-trip, parent-count limits, trailing-byte rejection |
| `crates/rahn-state/src/diff.rs` | Semantic diff: additions, removals, metadata changes, empty diff |
| `crates/rahn-state/src/history.rs` | Common ancestor: linear, branched, diamond, disjoint histories |
| `crates/rahn-state/tests/properties.rs` | Seeded property tests over random networks: canonical round-trip + identity stability (200 seeds); diff-derived operations reconstruct the target (200 seeds); merge of disjoint removals is commutative and valid (100 seeds) |
| `crates/rahn-store/src/lib.rs` | Content-addressed round-trip, deduplication, commit round-trip, **corrupted-object refusal**, refs/HEAD, index, re-init refusal |
| `crates/rahn-verify/src/constitution.rs` | Constitution parsing: round-trip, unknown keywords, arity, invalid ids |
| `crates/rahn-verify/src/invariants.rs` | Each invariant's pass and fail behavior, evidence strings, determinism, structural floor always checked |
| `crates/rahn-verify/src/merge.rs` | Fail-closed merge semantics: disjoint merges, identical-effect coalescing, conflict rejection, fail-closed application errors, metadata preservation |
| `crates/rahn-sim/src/lib.rs` | Plan ordering (removals before additions), empty plans, inspectable output |
| `crates/rahn-cli/src/args.rs` | Strict argument parsing: every subcommand, every rejection class |
| `crates/rahn-cli/tests/end_to_end.rs` | Full lifecycle, checkout, diverged branch merge, constitution-gated commits (including on branch states), `--force` recovery, invalid-transition explanations, corrupted-object detection via the CLI, cross-run determinism of commit ids |

Run everything:

```console
cargo test          # 81 tests
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

## What the tests deliberately do NOT cover yet

- **Benchmarks**: none (charter: no premature optimization). Benchmarks arrive with `benches/` when a hypothesis needs them (docs/research/benchmark-methodology.md).
- **Fuzzing**: structured malformed-input tests exist, but no continuous fuzzer. Planned when the parse surface grows (Stage 2+).
- **Cross-platform CI matrix**: determinism tests are designed for it; CI (GitHub Actions) runs on Linux and Windows.
- **Model checking / formal verification**: future methodology (docs/research/experiment-plan.md).

## Rules for contributors

- A change to canonical serialization that alters bytes for existing states MUST bump `CANONICAL_FORMAT_VERSION` via ADR and update pinned test vectors with justification.
- A new invariant MUST come with pass tests, fail tests, and evidence-string assertions.
- A new CLI command MUST come with usage-string coverage and at least one end-to-end flow.
- A bug fix MUST reference a regression test; the fix PR shows the test failing before the fix.
