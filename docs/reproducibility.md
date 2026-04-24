<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Reproducibility

Reproducibility is a design goal of the architecture, not an afterthought (charter §29). This document states what is reproducible today, how, and what is not yet.

## What is reproducible in v0.1

### 1. State identity and history

Given the same sequence of operations, RAHN produces byte-identical states, state ids, and commit ids — on any machine, at any time. This is enforced by:

- **Canonical serialization** (ADR 0003): total ordering everywhere, no timestamps, no hosts, no floating point, no ambient information.
- **Content-addressed identity** (ADR 0002): ids are SHA-256 over canonical bytes, including a format-version tag.
- **Test enforcement**: `determinism_same_commands_same_commit_ids` runs identical histories in separate repositories and asserts identical commit ids; `property_canonical_round_trip_and_identity` asserts identity stability over 200 seeded random networks.

Reproduce it yourself:

```console
cargo test -p rahn-cli determinism
```

### 2. Property tests

Property tests use a seeded xorshift PRNG (no `proptest`, no system randomness). A failing seed prints in the panic message and replays exactly. This keeps the "property-based testing where useful" requirement (docs/testing.md) deterministic by construction.

### 3. The test suite itself

`cargo test` requires only a Rust toolchain and network access to fetch the single dependency (`sha2`) on first build. No databases, services, emulators, or OS privileges.

### 4. Experiment reproducibility (policy)

Every future experiment must record — before its results are quotable — purpose, hypothesis, environment (hardware, OS, toolchain), repository commit, inputs (or generation procedure with seeds), method, raw outputs, and limitations (docs/research/experiment-plan.md, docs/research/benchmark-methodology.md). Numbers without this block are void per policy.

## What is not yet reproducible

| Area | Status | Plan |
|---|---|---|
| Release binaries | Not yet built reproducibly (Rust builds embed some paths by default) | Evaluate `cargo` reproducible-build options at the first tagged release with artifacts |
| Real-network experiments | None exist (no real execution before Stage 3) | Namespace-based experiments will be scripted end-to-end in-repo (E6) |
| Benchmarks | None exist | Will follow benchmark-methodology.md with recorded environments |
| Distributed replay | Out of scope (Stage 6+) | Requires the consistency model to be derived first (RQ7) |

## Canonical-format evolution rule

The canonical format is version-tagged. Any change that alters canonical bytes for existing states MUST (a) be recorded as an ADR, (b) bump `CANONICAL_FORMAT_VERSION`, and (c) update pinned vectors with justification. Identity of *historical* states remains stable because old objects are stored with their own bytes and verified against their own hashes — format evolution changes only newly written objects, with migration handled explicitly, never implicitly.
