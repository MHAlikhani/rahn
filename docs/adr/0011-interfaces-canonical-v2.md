<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# ADR 0011: Interfaces as Link Endpoints (Canonical Format v2)

- Status: Accepted
- Date: 2026-04-29
- Deciders: Stage 2 design (v0.2 network graph)

## Context

v0.1 links connect nodes directly. Real networks connect *interfaces*: a cable plugs into a port, an address lives on an interface, and link-state is per-interface. Stage 2 (charter: "richer topology, addressing, interfaces") requires this refinement before addressing and path semantics can be meaningful. Changing link endpoints changes canonical bytes for every existing state.

## Problem

How to introduce interfaces without (a) corrupting the identity guarantees of existing history, (b) growing the operation vocabulary beyond what is justified, or (c) blocking the Stage 2 graph work on a migration framework that nothing else needs yet.

## Alternatives

1. **Keep node-to-node links; model interfaces as node metadata.** Rejected — interfaces are structural (they participate in identity, diff, merge, and future observation); metadata cannot carry structure.
2. **Auto-migrate v1 stores on load** (synthesize one interface per linked node endpoint). Rejected — silent model translation violates the loud-corruption/explicitness principle and creates states whose provenance is a migration, not a recorded transition. Revisit if real deployments accumulate v1 history (none exist: alpha software).
3. **Accept both formats forever.** Rejected — dual-format readers double the adversarial surface permanently.
4. **Canonical format v2; v1 rejected loudly.** Chosen.

## Decision

1. **Interface** becomes a first-class object: owned by a node, uniquely named within it, carrying bounded metadata, participating in identity, diff, merge, and verification.
2. **Links connect interfaces.** A link's endpoints are `(node_id, interface_name)` pairs. Linking two interfaces of the *same* node is prohibited (topological self-loop). Endpoints are normalized (lexicographic) so a link is undirected.
3. **Canonical format version bumps to 2.** The v2 reader rejects v1 bytes with an explicit error naming the version and this ADR. v0.1 repositories are alpha-era artifacts; no automatic migration is provided. (Compatibility note in CHANGELOG.)
4. **Operation vocabulary extended** to `add_interface(node, name)`, `remove_interface(node, name)` (rejected while links reference it), and `add_link`/`remove_link` now address endpoints. This keeps every state change an explicit recorded transition.
5. **Node-level connectivity semantics are preserved**: `require-connectivity a b` and the new `prohibit-connectivity a b` operate over the node graph induced by interface links.

## Consequences

- Identity of states changes relative to v0.1 (format tag participates in the hash); historical v0.1 objects remain loadable *as objects* only by the v0.1 code lineage.
- Diff, merge, execution plans, and the CLI gain interface-aware cases (all test-covered).
- Addressing (Stage 2 follow-up) now has a natural home: addresses attach to interfaces.
- The v2 reader keeps every v1-era strictness property (total ordering, no ambient data, dangling-endpoint rejection, normalized endpoints).

## Rejected options

See Alternatives.

## Future reconsideration conditions

- If pre-1.0 adoption produces v1 repositories worth migrating, a `rahn migrate` command with explicit, per-state provenance recording may be added (its own ADR).
- When addressing arrives, interface metadata may need structure (typed address fields) — a further format version bump at that point.

## References

- docs/spec/objects.md; docs/spec/serialization.md; docs/adr/0003-canonical-serialization.md; docs/research/stages/v0.2.md
