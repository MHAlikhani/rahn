<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Problem Statement

Status: living research document (Stage 0).

## The problem in one paragraph

Modern networks are operated through fragmented views — configuration, topology, runtime state, telemetry, policy, identity, routing, incidents, and history each live in different systems with different models. As a result, changes are hard to verify before they execute, history is lost or non-causal, and incident diagnosis reconstructs causality by hand from telemetry that has no link to state evolution.

## The underlying question

> Can network state become a first-class computational object — and can that state be versioned, reasoned about, verified, replayed, branched, simulated, evolved, and safely executed?

## Why existing approaches fall short

(See [prior-art.md](prior-art.md) for detailed analysis.)

1. **Configuration management / IaC** versions *declarations*, not the resulting reconciled state; merges are textual; no causality.
2. **SDN controllers** centralize control but typically keep weak or ephemeral state history; verification of *evolution* (not just state) is absent.
3. **Intent-based networking** captures desired outcomes but usually without immutable provenance, semantic branching, or causal memory.
4. **Observability** answers "what happened" but not "which state transition caused it" — telemetry is not linked to state evolution.
5. **Digital twins** simulate but do not version, verify, or preserve causal history as first-class objects.
6. **Event sourcing** (from the systems world) has the right shape — immutable history, replay, projection — but has not been applied to network state with network semantics (topology, invariants, constitutions, semantic merge).

## What a solution must provide

- Deterministic, content-addressed network state.
- Explicit transitions as the only way state changes.
- A constitution of invariants enforced before execution.
- Semantic diff, branch, and merge (fail-closed on semantic conflict).
- Observation linked to transitions (provenance).
- A causal graph over observations and transitions.
- Replay of historical state from immutable history.
- Simulation-first execution; real execution only behind verified plans and a stable backend abstraction.

## Consequences if the problem is solved

Network changes become testable artifacts in CI; incidents become explainable chains from transition to effect; rollback becomes principled rather than ad hoc; automation and AI can propose but only verified transitions can execute.

## Scope boundaries

RAHN does not aim to replace routers, Linux networking, VPNs, SDN controllers, monitoring dashboards, or IaC tools. It aims to provide the **state substrate** they could share.
