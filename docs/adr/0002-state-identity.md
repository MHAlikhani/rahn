<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# ADR 0002: State Identity via Content Hashing

- Status: Accepted
- Date: 2026-04-15
- Deciders: project founding (charter §16.B)

## Context

Every RAHN state needs a stable, deterministic identity to support immutability, deduplication, verification of integrity, branching as references, and future replay. The identity must not depend on wall-clock time, host, process, or map iteration order.

## Decision

State identity is derived from the state's **canonical serialization** (ADR 0003) passed through a **content hash**. Committed states are content-addressed and immutable. The exact hash function is an internal implementation detail in v0.1 — not part of the public specification — so the scheme can evolve (e.g., hash length, domain separation, versioned hashing) before it is frozen. Identity records the scheme version so future migration is possible.

## Consequences

- Identical states deduplicate for free (content addressing).
- Corruption is detectable (hash mismatch ⇒ refuse to load, never silently repair).
- Any change to canonicalization changes identity — so canonicalization is itself versioned and stability-tested (E1).
- Choice of hash strength is deferred; cryptographic strength is anticipated for later signed transitions but is not required for v0.1 integrity checking.

## Alternatives considered

- **Sequential IDs / timestamps**: simple, but nondeterministic across hosts and hostile to dedup and integrity.
- **UUIDs generated at commit time**: stable per commit but content-unlinked; identical states get different IDs; corruption undetectable.
- **Database-assigned IDs**: couples identity to the storage backend; violates storage-model independence (ADR 0004).

## References

- docs/adr/0003-canonical-serialization.md; research question RQ2
