<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Specification: Transitions

Status: **Draft — normative for v0.1.** The v0.1 implementation in `crates/` satisfies the normative statements below (test-enforced where marked); later-stage documents remain reserved.

Normative keywords: MUST / MUST NOT / SHOULD / SHOULD NOT / MAY (RFC 2119 sense). Normative language is used only where behavior is intentionally defined.

## Normative seeds

Every state change MUST be an explicit transition (ADR 0005): (State A, Operation, Parameters) → candidate State B or structured Rejection. Transitions MUST be pure and total; implementations MUST NOT mutate a committed state. v0.2 operations (ADR 0011): add_node, remove_node, add_interface, remove_interface, add_link, remove_link (link operations address `node/interface` endpoints). An operation on a nonexistent object MUST produce a structured rejection, never a partial state. remove_interface MUST be rejected while any link references the interface; remove_node MUST be rejected while any link references any of its interfaces.
