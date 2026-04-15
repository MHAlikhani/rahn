<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Specification: Serialization

Status: **Draft — Stage 0.** Not yet normative; the normative portions below define intended v0.1 behavior and MUST be satisfied by the implementation once it exists. Changes to normative text require an ADR.

Normative keywords: MUST / MUST NOT / SHOULD / SHOULD NOT / MAY (RFC 2119 sense). Normative language is used only where behavior is intentionally defined.

## Normative seeds

Canonical serialization (ADR 0003) MUST be total-ordered, ambient-free, numerically exact, and format-versioned. canonicalize(x) MUST equal serialize(x). Malformed input MUST be rejected totally; implementations MUST NOT deserialize partial states. Display formats (JSON/YAML) MUST NOT be used as identity inputs.
