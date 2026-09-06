<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Specification: State

Status: **Draft — normative for v0.1.** The implementation in `crates/` satisfies the normative statements below (test-enforced where marked).

Normative keywords: MUST / MUST NOT / SHOULD / SHOULD NOT / MAY (RFC 2119 sense). Normative language is used only where behavior is intentionally defined.

## Normative seeds

A state MUST be deterministic, serializable, comparable, hashable, and reproducible. Committed states MUST be immutable and identified by their content hash (ADR 0002). A state MUST NOT embed wall-clock time, host identity, or nondeterministic ordering. The state model MUST include Node, Interface, Link, and Network with basic metadata (ADR 0011); all other concepts are deferred to later stages and MUST NOT enter the model.
