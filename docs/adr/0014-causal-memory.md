<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# ADR 0014: Causal Memory — Anchors, Edges, Epistemic Status

- Status: Accepted
- Date: 2026-06-17

## Context

Stage 5 (charter): causal events, causal edges, state-linked incidents, incident reconstruction — **without overstating what the system proves**. The architecture already has two immutable record classes: observations (ADR 0013, positioned log) and commits (content-addressed DAG, ADR 0005).

## Alternatives

1. **Full causal-inference engine** (do-calculus, Bayesian networks): rejected — research overreach; RAHN records *asserted* structure, it does not discover it.
2. **Causality embedded in observations:** rejected — conflates measurement records with asserted relations and breaks the observation format.
3. **Separate edge log with explicit epistemic status (chosen).**

## Decision

1. **CausalEdge** = `{ seq, from: Anchor, to: Anchor, status, note }`:
   - **Anchor** = `Observation(seq)` or `Commit(commit-id)` — both immutable, both checkable for existence at edge creation.
   - **status** ∈ {`temporal-correlation`, `hypothesis`, `verified`}. Semantics are strict:
     - `temporal-correlation` — the recorder asserts only ordering/adjacency in time;
     - `hypothesis` — the recorder asserts a suspected causal link;
     - `verified` — MAY only be recorded when both anchors are `Commit`s (structurally checkable provenance); observation-anchored edges cannot be "verified" because no mechanism proves them. Enforced at creation.
2. **DAG invariant:** the edge set MUST remain acyclic; insertion is rejected if it would close a cycle (deterministic DFS check).
3. **Storage:** append-only, versioned-framed log (`.rahn/causal.log`), same framing rules as observations; corruption refused loudly; records immutable.
4. **Incident (v0.5 scope):** an incident is the connected component around an anchor in the edge graph — a *query result*, not a stored type. Named incident records are deferred.
5. **No inference:** the system never creates edges automatically; all edges are explicit, human (or future AI-layer) assertions whose status labels their epistemic strength.

## Consequences

- `rahn explain`-style narratives become bounded walks over asserted, status-labeled edges — the output is honest about being hypotheses.
- Stage 5 gate (charter): represent and query causal hypotheses tied to state and observations without overstating proof — satisfied structurally by the status rule and anchor validation.
