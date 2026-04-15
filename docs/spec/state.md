<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Specification: State

Status: **Draft — Stage 0.** Not yet normative; the normative portions below define intended v0.1 behavior and MUST be satisfied by the implementation once it exists. Changes to normative text require an ADR.

Normative keywords: MUST / MUST NOT / SHOULD / SHOULD NOT / MAY (RFC 2119 sense). Normative language is used only where behavior is intentionally defined.

## Normative seeds

A state MUST be deterministic, serializable, comparable, hashable, and reproducible. Committed states MUST be immutable and identified by their content hash (ADR 0002). A state MUST NOT embed wall-clock time, host identity, or nondeterministic ordering. The v0.1 state model MUST include Node, Link, Network, and basic metadata; all other concepts are deferred to later stages and MUST NOT enter the v0.1 model.
