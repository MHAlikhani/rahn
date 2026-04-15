<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Experiment Plan

Status: living research document (Stage 0). Experiments must be reproducible: every experiment records its code commit, environment, inputs, and raw results alongside the analysis.

## Principles

- Experiments test **hypotheses** ([hypotheses.md](hypotheses.md)), not marketing points.
- Every experiment defines: question, method, metrics, controls, environment, and a refutation condition, *before* running.
- Results are recorded even when negative. Every major failure should teach something.
- No claims of novelty or superiority without a baseline and a recorded methodology.

## Planned experiment sequence

### E1 — Determinism and identity (v0.1)
**Question (H1):** Does canonical serialization + content hashing produce stable state identity across runs, platforms, and benign input variation (key order, whitespace)?
**Method:** property tests; cross-platform CI matrix; adversarial mutation corpus.
**Metrics:** identity stability rate; canonicalization cost.
**Refutes H1 if:** identity differs across environments for identical logical state.

### E2 — Transition and verification cost (v0.1)
**Question (H3):** Can constitution checking gate every transition at acceptable cost?
**Method:** benchmark invariant checking vs. topology size (10²–10⁵ objects); per [benchmark-methodology.md](benchmark-methodology.md).
**Metrics:** verification latency distribution; checks/sec.

### E3 — Merge conflict detection (v0.2)
**Question (H4):** What fraction of injected semantic conflicts does merge detect; how many clean merges are falsely rejected?
**Method:** generate branch pairs from topology+constraint scenarios with known ground truth (conflicting / non-conflicting); measure precision/recall of conflict detection.
**Metrics:** detection precision/recall; false-rejection rate.

### E4 — Causal attribution (Stage 5)
**Question (H5):** Does transition-linked causal memory improve root-cause identification over telemetry-only analysis?
**Method:** fault-injection scenarios with ground-truth causal chains; compare RAHN explanation vs. baseline (metrics + correlation) on attribution precision/recall and time-to-explanation.
**Metrics:** attribution precision/recall; explanation completeness.

### E5 — Replay fidelity (Stage 5)
**Question (H6):** At what cost and fidelity can `rahn replay --at T` reconstruct past state and answer incident questions?
**Method:** record full evolution of scenarios; replay at sampled timestamps; compare against ground truth.
**Metrics:** reconstruction error; replay latency; storage growth.

### E6 — Execution plan fidelity (Stage 3+)
**Question (H8/RQ8):** Do planned effects match observed effects when executing against Linux namespaces?
**Method:** differential testing: execution plan vs. observed namespace state after apply.
**Metrics:** plan/observation mismatch rate by operation class.

### E7 — Distributed consistency workloads (Stage 6)
**Question (RQ7):** What read/write/staleness patterns do realistic RAHN deployments exhibit?
**Method:** trace-driven study of candidate deployment patterns before choosing a consistency model.

## Reproducibility requirements

- Experiment harness lives in-repo (`experiments/` once the workspace exists); deterministic inputs or generated-with-seed inputs.
- Every result document records: commit, date, hardware, OS, toolchain, methodology, raw data.
- Benchmarks follow [benchmark-methodology.md](benchmark-methodology.md) — no meaningless benchmark numbers in project materials.
