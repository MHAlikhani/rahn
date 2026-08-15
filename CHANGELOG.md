<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Changelog

All notable changes to the RAHN project (architecture, documentation, and later software) are documented here. Format based on [Keep a Changelog](https://keepachangelog.com/); versioning is semantic once releases begin.

## [0.8.0-alpha] — Stage 8: Network CI / Verification

### Added
- **`rahn test [ref]` (ADR 0017):** machine-readable verification of a
  committed state (default HEAD) — deterministic TSV report
  (`PASS|FAIL<TAB>id<TAB>evidence`) with exit-code contract (0 pass,
  1 failure, 2 usage). Invariant ids are a stable machine contract.
- **CI example:** `examples/network-ci.yml` (GitHub Actions gate on
  `.rahn/**` changes).
- Stage report: `docs/research/stages/v0.8.md`.

### Compatibility
- No canonical-format change. Invariant ids in reports must not be
  renamed without an ADR (machine contract).

## [0.7.0-alpha] — Stage 7: Execution Backends

### Added
- **Execution backend abstraction (ADR 0016, `rahn-sim::backend`):**
  `ExecutionBackend` trait (name, capabilities, pure
  `plan(current, target) -> Vec<BackendStep>`); two-level step model
  (`Describe` model actions vs `Invoke` host invocations); capability
  negotiation with explicit refusal of unsupported capabilities.
- **Backends:** `simulation` (default; description-only; cannot execute
  by definition) and `linux-ns` (ADR 0012 namespace semantics unchanged,
  migrated behind the trait; Linux/root gate inside the backend).
- CLI: `rahn apply --backend <name>`; explicit registry with
  unknown-backend rejection; dry runs render either step kind.
- Stage report: `docs/research/stages/v0.7.md`.

### Compatibility
- No canonical-format change. `rahn apply` default output now includes a
  `backend:` line and renders model-level steps for the default backend
  (previously namespace commands were shown unconditionally).

## [0.6.0-alpha] — Stage 6: Distributed State

### Added
- **`rahn-dist` crate (ADR 0015):** deterministic two-way peer sync
  over content-addressed history — branch-tip offers, fetch of missing
  commits/states by hash (complete ancestries only), strict-ahead/behind
  adoption, and divergent-tip convergence via the fail-closed semantic
  merge producing **byte-identical merge commits on all replicas**
  (parents lexicographically ordered, message `sync merge`).
- **Fail-closed divergence:** merge conflicts and constitution
  rejections leave divergent tips in place with explainable reports;
  repeated syncs reproduce the conflict; nothing is auto-resolved.
- **Consistency model (documented, not claimed):** per-replica
  read-your-writes and monotonic history; no linearizability or
  cross-replica strong consistency; no leader/epoch concept (deliberate).
- Sync benchmark harness (`sync_scaling`); results in
  `docs/research/stages/v0.6.md` (~3–8 ms/object transport cost recorded
  as known performance debt; packing is a Stage 7 follow-up).
- Stage report: `docs/research/stages/v0.6.md`.

### Security
- Replica identity is self-asserted (documented trust boundary);
  integrity via content hashes; replay is idempotent/harmless. No PKI
  by design; signed transitions remain Future.

## [0.5.0-alpha] — Stage 5: Causal Memory

### Added
- **Causal edges (ADR 0014, `rahn-state::causal`):** explicit relations
  between immutable anchors (`obs:<seq>` / `commit:<id>`) with strict
  epistemic statuses — `temporal-correlation`, `hypothesis`,
  `verified` (the last only between Commit anchors, structurally
  enforced).
- **DAG invariant:** edge insertions that would close a cycle are
  rejected (deterministic DFS); dangling anchors are rejected against
  the observation log and commit store.
- **Append-only causal log** (`.rahn/causal.log`) with versioned
  framing and loud corruption refusal.
- **Incident query:** connected component around an anchor
  (`rahn explain`); reverse-adjacency index keeps it O(component)
  (an O(V·E) first implementation was found by benchmark and fixed:
  2 702 ms -> 5.7 ms on a 10k-edge chain).
- CLI: `rahn relate ... --note`, `rahn explain <anchor>` — output
  carries edge statuses and never presents hypotheses as proven.
- Graph benchmark harness (`causal_scaling`); results in
  `docs/research/stages/v0.5.md`.

### Compatibility
- No canonical-format change for states; `causal.log` is a new,
  separate artifact with its own versioned framing.

## [0.4.0-alpha] — Stage 4: Observability

### Added
- **Deterministic observation model (ADR 0013, `rahn-state::obs`):**
  `Observation {seq, time_ns, subject, metric, value}`; exact
  float-free values (counter/gauge/event); caller-supplied timestamps;
  positional `seq`; `(time_ns, seq)` total ordering; strict canonical
  parsing with loud malformed-input rejection.
- **Append-only observation log** (`.rahn/observations.log`) with
  versioned framing; corruption refused loudly, never repaired.
- **Provenance:** every record validated against and tagged with the
  HEAD state id; unknown subjects rejected at ingest.
- CLI: `rahn observe ... --at <unix-ns>`, `rahn observations [filter]`
  (deterministic TSV output).
- Ingestion benchmark harness (`obs_scaling`); results and methodology
  in `docs/research/stages/v0.4.md` (per-append open+flush dominates:
  ~144 µs/append — recorded as known performance debt).
- Stage report: `docs/research/stages/v0.4.md`.

### Compatibility
- No canonical-format change for states; the observation log is a new,
  separate artifact with its own versioned framing.

## [0.3.0-alpha] — Stage 3: Isolated Linux Execution

### Added
- **`rahn-exec` crate (ADR 0012):** pure, deterministic mapping from
  execution plans to iproute2 (`ip`) commands; nodes become network
  namespaces (`rahn-<node>`), interfaces become dummy interfaces, links
  become veth pairs with deterministic IFNAMSIZ-safe names.
- **Opt-in real execution:** `rahn apply <ref> --execute --yes-i-know`
  (Linux + root only; refuses elsewhere); `rahn destroy --yes-i-know
  <ref>` for teardown/recovery. Default remains simulation: `apply`
  prints the exact command sequence.
- **Structural host-safety invariant** (test-enforced): host-side
  operations limited to `ip netns add/del rahn-*`; all link/interface
  mutations namespace-scoped.
- CI job running the real-execution integration test on ubuntu with
  sudo. Stage report: `docs/research/stages/v0.3.md`.

### Compatibility
- No canonical-format change. The namespace backend requires Linux,
  iproute2, and root (CAP_SYS_ADMIN).

## [0.2.0-alpha] — Stage 2: Network Graph Foundation

### Added
- **Interface-endpoint model (ADR 0011):** interfaces are first-class
  objects owned by nodes; links connect `node/interface` endpoints;
  same-node links prohibited. Canonical format versioned to 2 (v1 bytes
  rejected explicitly — no silent migration; see ADR 0011 for rationale).
- **Graph queries:** deterministic shortest path (lexicographic
  tie-break), neighbors, reachability, connected components.
- **Isolation constraints:** `prohibit-connectivity a b` in the
  constitution, evaluated over the interface-induced node graph.
- CLI: `rahn interface add/remove`; link endpoints as `node/iface`;
  `rahn path <from> <to>`; interface-aware diff/state/inspect/apply.
- Scaling benchmarks (`cargo test -p rahn-state --release --test
  scaling -- --ignored`): 10 to 100 000 nodes; results and methodology in
  `docs/research/stages/v0.2.md`.
- Stage report: `docs/research/stages/v0.2.md`.

### Fixed
- Merge: added nodes now materialize their interfaces (previously links
  referencing them failed or nodes were interface-less).
- Diff: added/removed nodes report their interfaces (diff-derived
  operations now reconstruct the state exactly).

### Compatibility
- Canonical format v2 is NOT readable by v0.1.x binaries, and v0.1.x
  stores are NOT readable by v0.2 (explicit error referencing ADR 0011).
  Alpha-era repositories; no migration tool is provided by design.

## [0.1.0-alpha.2] — Stage 1 Hardening (post-v0.1 review)

### Added
- `rahn checkout [--force] <branch>` (ADR 0010): switch branches; fails
  closed on uncommitted working changes; `--force` is the explicit,
  documented recovery path for changes that can never be committed
  (e.g., constitution-rejected edits).
- Seeded property tests over random networks (docs/testing.md): canonical
  round-trip + identity stability (200 seeds), diff-derived operations
  reconstruct the target (200 seeds), merge commutativity for disjoint
  effects (100 seeds). Deterministic xorshift PRNG; no new dependencies.
- End-to-end tests: diverged-branch checkout+merge, dirty-checkout
  refusal, constitution gating of branch-state commits, --force recovery.
- `docs/testing.md` (test strategy) and `docs/reproducibility.md`.
- GitHub Actions CI: rustfmt, clippy (-D warnings), tests on Linux and
  Windows, plus license/SPDX documentation checks.
- Issue templates (bug, feature proposal, design proposal/RFC) and
  CODE_OF_CONDUCT.md (Contributor Covenant 2.1).

### Changed
- White paper upgraded from outline to initial technical draft (v0.2):
  claim-status markers throughout, explicit no-measurements statement,
  scoped novelty statement.
- docs/research/prior-art.md expanded into a structured per-system
  survey (Git, Terraform, Batfish, SDN, IBN/RFC 9315, NetBox-class,
  emulators, P4/eBPF/XDP, QUIC/SCION, observability, event sourcing,
  consensus, formal verification).
- docs/research/research-questions.md restructured to RQ1-RQ10, each
  with motivation, hypothesis, method, measurable result, limitations.
- Founder attribution resolved: RAHN was created and is initially led by
  Mohammad Hossein Alikhani (docs/licensing.md [LEGAL REVIEW] marker
  resolved; ADR 0009 amendment; README and GOVERNANCE updated).
- Code formatted with rustfmt; clippy clean at -D warnings.

### Fixed
- Merge no longer drops metadata of nodes added on a branch (property
  test coverage added).

## [0.1.0-alpha.1] — Stage 1: State Engine

### Added
- Rust workspace (`crates/`): `rahn-core` (object model), `rahn-state`
  (canonical serialization, content-hash identity, transitions, diff,
  commits, history graph), `rahn-store` (content-addressed filesystem
  store), `rahn-verify` (invariants, constitution, fail-closed semantic
  merge), `rahn-sim` (execution plans), `rahn-cli` (`rahn` binary).
- CLI: `init, node add/remove, link add/remove, commit, state, branch,
  diff, merge, verify, log, inspect, apply` — simulation-only.
- Deterministic invariant engine: referential integrity, link endpoints,
  duplicate links, self-loops, named connectivity requirements.
- Verification-gated commits: failing candidates are rejected with
  per-invariant evidence and never written to the store.
- Content-addressed storage with loud corruption refusal.
- 75 tests, including cross-platform determinism, canonical round-trip
  adversarial cases, corrupted-object detection, and end-to-end CLI flows.
- `examples/cli-walkthrough.md`.

### Documentation
- Spec status headers updated to reflect implementation (reserved areas
  marked as such); ADR 0007 implementation amendment; third-party
  inventory records `sha2 0.10.9 (Apache-2.0 OR MIT)`.

## [Unreleased] — Stage 0: Research / Foundation

### Added
- Initial documentation corpus: README, ARCHITECTURE, DESIGN, GLOSSARY, ROADMAP, CONTRIBUTING, SECURITY, GOVERNANCE.
- `docs/concepts.md` — core concepts.
- `docs/research/` — problem statement, prior art, research questions, hypotheses, experiment plan, benchmark methodology, limitations, future work.
- `docs/adr/` — ADRs 0001–0008 (project scope, state identity, canonical serialization, storage model, transition model, verification model, branching model, execution boundary).
- `docs/spec/` — specification skeleton (state, objects, transitions, invariants, constitution, storage, serialization, protocol, execution, observation, causality, security).
- `docs/whitepaper/RAHN-Whitepaper.md` — white paper v0.1 (structured outline).

### Changed
- **Licensing model established (ADR 0009):** source code Apache-2.0, documentation/research CC-BY-4.0. The provisional MIT root LICENSE was replaced with the canonical Apache License 2.0 text; canonical license texts added under `LICENSES/`; SPDX headers added to documentation; README and CONTRIBUTING updated; `docs/licensing.md` (authoritative policy) and `docs/third-party.md` (inventory, currently empty) added.
