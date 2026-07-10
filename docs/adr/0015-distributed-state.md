<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# ADR 0015: Distributed State — Sync, Merge, Fail-Closed Divergence

- Status: Accepted
- Date: 2026-07-10

## Context

Stage 6 (charter): replicated state across RAHN instances, requirements-first (RQ7). RAHN's existing artifacts are already distribution-friendly: content-addressed states/commits (CAS), a commit DAG with common-ancestor computation, fail-closed semantic merge (ADR 0007), append-only logs. The question is what consistency model those support honestly.

## Alternatives

1. **Raft-style leader consensus:** rejected for v0.6 — RAHN writes are low-frequency, human-scale, merge-aware; linearizable ordering of a leader adds liveness coupling (partitions block writes) without answering RAHN's real question (how do divergent *histories* converge?). Revisit only if the requirements study (below) shows a need for a single write authority.
2. **CRDTs:** rejected — state objects are not commutative semilattices, and constraint conflicts require explanation, not automatic convergence. Silent resolution violates ADR 0007.
3. **Event-sourcing replication (ship operations):** rejected for v0.6 — shipping immutable *records* (commits + states) is strictly simpler and content-verifiable; op-shipping would reintroduce ordering problems the DAG already solves.
4. **Chosen: peer sync over content-addressed history with fail-closed merge convergence.**

## Requirements (RQ7 answers)

- **Strongly consistent state:** each replica's own committed history (read-your-writes; commits are immutable and hash-verified). Nothing else is strongly consistent.
- **Eventually consistent:** cross-replica visibility of commits and branch tips, converging only through explicit synchronization.
- **Authority:** every replica is authoritative for its own history. No leader, no epoch/generation — without leader election there is nothing an epoch orders (documented; revisit if a single-writer mode is ever added).
- **Partition:** replicas continue independently (local commits always allowed); divergence is expected and represented, not prevented. CAP: under partition RAHN chooses availability of local writes + consistency of local history; cross-replica consistency is sacrificed until healing.
- **Conflicting transitions:** represented as divergent branch tips over a common ancestor; conflicts are the existing semantic merge conflicts (detectable, explainable, fail-closed). No automatic resolution — divergence *persists* when resolution is impossible, by design.
- **Recovery:** a replica restart reopens its store; missing objects are fetched by hash from peers; corrupted replication payloads are refused by content-hash checks (existing store semantics). Partial recovery (some branches synced, some conflicting) is the normal outcome and is reported.
- **Guarantees actually provided:** immutable hash-verified records; deterministic convergence — when a merge succeeds, both replicas commit a *byte-identical* merge commit (same parents, state, message); read-your-writes per replica; monotonic history per replica. **No linearizability, no serializability, no strong consistency across replicas** — explicitly not claimed.

## Distributed model (minimum concepts)

- **ReplicaId**: validated identifier, self-asserted (no auth in v0.6).
- **Offer**: a replica's advertised branch tips (branch → commit id). The only discovery mechanism.
- **SyncMessage**: deterministic, canonically serializable exchange — `Offer`, `RequestObjects(ids)`, `Objects(records)`.
- **SyncOutcome**: per-branch result — `UpToDate | Fetched | Merged(commit-id) | Conflict(report)`.
- Reused unchanged: StateId, CommitId, DAG ancestry, semantic merge, constitution verification (merged candidates must pass the constitution on both replicas).

## Convergence rule (determinism)

When two replicas' branch tips diverge and the semantic merge succeeds, each replica commits a merge commit with: parents = [own tip, other tip] ordered lexicographically by commit id, the merged state, message `sync merge`, verification passed. Identical inputs ⇒ identical CommitId on both replicas (test-enforced). On merge conflict: both replicas keep their own tips; the conflict report is returned; nothing is auto-resolved (fail closed).

## Security assumptions (v0.6)

Replica identity is self-asserted; there is no authentication, so the model trusts its transport peers. Integrity is provided by content hashes; forged *history* is detectable but unauthorized *authorship* is not — signed transitions remain Future (charter security list). Replay of sync messages is harmless (idempotent, content-addressed). This is a documented trust boundary, not an oversight.

## Reconsideration conditions

- A single-writer/leader requirement emerges from workload evidence (E7 follow-up).
- Signed transitions arrive (then replica authenticity upgrades).
- Observation/causal log replication becomes needed (currently per-replica local; documented Future).
