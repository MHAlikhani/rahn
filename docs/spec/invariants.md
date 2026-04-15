<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Specification: Invariants

Status: **Draft — Stage 0.** Not yet normative; the normative portions below define intended v0.1 behavior and MUST be satisfied by the implementation once it exists. Changes to normative text require an ADR.

Normative keywords: MUST / MUST NOT / SHOULD / SHOULD NOT / MAY (RFC 2119 sense). Normative language is used only where behavior is intentionally defined.

## Normative seeds

v0.1 invariant checks (ADR 0006): node-exists, link-endpoints-exist, no-duplicate-links, no-invalid-topology, named-connectivity. Invariant evaluation MUST be deterministic and MUST NOT depend on evaluation environment. A verification result MUST enumerate per-invariant outcomes with machine-readable evidence.
