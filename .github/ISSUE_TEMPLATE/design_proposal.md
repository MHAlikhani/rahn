---
name: Design proposal (RFC)
about: Propose an architectural change — this is the path to an ADR
labels: ["design", "rfc"]
---

<!-- SPDX-License-Identifier: CC-BY-4.0 -->

Architectural changes follow: **discussion here → accepted → ADR → spec/ARCHITECTURE update → implementation.** Implementation is never first.

**Classification:** architectural change / semantics change / research direction.

**Problem.** What architectural problem exists? What breaks or becomes incoherent without the change?

**Prior art.** What existing systems or literature address this? (docs/research/prior-art.md is the baseline.) Never claim "no one has done this" without a scoped survey.

**Proposed design.** The model, its invariants, and its failure modes.

**Alternatives considered.** At least two, with reasons for rejection.

**Impact on core semantics.** State identity, canonical serialization, transitions, verification, merge, storage — what changes? Canonical-byte changes require a format-version bump.

**ADR plan.** Which new ADR (or amendment to which existing ADR) would record the decision if accepted?

**Reconsideration conditions.** Under what future evidence should this decision be revisited?
