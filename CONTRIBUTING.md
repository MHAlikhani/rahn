<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Contributing to RAHN

Thank you for considering contributing. RAHN is an architectural research project first; contributions are judged by whether they strengthen the core idea: **verifiable, evolving network state**.

This document covers how to decide whether a contribution fits, how the workspace is laid out, what to run, how to land a change, and how the project is licensed. Test strategy and suite layout are in [docs/testing.md](docs/testing.md).

## How to decide if a contribution fits

Every feature must answer: *Does this strengthen the concept of verifiable, evolving network state?* If not, defer it.
Every abstraction must answer: *Is this fundamental?* If not, keep it outside the core.
Every automation feature must answer: *Can it remain deterministic and explainable?*
Every AI feature must answer: *Can the system remain correct without it?*
Every networking integration must answer: *Is it an execution backend for the architecture, or is it accidentally becoming the architecture itself?*

The maintainers will decline features that violate the architectural model, regardless of implementation quality. This is documented policy, not unfriendliness.

## Areas

Core runtime · state model · verification · networking adapters · eBPF · simulation · protocol · documentation · research · benchmarking · SDK · tooling.

## Ground rules

- **Determinism is non-negotiable.** No nondeterministic iteration order, timestamps-as-logic, or hidden global state.
- **No network side effects** outside approved execution backends (simulation is the default; the `linux-ns` backend is the approved opt-in, ADR 0012/0016).
- **No AI in the core.**
- **Small modules, strong types, explicit errors.** Make invalid states hard to represent.
- **Tests close to semantics:** unit, property, integration, deterministic fixtures, malformed-input tests, serialization round-trips. Correctness outranks speed (the priority order is set out in [ARCHITECTURE.md](ARCHITECTURE.md)).
- **Documentation is part of the architecture.** Architectural changes require an ADR plus updates to ARCHITECTURE.md and (when significant) the white paper. Do not silently change core assumptions.

## Workspace layout

A Cargo workspace (edition 2021) with one binary, `rahn`. One runtime dependency across the workspace: `sha2`. Crate boundaries are architectural: experimental features live behind clear crate and feature-gate boundaries.

| Crate | Responsibility |
|---|---|
| `rahn-core` | Object model: nodes, interfaces, links, network, state value. No internal dependencies. |
| `rahn-state` | Canonical serialization (format v2), state identity, transitions, commits, diff, history, graph queries, observations, causal records. |
| `rahn-store` | Content-addressed local persistence: objects, commits, refs/HEAD, index. |
| `rahn-verify` | Invariant engine, constitution parsing, fail-closed semantic merge. |
| `rahn-sim` | Simulation-only execution planning. Default backend; never touches the host. |
| `rahn-exec` | Opt-in isolated Linux namespace backend (ADR 0012, ADR 0016). |
| `rahn-dist` | Peer synchronization over content-addressed history (ADR 0015). |
| `rahn-sdk` | Curated public API facade (ADR 0018); the semver-stable surface for external code. |
| `rahn-cli` | The `rahn` binary: argument parsing, command dispatch, report formatting. |

Supporting paths: `docs/adr/` (the decision record), `docs/spec/` (normative), `docs/research/` (evidence), `examples/` (walkthrough, demo, CI example), `scripts/` (repository checks), `.github/workflows/ci.yml` (gates).

## Developer setup

Install a stable Rust toolchain via [rustup](https://rustup.rs), then:

```console
cargo build
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

| Task | Command |
|---|---|
| Build | `cargo build` (or `cargo build --release`) |
| Full test suite | `cargo test --all-features` |
| One crate | `cargo test -p rahn-verify` |
| Formatting gate | `cargo fmt --check` |
| Lint gate | `cargo clippy --all-targets -- -D warnings` |
| Benchmark harness (ignored, release) | `cargo test -p rahn-state --release --test scaling -- --ignored --nocapture` |
| Other ignored harnesses | `--test obs_scaling`, `--test causal_scaling` (`-p rahn-state`); `-p rahn-dist --test sync_scaling`; `-p rahn-exec --test linux_real` (Linux, requires root) |
| Runnable demo | `bash examples/demo/demo.sh` - simulation only, no root, no network |
| CLI from source | `cargo run -q -p rahn-cli -- <args>` |
| Documentation checks | `python3 scripts/check_docs.py` |

No other toolchain is required for documentation-only contributions. The CLI session is transcribed in [examples/cli-walkthrough.md](examples/cli-walkthrough.md); the demo's expected output and per-step specification references are in [examples/demo/README.md](examples/demo/README.md).

## Making a change safely

1. **Add the test first where you can.** A failing test pins the behavior and becomes the regression evidence; [docs/testing.md](docs/testing.md) states the rules per artifact (invariant, CLI command, canonical format, bug fix).
2. **Keep it inside one crate's responsibility.** The table above is the boundary; if a change needs two crates to change meaning, an ADR is likely required.
3. **Know which changes require an ADR:** core semantics (state identity, canonical serialization, transitions, merge, verification); a new invariant, backend, or public `rahn-sdk` surface change; a new dependency or crate; anything that changes canonical bytes for existing states. The path is design-proposal issue → accepted ADR → [docs/spec/](docs/spec/) and [ARCHITECTURE.md](ARCHITECTURE.md) updates (plus the white paper where significant) → implementation. Accepted ADRs are immutable; supersede with a new ADR that references the old one.
4. **Canonical-format changes** require bumping `CANONICAL_FORMAT_VERSION` (`crates/rahn-state/src/canonical.rs`) via ADR and updating the pinned test vectors with justification - see [docs/testing.md](docs/testing.md).
5. **Keep the documents in step.** When implementation and documentation disagree, the discrepancy is investigated, not assumed to favor the code ([GOVERNANCE.md](GOVERNANCE.md)); `python3 scripts/check_docs.py` catches broken links and missing SPDX headers.

## Verification matrix for a pull request

| Change | Run |
|---|---|
| Any | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --all-features` |
| Documentation, links, repository checks | `python3 scripts/check_docs.py` |
| User-visible behavior | `bash examples/demo/demo.sh` and, where relevant, the [CLI walkthrough](examples/cli-walkthrough.md) |
| Execution backend or host-safety | `cargo test -p rahn-exec`; the `linux-execution` CI job covers the namespace path |
| Canonical serialization | Pinned vectors plus `cargo test -p rahn-state`; state the version-bump justification |
| New or changed document | CC-BY-4.0 SPDX line as the first line (after YAML front matter in issue templates) |

CI runs the same gates (`.github/workflows/ci.yml`: `test`, `linux-execution`, `license-check`, `docs`). Evidence to include in a pull request: what changed and why, the exact commands run, their results including failures, what remains unverified, ADR/spec impact, and which documents were updated. [.github/PULL_REQUEST_TEMPLATE.md](.github/PULL_REQUEST_TEMPLATE.md) encodes that checklist.

## How a contributor grows

The stages mirror the roles in [GOVERNANCE.md](GOVERNANCE.md); nothing here overrides that document, and the rule that governs advancement is its own: maintainers are added "by consensus of existing maintainers based on sustained, high-quality contribution".

**Newcomer.** Read, build, and run - no contribution obligation yet. In order: [README.md](README.md) (what RAHN is, and what it is not), [ARCHITECTURE.md](ARCHITECTURE.md), [docs/concepts.md](docs/concepts.md), [GLOSSARY.md](GLOSSARY.md), then [docs/adr/](docs/adr/) - begin with ADR 0002, 0003, and 0005. Concrete first actions: `cargo build` and `cargo test` (the suite passes in full at v1.0 and needs only a Rust toolchain), work through [examples/cli-walkthrough.md](examples/cli-walkthrough.md) end to end, and run `bash examples/demo/demo.sh`. The first merged contribution makes you a Contributor.

**Contributor.** Pick a bounded task and land it. Entry points are issues labeled [`good first issue`](https://github.com/MHAlikhani/rahn/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22), chosen for bounded scope, no core-semantics changes, and clear verification. Complete one end to end - issue, PR, tests, review - and read the review you receive as documentation of the project's standards. Advancement is sustained, high-quality contribution; there is no quota and no time box.

**Regular contributor / Researcher (parallel tracks).** Own an area of attention rather than a single task. Both tracks are legitimate: recurring engineering work in one or more crates with reviews of others' PRs, or the research corpus in [docs/research/](docs/research/) (research questions, experiments E1-E7, the prior-art survey, the white paper). [GOVERNANCE.md](GOVERNANCE.md) is explicit that Researchers "may hold no merge rights" - the research track is real participation, not a waiting room for engineering roles.

**Subsystem maintainer.** Own a crate or docs area (state, verification, execution backends, simulation, docs, research). Review PRs in your area, keep its spec and tests honest, and represent it in ADR discussions. Appointed by consensus of existing maintainers.

**Core maintainer.** Own the architectural core, hold merge rights, and make final calls on ADRs. An ADR is accepted when at least one maintainer other than the author approves it (once the maintainer group exceeds one); accepted ADRs are immutable.

A subsystem opens up when its owner steps down or when the work outgrows them; [GOVERNANCE.md](GOVERNANCE.md) distributes ownership deliberately so no single person is a permanent bottleneck.

## Process

1. Open an issue using the relevant template (bug, feature, or **design proposal**). Design proposals that touch core semantics go through the ADR process.
2. Discuss; get alignment before large implementations.
3. Submit a pull request with tests and, where behavior is user-visible, documentation updates.
4. Every PR is reviewed for architectural fit, determinism, and test coverage.

Questions and early ideas belong in [GitHub Discussions](https://github.com/MHAlikhani/rahn/discussions); bug reports, feature requests, and design drafts belong in issues. A discussion never replaces an ADR: the formal path stays issue template → ADR → spec/ARCHITECTURE → implementation.

Design changes do not enter through PRs at all - they follow **design-proposal issue → ADR → spec/ARCHITECTURE update → implementation**. Implementation is never first.

## Licensing of Contributions

By contributing source code to RAHN, contributors make their contributions available under the Apache License 2.0.

By contributing documentation or other non-software content, contributors make those contributions available under CC BY 4.0, unless the contribution is explicitly identified otherwise.

Contributors retain copyright in their contributions; the project does not require copyright transfer. There is no CLA. The full policy is in [docs/licensing.md](docs/licensing.md); the licensing decision is recorded in [docs/adr/0009-licensing-model.md](docs/adr/0009-licensing-model.md).

## Reporting security issues

Do not open public issues for security problems - see [SECURITY.md](SECURITY.md).

## Conduct

Be respectful and precise. The community standard is [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
