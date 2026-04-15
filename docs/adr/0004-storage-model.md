<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# ADR 0004: Storage Model — Content-Addressed Local Store

- Status: Accepted
- Date: 2026-04-15
- Deciders: project founding (charter §16.C)

## Context

Committed states must be persisted immutably and cheaply. v0.1 is local, single-writer; distributed stores are explicitly out of scope (ROADMAP Stage 6).

## Decision

v0.1 persistence is a **filesystem-backed content-addressed store**: canonical serialized objects (ADR 0003) stored under their content hash; **branches are lightweight references** (named pointers to state IDs), never copies of state data; history is the chain/graph of commits with parent links. Objects are write-once; the store verifies hashes on load and refuses corrupted objects loudly.

## Consequences

- Identical states stored once (deduplication by construction).
- Rollback/branch/replay are pointer operations.
- Storage grows with unique state content, not with history length of identical content; pruning is deferred until real growth data exists (charter: no premature optimization).
- The store is a swappable backend behind a small interface, so later stages (0.6+) can introduce other engines without changing state semantics.
- Concurrency: v0.1 assumes a single writer; multi-process safety is out of scope until Stage 6 (documented limitation).

## Alternatives considered

- **Embedded SQL database (e.g., SQLite)**: strong tooling, but relational schemas fight content-addressed graph data in v0.1 and add migration burden to an unfrozen model. Reconsidered if query needs outgrow the object store (Stage 2+).
- **Git object store as backend**: attractive (CAS for free) but would entangle RAHN semantics with Git's object model and tooling; RAHN borrows concepts, not infrastructure.
- **Key-value embedded stores (sled/rocksdb)**: viable later; unnecessary dependency for v0.1's write-once workload.

## References

- docs/adr/0002-state-identity.md; docs/adr/0003-canonical-serialization.md; docs/spec/storage.md
