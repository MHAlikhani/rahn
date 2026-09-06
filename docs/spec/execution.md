<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Specification: Execution

Status: **Draft — normative for v0.8.** The simulation default and the Linux namespace backend (ADR 0012) are implemented; the statements below are test-enforced where marked.

Normative keywords: MUST / MUST NOT / SHOULD / SHOULD NOT / MAY (RFC 2119 sense). Normative language is used only where behavior is intentionally defined.

## Normative seeds

Execution is simulation by default (ADR 0008). rahn apply MUST produce an explicit, recorded execution plan and, unless real execution is explicitly opted into, MUST NOT modify the host system. The executor MUST accept only transitions carrying a passing verification result. Backends MUST implement the ExecutionBackend trait (ADR 0016) and MUST NOT leak backend concepts into the state model. The namespace backend (ADR 0012) MUST map plans to iproute2 argv deterministically, MUST restrict host-side operations to `ip netns add/del rahn-*`, MUST refuse execution on non-Linux platforms, and MUST refuse execution without an explicit opt-in flag pair (`--execute --yes-i-know`). The default `apply` MUST print the command sequence and perform no host mutation.
