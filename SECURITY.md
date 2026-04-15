<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Security Policy

## Reporting a vulnerability

Do **not** open a public GitHub issue for security reports. Contact the maintainers privately (add a private contact channel here before the first public release) with:

- affected component and version/commit
- a minimal reproduction or proof of concept
- impact assessment

You will receive an acknowledgment; we aim to triage within one week. We credit reporters by default unless anonymity is requested.

## Security model (v0.1 and Stage 0)

Security is a first-class architectural concern. The threat model lives in the specification ([docs/spec/security.md](docs/spec/security.md)); its v0.1 posture:

- **No production execution.** v0.1 cannot and must not touch real networks.
- **No network privileges, no automatic privileged operations.**
- The default and safest mode is **read-only / simulated**.
- State files are untrusted input: parsing is strict and total; malformed or corrupted state is rejected loudly (hash verification against content addresses), never partially loaded or silently repaired.

## Longer-term security considerations (design-level, not yet implemented)

Identity, authentication, authorization, capability-based execution, cryptographic state identity, signed transitions, provenance, rollback safety, replay-attack resistance, isolation between simulation and execution, least privilege, secure defaults. Each will be specified and ADR-recorded before the corresponding stage of the roadmap.

## Scope

In scope: the RAHN codebase, its specification, and its execution/simulation engines. Out of scope: the host OS, third-party dependencies (report those upstream and notify us), and the development infrastructure.
