<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Changelog

All notable changes to the RAHN project (architecture, documentation, and later software) are documented here. Format based on [Keep a Changelog](https://keepachangelog.com/); versioning is semantic once releases begin.

## [0.1.0-alpha.1] — Stage 1: State Engine

### Added
- Rust workspace (`crates/`): `rahn-core` (object model), `rahn-state`
  (canonical serialization, content-hash identity, transitions, diff,
  commits, history graph), `rahn-store` (content-addressed filesystem
  store), `rahn-verify` (invariants, constitution, fail-closed semantic
  merge), `rahn-sim` (execution plans), `rahn-cli` (`rahn` binary).
- CLI: `init, node add/remove, link add/remove, commit, state, branch,
  diff, merge, verify, log, inspect, apply` — simulation-only.
- Deterministic invariant engine: referential integrity, link endpoints,
  duplicate links, self-loops, named connectivity requirements.
- Verification-gated commits: failing candidates are rejected with
  per-invariant evidence and never written to the store.
- Content-addressed storage with loud corruption refusal.
- 75 tests, including cross-platform determinism, canonical round-trip
  adversarial cases, corrupted-object detection, and end-to-end CLI flows.
- `examples/cli-walkthrough.md`.

### Documentation
- Spec status headers updated to reflect implementation (reserved areas
  marked as such); ADR 0007 implementation amendment; third-party
  inventory records `sha2 0.10.9 (Apache-2.0 OR MIT)`.

## [Unreleased] — Stage 0: Research / Foundation

### Added
- Initial documentation corpus: README, ARCHITECTURE, DESIGN, GLOSSARY, ROADMAP, CONTRIBUTING, SECURITY, GOVERNANCE.
- `docs/concepts.md` — core concepts.
- `docs/research/` — problem statement, prior art, research questions, hypotheses, experiment plan, benchmark methodology, limitations, future work.
- `docs/adr/` — ADRs 0001–0008 (project scope, state identity, canonical serialization, storage model, transition model, verification model, branching model, execution boundary).
- `docs/spec/` — specification skeleton (state, objects, transitions, invariants, constitution, storage, serialization, protocol, execution, observation, causality, security).
- `docs/whitepaper/RAHN-Whitepaper.md` — white paper v0.1 (structured outline).

### Changed
- **Licensing model established (ADR 0009):** source code Apache-2.0, documentation/research CC-BY-4.0. The provisional MIT root LICENSE was replaced with the canonical Apache License 2.0 text; canonical license texts added under `LICENSES/`; SPDX headers added to documentation; README and CONTRIBUTING updated; `docs/licensing.md` (authoritative policy) and `docs/third-party.md` (inventory, currently empty) added.
