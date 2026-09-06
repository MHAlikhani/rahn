#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
#
# RAHN runnable demo: the core v1.0 workflow, end to end.
#
# Simulation only. No command here touches the host network or any other
# system: the script never passes --execute or --yes-i-know. It runs entirely
# in a temporary directory outside the repository and removes it on exit.
#
# Usage:
#   bash examples/demo/demo.sh
#   RAHN_BIN=./target/debug/rahn bash examples/demo/demo.sh   # use a built binary
#
# See examples/demo/README.md for what each step demonstrates and why.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

# --- Binary resolution: $RAHN_BIN if set, otherwise `cargo run` from the repo.
if [ -n "${RAHN_BIN:-}" ]; then
  # Resolve to an absolute path before changing directory, so a
  # repository-relative value (e.g. ./target/debug/rahn) keeps working
  # after the script moves into its temporary working directory.
  if [ ! -e "${RAHN_BIN}" ]; then
    printf 'demo: RAHN_BIN does not exist: %s\n' "${RAHN_BIN}" >&2
    exit 2
  fi
  RAHN_BIN_ABS="$(cd "$(dirname "${RAHN_BIN}")" && pwd)/$(basename "${RAHN_BIN}")"
  RAHN_HOW="RAHN_BIN=${RAHN_BIN}"
  rahn() { "${RAHN_BIN_ABS}" "$@"; }
else
  RAHN_HOW="cargo run -q -p rahn-cli -- (repo: ${REPO_ROOT})"
  # CARGO_INCREMENTAL=0 avoids cargo's incremental-session notes on Windows;
  # it changes nothing about what is built.
  rahn() { CARGO_INCREMENTAL=0 cargo run -q --manifest-path "${REPO_ROOT}/Cargo.toml" -p rahn-cli -- "$@"; }
fi

# --- Isolated working directory: never the repository, never the caller's cwd.
WORK="$(mktemp -d)"
cleanup() {
  cd /
  rm -rf "${WORK}" || true
}
trap cleanup EXIT INT TERM
cd "${WORK}"

step() { printf '\n=== %s\n' "$*"; }
note() { printf '    - %s\n' "$*"; }
cmd() { printf '\n$ rahn %s\n' "$*"; rahn "$@"; }

# Run a command that MUST fail (used only for verify-before-execute). stderr is
# merged into stdout so the rejection report appears inline in logs and CI.
cmd_expect_fail() {
  local rc=0
  printf '\n$ rahn %s\n' "$*"
  rahn "$@" 2>&1 || rc=$?
  if [ "${rc}" -ne 1 ]; then
    printf 'demo: expected exit 1, got %s\n' "${rc}" >&2
    exit 1
  fi
}

printf 'RAHN demo — simulation only; nothing is executed against any system\n'
printf 'cli: %s\n' "${RAHN_HOW}"
printf 'work dir (removed on exit): %s\n' "${WORK}"

# ---------------------------------------------------------------------------
step "1/10 init — an empty network state with a content-derived identity"
note "WHY: state is the first-class object; the repository is a store of"
note "content-addressed states, not a directory of configuration files."
cmd init

# ---------------------------------------------------------------------------
step "2/10 topology — nodes, interfaces, links"
note "WHY: the model is explicit. Endpoints are node/interface pairs; links are"
note "undirected and normalized, so endpoints sort and serialize identically"
note "on every machine (docs/spec/objects.md)."
cmd node add web role=frontend
cmd node add db role=database
cmd interface add web eth0
cmd interface add db eth0
cmd link add web/eth0 db/eth0
cmd state

# ---------------------------------------------------------------------------
step "3/10 constitution — encode policy that gates every candidate state"
note "WHY: validity is data, not convention. The constitution is a plain file"
note "at .rahn/constitution (docs/spec/constitution.md); v1.0 has no command"
note "that edits it, so the demo writes the file directly."
printf '# RAHN constitution: the web tier must reach the db tier.\nrequire-connectivity web db\n' > .rahn/constitution
note "require-connectivity web db — a node-level path must exist at all times."
cmd verify
note "verify checks the WORKING state, including uncommitted changes."

# ---------------------------------------------------------------------------
step "4/10 commit — a verified state becomes durable and records its operations"
note "WHY: every commit stores the state id, its parents, the exact operations"
note "applied, and the verification summary (docs/spec/transitions.md)."
cmd commit -m "web-db topology"
cmd inspect main
cmd test
note "test (no argument) verifies HEAD, the committed state. Its TSV report is"
note "the same check CI runs (ADR 0017, examples/network-ci.yml)."
note "verify/test print the verified state's own digest; log/inspect print its"
note "content address in the store. Same canonical bytes, different framing, so"
note "the two hex values differ. Both are deterministic."

# ---------------------------------------------------------------------------
step "5/10 branch — evolve separately, then diff semantically"
note "WHY: branches are independent lines of state; a branch is created but not"
note "switched to, so checkout selects it explicitly (ADR 0007, ADR 0010)."
cmd branch experiment
cmd checkout experiment
cmd node add cache role=cache
cmd interface add cache eth0
cmd link add cache/eth0 web/eth0
cmd commit -m "add cache tier"
cmd diff main experiment
note "diff is over the topology model — added/removed nodes, interfaces, links —"
note "not over text."

# ---------------------------------------------------------------------------
step "6/10 path — deterministic graph reasoning over the working state"
note "WHY: reachability is derived from links, so the same query returns the"
note "same path on every replica. No device is consulted."
cmd path cache db

# ---------------------------------------------------------------------------
step "7/10 VERIFY-BEFORE-EXECUTE — the constitution rejects an invalid transition"
note "WHY: this is the central claim. A candidate state that breaks an encoded"
note "invariant cannot become a commit; it is rejected before any durable write."
cmd checkout main
cmd link remove web/eth0 db/eth0
note "The working state now breaks require-connectivity web db:"
note "web and db are no longer connected."
printf '\n----- expected rejection -----\n'
cmd_expect_fail verify
cmd_expect_fail commit -m "remove db link"
printf '\n----- rejection confirmed -----\n'
note "The commit failed with exit 1 and wrote nothing: no state object, no"
note "commit object, no branch move. Restoring the link makes the state valid again."
cmd link add web/eth0 db/eth0
cmd verify
note "restored: working state equals HEAD, so there is nothing to commit."

# ---------------------------------------------------------------------------
step "8/10 apply — an inspectable execution plan, nothing executed"
note "WHY: execution is separated from intent. The default backend is simulation:"
note "apply renders the plan that realizing this state WOULD require and stops"
note "(ADR 0008, ADR 0016). No --execute, no --yes-i-know."
cmd apply experiment

# ---------------------------------------------------------------------------
step "9/10 memory — observations with provenance, and asserted causal edges"
note "WHY: observations are immutable records bound to the state they were"
note "validated against; causal edges are asserted and status-labeled, never"
note "inferred. Timestamps are caller-supplied, so ingest is deterministic."
cmd observe web cpu_pct gauge:37 --at 1700000000000000000
cmd observe web/eth0 rx_bytes counter:1024 --at 1700000000000000001
cmd observations
cmd relate obs:0 obs:1 temporal-correlation --note "same collection window"
main_commit="$(rahn inspect main | sed -n '1s/^commit //p')"
exp_commit="$(rahn inspect experiment | sed -n '1s/^commit //p')"
cmd relate "commit:${main_commit}" "commit:${exp_commit}" verified --note "experiment inherits main topology"
cmd explain "commit:${main_commit}"
note "'verified' is an edge status meaning both anchors are commits; the output"
note "is asserted structure, not proven causality (docs/spec/causality.md)."

# ---------------------------------------------------------------------------
step "10/10 summary — what was demonstrated"
cat <<'SUMMARY'
  A complete RAHN loop, entirely in simulation:

    state        content-addressed topology with a stable identity
    transitions  each commit records the exact operations applied
    constitution require-connectivity gates every candidate state
    rejection    an invalid transition is refused before it is durable
    evolution    branch, semantic diff, and an inspectable execution plan
    memory       provenance-bearing observations and causal edges

  Nothing was executed: apply printed a plan and stopped.
  "Verified" here means only "all encoded invariants hold".

  Guided reading : examples/demo/README.md
  Full session   : examples/cli-walkthrough.md
SUMMARY

main_state="$(rahn inspect main | sed -n '2s/^  state: //p')"
exp_state="$(rahn inspect experiment | sed -n '2s/^  state: //p')"
printf '\nfinal ids (stable across runs on every machine):\n'
printf '  main       commit %s\n' "${main_commit}"
printf '             state  %s\n' "${main_state}"
printf '  experiment commit %s\n' "${exp_commit}"
printf '             state  %s\n' "${exp_state}"
