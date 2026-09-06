<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN CLI Walkthrough (v1.0)

A complete v1.0 session. Everything below is simulation-only: no command
touches the host network or OS. Only the `init` line varies by machine (it
prints the absolute repository path); every state and commit id reproduces
on any machine, at any time. For the guided, runnable version of this
workflow, run `bash examples/demo/demo.sh` — see
[demo/README.md](demo/README.md).

```console
$ rahn init
initialized empty rahn repository in <session>/.rahn

$ rahn node add web role=api
ok: add_node web (working state, uncommitted — run `rahn commit -m "..."`)
$ rahn node add db role=database
ok: add_node db (working state, uncommitted — run `rahn commit -m "..."`)
$ rahn interface add web eth0
ok: add_interface web/eth0 (working state, uncommitted — run `rahn commit -m "..."`)
$ rahn interface add db eth0
ok: add_interface db/eth0 (working state, uncommitted — run `rahn commit -m "..."`)
$ rahn link add web/eth0 db/eth0
ok: add_link web/eth0 db/eth0 (working state, uncommitted — run `rahn commit -m "..."`)

$ rahn verify
state eb5801c5552c6d6ad18b619e4167ffa4f4b4225d5ec6a8fe6f7f532ad95f337d
  PASS referential-integrity
  PASS link-endpoints-exist
  PASS no-duplicate-links
  PASS no-self-loops
verification PASSED (4 invariants)

$ rahn commit -m "web-db topology"
[main c623cc4f59ac] web-db topology
  state e89e6db5c846 · 2 node(s), 2 interface(s), 1 link(s)

$ rahn branch experiment
branch "experiment" created at c623cc4f59ac8946e56f2522ecca7675c6b499f7c3e811aa14e491304a6f0e1a
$ rahn checkout experiment
switched to branch "experiment" (was "main") at commit c623cc4f59ac
$ rahn node add cache
ok: add_node cache (working state, uncommitted — run `rahn commit -m "..."`)
$ rahn interface add cache eth0
ok: add_interface cache/eth0 (working state, uncommitted — run `rahn commit -m "..."`)
$ rahn link add web/eth0 cache/eth0
ok: add_link web/eth0 cache/eth0 (working state, uncommitted — run `rahn commit -m "..."`)
$ rahn commit -m "add cache"
[experiment cf2ae4bcf5eb] add cache
  state be21c120a6ab · 3 node(s), 3 interface(s), 2 link(s)

$ rahn checkout main
switched to branch "main" (was "experiment") at commit c623cc4f59ac
$ rahn diff main experiment
+ node: cache
+ interface: cache/eth0
+ link: cache/eth0 <-> web/eth0

$ rahn merge experiment
merged experiment into main: commit 6605385d9d7b
  merged state: 3 node(s), 3 interface(s), 2 link(s)

$ rahn path web cache
path (1 hop(s)):
web -> cache

$ rahn apply main
target: main
backend: simulation
Execution plan: (no changes)

$ rahn log
-> commit 6605385d9d7b6c132ee74eca8821c783907fffefc213bf20b6ad258f67536387  state be21c120a6ab606a650881dcab8b1d52d46ee7160c0fda0d7f91c88c9fda67b2
       merge · [verification: passed]
       merge experiment
   commit c623cc4f59ac8946e56f2522ecca7675c6b499f7c3e811aa14e491304a6f0e1a  state e89e6db5c8466a099ec0978e0e17b90f43f52ee3c5ea283abb6cde2c7354e4b4
       [verification: passed]
       web-db topology
```

## What this demonstrates

- **Determinism**: the same command sequence on any machine produces the
  same state and commit ids (test-enforced; see
  `crates/rahn-cli/tests/end_to_end.rs`).
- **Verify before execute**: a commit that violates the constitution is
  rejected with a per-invariant explanation.
- **Explicit transitions**: every commit records the exact operations that
  produced the new state.
- **Branch then checkout**: `branch` creates a branch without switching;
  `checkout` selects it, so commits before a checkout land on the branch
  you are already on (ADR 0007, ADR 0010).
- **Semantic merge**: `merge` combines divergent tips through the
  fail-closed three-way merge (ADR 0007).
- **Simulation-only by default**: `apply` produces an inspectable plan and
  nothing else (ADR 0008). Real execution exists only behind
  `--execute --yes-i-know`, against isolated Linux network namespaces
  (ADR 0012, ADR 0016).

## Constitution example

Write `.rahn/constitution` (documented format, `docs/spec/constitution.md`):

```
require-connectivity web db
```

Now `rahn verify` and every commit check that `web` can reach `db` via
links. A candidate state that breaks the path is rejected with
`FAIL named-connectivity:web:db — no path between "web" and "db"`.
