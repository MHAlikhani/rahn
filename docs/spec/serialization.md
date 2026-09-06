<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Specification: Serialization

Status: **Draft — normative for v0.1.** The implementation in `crates/` satisfies the normative statements below (test-enforced where marked).

Normative keywords: MUST / MUST NOT / SHOULD / SHOULD NOT / MAY (RFC 2119 sense). Normative language is used only where behavior is intentionally defined.

## Normative seeds

Canonical serialization (ADR 0003) MUST be total-ordered, ambient-free, numerically exact, and format-versioned. The current version is 2 (ADR 0011): nodes carry owned interfaces before links, links carry endpoint pairs. Readers MUST reject v1 bytes explicitly and MUST NOT migrate silently. canonicalize(x) MUST equal serialize(x). Malformed input MUST be rejected totally; implementations MUST NOT deserialize partial states. Display formats (JSON/YAML) MUST NOT be used as identity inputs.
