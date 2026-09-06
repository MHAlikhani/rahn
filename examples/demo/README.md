<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Runnable demo: the core RAHN workflow

`demo.sh` drives the existing `rahn` CLI through the core v1.0 value chain -
content-addressed state, explicit transitions, a constitution that gates every
candidate state, branching and semantic diff, deterministic graph queries, an
inspectable execution plan, and observation/causal memory. Each step prints a
header stating what it does and why it matters, so the output itself is the
explanation.

The demo is **simulation only**. It never passes `--execute` or `--yes-i-know`,
never uses `sudo`, and never touches the host network. It runs in a temporary
directory outside the repository and removes that directory on exit.

## Prerequisites

- `bash` - Linux, macOS, or Windows via Git Bash or WSL.
- Either a prebuilt `rahn` binary, or a Rust toolchain so the script can run the
  CLI with `cargo run`.

No root privileges, no network access, and no external services are required.

## How to run

From anywhere in the repository:

```console
$ bash examples/demo/demo.sh
```

The script resolves the CLI as follows:

1. if `RAHN_BIN` is set, that binary is used;
2. otherwise the CLI is run from the repository root with
   `cargo run -q -p rahn-cli --`.

To skip a rebuild, point at an existing binary:

```console
$ RAHN_BIN=./target/debug/rahn bash examples/demo/demo.sh
```

The script prints which binary or command it is using, and the temporary working
directory. It creates that directory under the system temp location
(`mktemp -d`) and deletes it on exit, including on failure or interrupt. The
repository and the caller's working directory are never modified.

## Expected output

Excerpt from a run. State and commit identifiers are stable across machines and
runs; only the temporary directory path on the `init` line varies.

```console
=== 3/10 constitution — encode policy that gates every candidate state
$ rahn verify
state 4c5325fdd071e638129e94f4464f1654efce8bed218778609526c8e485df03a2
  PASS referential-integrity
  PASS link-endpoints-exist
  PASS no-duplicate-links
  PASS no-self-loops
  PASS named-connectivity:web:db
verification PASSED (5 invariants)

=== 4/10 commit — a verified state becomes durable and records its operations
$ rahn commit -m "web-db topology"
[main ad60e9186c67] web-db topology
  state fc9aac040061 · 2 node(s), 2 interface(s), 1 link(s)

=== 7/10 VERIFY-BEFORE-EXECUTE — the constitution rejects an invalid transition
$ rahn commit -m "remove db link"
error: commit rejected by verification:
state 409cccaaa0b5019e8795f3cef43b6d71cb78e7f4ae588377e51ea74eedf6cd2e
  PASS referential-integrity
  ...
  FAIL named-connectivity:web:db — no path between "web" and "db"
verification FAILED (1 of 5 invariants)

=== 8/10 apply — an inspectable execution plan, nothing executed
$ rahn apply experiment
target: experiment
backend: simulation
Execution plan (3 step(s)):
  1. create node cache
  2. create interface cache/eth0
  3. create link cache/eth0 <-> web/eth0
(dry run — pass --execute --yes-i-know to realize this plan)

final ids (stable across runs on every machine):
  main       commit ad60e9186c674c46e5dd92b85338050771a1030880532e681b9aa253f18520c0
             state  fc9aac0400618034b1d6fc8d147d6b04cafbc42b3bb81d308779b8b1c358880e
  experiment commit 684a0103807763afda8fcb6a496daf0383a5c3e33b54ff1995da8274785f5b68
             state  5446796f93d189fcd5e21ecf94e855be72f6348365ae378c9ff4480abc216237
```

## What each step demonstrates

| Step | What it shows | Where it is specified |
|---|---|---|
| 1 `init` | An empty state and a content-addressed store: state is the first-class object, not a directory of configuration. | [ARCHITECTURE.md](../../ARCHITECTURE.md), [ADR 0002](../../docs/adr/0002-state-identity.md), [docs/spec/storage.md](../../docs/spec/storage.md) |
| 2 topology | Explicit nodes, interfaces, and normalized undirected links; endpoint syntax is `node/interface`. | [docs/spec/objects.md](../../docs/spec/objects.md), [ADR 0011](../../docs/adr/0011-interfaces-canonical-v2.md) |
| 3 constitution | `require-connectivity web db` is data, read from `.rahn/constitution`, and gates the candidate state; `verify` checks the working state. | [docs/spec/constitution.md](../../docs/spec/constitution.md), [docs/spec/invariants.md](../../docs/spec/invariants.md), [ADR 0006](../../docs/adr/0006-verification-model.md) |
| 4 commit | A commit records the state id, parents, the exact operations applied, and the verification summary; `inspect` shows those operations. `test` verifies HEAD and emits the machine-readable report CI uses. | [docs/spec/transitions.md](../../docs/spec/transitions.md), [ADR 0017](../../docs/adr/0017-ci-verification.md) |
| 5 branch + diff | Independent lines of state; `branch` creates without switching, `checkout` selects, and `diff` compares the topology model rather than text. | [ADR 0007](../../docs/adr/0007-branching-model.md), [ADR 0010](../../docs/adr/0010-working-state-checkout.md), [ARCHITECTURE.md](../../ARCHITECTURE.md) |
| 6 `path` | Reachability derived from links, so the result is identical on every replica. | [ARCHITECTURE.md](../../ARCHITECTURE.md) |
| 7 rejection | **The central claim.** A state that breaks an encoded invariant is refused at commit time, before any durable write: no state object, no commit object, no branch move. | [ADR 0006](../../docs/adr/0006-verification-model.md), [docs/spec/invariants.md](../../docs/spec/invariants.md) |
| 8 `apply` | Execution is separated from intent: the simulation backend renders the plan that realizing the target state would require and stops. | [ADR 0008](../../docs/adr/0008-execution-boundary.md), [ADR 0016](../../docs/adr/0016-execution-backends.md), [docs/spec/execution.md](../../docs/spec/execution.md) |
| 9 memory | Observations are immutable, provenance-bearing records with caller-supplied timestamps; causal edges are asserted and status-labeled, never inferred; `explain` returns the connected component. | [docs/spec/observation.md](../../docs/spec/observation.md), [docs/spec/causality.md](../../docs/spec/causality.md), [ADR 0013](../../docs/adr/0013-observation-model.md), [ADR 0014](../../docs/adr/0014-causal-memory.md) |
| 10 summary | A recap of the loop and the final identifiers. | this file |

## Determinism and identifiers

Running the demo twice produces identical state and commit ids; the demo is
deterministic by construction (fixed topology, fixed commit messages,
caller-supplied observation timestamps). The same is expected of the
implementation, and is test-enforced; see
[docs/spec/serialization.md](../../docs/spec/serialization.md) and
[docs/reproducibility.md](../../docs/reproducibility.md).

Two state hex values appear in the output, and they are not equal for the same
state:

- `verify` and `test` print the digest of the state they checked;
- `log`, `inspect`, and `apply` print the state's content address in the store.

Both are deterministic hashes over the same canonical state bytes with
different framing. Neither is a timestamp or a random value.

## Non-goals

This demo is deliberately narrow.

- **Simulation only.** Nothing is executed against any system. Real namespace
  execution is a separate, explicit opt-in and is out of scope here.
- **No host network.** No command in the script contacts the network or
  reconfigures the host.
- **No AI.** RAHN has no AI dependency; nothing here involves one.
- **A simulation is not a production controller.** `apply` produces an
  inspectable plan, not a deployment.
- **"Verified" means only** that all encoded invariants hold - none of the
  demo's output claims more (see
  [docs/research/limitations.md](../../docs/research/limitations.md)).
- **No post-v1.0 capability is demonstrated or implied.** Addressing, traffic
  control, authentication, replicated logs, and further backends are not part of
  v1.0 and are not scheduled.
- **RAHN is not "Git for networks."** The versioning vocabulary is an analogy
  for state evolution, not a claim about the product.

## Where to go next

- [`examples/cli-walkthrough.md`](../cli-walkthrough.md) - the same workflow as
  a transcript, including merge.
- [`examples/network-ci.yml`](../network-ci.yml) - gating a topology change in
  CI.
- [`crates/rahn-sdk`](../../crates/rahn-sdk) - the curated public API for
  building against RAHN.
