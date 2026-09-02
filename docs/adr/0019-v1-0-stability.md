<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# ADR 0019: v1.0 Stability and Compatibility Commitment

- Status: Accepted
- Date: 2026-09-02

## Context

v1.0 (charter §25, §52) requires a compatibility strategy, an extension model, and a release process. Pre-1.0 policy (ADR 0018) was MINOR-breaking; 1.0 changes the contract.

## Decision

1. **Semver from 1.0.0:** `rahn-sdk` (the public facade) and the IR (canonical format v2 + `Operation` vocabulary) follow strict semver — breaking changes only in MAJOR. Internal crates (`rahn-core`, `rahn-state`, …) remain workspace-internal; only `rahn-sdk`, the CLI behavior contracts (exit codes, `rahn test` output schema), and the IR are stable surface.
2. **Canonical format v2 is frozen for 1.x.** A format change requires MAJOR + ADR + migration tooling.
3. **Extension model (already structural, now declared stable):**
   - new execution targets = `ExecutionBackend` trait implementations + `Capabilities` entries (ADR 0016);
   - new invariants = deterministic checkers with machine-contract ids (ADR 0006/0017);
   - new verification sources = observation records (ADR 0013), never implicit edges (ADR 0014);
   - distribution = peer sync over content-addressed history (ADR 0015).
4. **Release process** (charter §35): CHANGELOG entry → full validation (fmt, clippy `-D warnings`, all tests) → benchmark/evidence updates where behavior changed → tag `vX.Y.Z` on the feature commit → GitHub Release with notes from `docs/releases/` → CI green on the release commit. Pre-release flags stay on all `<1.0-style` alpha channels; 1.0.0 is the first non-prerelease.
5. **Deprecation:** surface items are deprecated (documented, still working) for at least one MINOR before removal.

## Consequences

- External code targeting `rahn-sdk` is stable across 1.x.
- The cost is process discipline: every surface change needs an ADR + CHANGELOG entry.
- Research continues post-1.0 inside these extension points.
