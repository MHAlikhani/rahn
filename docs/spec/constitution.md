<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Specification: Constitution

Status: **Draft — Stage 0.** Not yet normative; the normative portions below define intended v0.1 behavior and MUST be satisfied by the implementation once it exists. Changes to normative text require an ADR.

Normative keywords: MUST / MUST NOT / SHOULD / SHOULD NOT / MAY (RFC 2119 sense). Normative language is used only where behavior is intentionally defined.

## Normative seeds

The constitution is the named collection of invariants governing all valid states. A candidate state that violates any constitution invariant MUST be rejected before execution. v0.1 constitution checks MUST include at least the invariant set of invariants.md. The constitution MUST be stored as data and versioned with the state history.
