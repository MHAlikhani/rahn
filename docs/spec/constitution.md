<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Specification: Constitution

Status: **Draft — normative for v0.1.** The implementation in `crates/` satisfies the normative statements below (test-enforced where marked).

Normative keywords: MUST / MUST NOT / SHOULD / SHOULD NOT / MAY (RFC 2119 sense). Normative language is used only where behavior is intentionally defined.

## Normative seeds

The constitution is the named collection of invariants governing all valid states. A candidate state that violates any constitution invariant MUST be rejected before execution. v0.2 constitution checks MUST include the structural invariants plus `require-connectivity a b` (a node-level path MUST exist) and `prohibit-connectivity a b` (no node-level path MAY exist; isolation). Connectivity is evaluated over the node graph induced by interface links. The constitution MUST be stored as data and versioned with the state history.
