<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# ADR 0006: Verification Model — Deterministic, Constitution-Gated, Explainable

- Status: Accepted
- Date: 2026-04-15
- Deciders: project founding (charter §8, §16.H–I)

## Context

Verification gates execution. It must be deterministic (same candidate ⇒ same verdict) and its verdicts must be explainable, not bare booleans.

## Decision

1. **Constitution as data**: invariants are declared, named, first-class objects — not scattered assertions. A candidate state violating any invariant is rejected with an explanation naming the invariant, the offending objects, and the transition that produced them.
2. **v0.1 invariant vocabulary** (deliberately small, deterministic):
   - require node exists (referential integrity)
   - require link endpoints exist
   - prohibit duplicate links
   - prohibit invalid topology (e.g., self-loop links, dangling references)
   - require named connectivity (path existence between named nodes)
3. **Deterministic checking**: fixed evaluation order; no randomness, time, or I/O in the checker.
4. **Structured results**: verification returns per-invariant pass/fail with machine-readable evidence — enabling future `rahn explain`-style output and CI integration.
5. **Honesty rule**: "verified" means "all encoded invariants hold" — nothing more. Documentation must never overstate what verification guarantees (see research/limitations.md §2).

## Consequences

- The invariant engine is a growth path toward the full constitution concept (v0.2 stronger constraint engine), constrained by RQ4 (expressiveness vs. decidability).
- Checking cost must permit per-transition gating (benchmark E2); if it cannot, the model fails (H3) and must be revised, not bypassed.
- External policy engines (OPA/Cedar-class) may be *considered* later, but determinism and total, explainable results are non-negotiable requirements on any such dependency.

## Alternatives considered

- **Ad hoc assertions throughout the code**: unverifiable coverage, non-explainable failures, no constitution concept.
- **SMT/model-checking from day one**: powerful but heavy; premature for v0.1's topology-level invariants; noted as future methodology (Stage 2+).
- **Probabilistic/fuzzy verification**: rejected — determinism is architectural.

## References

- docs/adr/0005-transition-model.md; docs/spec/constitution.md; docs/spec/invariants.md
