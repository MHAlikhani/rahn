<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# ADR 0016: Execution Backend Abstraction

- Status: Accepted
- Date: 2026-07-28

## Context

Stage 7 (charter): separate abstract state from backend execution; multiple future targets (Linux namespaces, eBPF/XDP, programmable dataplanes). Today the CLI calls `rahn-exec`'s namespace functions directly — the backend is hard-wired, and simulation is a special case rather than a backend.

## Alternatives

1. **Keep free functions, add CLI flags:** no abstraction; every new backend duplicates the gating/safety wiring. Rejected.
2. **Trait returning a universal command list:** forces all backends into shell semantics (eBPF is not a shell). Rejected.
3. **Trait over a typed two-level step model (chosen):** a backend returns `BackendStep`s that are either *descriptions* of model-level actions or *concrete host invocations*. Simulation returns only descriptions; real backends return invocations. Dry-run renders both without executing; the execution gate stays at the CLI boundary.

## Decision

1. **`ExecutionBackend` trait** (in `rahn-sim`, next to the plan model): `name()`, `capabilities()`, `plan(current, target) -> Result<Vec<BackendStep>, BackendError>`.
2. **`BackendStep`**: `Describe(Action)` (model-level, from the existing dependency-safe plan) or `Invoke { program, args }` (concrete, host-scoped). A plan mixing both is invalid per backend contract — each backend returns one kind exclusively.
3. **Backends:**
   - `simulation` (default, in `rahn-sim`): `Describe` steps only; never executes; the dry-run of every other backend is its model-level counterpart.
   - `linux-ns` (in `rahn-exec`): `Invoke` steps (iproute2 argv, ADR 0012 rules unchanged — host safety asserted structurally; Linux/root gate enforced inside the backend).
4. **Selection:** by backend name; the default is always `simulation`. Unknown names fail explicitly with the list of registered backends. Real execution additionally requires the existing `--execute --yes-i-know` gate.
5. **Capability boundaries:** `Capabilities` reports what a backend can realize (v0.7: `link_topology: bool`). A plan requiring unsupported capabilities fails explicitly — no silent partial execution. Addressing/traffic-control capabilities are Future and their absence is reportable.
6. **No eBPF/XDP code in this stage:** they become new trait implementations plus capability entries; the extension point is the trait + capability enum, nothing more.

## Consequences

- The core state model remains backend-independent (no backend types leak into `rahn-core`/`rahn-state`).
- Simulation-first is structural: the default backend cannot touch the host, and a real backend's dry run is inspectable by construction.
- Determinism: `plan()` is a pure function of (current, target) for every backend; identical inputs produce identical steps.
- Migration is internal: ADR 0012 semantics (naming, host-safety, gates) are unchanged; only the call path changes.

## Reconsideration conditions

- A backend needs async/streaming effects (e.g., long-running observations) → revisit the step model.
- Capability negotiation between replicas (distributed execution) → extend `Capabilities` via ADR.
