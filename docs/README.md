<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Documentation

RAHN treats network state as a first-class value: deterministic, content-addressed, changed only by explicit transitions, verified against a constitution of invariants, and executed simulation-first.

This page is the index. Pick the path that matches why you are here.

## Start here

| If you are... | Read, in this order |
|---|---|
| Evaluating the idea | [../README.md](../README.md) → [concepts.md](concepts.md) → [../ARCHITECTURE.md](../ARCHITECTURE.md) → [research/limitations.md](research/limitations.md) |
| A contributor | [../CONTRIBUTING.md](../CONTRIBUTING.md) → [testing.md](testing.md) → [../ARCHITECTURE.md](../ARCHITECTURE.md) → [adr/](adr/) |
| A researcher | [research/problem-statement.md](research/problem-statement.md) → [research/research-questions.md](research/research-questions.md) → [research/prior-art.md](research/prior-art.md) → [whitepaper/RAHN-Whitepaper.md](whitepaper/RAHN-Whitepaper.md) |
| Building against the SDK | [../crates/rahn-sdk/src/lib.rs](../crates/rahn-sdk/src/lib.rs) → [spec/state.md](spec/state.md) → [spec/transitions.md](spec/transitions.md) |

New to the vocabulary? [../GLOSSARY.md](../GLOSSARY.md) is the canonical terminology; implementation and docs are expected to match it.

## What is in this directory

| Path | What it holds |
|---|---|
| [concepts.md](concepts.md) | The ideas behind the architecture, in prose |
| [adr/](adr/) | 19 Architecture Decision Records - the decision record |
| [spec/](spec/) | Normative specification: state, objects, transitions, serialization, storage, invariants, constitution, execution, observation, causality, protocol, security |
| [research/](research/) | Problem statement, prior art, research questions, hypotheses, experiments, benchmark methodology, per-stage evidence, limitations |
| [whitepaper/RAHN-Whitepaper.md](whitepaper/RAHN-Whitepaper.md) | Technical white paper, with claim-status markers |
| [testing.md](testing.md) | Test strategy and suite layout |
| [reproducibility.md](reproducibility.md) | How the project's results are reproduced |
| [licensing.md](licensing.md) | The licensing model |
| [third-party.md](third-party.md) | Third-party material and attributions |

## Outside this directory

[../README.md](../README.md) · [../ARCHITECTURE.md](../ARCHITECTURE.md) · [../CONTRIBUTING.md](../CONTRIBUTING.md) · [../GOVERNANCE.md](../GOVERNANCE.md) · [../ROADMAP.md](../ROADMAP.md) · [../CHANGELOG.md](../CHANGELOG.md) · [../SECURITY.md](../SECURITY.md)

## How to read these documents

- **[adr/](adr/) is the authority.** Accepted ADRs are immutable: they are superseded by a new ADR, never edited.
- **[spec/](spec/) is normative** and uses RFC 2119 keywords (MUST / MUST NOT / SHOULD / MAY) only where behavior is intentionally defined.
- **[research/](research/) is evidence, not commitment.** Claims there carry their own status, and open questions are stated as open.
- **Verified means what was encoded.** Verification covers the encoded constitution and nothing beyond it; documents say so where it matters.
