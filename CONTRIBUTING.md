<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Contributing to RAHN

Thank you for considering contributing. RAHN is an architectural research project first; contributions are judged by whether they strengthen the core idea: **verifiable, evolving network state**.

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
- **Tests close to semantics:** unit, property, integration, deterministic fixtures, malformed-input tests, serialization round-trips. Correctness outranks speed (DESIGN.md priority order).
- **Documentation is part of the architecture.** Architectural changes require an ADR plus updates to ARCHITECTURE.md and (when significant) the white paper. Do not silently change core assumptions.

## Developer setup

Install a stable Rust toolchain via [rustup](https://rustup.rs), then:

```console
cargo build
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

The benchmark harness runs with `cargo test -p rahn-state --release --test scaling -- --ignored --nocapture`. No other toolchain is required for documentation-only contributions. The workspace layout, the full command set (including the ignored benchmark harnesses and the runnable demo), and the pull-request verification matrix are in [DEVELOPMENT.md](DEVELOPMENT.md).

## Process

1. Open an issue using the relevant template (bug, feature, or **design proposal**). Design proposals that touch core semantics go through the ADR process.
2. Discuss; get alignment before large implementations.
3. Submit a pull request with tests and, where behavior is user-visible, documentation updates.
4. Every PR is reviewed for architectural fit, determinism, and test coverage.

Where a conversation belongs: questions → Discussions (Technical Questions); early ideas → Architecture & Design or Research Ideas; design drafts → Architecture & Design. A discussion never replaces an ADR. The formal path stays: issue template → ADR → spec/ARCHITECTURE → implementation. Categories and their rules: [docs/community/discussions.md](docs/community/discussions.md).

Good first issues are labeled `good first issue` (GitHub's default).

## Agent-assisted contributions

Contributions made with coding agents (any tool) are welcome and held to the same bar as any other contribution.

- The human submitting the pull request is responsible for the change and must be able to explain and defend it, including why each design choice is correct.
- Disclose agent assistance in the pull request: which parts were agent-produced, and with what tool.
- Agents must not weaken tests, CI, determinism, or architectural boundaries to make a change pass. A change that does so is rejected regardless of its quality otherwise.
- Architectural changes still follow design-proposal issue → accepted ADR → spec/ARCHITECTURE update → implementation, no matter who or what authored them. Accepted ADRs are immutable.
- Documentation claims must be verified against the implementation; anything unverified is reported as unverified.

[AGENTS.md](AGENTS.md) is how agents work in this repository (reading order, commands, prohibitions, reporting); this section is the project's policy for the contributions they produce.

## Licensing of Contributions

By contributing source code to RAHN, contributors make their contributions available under the Apache License 2.0.

By contributing documentation or other non-software content, contributors make those contributions available under CC BY 4.0, unless the contribution is explicitly identified otherwise.

Contributors retain copyright in their contributions; the project does not require copyright transfer. There is no CLA. The full policy is in [docs/licensing.md](docs/licensing.md); the licensing decision is recorded in [docs/adr/0009-licensing-model.md](docs/adr/0009-licensing-model.md).

## Reporting security issues

Do not open public issues for security problems — see [SECURITY.md](SECURITY.md).

## Conduct

Be respectful and precise. The community standard is [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## Community structure

- [CONTRIBUTOR_PATH.md](CONTRIBUTOR_PATH.md) — how to grow from a first contribution to maintainer, aligned with GOVERNANCE.md.
- [docs/community/good-first-issues.md](docs/community/good-first-issues.md) — curated entry points for a first contribution.
- [docs/community/issue-drafts.md](docs/community/issue-drafts.md) — planned issues, drafted and grounded before filing.
- [docs/community/discussions.md](docs/community/discussions.md) — discussion categories, and where questions belong.
