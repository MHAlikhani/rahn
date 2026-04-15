<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Future Work

Status: living research document (Stage 0). Ordered by the staged roadmap; see [../ROADMAP.md](../ROADMAP.md) for the authoritative sequence.

## Near term (Stages 0–2)

- Deepen prior-art analysis with literature review (SIGCOMM/NSDI/HotNets, formal-methods venues); reclassify [prior-art.md](prior-art.md) provisional claims.
- Build the v0.1 state engine per the strict scope: state model, identity, persistence, transitions, semantic diff, branches, conservative merge, invariant engine, verification, simulation-only execution, CLI, extensive tests.
- Run E1–E3 experiments; promote or refute H1–H4.
- Enrich the model (0.2): addressing, interfaces, paths, services, stronger constraints, semantic merge conflicts, graph algorithms.

## Middle term (Stages 3–6)

- Linux namespace execution prototype (0.3); execution-plan vs. observed-effect differential testing (E6).
- Observability and provenance (0.4): link observations to transitions.
- Causal memory and replay (0.5): causal events/edges, incident reconstruction, `rahn explain`; E4–E5.
- Distributed state (0.6): derive consistency requirements from workloads (E7) *before* selecting replication/consensus mechanisms.

## Long term (Stages 7–10)

- Execution backends (0.7): Linux, namespaces, eBPF/XDP, selected programmable dataplanes — behind the backend abstraction.
- Network CI (0.8): `rahn test/verify/simulate/replay` in CI pipelines.
- Programmability (0.9): API, SDK, formalized IR, policy language; evaluate DSL design only once the IR is understood.
- 1.0 gate: stable semantics, documented APIs, trustworthy execution, credible security model, comprehensive tests, mature docs.

## Research directions

Causal network graphs · network constitutions · semantic network merge · network time travel · explainable routing · state provenance · safe execution · failure forecasting · network digital evolution (intended/observed/actual/historical alignment) · neuro-symbolic networking (AI proposes, architecture verifies). See [research-questions.md](research-questions.md) for the formal list.

## Publication pipeline

If experiments produce genuinely novel and reproducible findings (E4/E5 are the most likely candidates: causal network memory, semantic network merge), pursue: technical report → workshop paper → conference/systems paper. Do **not** manufacture novelty; claims require reproduced results and a completed prior-art check.

## Documentation evolution

Documentation site only if cheap and non-distracting (after v0.1); versioned white paper revisions tied to meaningful architectural changes; normative spec maturing from draft to implementable as stages complete.
