<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# AGENTS.md — how to work in RAHN

Canonical, vendor-neutral instructions for coding agents. `CLAUDE.md` and `.github/copilot-instructions.md` point here; this file is the source.

RAHN is a stateful execution architecture for evolving networks: network state is a deterministic, content-addressed value, changed only by explicit transitions, gated by a constitution of invariants, and executed simulation-first. It is an architectural research project in a Rust workspace, released as v1.0.0.

**Boundary:** RAHN is a stable v1.0 architecture — not a production network controller, not a router/SDN controller/IaC tool, and not "Git for networks". Every tracked `*.md` except root `README.md` begins with the CC-BY-4.0 SPDX line; source is Apache-2.0.

## Repository map

| Path | Owns |
|---|---|
| `crates/rahn-core` | Object model: nodes, interfaces, links, network, state value |
| `crates/rahn-state` | Canonical serialization (format v2), state identity, transitions, commits, diff, history, graph queries, observations, causal records |
| `crates/rahn-store` | Content-addressed local persistence: objects, commits, refs/HEAD, index |
| `crates/rahn-verify` | Invariant engine, constitution parsing, fail-closed semantic merge |
| `crates/rahn-sim` | Simulation-only execution planning; default backend; never touches the host |
| `crates/rahn-exec` | Opt-in isolated Linux namespace backend (ADR 0012/0016) |
| `crates/rahn-dist` | Peer sync over content-addressed history (ADR 0015) |
| `crates/rahn-sdk` | Curated public API facade (ADR 0018); the semver-stable surface |
| `crates/rahn-cli` | The `rahn` binary and its argument parsing |
| `docs/adr/` | 19 immutable architecture decision records — the authority |
| `docs/spec/` | Normative specification (state, objects, transitions, serialization, storage, invariants, execution, observation, causality, protocol, security) |
| `docs/research/` | Problem statement, prior art, research questions, limitations, v1.0 review, stage reports |
| `docs/community/` | Discussion categories, curated good-first-issues, grounded issue drafts |
| `docs/whitepaper/` | Claim-status-marked white paper |
| `examples/` | `cli-walkthrough.md`, `network-ci.yml`, runnable `demo/demo.sh` |
| `.github/workflows/ci.yml` | CI: `test` (fmt, clippy, tests on Ubuntu + Windows), `linux-execution`, `license-check`, `docs` |

## Mandatory reading order

1. `README.md` — what RAHN is, and what it is not.
2. `ARCHITECTURE.md` — goals, chosen design, trade-offs, failure modes.
3. `GLOSSARY.md` — canonical terminology; code and docs must match it.
4. `docs/research/limitations.md` — the honesty baseline; never claim past it.
5. `CONTRIBUTING.md` — fit questions, ground rules, process, licensing.
6. `DEVELOPMENT.md` — layout, commands, how to make a change safely.
7. `docs/adr/` — the ADRs governing the area you touch (start with 0002, 0003, 0005, 0019).
8. `docs/testing.md` — suite layout and the rules for extending it.

## Non-negotiable rules

- **Determinism.** No nondeterministic iteration order, timestamps-as-logic, wall-clock, RNG, or hidden global mutable state. Same inputs produce byte-identical states, ids, and reports.
- **Explicit transitions.** Committed state changes only through the Operation vocabulary; nothing mutates a state in place.
- **No AI in the core.** The system must remain correct and useful without any model.
- **No network side effects by default.** Simulation is the default; real execution is opt-in (`--execute --yes-i-know`) against isolated Linux namespaces only.
- **Verify before execute.** The execution path is structurally unreachable without a verification result.
- **Tests close to semantics.** Unit, property (seeded), integration, malformed-input, and round-trip tests. A behavior change without a test is incomplete.
- **SPDX headers.** New `*.md` files start with `<!-- SPDX-License-Identifier: CC-BY-4.0 -->` (after YAML front matter in issue templates). New source files start with `// SPDX-License-Identifier: Apache-2.0`.
- **Architecture goes through the record.** Design-proposal issue → accepted ADR → `docs/spec/` + `ARCHITECTURE.md` (+ white paper where significant) → implementation. Implementation is never first, and accepted ADRs are immutable.

## Commands

```console
cargo build                                   # debug build of the workspace
cargo test --all-features                     # full suite (136 tests plus ignored harnesses at v1.0)
cargo fmt --check                             # formatting gate
cargo clippy --all-targets -- -D warnings     # lint gate
cargo test -p rahn-state --release --test scaling -- --ignored --nocapture   # one benchmark harness
bash examples/demo/demo.sh                    # end-to-end workflow, simulation only
python3 scripts/check_docs.py                 # links, SPDX headers, roadmap drift, llms.txt
```

No other toolchain is required for documentation-only work. Full guidance: `DEVELOPMENT.md`.

## Never

- Rewrite, delete, or renumber an accepted ADR; supersede it with a new ADR that references it.
- Weaken, skip, or delete a test, assertion, or CI gate to make a change pass. That change is rejected.
- Change `CANONICAL_FORMAT_VERSION` or the canonical byte format without an ADR and pinned-vector updates.
- Introduce a dependency, crate, or abstraction without an ADR where the bar requires one.
- Claim a result you did not verify; present a limitation as solved; imply a schedule for any post-v1.0 candidate.
- Commit, push, open issues, or open pull requests unless the human asked for it.
- Add AI as a dependency of the core, or describe RAHN as a production controller.

## Picking work

- `docs/community/good-first-issues.md` — curated, bounded, clearly verifiable entry points (label `good first issue`).
- `docs/community/issue-drafts.md` — grounded drafts; each becomes one issue, and filing does not schedule it.
- `roadmap.json` — machine-readable stages, post-v1.0 candidates, open experiments, known gaps.
- `docs/research/research-questions.md` — RQ status; `docs/research/experiment-plan.md` — E-program.
- Nothing post-v1.0 is scheduled or promised (ADR 0019). A candidate stays a candidate until its ADR is accepted.

## Reporting results

- Give evidence per claim: the exact command, its exit status, and the relevant output. Verified and unverified must be visibly distinct.
- State deviations from an instruction and why; do not silently substitute an approach.
- When a check fails, report the failing output rather than describing it as passing.
- Leave the change uncommitted unless asked to commit. The maintainer commits.
