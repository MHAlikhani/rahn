---
name: Research proposal
about: Propose or advance a research question or experiment
labels: ["research"]
---

<!-- SPDX-License-Identifier: CC-BY-4.0 -->

**Research question.** Link the RQ number it advances ([docs/research/research-questions.md](../../docs/research/research-questions.md)) or propose a new one. State the question as something evidence can answer.

**Motivation and hypothesis.** Why the question matters to the architecture, and the hypothesis in one falsifiable sentence.

**Relationship to existing work.** [docs/research/prior-art.md](../../docs/research/prior-art.md) is the baseline: what is already known or built here, and what this proposal adds or reclassifies. Never claim "no one has done this" without a scoped survey.

**Proposed method.** How the question would be investigated: inputs, procedure, tooling. Deterministic and seeded by construction (no wall-clock, no network, no system randomness in inputs).

**Measurable result.** What evidence would answer the question — the number, distribution, or classification that counts as an answer, and what refutation would look like.

**Reproducibility plan.** Per [docs/reproducibility.md](../../docs/reproducibility.md): seeded inputs or a generation procedure with seeds, a recorded environment block (docs/research/benchmark-methodology.md rule 1 — a result without it is void), and where raw outputs will live. A result that cannot be rerun from the repository is not quotable.

**Limitations.** What this method cannot show, and which parts of the question it leaves open.
