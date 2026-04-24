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
- **No network side effects** outside approved execution backends (none exist before v0.3).
- **No AI in the core.**
- **Small modules, strong types, explicit errors.** Make invalid states hard to represent.
- **Tests close to semantics:** unit, property, integration, deterministic fixtures, malformed-input tests, serialization round-trips. For v0.1, correctness outranks speed.
- **Documentation is part of the architecture.** Architectural changes require an ADR plus updates to ARCHITECTURE.md and (when significant) the white paper. Do not silently change core assumptions.

## Developer setup

(Once the Rust workspace lands: install a stable Rust toolchain via [rustup](https://rustup.rs), then `cargo build && cargo test`.) No other toolchain is required for documentation-only contributions.

## Process

1. Open an issue using the relevant template (bug, feature, or **design proposal**). Design proposals that touch core semantics go through the ADR process.
2. Discuss; get alignment before large implementations.
3. Submit a pull request with tests and, where behavior is user-visible, documentation updates.
4. Every PR is reviewed for architectural fit, determinism, and test coverage.

Good first issues are labeled `good-first-issue`.

## Licensing of Contributions

By contributing source code to RAHN, contributors make their contributions available under the Apache License 2.0.

By contributing documentation or other non-software content, contributors make those contributions available under CC BY 4.0, unless the contribution is explicitly identified otherwise.

Contributors retain copyright in their contributions; the project does not require copyright transfer. There is no CLA. The full policy is in [docs/licensing.md](docs/licensing.md); the licensing decision is recorded in [docs/adr/0009-licensing-model.md](docs/adr/0009-licensing-model.md).

## Reporting security issues

Do not open public issues for security problems — see [SECURITY.md](SECURITY.md).

## Conduct

Be respectful and precise. The community standard is [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
