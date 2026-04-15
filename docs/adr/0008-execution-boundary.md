<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# ADR 0008: Execution Boundary — Simulation Only Until the Model Can Perceive

- Status: Accepted
- Date: 2026-04-15
- Deciders: project founding (charter §16.J, §20)

## Context

RAHN's lifecycle ends in execution against a real network. Executing against reality before the state model, verification, and provenance are sound would be dangerous and would entangle representation with backend concerns.

## Decision

1. **v0.1 (and v0.2) execution is simulation-only.** `rahn apply <state>` produces an explicit **execution plan** (e.g., "create node", "create link", "remove link") that is inspectable and recorded — and touches nothing. The operating system and any network are untouched.
2. **Structural gate**: the executor accepts only *verified* transitions (type-enforced, ADR 0005); unverified candidates cannot reach it.
3. **Representation is separated from execution**: the state model must not contain execution backend concepts. Backends implement a small plan-execution interface (Stage 3+).
4. **First real backend** is the most reversible, most isolated substrate available: Linux network namespaces (Stage 3) — not production devices, not cloud.
5. **No privileges by default**: even future real execution requires explicit opt-in; the safest mode (read-only/simulated) is the default forever.
6. **Security posture** (charter §20): v0.1 has no network privileges, no automatic privileged operations, treats all loaded state as untrusted input.

## Consequences

- Simulation fidelity becomes a research object (plan/observation differential testing, E6) before real networks are touched.
- The execution-plan format becomes part of the architecture early — it must be specified and stable before Stage 3 backends exist.
- RAHN cannot demo "real networking" until Stage 3; accepted (see ADR 0001).

## Alternatives considered

- **Direct netlink/device execution in v0.1**: fastest path to a "real" demo; rejected — couples the unfrozen state model to kernel behavior and creates real harm potential with unverified tooling.
- **Defer execution planning entirely**: rejected — the plan is where representation meets execution; designing it early keeps the boundary honest and testable in simulation.

## References

- docs/adr/0005-transition-model.md; docs/spec/execution.md; docs/spec/security.md; ROADMAP Stage 3
