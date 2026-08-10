<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# ADR 0017: CI Verification Semantics

- Status: Accepted
- Date: 2026-08-10

## Context

Stage 8 (charter): network changes must participate in CI — testable before deployment. RAHN already gates every commit through deterministic verification; what is missing is a machine-facing surface that CI systems can invoke without parsing prose.

## Decision

1. **`rahn test [ref]`** verifies a committed state (default: HEAD; `ref` = branch name or commit id) against the constitution and structural invariants, and prints a deterministic report:
   - header: `state <hex>`;
   - one line per invariant: `PASS|FAIL<TAB>id<TAB>evidence` (evidence empty on pass);
   - summary: `test PASSED|FAILED (n invariants)`.
2. **Exit codes are the contract:** `0` = all invariants pass; `1` = at least one failure (report still printed); `2` = usage error. CI pipelines branch on exit codes only; output format is deterministic TSV, never locale- or tty-dependent.
3. **Determinism:** identical repository state ⇒ identical report bytes. No wall clock, no environment data in output.
4. **Scope:** v0.8 verifies committed states only (the working-state flow already exists as `rahn verify`). Reachability/latency-class checks arrive with addressing; the report schema is stable for them (invariant ids).
5. **No dashboards, no servers.** CI integration is a documented example workflow consuming the CLI's exit codes.

## Consequences

- Any CI system (GitHub Actions, GitLab, local) can gate network changes today for topology + constitution violations.
- Invariant ids are the machine contract; renaming one requires an ADR.
