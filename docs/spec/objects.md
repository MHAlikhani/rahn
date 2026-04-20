<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Specification: Objects

Status: **Draft — normative for v0.1.** The v0.1 implementation in `crates/` satisfies the normative statements below (test-enforced where marked); later-stage documents remain reserved.

Normative keywords: MUST / MUST NOT / SHOULD / SHOULD NOT / MAY (RFC 2119 sense). Normative language is used only where behavior is intentionally defined.

## Normative seeds

v0.1 defines: Node (a named network node with metadata), Link (an ordered pair of Node references with metadata), Network (a collection of Nodes and Links). A Link MUST reference existing Nodes at validation time; invalid references MUST NOT be representable in a validated state. Identifiers MUST be stable across serialization round-trips.
