<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Concepts

An introduction to the ideas behind RAHN. Definitions in [../GLOSSARY.md](../GLOSSARY.md); the normative model will live in [spec/](spec/).

## The core idea

A network should not be treated merely as a collection of devices and configurations. It should be modeled as an **evolving computational system** with state, history, intent, constraints, invariants, causality, observations, executable transitions, verification, and controlled evolution.

Formally: *RAHN is a stateful execution architecture for evolving networks.*

## The lifecycle

```
Intent → State → Constraints → Candidate Transition → Verification
       → Execution → Real Network → Observation → Causal Memory → New State
```

At a higher level, intent shapes RAHN state; constraints and memory feed verification; verification gates execution; execution affects the real network; observation feeds the causal graph; the causal graph produces new state. The loop is explicit, recorded, and inspectable at every stage.

## State

State is the central value. It is more than configuration — it eventually spans topology, nodes, links, interfaces, addresses, routes, policies, identities, capabilities, service relationships, flows, constraints, health and performance information, observations, historical metadata, and causal relationships.

A state must be: inspectable, serializable, comparable, hashable, reproducible, verifiable, versionable, replayable.

Conceptual shape (deliberately not frozen):

```
State_t = { topology, policy, identity, intent,
            observations, constraints, history_reference }
```

## Transitions — not configuration

The most important primitive is the **state transition**:

```
State A + Intent/Operation + Constraints + Preconditions → Candidate State B
Verify(B) ? Execute(B) : Reject(B)
```

Every transition is explicit, recorded, and produces a new state. The architecture makes unsafe transitions difficult to *express*, not merely difficult to execute.

## The network constitution

A constitution is a collection of **architectural invariants** that must hold across all valid states — e.g., "databases must never be publicly reachable", "payment traffic must remain encrypted", "critical services require multiple failure domains", "control-plane connectivity must be preserved". A candidate state that violates the constitution is rejected. These are not configuration directives; they are constraints on what the network is allowed to *be*.

## Causal memory

Telemetry answers "what happened?". RAHN's long-term goal is to answer "what changed, what happened before that, which transition introduced the change, which event caused downstream effects, which policy or path decision was responsible?" — via a **causal graph** connected to state evolution:

```
link degradation → path selection change → congestion → packet loss
→ retransmission → application latency → service timeout
```

This is not a metrics graph; it is causality linked to state transitions.

## Evolution, branching, semantic merge

Evolution is a graph: S0 → S1 → S2 → S3, and it may branch. Branches are lightweight references. Diff and merge are semantic operations on network meaning, not text. Merges can produce **semantic conflicts** — e.g., branch A adds `latency <= 10ms`, branch B requires post-quantum encryption, and no available path satisfies both. RAHN must detect and explain such conflicts; v0.1 fails closed rather than auto-resolving.

## Time travel and replay

`rahn replay --at <timestamp>` should reconstruct relevant state and observations for an earlier point; `rahn explain <incident>` should trace an incident to its origin transition and causal chain. Not implemented in v0.1 — but immutable, content-addressed history is designed for it from day one.

## AI as an optional layer

AI is not the foundation. Neural models may propose (diagnose, rank, hypothesize); the symbolic, deterministic layer (constraints, invariants, verification, execution policy) disposes. AI consumes RAHN state; it never bypasses deterministic safety mechanisms. RAHN is fully useful without AI — this is a requirement, not a concession.

## The primitive question

Candidate first-class primitives: Node, Link, Path, Flow, Service, Identity, Capability, Policy, Intent, Constraint, Invariant, Observation, Event, Transition, State, State Commit, Branch, Merge, Causal Relation, Execution Plan, Verification Result.

Not all of these are fundamental. Determining which are fundamental and which are derived — and avoiding unnecessary abstractions — is an explicit research goal ([research/research-questions.md](research/research-questions.md)).
