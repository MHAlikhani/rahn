<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Specification: Constitution

Status: **Draft — normative for v0.1.** The v0.1 implementation in `crates/` satisfies the normative statements below (test-enforced where marked); later-stage documents remain reserved.

Normative keywords: MUST / MUST NOT / SHOULD / SHOULD NOT / MAY (RFC 2119 sense). Normative language is used only where behavior is intentionally defined.

## Normative seeds

The constitution is the named collection of invariants governing all valid states. A candidate state that violates any constitution invariant MUST be rejected before execution. v0.1 constitution checks MUST include at least the invariant set of invariants.md. The constitution MUST be stored as data and versioned with the state history.
