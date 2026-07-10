<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Specification: Protocol

Status: **Draft — normative for v0.6.** The peer-sync model (ADR 0015) is implemented; authentication and log replication remain reserved.

Normative keywords: MUST / MUST NOT / SHOULD / SHOULD NOT / MAY (RFC 2119 sense). Normative language is used only where behavior is intentionally defined.

## Reserved

No wire protocol is defined in v0.1 (networking and distribution are later-stage). This document is reserved; when a protocol is specified, it MUST reuse existing transports (e.g., QUIC/TLS) unless an architectural requirement cannot be met otherwise.

## Normative seeds (v0.6, ADR 0015)

1. Synchronization MUST exchange only immutable, content-addressed records (commits, states) and branch-tip offers.
2. A replica MUST treat its own committed history as authoritative; it MUST NOT adopt a remote tip whose ancestry is incomplete or whose objects fail hash verification.
3. Diverged branch tips MUST converge only through the fail-closed semantic merge (ADR 0007); a successful convergence MUST produce a byte-identical merge commit on every replica (parents ordered lexicographically by commit id, message `sync merge`).
4. Merge conflicts MUST be reported and MUST leave divergent tips in place; automatic conflict resolution is forbidden.
5. Implementations MUST NOT claim linearizability, serializability, or strong consistency across replicas; per-replica read-your-writes and monotonic history are the only read guarantees.
6. Corrupted replication payloads MUST be refused (hash verification); replica identity is self-asserted in v0.6 (documented trust boundary).
