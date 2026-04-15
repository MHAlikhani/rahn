<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Glossary

Canonical terminology. Terms are defined here first; implementation and docs must match these definitions.

- **State** — a first-class, deterministic representation of a network at a point in its evolution: topology, policy, identity, intent, observations, constraints, and history references. Inspectable, serializable, comparable, hashable, reproducible, verifiable, versionable, replayable.
- **State commit** — an immutable, content-addressed record of a state plus its provenance (parent state, transition, verification result).
- **State identity** — the stable deterministic identifier derived from a state's canonical serialization and content hash.
- **Transition** — the primary primitive: State A + intent/operation + constraints + preconditions → candidate State B, subject to verification. Not "configuration".
- **Candidate state** — a state produced by a transition but not yet verified or committed.
- **Verification** — deterministic checking of a candidate state against the constitution and preconditions. Produces an explainable result, not just pass/fail.
- **Constitution** — a collection of architectural invariants that must remain true across all valid network states (e.g., "databases must never be publicly reachable").
- **Invariant** — a predicate over state that must hold for the state to be valid.
- **Intent** — a declaration of desired network properties/outcomes, distinct from the mechanism used to reach them.
- **Constraint** — a restriction on transitions or states, narrower than an invariant (typically scoped to a change or object).
- **Branch** — a lightweight reference to a state, from which alternative evolution proceeds. Not a copy of state data.
- **Merge** — combination of two branches of state evolution. Network-aware and semantic: conflicting constraints/infeasible combinations are detected and explained; never a textual merge.
- **Diff** — a semantic comparison of two states (objects added/removed/changed), not a text diff.
- **Observation** — a measurement or event associated with a state or transition (health, latency, statistics).
- **Causal memory / causal graph** — the graph linking observations, events, and state transitions so that "what changed and why" is answerable. Not a metrics graph.
- **Causal relation** — a directed edge asserting that one event/transition contributed to another.
- **Replay / time travel** — reconstruction of network state (and associated observations) as of an earlier point, from immutable history.
- **Execution plan** — the explicit sequence of low-level actions that would realize a transition on a real network. Produced for every transition; inspected in simulation.
- **Execution backend** — an adapter that executes a plan against a substrate (simulation in v0.1; Linux namespaces later). Separated from state representation.
- **Simulation** — execution of a plan against a model, with no effect on the host or real network.
- **Provenance** — the recorded linkage from any network behavior or state to the transition that produced it.
- **Node / Link / Path / Flow / Service / Identity / Capability** — candidate core primitives (see [concepts.md](concepts.md)); which are fundamental vs. derived is an open research question.
- **ADR** — Architecture Decision Record; the immutable record of an architectural decision and its rationale ([docs/adr/](adr/)).
