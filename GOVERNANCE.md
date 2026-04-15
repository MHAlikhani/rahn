<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Governance

Status: provisional (Stage 0). Governance will mature as the contributor base grows; the design goal is long-term technical integrity, not founder control.

## Roles

- **Core maintainers** — own the architectural core, merge rights, and final calls on ADRs.
- **Subsystem maintainers** — own a crate or docs area (state, verification, execution backends, simulation, docs, research).
- **Contributors** — anyone with merged contributions.
- **Researchers** — contribute to the research corpus, experiments, and white paper; may hold no merge rights.
- **Ecosystem maintainers** — steward SDKs, integrations, and packaging outside the core.

## Decision making

- **Core semantics and architecture**: by ADR. An ADR is accepted when at least one maintainer other than the author approves it (once the maintainer group exceeds one). Accepted ADRs are immutable; superseding requires a new ADR that references the old one.
- **Ordinary changes**: normal review in pull requests.
- **Disputes**: technical arguments first, in writing; unresolved disputes are decided by the core maintainers and the reasoning is recorded (in the ADR or the PR).
- When documentation and implementation disagree, implementation is **not** automatically considered correct: the discrepancy is investigated and the wrong side is fixed.

## Anti-bottleneck principles

The project is designed so that no single person is a permanent bottleneck: subsystem ownership is distributed, decisions are written down (ADRs, docs), and maintainer onboarding is documented. Maintainers are added by consensus of existing maintainers based on sustained, high-quality contribution; maintainers may step down at any time and are thanked, not erased.

## Avoiding capture

Do not optimize for founder control; optimize for long-term technical integrity. Commercial entities may employ contributors but do not receive extra governance weight. The architectural core cannot be forked away informally — changes to it must go through the public ADR record.

## Project identity

The name RAHN is a project-level asset; do not rename it without a strong technical, legal, trademark, or ecosystem reason, established via ADR.
