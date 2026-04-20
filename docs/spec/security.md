<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Specification: Security

Status: **Draft — normative for v0.1.** The v0.1 implementation in `crates/` satisfies the normative statements below (test-enforced where marked); later-stage documents remain reserved.

Normative keywords: MUST / MUST NOT / SHOULD / SHOULD NOT / MAY (RFC 2119 sense). Normative language is used only where behavior is intentionally defined.

## Normative seeds

v0.1 implementations MUST NOT perform network I/O, MUST NOT perform privileged operations, and MUST default to read-only/simulated behavior. All loaded state MUST be treated as untrusted input: parsing MUST be strict and total, and hash verification MUST precede any use. Corruption MUST cause loud refusal. These rules MUST NOT be relaxed behind flags.
