<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Prior Art

Status: living research document. **Rules: never claim novelty without evidence; never write "no one has ever done this"; use scoped language such as "not identified in the surveyed sources."** The review below is a working survey, not a systematic literature review; its claims are scoped to the sources consulted and will be deepened before the white paper asserts any novelty.

For every major system: problem solved, abstraction, strengths, limitations, overlap with RAHN, differences, integration opportunities, and architectural conflicts.

---

## 1. Version control and content-addressed systems

### Git (and Mercurial)

- **Problem solved:** versioning source-code files; distributed collaboration.
- **Abstraction:** content-addressed object store; commits as DAG snapshots; branches as refs; three-way textual merge.
- **Strengths:** proven model; immutability; cheap branching; tooling ubiquity.
- **Limitations:** semantics end at bytes/lines; merge is textual and cannot see meaning; no verification concept; history is non-causal beyond parentage.
- **Overlap with RAHN:** content-addressed objects, commit DAG, branch refs, three-way merge shape, diff-driven workflows. RAHN's store and history machinery consciously borrows these mechanics.
- **Differences:** RAHN's merge operates on **network semantics** (objects and constraint feasibility), fails closed, and re-verifies merged candidates; RAHN states carry invariant obligations ("constitution") that have no Git analogue; RAHN's roadmap extends to causality and observation linkage.
- **Integration:** none required; conceptual borrowing acknowledged.
- **Conflicts:** presenting RAHN as "Git for networks" (explicitly rejected in the charter — it understates the semantic layer and misleads about merge semantics).

## 2. Infrastructure as Code and declarative configuration

### Terraform / OpenTofu

- **Problem solved:** provisioning infrastructure declaratively.
- **Abstraction:** declarative resource graph + persisted state file + plan/apply cycle.
- **Strengths:** plan-before-apply resembles RAHN's candidate→verify→execute; drift detection; provider ecosystem.
- **Limitations:** the state file is a flat reconciled snapshot — not content-addressed history; no branches of infrastructure state; no causal memory; plan correctness depends on provider implementations; merge is not a concept.
- **Overlap:** plan/apply shape; state persistence.
- **Differences:** RAHN versions the *evolving state itself* with identity and provenance, branches it, verifies it against a constitution, and treats execution as one lifecycle stage.
- **Integration:** future execution backends could be Terraform-like providers; not a design dependency.
- **Conflicts:** adopting an IaC state-file model wholesale would lose immutability and history.

### Ansible / Nornir / network automation frameworks

- **Problem solved:** automating device configuration changes.
- **Abstraction:** procedural/declarative tasks executed against devices.
- **Strengths:** broad device coverage; operational maturity.
- **Limitations:** runs are the unit of record; state is whatever is on the devices; no versioned semantic state; idempotence approximated, not verified.
- **Overlap:** the execution-plan stage produces a similar action list.
- **Differences:** RAHN keeps representation independent of execution and records history.
- **Integration:** plausible adapter target for RAHN execution backends (Stage 7).
- **Conflicts:** none architectural.

### Batfish

- **Problem solved:** analyzing and verifying network *configurations* (reachability, routing correctness) without deploying.
- **Abstraction:** vendor config → vendor-independent model → data-plane analysis; question/answer interface.
- **Strengths:** serious formal analysis of data planes; differential and impact analysis; CI-for-networks precedent.
- **Limitations:** analyzes configurations as inputs; not a versioned evolving state with identity, branches, causality, or a constitution; no state history.
- **Overlap:** verification-before-execution; CI-for-networks vision (RAHN Stage 8).
- **Differences:** Batfish verifies the data plane implied by configs; RAHN verifies candidate *states* against declared invariants and versions the evolution. Complementary rather than competing.
- **Integration:** a Batfish-class engine is a candidate *verification backend* behind RAHN's verification layer (Stage 2+).
- **Conflicts:** none; distinct layers.

## 3. SDN, network OSes, and intent-based networking

### OpenDaylight / ONOS (SDN controllers)

- **Problem solved:** centralized control of forwarding via southbound protocols.
- **Abstraction:** topology + flow programming; cluster-distributed controllers.
- **Strengths:** global view; programmatic control; mature southbound abstractions.
- **Limitations:** the "global view" is operational, mutable, and history-poor; no content-addressed state; no semantic branching/merging; no constitution of invariants governing all states.
- **Overlap:** network state as a first-class controller artifact.
- **Differences:** SDN is execution-first; RAHN is state-semantics-first and simulation-only until Stage 3.
- **Integration:** an SDN controller could be an execution backend consuming verified RAHN plans.
- **Conflicts:** RAHN must not become a controller clone (charter §3).

### Intent-based networking (IETF RFC 9315 analysis; ONF IBN work)

- **Problem solved:** expressing desired outcomes and (partially) verifying fulfillment.
- **Abstraction:** intent decomposition (intent, expectation, mechanism) per RFC 9315; intent translation and assurance loops.
- **Strengths:** outcome-oriented operation; continuous assurance concept.
- **Limitations (per the RFC's own analysis):** verification of intent fulfillment is often partial; intent lifecycles lack immutable versioned history and causal linkage; no semantic branch/merge of network states.
- **Overlap:** intent shapes RAHN's lifecycle entry point; "assurance" overlaps RAHN's verification + observation stages.
- **Differences:** RAHN makes the state — not the intent — the versioned first-class object, and records evolution with provenance.
- **Integration:** RAHN could serve as the durable, versioned substrate beneath an IBN system's intent lifecycle.
- **Conflicts:** none identified.

## 4. Network source of truth / state stores

### Network Source of Truth systems (Nautobot/NetBox class)

- **Problem solved:** authoritative inventory (devices, interfaces, IPs, circuits).
- **Abstraction:** relational models + APIs + change logs.
- **Strengths:** operational adoption; data completeness for inventories.
- **Limitations:** relational rows, not content-addressed semantic states; change logs are audit trails, not a verifiable evolution graph; no invariants engine over candidate futures.
- **Overlap:** RAHN's node/link/identity objects overlap inventory data.
- **Differences:** RAHN's states are immutable, identity-bearing, branchable, and verifiable as wholes.
- **Integration:** a SoT is a natural *feeder* of intended state into RAHN (Stage 2+).
- **Conflicts:** none.

## 5. Digital twins, simulators, emulators

### GNS3 / Containerlab / Mininet / Cisco CML

- **Problem solved:** building runnable lab topologies.
- **Abstraction:** topology description → containers/VMs/namespaces with virtual links.
- **Strengths:** realistic behavior; broad ecosystem.
- **Limitations:** the lab *is* the artifact; no versioned semantic state, no invariants, no causality; topology files are configs, not content-addressed states.
- **Overlap:** RAHN Stage 3 (Linux namespaces) will build on similar primitives (veth, netns).
- **Differences:** for these tools, execution is the product; for RAHN, execution is a stage gated by verification and recorded for causality.
- **Integration:** containerlab-style topologies could be import/export formats for RAHN states.
- **Conflicts:** RAHN is not a simulator (charter §3).

## 6. Programmable dataplanes and protocols

### P4 / eBPF / XDP

- **Problem solved:** programming packet processing behavior.
- **Abstraction:** dataplane programs (P4 programs; eBPF programs attached to hooks).
- **Strengths:** extreme flexibility and performance at the dataplane.
- **Limitations:** orthogonal abstraction level; nothing about state evolution, versioning, or verification of network-wide semantics.
- **Overlap:** none at the core; both are candidate **execution backends** (Stage 7).
- **Differences:** dataplane programs are mechanism; RAHN state is policy/representation.
- **Integration:** planned adapter boundary.
- **Conflicts:** none.

### QUIC, SCION, multipath transport

- **Problem solved:** transport (QUIC), path-aware internetworking (SCION), multipath (MPTCP/Multipath QUIC).
- **Abstraction:** protocol-level.
- **Relevance:** RAHN plans to *reuse* these rather than design transports (charter §25). SCION's path-aware architecture shares vocabulary (paths as first-class), but at the packet level.
- **Integration:** candidate control-plane transports (Stage 6+); SCION-style path concepts may inform Stage 2 path objects.
- **Conflicts:** RAHN must not invent a transport without an architectural requirement (ADR-worthy if ever proposed).

## 7. Observability and causal analysis

### Metrics/tracing systems (Prometheus, OpenTelemetry)

- **Problem solved:** recording and querying what happened.
- **Abstraction:** time series; spans/traces with parent-child causality *within* requests.
- **Strengths:** mature ecosystems; distributed tracing has real causal semantics for call graphs.
- **Limitations:** telemetry is not linked to network *state transitions*; correlation stops at dashboards; tracing causality is per-request, not per-state-evolution.
- **Overlap:** RAHN Stage 4 observation; Stage 5 causal memory is the research delta.
- **Differences:** RAHN's causal graph is anchored in state transitions (provenance), which telemetry systems lack.
- **Integration:** observation ingestion could consume OpenTelemetry-class data (Stage 4).
- **Conflicts:** none.

### Event-correlation / AIOps products

- **Problem solved:** reducing alert noise; hypothesizing root causes.
- **Abstraction:** correlation rules or learned models over event streams.
- **Limitations:** rarely expose a verifiable causal model; outputs are suggestions without provenance to state changes.
- **Overlap/Difference:** RAHN aims for *explainable, state-anchored* causality with explicit confidence distinctions (temporal correlation vs. causal hypothesis vs. verified relation — RQ5).
- **Integration:** possible presentation layer; not core.

## 8. Distributed systems and formal methods

### Event sourcing / CQRS

- **Problem solved:** auditable, replayable application state evolution.
- **Abstraction:** append-only event log; state as fold over events.
- **Strengths:** replay and audit are native; the closest existing-pattern relative to RAHN's history.
- **Limitations:** designed for application entities and command handlers; no network semantics (topology, invariants, constitutions, semantic merge); snapshots are not content-addressed by default.
- **Overlap/Difference:** RAHN keeps *both* explicit states and their generating transitions, content-addressed.
- **Integration:** pattern-level borrowing only.

### Consensus and replication (Raft, Paxos, CRDTs)

- **Problem solved:** replicating state under failure.
- **Relevance:** Stage 6 — explicitly *not to be chosen by familiarity* (charter §21). RAHN must first derive its consistency requirements (RQ7: who writes network state, how stale may views be, what conflict semantics do network merges need?).
- **Conflicts:** premature adoption would bake in assumptions before requirements exist.

### Formal verification of networks (Header Space Analysis, VeriFlow, NetKAT, Minesweeper)

- **Problem solved:** proving data-plane properties of given configurations.
- **Strengths:** rigorous semantics; genuine proofs for scoped questions.
- **Relevance:** candidate engines and theory behind RAHN's verification layer as it deepens (Stage 2+); NetKAT-style algebra is relevant background for path/reachability invariants.
- **Differences:** these verify static configurations; RAHN verifies candidate states inside a versioned, branching lifecycle.
- **Integration:** likely the most important prior-art family for RAHN's verification research.

### TLA+ / P / model checking

- **Relevance:** methodology for verifying RAHN's own transition/merge semantics (planned, not yet applied).

## 9. RAHN-specific ideas (provisional; each requires verification against literature before any novelty claim)

1. **Network constitution** as a named, first-class, versioned collection of architectural invariants governing all valid states.
2. **Semantic network merge** over content-addressed network states with constraint-aware, fail-closed conflict detection.
3. **Causal network memory**: a causal graph anchored in state-transition provenance.
4. **Network time travel** as a first-class operation over content-addressed state history.
5. Treating *the network's evolving state itself* as the primary computational object across the whole lifecycle (vs. configs, intents, or packets).

**Scoping statement:** the *combination and emphasis* in (5) — a full lifecycle of identity, branching, semantic merging, constitution-gated verification, and simulation-first execution around network state — was not identified in the surveyed sources. Individual elements have close relatives documented above. This statement is bounded by the survey's scope and must be re-validated against a systematic literature review before publication-level novelty claims.

## 10. Unresolved research questions

See [research-questions.md](research-questions.md): merge completeness (RQ3), causal attribution fidelity (RQ5), replay fidelity (RQ6), distributed consistency requirements (RQ7), plan-safety assurance (RQ8).

## Survey method and sources note

This survey is based on the systems' published documentation and widely known literature summaries as of 2026-09; primary citations (papers, RFCs, specification documents) are to be added per-section as the white paper matures. Where a claim here rests on general knowledge rather than a consulted primary source, it is provisional and marked by the absence of a citation.
