<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Development Guide

How the workspace is laid out, what to run, and how to land a change. Contribution policy, fit questions, and review expectations are in [CONTRIBUTING.md](CONTRIBUTING.md); test strategy and suite layout are in [docs/testing.md](docs/testing.md). This document references both instead of repeating them.

## Workspace layout

A Cargo workspace (edition 2021) with one binary, `rahn`. One runtime dependency across the workspace: `sha2`. Crate boundaries are architectural (DESIGN.md 19: experimental features behind clear boundaries).

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

Supporting paths: `docs/adr/` (the authority), `docs/spec/` (normative), `docs/research/` (evidence), `docs/community/` (entry points), `examples/` (walkthrough, demo, CI example), `scripts/` (repository checks), `.github/workflows/ci.yml` (gates).

## Commands

| Task | Command |
|---|---|
| Build | `cargo build` (or `cargo build --release`) |
| Full test suite | `cargo test --all-features` |
| One crate | `cargo test -p rahn-verify` |
| Formatting gate | `cargo fmt --check` |
| Lint gate | `cargo clippy --all-targets -- -D warnings` |
| Benchmark harness (ignored, release) | `cargo test -p rahn-state --release --test scaling -- --ignored --nocapture` |
| Other ignored harnesses | `--test obs_scaling`, `--test causal_scaling` (`-p rahn-state`); `-p rahn-dist --test sync_scaling`; `-p rahn-exec --test linux_real` (Linux, requires root) |
| Runnable demo | `bash examples/demo/demo.sh` — simulation only, no root, no network |
| CLI from source | `cargo run -q -p rahn-cli -- <args>` |
| Documentation checks | `python3 scripts/check_docs.py` |

The CLI session is transcribed in [examples/cli-walkthrough.md](examples/cli-walkthrough.md); the demo's expected output and per-step specification references are in [examples/demo/README.md](examples/demo/README.md).

## Making a change safely

1. **Add the test first where you can.** A failing test pins the behavior and becomes the regression evidence; [docs/testing.md](docs/testing.md) states the rules per artifact (invariant, CLI command, canonical format, bug fix).
2. **Keep it inside one crate's responsibility.** The table above is the boundary; if a change needs two crates to change meaning, an ADR is likely required.
3. **Know which changes require an ADR:** core semantics (state identity, canonical serialization, transitions, merge, verification); a new invariant, backend, or public `rahn-sdk` surface change; a new dependency or crate; anything that changes canonical bytes for existing states. The path is design-proposal issue → accepted ADR → [docs/spec/](docs/spec/) and [ARCHITECTURE.md](ARCHITECTURE.md) updates (plus the white paper where significant) → implementation. Accepted ADRs are immutable; supersede with a new ADR that references the old one.
4. **Canonical-format changes** require bumping `CANONICAL_FORMAT_VERSION` (`crates/rahn-state/src/canonical.rs`) via ADR and updating the pinned test vectors with justification — see [docs/testing.md](docs/testing.md).
5. **Keep the documents in step.** When implementation and documentation disagree, the discrepancy is investigated, not assumed to favor the code ([GOVERNANCE.md](GOVERNANCE.md)); `python3 scripts/check_docs.py` catches broken links, missing SPDX headers, and roadmap drift.

## Verification matrix for a pull request

| Change | Run |
|---|---|
| Any | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --all-features` |
| Documentation, links, roadmap, agent files | `python3 scripts/check_docs.py` |
| User-visible behavior | `bash examples/demo/demo.sh` and, where relevant, the [CLI walkthrough](examples/cli-walkthrough.md) |
| Execution backend or host-safety | `cargo test -p rahn-exec`; the `linux-execution` CI job covers the namespace path |
| Canonical serialization | Pinned vectors plus `cargo test -p rahn-state`; state the version-bump justification |
| New or changed document | CC-BY-4.0 SPDX line as the first line (after YAML front matter in issue templates) |

CI runs the same gates (`.github/workflows/ci.yml`: `test`, `linux-execution`, `license-check`, `docs`). Evidence to include in a pull request: what changed and why, the exact commands run, their results including failures, what remains unverified, ADR/spec impact, and which documents were updated. [.github/PULL_REQUEST_TEMPLATE.md](.github/PULL_REQUEST_TEMPLATE.md) encodes that checklist.

## For coding agents

[AGENTS.md](AGENTS.md) is the canonical, vendor-neutral instruction file: repository map, mandatory reading order, non-negotiable rules, prohibitions, and reporting expectations. Read it before acting rather than working from this guide alone. `CLAUDE.md` and [.github/copilot-instructions.md](.github/copilot-instructions.md) point there. Agent-assisted contributions are welcome and held to the same bar; the policy is in [CONTRIBUTING.md](CONTRIBUTING.md).
