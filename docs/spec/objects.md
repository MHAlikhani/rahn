<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Specification: Objects

Status: **Draft — Stage 0.** Not yet normative; the normative portions below define intended v0.1 behavior and MUST be satisfied by the implementation once it exists. Changes to normative text require an ADR.

Normative keywords: MUST / MUST NOT / SHOULD / SHOULD NOT / MAY (RFC 2119 sense). Normative language is used only where behavior is intentionally defined.

## Normative seeds

v0.1 defines: Node (a named network node with metadata), Link (an ordered pair of Node references with metadata), Network (a collection of Nodes and Links). A Link MUST reference existing Nodes at validation time; invalid references MUST NOT be representable in a validated state. Identifiers MUST be stable across serialization round-trips.
