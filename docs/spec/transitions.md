<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Specification: Transitions

Status: **Draft — Stage 0.** Not yet normative; the normative portions below define intended v0.1 behavior and MUST be satisfied by the implementation once it exists. Changes to normative text require an ADR.

Normative keywords: MUST / MUST NOT / SHOULD / SHOULD NOT / MAY (RFC 2119 sense). Normative language is used only where behavior is intentionally defined.

## Normative seeds

Every state change MUST be an explicit transition (ADR 0005): (State A, Operation, Parameters) → candidate State B or structured Rejection. Transitions MUST be pure and total; implementations MUST NOT mutate a committed state. v0.1 operations: add_node, remove_node, add_link, remove_link. An operation on a nonexistent object MUST produce a structured rejection, never a partial state.
