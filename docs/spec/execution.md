<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Specification: Execution

Status: **Draft — normative for v0.1 (simulation-only).** The v0.1 implementation satisfies the normative statements below; real backends (Stage 3+) do not exist yet.

Normative keywords: MUST / MUST NOT / SHOULD / SHOULD NOT / MAY (RFC 2119 sense). Normative language is used only where behavior is intentionally defined.

## Normative seeds

v0.1 execution is simulation-only (ADR 0008). rahn apply MUST produce an explicit, recorded execution plan and MUST NOT modify the host system. The executor MUST accept only transitions carrying a passing verification result. Real backends (Stage 3+) MUST implement a plan-execution interface and MUST NOT leak backend concepts into the state model.
