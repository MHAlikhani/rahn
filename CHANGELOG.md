<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Changelog

All notable changes to the RAHN project (architecture, documentation, and later software) are documented here. Format based on [Keep a Changelog](https://keepachangelog.com/); versioning is semantic once releases begin.

## [0.2.0-alpha] — Stage 2: Network Graph Foundation

### Added
- **Interface-endpoint model (ADR 0011):** interfaces are first-class
  objects owned by nodes; links connect `node/interface` endpoints;
  same-node links prohibited. Canonical format versioned to 2 (v1 bytes
  rejected explicitly — no silent migration; see ADR 0011 for rationale).
- **Graph queries:** deterministic shortest path (lexicographic
  tie-break), neighbors, reachability, connected components.
- **Isolation constraints:** `prohibit-connectivity a b` in the
  constitution, evaluated over the interface-induced node graph.
- CLI: `rahn interface add/remove`; link endpoints as `node/iface`;
  `rahn path <from> <to>`; interface-aware diff/state/inspect/apply.
- Scaling benchmarks (`cargo test -p rahn-state --release --test
  scaling -- --ignored`): 10 to 100 000 nodes; results and methodology in
  `docs/research/stages/v0.2.md`.
- Stage report: `docs/research/stages/v0.2.md`.

### Fixed
- Merge: added nodes now materialize their interfaces (previously links
  referencing them failed or nodes were interface-less).
- Diff: added/removed nodes report their interfaces (diff-derived
  operations now reconstruct the state exactly).

### Compatibility
- Canonical format v2 is NOT readable by v0.1.x binaries, and v0.1.x
  stores are NOT readable by v0.2 (explicit error referencing ADR 0011).
  Alpha-era repositories; no migration tool is provided by design.

## [0.1.0-alpha.2] — Stage 1 Hardening (post-v0.1 review)

### Added
- `rahn checkout [--force] <branch>` (ADR 0010): switch branches; fails
  closed on uncommitted working changes; `--force` is the explicit,
  documented recovery path for changes that can never be committed
  (e.g., constitution-rejected edits).
- Seeded property tests over random networks (docs/testing.md): canonical
  round-trip + identity stability (200 seeds), diff-derived operations
  reconstruct the target (200 seeds), merge commutativity for disjoint
  effects (100 seeds). Deterministic xorshift PRNG; no new dependencies.
- End-to-end tests: diverged-branch checkout+merge, dirty-checkout
  refusal, constitution gating of branch-state commits, --force recovery.
- `docs/testing.md` (test strategy) and `docs/reproducibility.md`.
- GitHub Actions CI: rustfmt, clippy (-D warnings), tests on Linux and
  Windows, plus license/SPDX documentation checks.
- Issue templates (bug, feature proposal, design proposal/RFC) and
  CODE_OF_CONDUCT.md (Contributor Covenant 2.1).

### Changed
- White paper upgraded from outline to initial technical draft (v0.2):
  claim-status markers throughout, explicit no-measurements statement,
  scoped novelty statement.
- docs/research/prior-art.md expanded into a structured per-system
  survey (Git, Terraform, Batfish, SDN, IBN/RFC 9315, NetBox-class,
  emulators, P4/eBPF/XDP, QUIC/SCION, observability, event sourcing,
  consensus, formal verification).
- docs/research/research-questions.md restructured to RQ1-RQ10, each
  with motivation, hypothesis, method, measurable result, limitations.
- Founder attribution resolved: RAHN was created and is initially led by
  Mohammad Hossein Alikhani (docs/licensing.md [LEGAL REVIEW] marker
  resolved; ADR 0009 amendment; README and GOVERNANCE updated).
- Code formatted with rustfmt; clippy clean at -D warnings.

### Fixed
- Merge no longer drops metadata of nodes added on a branch (property
  test coverage added).

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
