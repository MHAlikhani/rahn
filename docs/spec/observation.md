<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Specification: Observation

Status: **Draft — normative for v0.4.** The observation model (ADR 0013) is implemented; statements below are test-enforced where marked.

Normative keywords: MUST / MUST NOT / SHOULD / SHOULD NOT / MAY (RFC 2119 sense).

## Records

1. An observation MUST be the record `{ seq, time_ns, subject, metric, value }`.
2. `time_ns` MUST be supplied by the caller; implementations MUST NOT generate timestamps implicitly.
3. `seq` MUST be assigned deterministically at ingest (existing record count) and MUST be immutable.
4. `subject` MUST be `node/<id>` or `node/<id>/<iface>` and MUST reference a resource existing in the referenced state; a missing subject MUST be rejected with a structured error.
5. `metric` MUST satisfy the identifier rules (non-empty, ≤64 chars, ASCII alphanumeric plus `-`, `_`, `.`).
6. `value` MUST be `Counter(u64)`, `Gauge(i64)`, or `Event(String)`; floating point MUST NOT be representable.
7. Events MUST be bounded (≤256 bytes UTF-8).

## Ordering and storage

8. Records MUST be ordered by `(time_ns, seq)` ascending; `seq` MUST break timestamp ties.
9. The observation log MUST be append-only; written records MUST be immutable.
10. Storage MUST use versioned, canonical record framing (u64 length + canonical bytes) under the repository root.
11. Malformed framing or record content MUST cause loud refusal; implementations MUST NOT truncate or silently repair the log.
12. Reading MUST return records in stored order; clients MAY sort by `(time_ns, seq)`.

## Provenance

13. Each record MUST record the state id it was validated against.
14. Ingest MUST NOT require any network I/O, external collectors, or privileged operations.

## Reserved

Causal relations between observations/events/transitions are specified separately (docs/spec/causality.md, ADR 0014) and MUST NOT be implied by this specification.
