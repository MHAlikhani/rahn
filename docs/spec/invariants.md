<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Specification: Invariants

Status: **Draft — normative for v0.1.** The v0.1 implementation in `crates/` satisfies the normative statements below (test-enforced where marked); later-stage documents remain reserved.

Normative keywords: MUST / MUST NOT / SHOULD / SHOULD NOT / MAY (RFC 2119 sense). Normative language is used only where behavior is intentionally defined.

## Normative seeds

v0.1 invariant checks (ADR 0006): node-exists, link-endpoints-exist, no-duplicate-links, no-invalid-topology, named-connectivity. Invariant evaluation MUST be deterministic and MUST NOT depend on evaluation environment. A verification result MUST enumerate per-invariant outcomes with machine-readable evidence.

## CI verification (v0.8, ADR 0017)

1. `rahn test [ref]` MUST verify the referenced committed state against the structural invariant floor plus the constitution.
2. Output MUST be deterministic TSV (`PASS|FAIL<TAB>id<TAB>evidence`) plus a `test PASSED|FAILED (n invariants)` summary line; identical repository state MUST produce identical bytes.
3. Exit codes: 0 = pass, 1 = at least one failed invariant, 2 = usage error. CI systems MUST be able to rely on exit codes alone.
4. Invariant identifiers (e.g. `named-connectivity:a:b`) are a machine contract; changing one requires an ADR.
5. Verification of the working (uncommitted) state remains available via `rahn verify`.
