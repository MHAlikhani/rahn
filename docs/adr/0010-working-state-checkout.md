<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# ADR 0010: Working State Model and Branch Checkout

- Status: Accepted
- Date: 2026-04-24
- Deciders: v0.1 hardening review

## Context

The v0.1 flow distinguishes a **working state** (`.rahn/index`, mutable, unverified) from **committed states** (immutable, content-addressed). Branches existed as references, but the CLI provided no way to switch between them, which made two things true: diverged-branch merges were unreachable from the CLI, and branch semantics were undertested end-to-end. Adding checkout raises a safety question: what happens to uncommitted working changes when the user switches branches?

## Problem

Checkout must not silently destroy work. But if a working change can *never* be committed (e.g., the constitution rejects it), refusing checkout unconditionally would strand the user with no recovery path.

## Alternatives

1. **Auto-carry working changes across checkout** (Git-like carry-over): rejected — silently mixing state from two branches violates explicitness and can produce states that are neither branch's evolution.
2. **Auto-stash**: rejected — stash is extra machinery and hidden state for a v0.1 whose working flows are tiny.
3. **Refuse checkout unconditionally when dirty**: safe but can strand the user permanently (a constitution-rejected change cannot be committed and cannot be abandoned).
4. **Refuse when dirty, with explicit `--force` discard** — chosen.

## Decision

1. `rahn checkout <branch>`:
   - requires the working index to be **clean** (identical to the current HEAD commit's state); otherwise it fails closed with an explanation;
   - requires the target branch to exist;
   - sets HEAD to the branch and loads the branch tip's state into the index.
2. `rahn checkout --force <branch>` is the **explicit** recovery path: it discards the working index and restores the target branch's state. It touches only `.rahn/index` — never committed objects, which are immutable by construction.
3. Checkout is a local pointer/index operation; it performs no verification (committed states were verified at commit time) and has no execution effects (ADR 0008).

## Consequences

- Diverged-branch merge flows are fully reachable and testable from the CLI.
- The recovery path for constitution-rejected working changes exists but is opt-in and named (`--force`), matching the project's "explicit over convenient" principle.
- A future stash or worktree feature is additive and does not change these semantics.

## Rejected options

See Alternatives 1–3.

## Future reconsideration conditions

- If Stage 2+ introduces richer working flows (partial staging, metadata edits), revisit whether the clean-index rule still suffices or a stash model is warranted.
- If real execution backends arrive (Stage 3+), checkout semantics must be re-examined: switching branches may eventually imply reconciling backend state, which is a much stronger requirement.

## Verification

- `checkout_and_merge_of_diverged_branches`, `checkout_refuses_dirty_working_state`, and `constitution_survives_checkout_and_gates_merge` in `crates/rahn-cli/tests/end_to_end.rs` encode each rule above, including the `--force` recovery path.

## References

- docs/adr/0007-branching-model.md; docs/spec/transitions.md; docs/testing.md
