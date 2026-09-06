<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Research Questions

Status: living research document. Each question carries: motivation, hypothesis, proposed method, measurable result, and limitations. Evidence lands in [experiment-plan.md](experiment-plan.md) and the white paper's results section; **no question is considered answered without reproducible evidence** (docs/reproducibility.md).

---

## RQ1 — Deterministic, content-addressed network state

Can network state be modeled as a deterministic, content-addressed computational object?

- **Motivation:** every other goal (verification, branching, replay, provenance) presupposes that one logical state has exactly one identity, stable across machines and time.
- **Hypothesis (H1):** a useful fraction of network meaning (topology, metadata, and later policy/intent) fits a canonical, ambient-free representation whose identity is stable under benign variation.
- **Method:** canonical serialization with adversarial strictness tests; seeded property tests over random networks; cross-platform CI determinism runs (E1).
- **Measurable result:** identity stability rate across 200+ generated states and both CI platforms; round-trip exactness.
- **Limitations:** covers only modeled aspects of networks; unmodelable state (charter §2) is explicitly out of scope.
- **Status:** partially answered — implemented and test-enforced for topology+metadata; policy/intent/observation components remain.

## RQ2 — Verification before execution

Can network transitions be verified before execution?

- **Motivation:** the architecture's central safety claim; without a working gate, "verify before execute" is slogan, not property.
- **Hypothesis (H3):** real network invariants are expressible as deterministic predicates, checkable fast enough to gate every transition.
- **Method:** v0.1 invariant engine (structural + connectivity); grow the vocabulary in Stage 2; measure checking cost against topology scale (E2, benches/).
- **Measurable result:** per-invariant pass/fail with evidence on all candidate states; verification latency distribution at 10²–10⁵ objects.
- **Limitations:** verification covers only encoded invariants ("verified" means exactly that and nothing more); expressiveness vs. decidability tension (RQ4-related).
- **Status:** implemented; E2 partial (verification/path cost measured at 10²–10⁵ objects in the Stage 2 scaling run — [stages/v0.2.md](stages/v0.2.md)), full constraint-budget measurement pending.

## RQ3 — Semantic branches and merges

Can network branches and merges be defined semantically rather than textually?

- **Motivation:** textual merge cannot see constraint feasibility; wrong merges are worse than rejected merges.
- **Hypothesis (H4):** for non-conflicting topology changes, three-way semantic merge succeeds; for touching changes, conflicts are detected and explained, never silently resolved.
- **Method:** fail-closed merge with content-aware effect comparison; seeded property tests (commutativity for disjoint effects); conflict-class coverage tests; constraint-level infeasibility analysis remains open (the Stage 2 roadmap item was not delivered — [stages/v0.2.md](stages/v0.2.md)).
- **Measurable result:** detection precision/recall on injected conflict scenarios (E3); false-rejection rate on known-clean merges.
- **Limitations:** completeness is impossible in general; the design compensates by failing closed — measurable as the false-rejection rate.
- **Status:** implemented for the v0.1 vocabulary; E3 study pending.

## RQ4 — Constitution expressiveness

Can a constitution/invariant layer prevent invalid network states? (What invariant language is expressive enough for real constitutions yet decidable and deterministic?)

- **Motivation:** the constitution is the architecture's normative core; too weak is useless, too strong is undecidable.
- **Hypothesis:** a useful class (isolation, reachability, redundancy, control-plane preservation) is expressible in decidable predicates over the state model.
- **Method:** v0.1 vocabulary (structural + named connectivity); Stage 2 constraint engine; formal analysis of the language's decision procedures.
- **Measurable result:** set of expressible real-world invariants (drawn from incident literature) that check within per-transition budget; refutation = invariants that resist decidability.
- **Limitations:** "prevents" is scoped to encoded invariants and candidate states RAHN sees; unmodelable state escapes (see RQ12-analog in limitations).
- **Status:** open; v0.1 vocabulary implemented.

## RQ5 — Causal attribution fidelity

Can observations be linked reliably to state transitions?

- **Motivation:** `rahn explain` is trustworthy only if attribution is.
- **Hypothesis (H5):** transition-anchored causal memory yields root-cause explanations measurably better than telemetry-only analysis.
- **Method:** fault-injection scenarios with ground-truth causal chains (E4); compare attribution precision/recall against a telemetry-only baseline.
- **Measurable result:** attribution precision/recall; time-to-explanation.
- **Limitations:** imperfect telemetry and concurrent changes bound fidelity; output must distinguish temporal correlation, causal hypothesis, and verified relation.
- **Status:** open; the mechanisms exist (observations per ADR 0013, asserted edges per ADR 0014), but E4 has not run.

## RQ6 — Incident reconstruction / replay

Can network incidents be reconstructed from state + observations?

- **Motivation:** network time travel is a headline capability; its feasibility must be measured, not assumed.
- **Hypothesis (H6):** replay from immutable history reproduces state (and useful observations) at acceptable cost and fidelity.
- **Method:** record full scenario evolution; replay at sampled timestamps (E5); compare against ground truth.
- **Measurable result:** reconstruction error; replay latency; storage growth.
- **Limitations:** observations capture only what was instrumented; fidelity is intrinsically partial.
- **Status:** open; E5 has not run. (State-only replay of committed history is already possible via `rahn inspect`.)

## RQ7 — Consistency requirements for distributed state

What consistency model does replicated RAHN state actually need?

- **Motivation:** charter forbids adopting Raft/Paxos/CRDTs by familiarity; requirements must come first.
- **Hypothesis:** RAHN's write patterns (low-frequency, human-scale, merge-aware) admit weaker or different consistency than generic consensus assumes.
- **Method:** workload study of candidate deployment patterns (E7) *before* mechanism selection; document findings as ADR.
- **Measurable result:** a written requirements specification (write rates, staleness tolerance, conflict semantics) plus an evaluated candidate list.
- **Limitations:** requirements derived from assumed workloads may miss real ones; revisited on evidence.
- **Status:** requirements derived and recorded (ADR 0015); the workload study (E7) has not run.

## RQ8 — Multi-backend execution

Can the same abstract state model target multiple execution backends?

- **Motivation:** representation must not be captured by any one substrate (Linux, eBPF, SDN controllers…).
- **Hypothesis (H8 until Stage 3):** simulation-only execution satisfies v0.1–v0.2; the execution-plan format is a stable seam for backends.
- **Method:** keep plan format specified and test-frozen; Stage 3 namespace backend as first real target; differential testing of planned vs. observed effects (E6).
- **Measurable result:** plan/observation mismatch rate by operation class; number of backends implementable without core changes.
- **Limitations:** backend fidelity varies; mismatches are expected and must be reported, not hidden.
- **Status:** open; the seam and two backends are implemented (`rahn-sim`, `linux-ns` — ADR 0012/0016), and the plan/observation differential (E6) is partial.

## RQ9 — AI as advisor, not authority

Can AI assist with network reasoning without becoming a trusted execution authority?

- **Motivation:** charter mandates AI-optional, verify-gated; the interesting question is whether AI adds measurable value under that constraint.
- **Hypothesis:** AI-proposed candidates, forced through the same deterministic verifier, add value (candidate quality, diagnosis speed) without weakening safety.
- **Method:** only after Stages 2+; measure candidate acceptance rate and verifier catch rate of AI proposals vs. heuristics; AI never bypasses the gate by construction.
- **Measurable result:** value-add metrics with the verifier held fixed; safety metrics provably unchanged.
- **Limitations:** value depends on model quality; the architecture's correctness must be shown independent of it (RQ10).
- **Status:** open; deliberately deferred.

## RQ10 — Usefulness without AI

Can this abstraction remain useful without AI?

- **Motivation:** the inverse test of RQ9 and a charter requirement.
- **Hypothesis (H9):** a fully deterministic core (no AI, no dynamic behavior) supports real operator workflows.
- **Method:** the v0.1/v0.2 feature set is intentionally AI-free; measure whether documented workflows (validate change, branch experiment, diagnose with evidence) complete without any model.
- **Measurable result:** workflows completed AI-free; refutation = workflow gaps that demand nondeterministic help inside the core.
- **Limitations:** "useful" is partly qualitative; anchor to concrete scenario completion.
- **Status:** holding by design; re-examined each stage.
