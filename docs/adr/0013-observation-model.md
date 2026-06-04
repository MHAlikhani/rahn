<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# ADR 0013: Deterministic Observation Model

- Status: Accepted
- Date: 2026-06-04

## Context

Stage 4 (charter) introduces Observation as a first-class concept, linked to network state, without dashboards, external telemetry, or causal inference (Stage 5). The architecture's determinism rules constrain the design: no floating point, no hidden clocks, canonical serialization.

## Alternatives

1. **Wall-clock timestamps generated at ingest:** rejected — nondeterministic logs (same command sequence, different bytes), contradicting reproducibility policy.
2. **Float/double metric values:** rejected — banned from canonical encoding (ADR 0003).
3. **External telemetry integration (OTLP ingestion):** rejected for v0.4 — local, replayable data first; an importer MAY come later behind the same model.
4. **Single Observation record with typed values (chosen):** Events and metric samples share one record type; discrete events are a value variant. Keeps the schema small and the ordering rule single.

## Decision

1. **Observation** = `{ seq, time_ns, subject, metric, value }`:
   - `time_ns`: u64 nanoseconds since the UNIX epoch, **supplied by the caller** (`--at`); never generated implicitly.
   - `seq`: u64 assigned at ingest as the count of existing records — a deterministic positional order that also total-orders equal timestamps.
   - `subject`: `node/<id>` or `node/<id>/<iface>` — validated to exist in the referenced state at ingest (provenance).
   - `metric`: bounded identifier string (same charset as node ids).
   - `value`: `Counter(u64) | Gauge(i64) | Event(String)`. Counters are monotonically non-decreasing by convention (not enforced); gauges are signed; events are bounded UTF-8 text.
2. **Ordering:** `(time_ns, seq)`, ascending; equal timestamps permitted; `seq` breaks ties deterministically. The log is append-only; records are immutable once written.
3. **Provenance:** each record is ingested against a RAHN state id (default: the current HEAD commit's state); the association is validated at ingest (unknown subject → structured rejection) and stored in the record.
4. **Persistence:** append-only log at `.rahn/observations.log`, framed records (u64 length + canonical bytes, format versioned). Malformed framing or parse failure → loud refusal of that ingest/read; the log is never silently truncated or repaired.
5. **No aggregation, no causal edges, no external collection** — Stage 5 concerns.

## Consequences

- Determinism holds end-to-end: identical command sequences produce byte-identical logs.
- Callers must own time (documented; a future importer may normalize external clocks, explicitly recorded).
- Values are exact integers; units live in metric names by convention (e.g., `latency_ns`).
- Causal reasoning (Stage 5) will consume these records as inputs; nothing here asserts causality.
