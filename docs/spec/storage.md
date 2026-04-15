<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Specification: Storage

Status: **Draft — Stage 0.** Not yet normative; the normative portions below define intended v0.1 behavior and MUST be satisfied by the implementation once it exists. Changes to normative text require an ADR.

Normative keywords: MUST / MUST NOT / SHOULD / SHOULD NOT / MAY (RFC 2119 sense). Normative language is used only where behavior is intentionally defined.

## Normative seeds

Committed states MUST be stored content-addressed under their identity (ADR 0004). Objects MUST be write-once. On load, implementations MUST verify content hashes and MUST refuse corrupted objects with an explicit error; silent repair is forbidden. Branches MUST be references to state IDs. v0.1 assumes a single writer; concurrent writers MUST NOT be assumed safe.
