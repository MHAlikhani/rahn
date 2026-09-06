<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Specification: Storage

Status: **Draft — normative for v0.1.** The implementation in `crates/` satisfies the normative statements below (test-enforced where marked).

Normative keywords: MUST / MUST NOT / SHOULD / SHOULD NOT / MAY (RFC 2119 sense). Normative language is used only where behavior is intentionally defined.

## Normative seeds

Committed states MUST be stored content-addressed under their identity (ADR 0004). Objects MUST be write-once. On load, implementations MUST verify content hashes and MUST refuse corrupted objects with an explicit error; silent repair is forbidden. Branches MUST be references to state IDs. v0.1 assumes a single writer; concurrent writers MUST NOT be assumed safe.
