<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# ADR 0012: Isolated Linux Execution via Network Namespaces

- Status: Accepted
- Date: 2026-05-21

## Context

Stage 3 (charter) is the first real-network interaction: instantiate an abstract topology inside isolated Linux network namespaces. Constraints: simulation remains the default; execution is explicit opt-in; the host network must never be mutated silently; every step inspectable.

## Alternatives

1. **Raw netlink via a Rust crate** (e.g., rtnetlink): precise, but a heavyweight dependency for v0.3 and harder to inspect.
2. **Shell out to `ip` (iproute2)**: every action is a human-readable, inspectable argv; no new dependencies; namespace tooling is battle-tested. Chosen.
3. **Container runtime (Director/containerlab-style)**: rejected — external orchestration hides per-step causality and adds deployment requirements.

## Decision

1. New crate **`rahn-exec`**: pure mapping from an [`ExecutionPlan`](rahn-sim) to an ordered list of `ip`-command argv vectors. No I/O in the mapping — fully deterministic and testable on any OS.
2. **Naming rules (deterministic):** namespace per node = `rahn-<node-id>`; veth interface per link endpoint = `"r" + 7 lowercase hex chars of SHA-256(endpoint string)`, satisfying IFNAMSIZ (≤15) and collision resistance; loopback raised in every created namespace.
3. **Per-link topology:** one veth pair per link; first end created inside namespace A, peer moved into namespace B (`ip link set ... netns`); both ends raised. Nodes map to namespaces; interfaces map to `dummy` interfaces inside the node's namespace.
4. **Opt-in gating (structural):** `rahn apply <ref> --execute` requires `--yes-i-know` in the same invocation; non-Linux platforms refuse at runtime; the dry run (default) prints the exact argv sequence. Execution stops at the first failing command with its position in the plan.
5. **Host-safety invariant:** host-side operations are limited to `ip netns add/del rahn-*`. All link/interface mutations run inside `ip netns exec rahn-*`. Tests assert this structurally (no non-exempt argv touches the host namespace).
6. **Cleanup:** `rahn destroy <ref>` deletes the namespaces of a state's nodes (veths die with their namespace).

## Consequences

- Requires root (CAP_SYS_ADMIN) and Linux; validated in CI (ubuntu, sudo), not on the Windows dev machine.
- `ip` availability is a runtime requirement, checked before execution.
- No addressing yet (Stage 2 deferral): namespaces are created but have no IPs; connectivity is link-existence only. Ping-based validation arrives with addressing.
- Reversibility: every create has a destroy path; partial failures are recoverable via `rahn destroy`.

## Reconsideration conditions

- When addressing or tc features outgrow `ip` argv expressiveness, evaluate netlink (new ADR).
