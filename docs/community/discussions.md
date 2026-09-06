<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# GitHub Discussions Categories

Discussions are for thinking in the open: comparing approaches, testing ideas against readers, answering questions, and showing work. They are not the decision record.

**The formal decision path remains: design-proposal issue - ADR** (CONTRIBUTING.md; GOVERNANCE.md). A discussion never replaces an ADR, and a popular discussion is not a decision. Where a conversation converges on a change, the change still enters through the issue template and, if it touches core semantics, through an ADR that is accepted on its own merits.

The repository has six categories. Three kinds of post once considered for their own category belong elsewhere: benchmark and experiment reports go to **Research Ideas**, roadmap reactions to **General Discussion**, and show-and-tell posts to **General Discussion**.

## Categories

| Category | Purpose | What belongs | What does not belong | Example of a good post |
|---|---|---|---|---|
| Announcements | Releases, accepted ADRs, milestone notes, governance updates | Maintainer announcements: tagged releases, newly accepted ADRs, notable limitation changes | Questions, proposals, or anything requiring a reply; non-maintainer posts | "RAHN v1.0.0 is released: stable architecture, canonical format v2 frozen for 1.x, 19 ADRs. Link to CHANGELOG and the v1.0 review." |
| Architecture & Design | Architectural ideas, trade-offs, and design reasoning before a formal proposal | Trade-off analyses; comparisons against the alternatives in ARCHITECTURE.md 5; failure-mode reasoning; "is this fundamental?" discussion | Implementation PRs; feature requests (use the feature template); anything presented as a decision | "Comparing event-sourcing projection against RAHN's commit-DAG for replay: what each makes cheap, and where the cost lands." |
| Research Ideas | Hypotheses, prior art, experiments, and research directions | Anything that could become a research-proposal issue or an E-experiment; literature pointers; hypotheses with a measurable result; benchmark and experiment reports | Design commitments; peak-number advertising | "Could E3's conflict corpus be generated from real config diffs? Here is what the incident literature offers and where it breaks." |
| Technical Questions | How do I build, run, or reason about RAHN | Build/setup problems; how a transition or invariant works; how to read a spec section; CLI behavior | Bug reports (use the bug template); design debates; security reports (private, per SECURITY.md) | "Why does `rahn test` report the structural floor as separate invariants from my constitution checks?" (accepted answer marked) |
| Community Introductions | Who you are and what you want to contribute | Short introductions: background, what you work on, what you would like to contribute, what you are reading first | Long project proposals (use Architecture & Design); support requests | "I work on network verification; I read the prior-art survey and would like to help reclassify the provisional claims in 9." |
| General Discussion | Everything that is none of the above | Meta discussion about the project and the community that has no better home; roadmap reactions; show-and-tell posts | Anything that fits a category above - put it there so it can be found | "Should the curated list in docs/community/good-first-issues.md stay short enough to stay accurate, or grow with the backlog?" |

## Standing rules

- **Announcements are maintainer-only.** Everyone else posts to the category that fits.
- **Measurement posts follow the methodology.** docs/research/benchmark-methodology.md rule 1 makes a result without a recorded environment and method void; a discussion post is held to the same standard the project holds its own results to. Negative results are explicitly welcome - they are the point of running experiments.
- **Roadmap replies imply no promises.** The post-v1.0 extensions are candidates; each requires its own ADR, specification, research evidence, and compatibility review before implementation, and nothing is scheduled (ROADMAP.md, ADR 0019).
- **Security issues do not go in Discussions.** Use the private channel in SECURITY.md.
- **Discussions do not decide.** Where a thread converges, take it to the relevant issue template; architectural change takes the design-proposal - ADR path.

## Repository setup

Discussions are enabled on the repository, with these categories and slugs:

| Category | Slug |
|---|---|
| Announcements | `announcements` |
| Architecture & Design | `architecture-design` |
| Research Ideas | `research-ideas` |
| Technical Questions | `technical-questions` |
| Community Introductions | `community-introductions` |
| General Discussion | `general-discussion` |

A welcome thread is pinned in **Community Introductions**, and release announcements are posted in **Announcements**. `.github/ISSUE_TEMPLATE/config.yml` sends issue writers to **Technical Questions** and **Research Ideas** by category URL.

New categories are created in the repository UI (**Settings - General - Features - Discussions**); the GitHub API exposes no create-category mutation, so this stays a manual maintainer action. Add a category here when the repository gains one, and keep the slugs above accurate - they are used in links.