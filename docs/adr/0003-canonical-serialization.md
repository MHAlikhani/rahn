<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# ADR 0003: Canonical Serialization

- Status: Accepted
- Date: 2026-04-15
- Deciders: project founding (charter §16.B)

## Context

State identity (ADR 0002) requires that one logical state has exactly one byte sequence. Ad hoc serialization (default struct layout, hash-map iteration order, floating-point formatting) makes identity unstable across runs and platforms.

## Decision

RAHN defines a **canonical serialization** with these properties:

1. **Total ordering everywhere**: all collections serialize in a defined order (sorted keys; deterministic graph object ordering by identity, with a defined tie-break rule).
2. **No ambient information**: no timestamps, no host data, no uninitialized memory, no pointer values.
3. **Explicit numeric rules**: fixed-width integers; no floating point in state identity paths (measurements/observations, if ever included, are represented exactly or excluded from identity).
4. **Versioned format**: a format version tag precedes content, enabling evolution without silent reinterpretation.
5. **Round-trip guarantee**: canonicalize(serialize(x)) == serialize(x); deserialization of malformed input is a total, strict error — never a partial state.

The concrete encoding is internal in v0.1 and stability-tested (property tests, cross-platform CI — experiment E1).

## Consequences

- Implementation discipline: every new state-model field must specify its canonical form; enforced by review and round-trip property tests.
- Performance cost accepted (ARCHITECTURE.md §7).
- Human-readability is not a goal for identity serialization; separate presentation formats (JSON/YAML) are for display only and never hashed.

## Alternatives considered

- **JSON with sorted keys**: workable but weak on binary data, numeric edge cases, and graph references; kept as a display format.
- **Protobuf/FlatBuffers**: excellent wire formats but not canonical by default (map ordering, unknown-field behavior); considered for the future protocol layer, not for identity.
- **Opaque engine-internal representation**: fastest, but blocks cross-implementation identity and corruption detection.

## References

- docs/adr/0002-state-identity.md; docs/spec/serialization.md; RQ2
