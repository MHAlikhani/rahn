<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Specification: Causality

Status: **Draft — normative for v0.5.** The causal-edge model (ADR 0014) is implemented; causal *inference* remains out of scope.

Normative keywords: MUST / MUST NOT / SHOULD / SHOULD NOT / MAY (RFC 2119 sense). Normative language is used only where behavior is intentionally defined.

## Reserved

Causal *inference* is a research direction (RQ5, H5) and is reserved: implementations MUST NOT infer edges. Causal edges MUST reference recorded events/transitions, attribution SHOULD carry confidence information (the `status` label), and `rahn explain` output MUST distinguish established causality from hypothesis.

## Normative seeds (v0.5)

1. A causal edge MUST be `{ seq, from: Anchor, to: Anchor, status, note }` where Anchor is `Observation(seq)` or `Commit(commit-id)`.
2. Anchors MUST exist at edge creation (observation seq in the observation log; commit id in the store); dangling anchors MUST be rejected.
3. `status` MUST be `temporal-correlation`, `hypothesis`, or `verified`. A `verified` edge MUST have `Commit` anchors on both ends.
4. The edge set MUST remain acyclic; an insertion that closes a cycle MUST be rejected.
5. Edges MUST be stored append-only with versioned framing; corruption MUST be refused loudly.
6. Implementations MUST NOT generate edges automatically and MUST NOT present hypotheses as verified causality. Query output MUST carry edge status.
