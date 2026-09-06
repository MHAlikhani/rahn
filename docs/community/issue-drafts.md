<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Community Issue Drafts

Ready-to-file GitHub issues for RAHN. Each draft below is meant to be filed as **one issue**, using the seven-part format (title, problem statement, why this matters, technical context, possible approach, acceptance criteria, difficulty, labels).

These are drafts, not commitments. Filing an issue does not schedule it; a research or extension item stays a candidate until the work is done and recorded (see ADR 0019 for the extension model). Every grounding reference below was checked against the repository at the time of writing; file-time re-checking is part of the filing process.

## Filed issues

All fourteen drafts are filed. Two titles were reworded when filed (draft 9 appears as "Document rahn-sdk public API examples", draft 12 as "Improve error messages with actionable failure explanations"). Issues #19-#21 were filed by the maintainer and did not come from these drafts.

| # | Draft | Issue |
|---|---|---|
| 1 | Run E2: verification-latency measurement | [#17](https://github.com/MHAlikhani/rahn/issues/17) |
| 2 | Run E3: merge conflict-detection study | [#18](https://github.com/MHAlikhani/rahn/issues/18) |
| 3 | Design RFC: copy-on-write or batched apply | [#22](https://github.com/MHAlikhani/rahn/issues/22) |
| 4 | RQ4 study: constitution expressiveness | [#23](https://github.com/MHAlikhani/rahn/issues/23) |
| 5 | Independent host-safety review of the linux-ns backend | [#24](https://github.com/MHAlikhani/rahn/issues/24) |
| 6 | Structured fuzzing for the canonical deserializer | [#16](https://github.com/MHAlikhani/rahn/issues/16) |
| 7 | Deepen the prior-art survey | [#11](https://github.com/MHAlikhani/rahn/issues/11) |
| 8 | ADR index and reading guide | [#12](https://github.com/MHAlikhani/rahn/issues/12) |
| 9 | rahn-sdk public API documentation | [#13](https://github.com/MHAlikhani/rahn/issues/13) |
| 10 | Verify binary reproducibility | [#25](https://github.com/MHAlikhani/rahn/issues/25) |
| 11 | Add macOS to the CI determinism matrix | [#14](https://github.com/MHAlikhani/rahn/issues/14) |
| 12 | Error-message audit | [#15](https://github.com/MHAlikhani/rahn/issues/15) |
| 13 | Research proposal: typed addressing | [#26](https://github.com/MHAlikhani/rahn/issues/26) |
| 14 | Research proposal: signed transitions | [#27](https://github.com/MHAlikhani/rahn/issues/27) |

## Label setup

`bug`, `documentation`, `good first issue`, and `help wanted` are GitHub defaults and need no setup. The labels used by these drafts are not defaults; create them once in the repository before filing:

```console
gh label create research     --color 5319e7 --description "Research questions, experiments, and studies (RQ/E program)"
gh label create engineering  --color 1d76db --description "Implementation work in the Rust crates"
gh label create performance  --color fbca04 --description "Measurement, benchmarks, and recorded performance debt"
gh label create security     --color ee0701 --description "Security posture, reviews, and hardening"
gh label create design       --color c5def5 --description "Architectural design work (ADR path)"
gh label create rfc          --color d4c5f9 --description "Requests for comment on proposed changes"
gh label create feature      --color a2eeef --description "Feature proposals (feature_proposal template)"
gh label create benchmark    --color f9d0c4 --description "Benchmark runs and methodology work"
gh label create dx           --color 0e8a16 --description "Developer and contributor experience"
```

`design`, `rfc`, and `feature` are referenced by the existing issue templates' front matter, so they must exist for `gh issue create --label` to succeed; skip any `gh label create` invocation that reports the label already exists.

---

## 1. Run E2: verification-latency measurement across 10²–10⁵ objects

**Title:** Run E2: verification-latency measurement across 10²–10⁵ objects

**Problem statement.** RQ2's measurable result — "verification latency distribution at 10²–10⁵ objects" — has not been measured. RQ2 is implemented for the v0.1 vocabulary but explicitly records "cost measurement pending", so hypothesis H3 ("real network invariants are expressible as deterministic predicates, checkable fast enough to gate every transition") has no evidence behind it.

**Why this matters.** Verification gates every transition; it is the architecture's central safety claim, and without a cost distribution the claim is untested. Because verification sits in the critical path, tail latency matters more than the mean (benchmark-methodology rule 5: "verification gates every transition — p99 is the story"). This is also one of the named gaps in the current evidence base (limitations.md: the full experiment program, E2 completion and E3–E7, has not run).

**Technical context.**
- `docs/research/research-questions.md` — RQ2, status "implemented for the v0.1 vocabulary; cost measurement pending".
- `docs/research/benchmark-methodology.md` — rules 1–7: environment block or the result is void; recorded method with median + intervals; 10²–10⁵ scale; baseline comparison; never average away the tail; publish negative results; reproducibility over peak numbers. Reporting format: hypothesis · environment · methodology · results with intervals · baseline comparison · interpretation and limitations · raw data location.
- `crates/rahn-verify/` — the invariant engine being measured (structural floor plus named connectivity).
- `crates/rahn-state/tests/scaling.rs` — existing ignored benchmark harness, the precedent for a repo-runnable harness.
- `docs/research/limitations.md` — recorded performance debt: repeated operation application clones the whole network (~O(m·n) for m batched operations on n objects).

**Possible approach.** Build a criterion-style harness per methodology rule 2, driving the existing verification path over seeded synthetic topologies spanning 10² to 10⁵ objects, with and without a constitution. Record the full environment block, run warmup plus repeated iterations, and report median with intervals plus p99 per invariant and per scale. Compare a no-constitution run against a constitution-bearing run as the baseline pair in the same environment. Publish the results as a research document in `docs/research/` with raw data location. Because the recorded O(m·n) apply debt is present in this build, state explicitly that these results establish the **pre-optimization baseline** (they are not a marketing benchmark and should not be cited as the architecture's cost without noting the debt). Negative results — for example latency that does not meet a per-transition budget — are recorded, not hidden.

**Acceptance criteria.**
- A results document exists in `docs/research/` containing: hypothesis, environment block, methodology, results table or plot with intervals, baseline comparison, interpretation and limitations, and the raw data location.
- Measurements cover at least the 10², 10³, 10⁴, and 10⁵ object scales, with p99 reported separately from the median.
- The recorded O(m·n) apply debt is named in the interpretation, and the results are labeled as the pre-optimization baseline.
- RQ2's status line in `docs/research/research-questions.md` is updated to reflect what was measured and what remains open.

**Difficulty:** Intermediate.

**Labels:** `performance`, `engineering`, `help wanted`

---

## 2. Run E3: semantic merge conflict-detection study on injected conflict scenarios

**Title:** Run E3: semantic merge conflict-detection study on injected conflict scenarios

**Problem statement.** RQ3's measurable result — "detection precision/recall on injected conflict scenarios (E3); false-rejection rate on known-clean merges" — has not been measured. RQ3 is implemented for the v0.1 vocabulary with status "E3 study pending", so hypothesis H4 ("conflicts are detected and explained, never silently resolved") is untested, and the known cost of the fail-closed design (rejecting some mergeable states) is unquantified.

**Why this matters.** Wrong merges are worse than rejected merges, so the design deliberately fails closed — but that choice has a measurable price: the false-rejection rate. Measuring precision, recall, and false rejections is the honest counterweight to the design's conservatism, and it directly addresses the architectural limitation that semantic merge is incomplete in principle (limitations.md item 3).

**Technical context.**
- `docs/research/research-questions.md` — RQ3, status "implemented for the v0.1 vocabulary; E3 study pending"; measurable result and limitations stated there.
- `docs/testing.md` — the merge test row for `crates/rahn-verify/src/merge.rs`: disjoint merges, identical-effect coalescing, conflict rejection, fail-closed application errors, metadata preservation.
- `crates/rahn-verify/src/merge.rs` — the fail-closed three-way merge; its header rules state that both sides touching the same object succeeds only if effects are identical, and "false rejections are acceptable; false accepts are not".
- `crates/rahn-state/tests/properties.rs` — seeded property tests including merge of disjoint removals being commutative (100 seeds), the closest existing evidence.
- `docs/reproducibility.md` — the experiment reproducibility policy (seeds, environment, method recorded before results are quotable).

**Possible approach.** Construct two corpora: an injected-conflict corpus (both sides editing the same object; identical-effect coalescing; disjoint edits; metadata collisions) and a known-clean corpus (merges that are demonstrably satisfiable). Generate scenarios from seeded inputs so any case replays exactly (testing.md principle 3). Run them through the merge path and compute detection precision, detection recall, and the false-rejection rate on the clean corpus. Record the environment and method per benchmark-methodology rules and reproducibility policy.

**Acceptance criteria.**
- Both corpora are committed to the repository with their generation seeds and the procedure to regenerate them.
- A results document reports detection precision, detection recall, and false-rejection rate, with the environment and method recorded.
- Each conflict class in the injected corpus is labeled, and the results are reported per class.
- RQ3's status line is updated with what the study answered and what remains open (constraint-level conflicts, if the vocabulary does not yet reach them).

**Difficulty:** Intermediate.

**Labels:** `research`, `help wanted`

---

## 3. Design RFC: copy-on-write or batched apply for repeated transition application

**Title:** Design RFC: copy-on-write or batched apply for repeated transition application

**Problem statement.** Repeated operation application currently clones the whole network, giving roughly O(m·n) cost for m batched operations on an n-object network. This is recorded, known performance debt rather than an accident: limitations.md flags it explicitly and states that a copy-on-write or batched-apply design "would require its own ADR". No design work has been started.

**Why this matters.** The debt bounds batch throughput and interacts with the E2 verification-latency baseline; it also constrains how large a network a single transition can practically address. Any change here touches core semantics (purity, determinism, transition behavior), which is exactly the class of change that must go through the ADR path.

**Technical context.**
- `docs/research/limitations.md` — the recorded performance-debt paragraph, including the instruction "Do not benchmark future changes against this as-is without noting the debt".
- `docs/research/v1.0-review.md` — carried limitation 2: "O(m·n) batch-apply and per-object sync transport (recorded debts)".
- `crates/rahn-sdk/src/lib.rs` — `apply_all` is the public batch entry point.
- `docs/testing.md` — `crates/rahn-state/src/transition.rs` pins purity (input untouched), totality, and structured rejections; any redesign must preserve those properties.
- `DESIGN.md` principles 1–2 — determinism first; explicit transitions, no in-place mutation.
- `.github/ISSUE_TEMPLATE/design_proposal.md` — the template for the output of this work.

**Possible approach.** Benchmark the current apply cost first (this is the pre-optimization baseline; coordinate with the E2 issue so both use the same environment and method). Then evaluate at least three directions: copy-on-write structural sharing, an explicit batched-apply API that avoids per-operation cloning, and accepting-and-documenting the cost with the debt recorded. The comparison must state the effect on determinism and purity, the public-API surface impact (`rahn-sdk` is semver-stable), and a benchmark plan that can show the change is a net improvement. The deliverable is an **ADR draft filed through the design_proposal template**, with alternatives considered and reconsideration conditions. **No implementation before the ADR is accepted** — this issue produces design evidence, not code.

**Acceptance criteria.**
- Current apply cost is recorded with an environment block under the benchmark methodology.
- An ADR draft is filed covering at least the three directions above, with trade-offs and rejected alternatives.
- The draft states the determinism and purity impact and how the tests in `transition.rs` would constrain the change.
- No implementation PR accompanies the ADR draft; implementation begins only after acceptance.

**Difficulty:** Advanced.

**Labels:** `performance`, `engineering`

---

## 4. RQ4 study: constitution expressiveness against real-world network invariants

**Title:** RQ4 study: constitution expressiveness against real-world network invariants

**Problem statement.** RQ4 is open: can a constitution/invariant layer prevent invalid network states — that is, what invariant language is expressive enough for real constitutions yet decidable and deterministic? The current vocabulary is a structural floor plus named connectivity checks, and it has never been evaluated against invariants drawn from real network incidents and operations.

**Why this matters.** The constitution is the architecture's normative core, and the failure mode runs both ways: a vocabulary too weak does not prevent the states that matter, and one too strong becomes undecidable. RQ4's measurable result is a set of expressible real-world invariants that check within a per-transition budget, with refutation being invariants that resist decidability. This study produces the evidence that would inform any future constraint-engine design.

**Technical context.**
- `docs/research/research-questions.md` — RQ4, status "open; v0.1 vocabulary implemented".
- `docs/spec/invariants.md` — the structural invariant floor (`referential-integrity`, `link-endpoints-exist`, `no-duplicate-links`, `no-self-loops`) plus `named-connectivity`/`prohibited-connectivity` from the constitution.
- `docs/spec/constitution.md` — the structural invariants plus `require-connectivity a b` and `prohibit-connectivity a b`, evaluated over the node graph induced by interface links.
- `crates/rahn-verify/` — `constitution.rs` (parsing) and `invariants.rs` (checks).
- `docs/research/prior-art.md` — the formal-methods family (Header Space Analysis, VeriFlow, NetKAT, Minesweeper) as the reference for what formal verification of networks can express.

**Possible approach.** Collect candidate invariants from the incident literature and the prior-art survey (isolation, reachability, redundancy, control-plane preservation, and the examples named in docs/concepts.md). For each, attempt to express it in the current vocabulary and classify the result as expressible, inexpressible, or undecidable, citing the source of the invariant. Where an invariant is expressible, note whether it can be checked within a per-transition budget; where it is inexpressible, state what vocabulary extension would be required (without designing it). Produce an expressibility report that feeds future constraint-engine design. This is a study: no implementation is required, and no novelty claims may be made.

**Acceptance criteria.**
- An expressibility report exists with each collected invariant cited to its source and classified expressible / inexpressible / undecidable.
- Expressibility claims are demonstrated against the actual vocabulary in `docs/spec/constitution.md` and `crates/rahn-verify/`, not paraphrased.
- The report states which inexpressible invariants would require vocabulary extensions, as input to a future design, without committing to one.
- RQ4's status line is updated.

**Difficulty:** Intermediate.

**Labels:** `research`, `help wanted`

---

## 5. Independent host-safety review of the linux-ns execution backend

**Title:** Independent host-safety review of the linux-ns execution backend

**Problem statement.** The Linux execution path — the only code path in RAHN that touches a real network or OS — is validated by CI only. It has not been independently reviewed for host-safety, namespace isolation, or cleanup behavior. An independent adversarial review is the step that turns "our tests assert this" into "someone else checked our tests".

**Why this matters.** ADR 0012 rests on a structural host-safety invariant: host-side operations are limited to `ip netns add/del rahn-*`, with all link and interface mutations running inside `ip netns exec rahn-*`. Tests assert this structurally, but structural tests can encode the same blind spot as the implementation they check. Namespace isolation, failure cleanup, and the boundary of what can reach the host are exactly the areas where a second set of eyes is worth most.

**Technical context.**
- `docs/research/limitations.md` — "the Linux execution path is validated by CI only".
- `docs/adr/0012-linux-ns-execution.md` — the decision: shelling out to `ip`; deterministic `rahn-<node-id>` namespace naming; the host-safety invariant; `--execute` requires `--yes-i-know`; non-Linux platforms refuse at runtime; execution stops at the first failing command; `rahn destroy` cleanup.
- `.github/workflows/ci.yml` — the `linux-execution` job (requires root for netns) and the host-safety unit tests.
- `crates/rahn-exec/` — the mapping from plans to argv, plus the host-safety tests.
- `SECURITY.md` — the private reporting path for security-sensitive findings.

**Possible approach.** Perform an adversarial review of: namespace isolation (can a created topology affect anything outside its namespaces?); the host-side `rahn-*` restriction (can any non-exempt argv reach the host namespace, including through argument injection in node identifiers?); cleanup paths (interrupted runs, partial topologies, `rahn destroy`); and failure modes (first-failing-command stop, half-created state). The output is a written review report. Security-sensitive discoveries go to the maintainers privately per SECURITY.md; only non-sensitive findings become public follow-up issues. The report should not overstate: it may conclude the invariant holds, and it may conclude it does not.

**Acceptance criteria.**
- A written review report exists, public to the extent its findings are non-sensitive.
- Each finding is triaged as a fix, an ADR-worthy design issue, or accepted-and-documented, with a linked issue or PR.
- Security-sensitive items were routed privately per SECURITY.md and are noted as such in the public report.
- The "validated by CI only" wording in limitations.md is updated only if the review provides evidence beyond CI.

**Difficulty:** Advanced.

**Labels:** `security`, `help wanted`

---

## 6. Introduce structured fuzzing for the canonical state deserializer

**Title:** Introduce structured fuzzing for the canonical state deserializer

**Problem statement.** The canonical state parser is the untrusted-input boundary, and structured malformed-input tests cover the classes the authors thought of. There is no continuous fuzzer. docs/testing.md records this explicitly: "structured malformed-input tests exist, but no continuous fuzzer. Planned when the parse surface grows."

**Why this matters.** Parsing is required to be strict and total (docs/spec/security.md: all loaded state is untrusted input; malformed or corrupted state is rejected loudly, never partially loaded). Enumerated tests verify that; a fuzzer searches for the classes nobody enumerated — panics, hangs, or partial states on inputs that are neither well-formed nor in the test set. Determinism helps here: with seeded inputs, every crash reproduces exactly.

**Technical context.**
- `docs/testing.md` — the fuzzing paragraph and the regression rule ("Every regression gets a test").
- `crates/rahn-state/src/canonical.rs` — the format-v2 parser and its malformed-input tests: rejection of unknown versions, trailing bytes, truncation, unsorted content, dangling endpoints, unnormalized endpoints.
- `docs/spec/security.md` — strict, total parsing as a MUST.
- `docs/spec/serialization.md` — the normative format being parsed.
- `docs/reproducibility.md` — the determinism basis: no system randomness in test inputs; seeded PRNG for generation.

**Possible approach.** Add a `cargo-fuzz` (or `cargo-afl`) target over `parse_canonical` for the format-v2 input. Seed the corpus from the existing malformed-input classes plus valid canonical round-trip outputs, so the fuzzer starts from both sides of the boundary. Run it locally and in a non-blocking CI job initially. Any crash or hang becomes a named regression test per the testing.md rule; because the property tests are seeded and deterministic, every crash must be reproducible from a seed.

**Acceptance criteria.**
- A fuzz target is committed with a seed corpus and documented run instructions.
- The fuzzer has been run long enough to state a result honestly (coverage reached, crashes found or none found within the stated budget).
- Every confirmed finding is fixed with a regression test that fails before the fix.
- `docs/testing.md` is updated to reflect that a fuzzer now exists.

**Difficulty:** Intermediate.

**Labels:** `engineering`, `security`

---

## 7. Deepen the prior-art survey with a systems/formal-methods literature pass

**Title:** Deepen the prior-art survey with a systems/formal-methods literature pass

**Problem statement.** docs/research/limitations.md states plainly that the prior-art analysis "must be deepened before the white paper asserts any novelty", and prior-art.md identifies itself as a working survey whose claims are provisional and whose primary citations are still to be added. The white paper therefore cannot yet support novelty claims, and the survey's own scoping statement requires re-validation against a systematic literature review.

**Why this matters.** The publication pipeline is explicit that novelty must not be manufactured; claims require reproduced results and a completed prior-art check. Until the survey is deepened, the project is limited to scoped language, and the provisional RAHN-specific claims in prior-art.md §9 cannot be promoted, corrected, or scoped down.

**Technical context.**
- `docs/research/prior-art.md` — the survey-method note ("primary citations (papers, RFCs, specification documents) are to be added per-section"), the provisional claims in §9, and the scoping statement.
- `docs/research/future-work.md` — near-term list: "Deepen prior-art analysis with literature review (SIGCOMM/NSDI/HotNets, formal-methods venues); reclassify prior-art.md provisional claims."
- `docs/research/research-questions.md` — the RQs the survey feeds.
- `docs/whitepaper/RAHN-Whitepaper.md` — claim-status markers that depend on this survey.

**Possible approach.** Work claim by claim: for each major section and each §9 provisional claim, check the systems and formal-methods literature (SIGCOMM/NSDI/HotNets and formal-methods venues) and attach primary citations. Reclassify each provisional claim as supported, refuted, or scoped down. Where a claim collapses, record the collapse — do not defend it. No Rust is required; this is a reading and writing task.

**Acceptance criteria.**
- prior-art.md has primary citations per major section (papers, RFCs, or specification documents).
- Each §9 provisional claim is reclassified with the evidence that supports the classification.
- The survey-method note reflects the deepened state; remaining gaps are stated.
- No uncited claim is presented as verified, and no new novelty claim is asserted.

**Difficulty:** Beginner (no Rust needed).

**Labels:** `research`, `documentation`, `good first issue`

---

## 8. Write an ADR index and reading guide for docs/adr/

**Title:** Write an ADR index and reading guide for docs/adr/

**Problem statement.** There are 19 ADRs in docs/adr/ (0001–0019) and no index or README. A newcomer following the contribution path reaches a directory of decision records with no guided entry point and no statement of how the records relate to each other.

**Why this matters.** ADRs are the authority in this project — GOVERNANCE.md makes them the record that outranks any individual, and ARCHITECTURE.md "summarizes, it does not decide". The ADR path is also how architectural change happens, so the cost of a reader getting lost in that directory is the cost of the contribution model itself.

**Technical context.**
- `docs/adr/0001-project-scope.md` through `docs/adr/0019-v1-0-stability.md`.
- `ARCHITECTURE.md` — "Decisions are recorded in docs/adr/; this document summarizes, it does not decide."
- `CONTRIBUTING.md` — the design-proposal → ADR → spec/ARCHITECTURE → implementation path.
- `GOVERNANCE.md` — accepted ADRs are immutable; superseding requires a new ADR that references the old one.
- `docs/concepts.md` — the concept prose that an index can cross-link.

**Possible approach.** Add an index document in docs/adr/ with a one-line summary per ADR, grouped by theme (state model, execution, observability, distribution, stability), plus a suggested reading order — one order for newcomers who want the shape of the system, another for implementers who need the binding decisions. Cross-link from the concepts in docs/concepts.md where a concept maps naturally to an ADR. Do not alter ADR content: accepted ADRs are immutable.

**Acceptance criteria.**
- An index document exists in docs/adr/ listing all 19 ADRs with a one-line summary each.
- The summaries are grouped by theme and a reading order is given for at least two audiences (newcomer, implementer).
- docs/concepts.md links to the index where natural, without changing any concept definition.
- No ADR file content is modified.

**Difficulty:** Beginner.

**Labels:** `documentation`, `good first issue`

---

## 9. Audit rahn-sdk public API documentation

**Title:** Audit rahn-sdk public API documentation

**Problem statement.** README designates `rahn-sdk` as "the curated, doctest-covered public API", and ADR 0019 makes it the stable semver surface for external code. Its documentation coverage per public item has not been audited, and the `missing_docs` lint is not enabled in the crate, so undocumented surface can drift silently between releases.

**Why this matters.** The SDK is the contract external code builds against. A public item without documented semantics or a runnable example is a contract with an unwritten clause; because the SDK is semver-stable from 1.0, drift here is expensive to correct later.

**Technical context.**
- `crates/rahn-sdk/src/lib.rs` — the prelude re-exporting types from rahn-core, rahn-state, rahn-verify, rahn-sim, and rahn-store; crate-level docs with doctests already exist, and there is currently no `#![warn(missing_docs)]`.
- `docs/adr/0018-programmability-ir.md` — the curated facade decision.
- `docs/adr/0019-v1-0-stability.md` — semver stability and the stable surface definition (`rahn-sdk`, CLI behavior contracts, the IR).
- `docs/spec/` — the normative documents that per-item docs should cross-link (serialization and objects for identity, transitions for the IR).
- `docs/testing.md` principle 3 — determinism extends to tests: no wall-clock, network, or system randomness in doctests.

**Possible approach.** Walk every item reachable from the prelude and check that it documents its semantics and carries at least one doctest example. Cross-link each item to the normative spec section or ADR that governs it (ADR 0002/0003 for identity, ADR 0006 for verification, ADR 0016 for backends, ADR 0018 for the IR). Add `#![warn(missing_docs)]` (or `#![deny(missing_docs)]`) to the crate if it is not present, and fix what it reports. Keep every doctest deterministic.

**Acceptance criteria.**
- Every public item reachable from the SDK prelude has rustdoc semantics and at least one doctest example.
- `missing_docs` is enabled for the crate and the build is clean under it.
- Documented items cross-link to the governing spec section or ADR where one exists.
- `cargo test -p rahn-sdk` passes, including doctests, and adds no nondeterminism.

**Difficulty:** Beginner.

**Labels:** `documentation`, `dx`, `good first issue`

---

## 10. Verify binary reproducibility of release builds

**Title:** Verify binary reproducibility of release builds

**Problem statement.** Binary build reproducibility is unverified. docs/research/v1.0-review.md records it as a carried limitation ("Binary build reproducibility unverified"), and docs/reproducibility.md lists release binaries as "Not yet built reproducibly (Rust builds embed some paths by default)". The plan recorded there is to evaluate reproducible-build options "at the first tagged release with artifacts" — v1.0.0 is that release.

**Why this matters.** Determinism is architectural, and byte-identical history reproducibility is already test-enforced; binary reproducibility is the remaining, honestly-labeled gap between the two. Whether it is achievable affects what a downstream consumer can independently verify about a release artifact.

**Technical context.**
- `docs/research/v1.0-review.md` — carried limitation 5 and the gate-checklist entry: "Reproducible builds — partial... Binary reproducibility not verified (recorded limitation)."
- `docs/reproducibility.md` — the "What is not yet reproducible" table and the recorded plan for release binaries.
- `docs/adr/0019-v1-0-stability.md` — the release process (full validation, tag on the feature commit, CI green on the release commit).
- `docs/third-party.md` — the single direct dependency (`sha2`) and audited transitive closure, which bounds the toolchain surface.
- `docs/research/benchmark-methodology.md` rule 6 — publish negative results.

**Possible approach.** Attempt to produce byte-identical `rahn` release binaries from the same source commit on two machines or toolchains. Document the toolchain pins and the environmental inputs that break identity (embedded build paths are the known suspect; record exactly which flags or settings matter). Evaluate the cargo reproducible-build options that reproducibility.md names as the open question. Record the result in docs/reproducibility.md whether positive or negative: a negative result with a concrete list of the changes it would take is a legitimate outcome, not a failure to hide.

**Acceptance criteria.**
- docs/reproducibility.md is updated with a dated result, positive or negative, and the environment/method used.
- Toolchain pins, cargo flags, and the specific inputs found to break or preserve byte identity are recorded.
- If the result is negative, the concrete changes required for reproducibility are listed.
- The corresponding entry in the v1.0-review limitations list is annotated or updated.

**Difficulty:** Intermediate.

**Labels:** `engineering`, `dx`

---

## 11. Add macOS to the CI determinism matrix

**Title:** Add macOS to the CI determinism matrix

**Problem statement.** The determinism tests are designed to be run cross-platform, but CI runs on Linux and Windows only (`.github/workflows/ci.yml` matrix: `[ubuntu-latest, windows-latest]`; docs/testing.md: "CI (GitHub Actions) runs on Linux and Windows"). Cross-platform evidence for identity stability therefore covers two platforms.

**Why this matters.** RQ1's hypothesis is that state identity is stable across machines and time, and E1 is the cross-platform CI run that tests it. A third platform either strengthens that evidence or surfaces a divergence between platforms — and a divergence is itself a finding worth having, provided it is reported rather than hidden.

**Technical context.**
- `.github/workflows/ci.yml` — the `test` job matrix.
- `docs/testing.md` — the cross-platform paragraph.
- `docs/research/research-questions.md` — RQ1 method: "cross-platform CI determinism runs (E1)"; measurable result: "identity stability rate across 200+ generated states and both CI platforms".
- `crates/rahn-state/tests/properties.rs` — the seeded identity-stability property tests.
- `docs/reproducibility.md` — the identity/history reproducibility guarantees.
- `docs/adr/0012-linux-ns-execution.md` — non-Linux platforms refuse execution at runtime, so the `linux-execution` job stays Linux-only.

**Possible approach.** Add `macos-latest` to the `test` job matrix and confirm the seeded identity-stability and canonical round-trip property tests pass there. Leave the `linux-execution` job on `ubuntu-latest`; the namespace backend refuses to run on non-Linux at runtime, so it has nothing to do on macOS. If any platform divergence appears, file it as a determinism finding with the failing seed, rather than smoothing it over in the PR.

**Acceptance criteria.**
- The CI `test` matrix includes macOS and the job is green, or
- Any divergence is documented as a filed determinism finding with a reproducible failing seed, and the matrix entry is annotated accordingly.
- The cross-platform note in docs/testing.md reflects the platforms CI now covers.

**Difficulty:** Beginner.

**Labels:** `dx`, `engineering`, `good first issue`

---

## 12. Error-message audit against the explainable-failures principle

**Title:** Error-message audit against the explainable-failures principle

**Problem statement.** DESIGN.md principle 14 states that errors carry the violated invariant, the offending objects, and the transition that produced them. Structured-rejection tests exist in places (docs/testing.md records them for `transition.rs`), but no systematic audit confirms that every rejection path across the crates meets the principle.

**Why this matters.** Explainability is a first-class design goal, and an error message is the interface through which a user sees the architecture's reasoning. A rejection that says only "invalid" is a silent failure of the principle, even when the underlying check is correct.

**Technical context.**
- `DESIGN.md` principle 14 — "Explainable failures. Errors carry the violated invariant, the offending objects, and the transition that produced them."
- `docs/testing.md` — structured-rejection tests in `crates/rahn-state/src/transition.rs`; "invalid-transition explanations" in the CLI end-to-end tests.
- `crates/rahn-verify/src/merge.rs` — `MergeConflict` enumerates the conflicting effects it rejected.
- `crates/rahn-verify/src/invariants.rs` — per-invariant evidence strings.
- `crates/rahn-state/src/canonical.rs` and `crates/rahn-store/` — parsing and corrupted-object refusal messages.

**Possible approach.** Enumerate the rejection classes across the crates: transition errors, invariant and constitution failures, merge conflicts, canonical parse rejection, store corruption refusal, and CLI argument rejection. Audit each against principle 14 and note which of the three required pieces (violated invariant, offending objects, producing transition) each message carries. Fix the gaps and add tests for the fixes, then present a before/after table in the PR so the change is reviewable at a glance.

**Acceptance criteria.**
- The PR contains a complete table of rejection classes and, for each, which of the three principle-14 elements the message carries.
- Every gap is either fixed with a test asserting the message content, or recorded as an accepted exception with a reason.
- Where an audit reveals a doc or spec statement that does not match the implementation, the discrepancy is investigated per GOVERNANCE.md rather than assumed to favor the code.

**Difficulty:** Beginner.

**Labels:** `dx`, `engineering`, `good first issue`

---

## 13. Research proposal: requirements for typed addressing on interfaces

**Title:** Research proposal: requirements for typed addressing on interfaces

**Problem statement.** Addressing is a post-v1.0 extension candidate: without it, connectivity in RAHN is link-existence only, and the roadmap's addressing item (typed address fields, canonical format bump) has no requirements work behind it. There is no document that states what addressing must carry, what it would cost the canonical format, or which prior art applies.

**Why this matters.** ADR 0019 requires that each major extension have its own ADR, specification, research evidence, and compatibility review before any implementation, and it freezes canonical format v2 for 1.x. Addressing touches the format directly, so the requirements step is not optional — and ADR 0011 already anticipates that interface metadata "may need structure (typed address fields)" with a further format version bump. This issue performs the requirements gathering only. **Nothing is scheduled; nothing is promised** — the output feeds a future ADR, which may or may not be accepted.

**Technical context.**
- `ROADMAP.md` — post-v1.0 extensions, independently scoped: "Addressing on interfaces (typed address fields; canonical format bump)."
- `docs/adr/0019-v1-0-stability.md` — the extension model and the frozen-for-1.x format commitment (a format change requires MAJOR + ADR + migration tooling).
- `docs/adr/0011-interfaces-canonical-v2.md` — the interface model and its format-bump note.
- `docs/adr/0003-canonical-serialization.md` — the versioned-format rule; `docs/reproducibility.md` — the canonical-format evolution rule.
- `docs/research/prior-art.md` — the source-of-truth and IBN sections, as the prior art for address modeling.
- `docs/spec/objects.md` and `docs/spec/serialization.md` — the current object and encoding rules addressing would extend.

**Possible approach.** Gather requirements and prior art only: inventory the network meaning addressing must carry, drawn from incident and operations literature and the prior-art survey; derive the constraints addressing places on canonical serialization (total ordering, no ambient information, identity stability); sketch the compatibility impact of a format bump per ADR 0003/0011/0019; and record the open questions. The output is a requirements document that would feed a future ADR. Explicitly no implementation, and no commitment that the extension will happen.

**Acceptance criteria.**
- A requirements document exists under docs/research/ (or an RQ entry, if one is proposed) covering meaning inventory, prior-art citations, format constraints, and open questions.
- The document states explicitly that this is requirements gathering, not an implementation commitment, and that nothing is scheduled.
- The compatibility implications of a canonical format bump are analyzed against ADR 0003/0011/0019, not assumed.

**Difficulty:** Intermediate.

**Labels:** `research`

---

## 14. Research proposal: threat model and requirements for signed transitions / authenticated replicas

**Title:** Research proposal: threat model and requirements for signed transitions / authenticated replicas

**Problem statement.** The v0.6 distributed-state trust boundary is self-asserted: ADR 0015 gives every replica authority over its own history and makes divergence fail closed, but nothing authenticates who produced a transition or a replica. Signed transitions and authenticated replicas are a post-v1.0 extension candidate, and no threat model or requirements work exists for them.

**Why this matters.** The project derives requirements before mechanisms — ADR 0015 rejected Raft, CRDTs, and op-shipping on requirements grounds, and the roadmap's signed-transitions item is explicitly framed as replacing the self-asserted trust boundary. Proposing authentication before modeling the adversaries and assets would repeat the mistake the distributed-state ADR avoided. This issue performs threat modeling and requirements gathering only. **Nothing is scheduled; nothing is promised** — the output feeds a future ADR.

**Technical context.**
- `ROADMAP.md` — post-v1.0 extensions: "Signed transitions / authenticated replicas (replaces the self-asserted v0.6 trust boundary)."
- `docs/adr/0015-distributed-state.md` — per-replica authority, fail-closed divergence, and the stated CAP posture, which define what is and is not defended today.
- `docs/spec/security.md`, `SECURITY.md` — the current posture (no network I/O by default, untrusted input, hash-verified integrity, simulation default) and the longer-term security considerations list, which names signed transitions.
- `docs/research/v1.0-review.md` — carried limitation 4: "No signed transitions/authenticated replicas."
- `docs/adr/0002-state-identity.md` and `docs/adr/0003-canonical-serialization.md` — content-addressed identity, which any signing scheme must compose with deterministically.
- `docs/research/limitations.md` — the honest-limitations framing this document must follow.

**Possible approach.** Threat-model and requirements only: enumerate the assets (state store, peer sync, commit integrity), the adversaries (malicious replica, tampered store, network attacker), and the trust assumptions ADR 0015 currently makes. Analyze what fail-closed divergence does and does not defend against. Derive requirements for signature and identity primitives that remain consistent with content-addressed identity and the determinism requirement, and record the open questions and the unresolved primitives. No implementation, and no commitment that the extension will happen.

**Acceptance criteria.**
- A threat-model and requirements document exists under docs/research/ with an adversary/asset model, a gap analysis against the ADR 0015 posture, and derived requirements.
- The analysis states what the current design already handles versus what genuine authentication would add.
- The document states explicitly that this is threat modeling and requirements gathering, not an implementation commitment, and that nothing is scheduled.
- Open questions are recorded for a future ADR, including any primitive that remains unresolved.

**Difficulty:** Advanced.

**Labels:** `research`, `security`

---

## Filing process

1. New drafts are added here first: grounded in the repository, reviewed by a maintainer, then filed as one issue each (`gh issue create --title ... --label ... --body-file ...`). The table above records what is filed.
2. Filing does not schedule work. A research or extension issue stays a candidate until the work is done and recorded (ADR 0019).
3. Issues are living documents. Update the body as work progresses, record what was learned, narrow the scope when evidence warrants it, and close an issue when its acceptance criteria are met rather than when it is convenient.
4. Keep the mapping current: if an issue is superseded or closed unmerged, note it here rather than deleting the draft.
