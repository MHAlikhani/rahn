<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Contributor Path

How a person grows in RAHN: **Newcomer → Contributor → Regular contributor → Subsystem maintainer → Core maintainer**, with a parallel **Researcher** track. The stages mirror the roles in [GOVERNANCE.md](GOVERNANCE.md); nothing here overrides that document, and the quote that governs advancement is its own: maintainers are added "by consensus of existing maintainers based on sustained, high-quality contribution".

## Newcomer

**What you do:** read, build, and run. No contribution obligation yet.

**What you learn, in order:**
1. [README.md](README.md) — what RAHN is and, just as important, what it is not.
2. [ARCHITECTURE.md](ARCHITECTURE.md) — the goals, trade-offs, and failure modes.
3. [docs/concepts.md](docs/concepts.md) — the ideas behind the architecture.
4. [GLOSSARY.md](GLOSSARY.md) — canonical terminology; implementation and docs must match it.
5. [docs/adr/](docs/adr/) — once an index and reading guide exists (see issue [#12](https://github.com/MHAlikhani/rahn/issues/12)), start there; otherwise begin with ADR 0002, 0003, and 0005.

**Concrete first actions:**
- `cargo build` and `cargo test` — the suite passes in full at v1.0 (136 tests plus ignored benchmark harnesses; docs/research/v1.0-review.md) and needs only a Rust toolchain.
- Work through [examples/cli-walkthrough.md](examples/cli-walkthrough.md) end to end.
- Run the guided demo, `bash examples/demo/demo.sh` (simulation only), and read [examples/demo/README.md](examples/demo/README.md).
- Ask questions in Discussions under Technical Questions.

**Advancement:** there is nothing to advance through; the first merged contribution makes you a Contributor.

## Contributor

**What you do:** pick a bounded task and land it. [docs/community/good-first-issues.md](docs/community/good-first-issues.md) curates entry points; issues labeled `good first issue` are chosen for bounded scope, no core-semantics changes, and clear verification.

**What you learn:** [CONTRIBUTING.md](CONTRIBUTING.md) — the fit questions every contribution must answer, the ground rules (determinism is non-negotiable; no AI in the core), and the review process. [docs/testing.md](docs/testing.md) — how tests are laid out and the rules for extending the suite.

**Concrete first actions:**
- Complete one good first issue end to end: issue, PR, tests, review.
- Read the review you receive as documentation of the project's standards.

**Advancement:** sustained, high-quality contribution (GOVERNANCE.md). There is no quota and no time box.

## Regular contributor / Researcher (parallel tracks)

**What you do:** own an area of attention rather than a single task. Two paths exist here and both are legitimate:

- **Regular contributor (engineering):** recurring work in one or more crates; reviews of others' PRs; issues with real designs behind them.
- **Researcher (research track):** the research corpus in [docs/research/](docs/research/) — research questions, experiments (E1–E7), the prior-art survey, the white paper. GOVERNANCE.md is explicit that Researchers "may hold no merge rights": the research track is real participation, not a waiting room for engineering roles. The research-proposal issue template and the Research Ideas discussion category are its entry points.

**What you learn:** [docs/spec/](docs/spec/) — the normative specification — for implementers; [docs/research/](docs/research/) (start with research-questions.md and limitations.md) for researchers; the ADRs governing your area.

**How advancement works:** a subsystem opens up when its owner steps down or when the work outgrows them; GOVERNANCE.md distributes ownership deliberately so no single person is a permanent bottleneck.

## Subsystem maintainer

**What you do:** own a crate or docs area (state, verification, execution backends, simulation, docs, research — the list in GOVERNANCE.md). Review PRs in your area, keep its spec and tests honest, and represent it in ADR discussions.

**What you learn:** everything that touches your subsystem — its crate, its spec documents, its ADRs, its recorded limitations.

**Advancement:** appointed by consensus of existing maintainers based on sustained, high-quality contribution (GOVERNANCE.md). Maintainers may step down at any time and are thanked, not erased.

## Core maintainer

**What you do:** own the architectural core, hold merge rights, and make final calls on ADRs. An ADR is accepted when at least one maintainer other than the author approves it (once the maintainer group exceeds one); accepted ADRs are immutable (GOVERNANCE.md).

**What you learn:** the whole decision record, and the discipline of writing decisions down instead of deciding in private.

## How RAHN reviews

Every PR is reviewed for three things (CONTRIBUTING.md):

- **Architectural fit** — does it strengthen verifiable, evolving network state? The maintainers decline features that violate the architectural model regardless of implementation quality; that is documented policy, not unfriendliness.
- **Determinism** — no nondeterministic iteration order, timestamps-as-logic, or hidden global state.
- **Test coverage** — tests close to semantics: unit, property, integration, deterministic fixtures, malformed-input tests, round-trips.

Design changes do not enter through PRs at all: they follow **design-proposal issue → ADR → spec/ARCHITECTURE update → implementation**. Implementation is never first.

## Where to start today

If the path above feels abstract, this is the shortest concrete version: build and test, walk the CLI example, read README → ARCHITECTURE → concepts → GLOSSARY, then pick one entry from [docs/community/good-first-issues.md](docs/community/good-first-issues.md). If nothing there fits, propose a task in Discussions — maintainers curate that file, and a well-argued proposal can become its next entry.
