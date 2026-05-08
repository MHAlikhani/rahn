<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN CLI Walkthrough (v0.1)

A complete v0.1 session. Everything below is simulation-only: no command
touches the host network or OS.

```console
$ rahn init
initialized empty rahn repository in ./.rahn

$ rahn node add web role=api
$ rahn node add db role=database
$ rahn interface add web eth0
$ rahn interface add db eth0
$ rahn link add web/eth0 db/eth0

$ rahn verify
state e65660d7193be9e9e38631ed19a18914c4b2f6efe2f852a012a40aad073990d3
  PASS referential-integrity
  PASS link-endpoints-exist
  PASS no-duplicate-links
  PASS no-self-loops
verification PASSED (4 invariants)

$ rahn commit -m "web-db topology"
[main 0111627322a9] web-db topology
  state b2fea01105a0 - 2 node(s), 1 link(s)

$ rahn branch experiment
$ rahn node add cache
$ rahn link add web cache
$ rahn commit -m "add cache"
[main d7a594d0ff05] add cache

$ rahn diff experiment main
+ node: cache
+ link: cache/eth0 <-> web/eth0

$ rahn apply experiment
target: experiment
Execution plan:
  1. remove link cache/eth0 <-> web/eth0
  2. remove interface cache/eth0
  3. remove node cache
(simulation only - no action is taken against any system)

$ rahn log
-> commit d7a594d0ff05...  state b9c92fe47e3d...
        [verification: passed]
       add cache
   commit 0111627322a9...  state b2fea01105a0...
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
- **Simulation-only execution**: `apply` produces an inspectable plan and
  nothing else (ADR 0008).

## Constitution example

Write `.rahn/constitution` (documented format, `docs/spec/constitution.md`):

```
require-connectivity web db
```

Now `rahn verify` and every commit check that `web` can reach `db` via
links. A candidate state that breaks the path is rejected with
`FAIL named-connectivity:web:db - no path between "web" and "db"`.
