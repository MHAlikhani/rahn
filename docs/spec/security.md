<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Specification: Security

Status: **Draft — normative for v0.1.** The implementation in `crates/` satisfies the normative statements below (test-enforced where marked).

Normative keywords: MUST / MUST NOT / SHOULD / SHOULD NOT / MAY (RFC 2119 sense). Normative language is used only where behavior is intentionally defined.

## Normative seeds

Implementations MUST NOT perform network I/O or privileged operations by default, and MUST default to read-only/simulated behavior. The single exception is the explicit opt-in namespace backend (ADR 0012): it requires `--execute --yes-i-know`, is confined to isolated Linux network namespaces, refuses on non-Linux platforms, and never touches the host network. All loaded state MUST be treated as untrusted input: parsing MUST be strict and total, and hash verification MUST precede any use. Corruption MUST cause loud refusal. These untrusted-input rules MUST NOT be relaxed behind flags.
