<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Future Work

Status: living research document (updated at v1.0.0). The staged program below is delivered through v1.0.0; see [../ROADMAP.md](../../ROADMAP.md) for what shipped and for the post-v1.0 extension list.

## Near term (Stages 0–2) — delivered

- Deepen prior-art analysis with literature review (SIGCOMM/NSDI/HotNets, formal-methods venues); reclassify [prior-art.md](prior-art.md) provisional claims. *(Still open.)*
- Build the v0.1 state engine per the strict scope: state model, identity, persistence, transitions, semantic diff, branches, conservative merge, invariant engine, verification, simulation-only execution, CLI, extensive tests. *(Shipped in `v0.1.0-alpha.1`.)*
- Run E1–E3 experiments; promote or refute H1–H4. *(E1 ran as tests; E2 is partial (Stage 2 scaling); E3 is still open.)*
- Enrich the model (0.2): addressing, interfaces, paths, services, stronger constraints, semantic merge conflicts, graph algorithms. *(Interfaces, paths, isolation constraints, and graph algorithms shipped in `v0.2.0-alpha`; addressing, services, and constraint-level merge analysis remain post-v1.0 extensions.)*

## Middle term (Stages 3–6) — delivered

- Linux namespace execution prototype (0.3); execution-plan vs. observed-effect differential testing (E6). *(Backend shipped in `v0.3.0-alpha`; E6 is partial — one CI-validated scenario.)*
- Observability and provenance (0.4): link observations to transitions. *(Shipped in `v0.4.0-alpha`.)*
- Causal memory and replay (0.5): causal events/edges, incident reconstruction, `rahn explain`; E4–E5. *(Edges and `rahn explain` shipped in `v0.5.0-alpha`; narrative incident reconstruction and E4–E5 remain open.)*
- Distributed state (0.6): derive consistency requirements from workloads (E7) *before* selecting replication/consensus mechanisms. *(Requirements derived and recorded in ADR 0015; peer sync shipped in `v0.6.0-alpha`; E7 has not run.)*

## Long term (Stages 7–10) — delivered

- Execution backends (0.7): Linux, namespaces, eBPF/XDP, selected programmable dataplanes — behind the backend abstraction. *(`ExecutionBackend` with `simulation` and `linux-ns` shipped in `v0.7.0-alpha`; dataplane backends remain candidates.)*
- Network CI (0.8): `rahn test/verify/simulate/replay` in CI pipelines. *(`rahn test` shipped in `v0.8.0-alpha`, ADR 0017; `simulate`/`replay` do not exist.)*
- Programmability (0.9): API, SDK, formalized IR, policy language; evaluate DSL design only once the IR is understood. *(SDK and IR shipped in `v0.9.0-alpha`, ADR 0018; the DSL is deliberately deferred.)*
- 1.0 gate: stable semantics, documented APIs, trustworthy execution, credible security model, comprehensive tests, mature docs. *(Evaluated and met in [v1.0-review.md](v1.0-review.md), with limitations carried forward explicitly.)*

## Research directions

Causal network graphs · network constitutions · semantic network merge · network time travel · explainable routing · state provenance · safe execution · failure forecasting · network digital evolution (intended/observed/actual/historical alignment) · neuro-symbolic networking (AI proposes, architecture verifies). See [research-questions.md](research-questions.md) for the formal list.

## Publication pipeline

If experiments produce genuinely novel and reproducible findings (E4/E5 are the most likely candidates: causal network memory, semantic network merge), pursue: technical report → workshop paper → conference/systems paper. Do **not** manufacture novelty; claims require reproduced results and a completed prior-art check.

## Documentation evolution

Documentation site only if cheap and non-distracting (post-v1.0); versioned white paper revisions tied to meaningful architectural changes; the normative spec matured with the stages and remains marked Draft.
