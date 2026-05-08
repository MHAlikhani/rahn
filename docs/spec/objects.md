<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Specification: Objects

Status: **Draft — normative for v0.1.** The v0.1 implementation in `crates/` satisfies the normative statements below (test-enforced where marked); later-stage documents remain reserved.

Normative keywords: MUST / MUST NOT / SHOULD / SHOULD NOT / MAY (RFC 2119 sense). Normative language is used only where behavior is intentionally defined.

## Normative seeds

v0.2 defines (ADR 0011): Node (a named network node with metadata and owned interfaces), Interface (a node-scoped named attachment point with metadata), Link (an undirected pair of interface endpoints with metadata), Network (a collection of Nodes, their Interfaces, and Links).

- Interface names MUST be unique within their node and MUST satisfy the identifier rules; the same name MAY appear on different nodes.
- A Link's endpoints MUST be `(node, interface)` pairs; both interfaces MUST exist at validation time; dangling endpoints MUST NOT be representable in a validated state.
- Link endpoints MUST be normalized (lexicographically ordered) so a link is undirected; a link between two interfaces of the same node MUST be rejected (topological loop).
- Identifiers and endpoint pairs MUST be stable across serialization round-trips.
- Addresses (typed, per interface) are deferred; interface metadata MAY carry them as free-form values until then.
