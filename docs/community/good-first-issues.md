<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Good First Issues

Curated entry points for a first contribution. RAHN labels these tasks `good first issue` (GitHub's default).

## What makes a task beginner-friendly here

A first task in this repository should have:

- **Bounded scope** — a deliverable you can finish without touching core semantics (state identity, canonical serialization, transitions, merge, verification).
- **Clear verification** — success is checkable: a test passes, a document exists with named sections, a CI matrix goes green.
- **Existing scaffolding** — the task extends or audits something that already exists; you are not designing an abstraction.

Anything that changes core semantics goes through the ADR path instead and is never labeled `good first issue`.

## Current set

These are live issues, listed with their numbers. The drafts they came from, and the rest of the filed backlog, are in [issue-drafts.md](issue-drafts.md).

- **"Deepen the prior-art survey with a systems/formal-methods literature pass"** - [issue #11](https://github.com/MHAlikhani/rahn/issues/11)
  *Why a good start:* reading and writing only — no Rust, no toolchain beyond a Markdown editor.
  *What you'll learn:* the research-corpus discipline this project runs on: claims scoped to sources, provisional claims marked, novelty never manufactured.
  *Where to ask:* Discussions → Technical Questions.

- **"Write an ADR index and reading guide for docs/adr/"** - [issue #12](https://github.com/MHAlikhani/rahn/issues/12)
  *Why a good start:* 19 short documents to summarize, grouped by theme; nothing may be changed, only described and cross-linked.
  *What you'll learn:* the ADRs themselves — the fastest way to learn the architecture is to index its decisions.
  *Where to ask:* Discussions → Technical Questions.

- **"Audit rahn-sdk public API documentation"** - [issue #13](https://github.com/MHAlikhani/rahn/issues/13)
  *Why a good start:* one crate, a checklist-shaped task (every public item: semantics plus a doctest), and a compiler lint (`missing_docs`) that tells you what is left.
  *What you'll learn:* the public API surface, the IR, and how doctests double as executable documentation.
  *Where to ask:* Discussions → Technical Questions.

- **"Add macOS to the CI determinism matrix"** - [issue #14](https://github.com/MHAlikhani/rahn/issues/14)
  *Why a good start:* a one-line workflow change plus verifying the seeded property tests still pass — and a clearly defined failure path (divergence becomes a filed determinism finding, not a stalled PR).
  *What you'll learn:* how determinism is test-enforced and what the CI gate actually checks.
  *Where to ask:* Discussions → Technical Questions.

- **"Error-message audit against the explainable-failures principle"** - [issue #15](https://github.com/MHAlikhani/rahn/issues/15)
  *Why a good start:* enumerate, audit, fix, and tabulate — the deliverable (a before/after table in the PR) is defined before you start.
  *What you'll learn:* every rejection path in the crates, and the design principle (DESIGN.md 14) that governs them.
  *Where to ask:* Discussions → Technical Questions.

## After the first one or two

The natural progression, by level:

1. **Beginner (this file):** one or two of the tasks above.
2. **Intermediate:** the experiment studies and fuzzing work from the issue drafts — E2 verification-latency measurement ([#17](https://github.com/MHAlikhani/rahn/issues/17)), E3 merge conflict-detection study ([#18](https://github.com/MHAlikhani/rahn/issues/18)), and structured fuzzing for the canonical deserializer ([#16](https://github.com/MHAlikhani/rahn/issues/16)) These require Rust and the benchmark methodology, but have defined methods and acceptance criteria.
3. **Design work:** proposals through the RFC path (design-proposal issue → ADR), ideally after an intermediate issue has shown you how the core constraints bite. Start in Discussions under Architecture & Design before filing the template.

## Curation

Maintainers curate this file. When a listed issue is claimed, completed, or superseded, the entry is updated or removed. If nothing here fits you, propose a task in Discussions — a well-argued proposal with bounded scope and clear verification can become the next entry.
