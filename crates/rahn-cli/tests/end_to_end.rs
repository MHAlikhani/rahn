// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use rahn_cli::{parse, run, CliError};

/// Run `rahn <args>` in `dir`, asserting success; returns stdout.
fn ok(dir: &Path, args_list: &[&str]) -> String {
    let argv: Vec<String> = args_list.iter().map(|s| s.to_string()).collect();
    let cmd = parse(&argv).unwrap_or_else(|e| panic!("parse {:?}: {e}", args_list));
    match run(dir, cmd) {
        Ok(out) => out,
        Err(CliError::Usage(m)) => panic!("usage error for {:?}: {m}", args_list),
        Err(CliError::Runtime(m)) => panic!("runtime error for {:?}: {m}", args_list),
    }
}

/// Run `rahn <args>` expecting a runtime failure; returns the message.
fn fail(dir: &Path, args_list: &[&str]) -> String {
    let argv: Vec<String> = args_list.iter().map(|s| s.to_string()).collect();
    let cmd = parse(&argv).unwrap_or_else(|e| panic!("parse {:?}: {e}", args_list));
    match run(dir, cmd) {
        Ok(out) => panic!("expected failure for {:?}, got: {out}", args_list),
        Err(CliError::Runtime(m)) => m,
        Err(CliError::Usage(m)) => {
            panic!(
                "expected runtime failure for {:?}, got usage error: {m}",
                args_list
            )
        }
    }
}

fn temp_repo(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rahn-cli-test-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn full_lifecycle_init_commit_branch_diff_verify_apply_log() {
    let dir = temp_repo("lifecycle");
    ok(&dir, &["init"]);

    // Build a small topology: a - b - c.
    ok(&dir, &["node", "add", "a"]);
    ok(&dir, &["node", "add", "b", "role=api"]);
    ok(&dir, &["node", "add", "c"]);
    ok(&dir, &["interface", "add", "a", "eth0"]);
    ok(&dir, &["interface", "add", "b", "eth0"]);
    ok(&dir, &["interface", "add", "b", "eth1"]);
    ok(&dir, &["interface", "add", "c", "eth0"]);
    ok(&dir, &["link", "add", "a/eth0", "b/eth0"]);
    ok(&dir, &["link", "add", "b/eth1", "c/eth0"]);

    // Verify the working state before committing.
    let out = ok(&dir, &["verify"]);
    assert!(out.contains("verification PASSED"), "{out}");

    // Path discovery over the working state.
    let out = ok(&dir, &["path", "a", "c"]);
    assert!(out.contains("a -> b -> c"), "{out}");

    // Commit.
    let out = ok(&dir, &["commit", "-m", "topology a-b-c"]);
    assert!(out.contains("[main"), "{out}");
    assert!(out.contains("4 interface(s)"), "{out}");

    // Working state is clean now.
    let out = ok(&dir, &["state"]);
    assert!(out.contains("clean"), "{out}");
    assert!(out.contains("2 link(s)"), "{out}");
    assert!(out.contains("iface eth1"), "{out}");

    // Branch and diverge.
    ok(&dir, &["branch", "experiment"]);
    ok(&dir, &["node", "add", "d"]);
    ok(&dir, &["interface", "add", "d", "eth0"]);
    ok(&dir, &["link", "add", "c/eth0", "d/eth0"]);
    ok(&dir, &["commit", "-m", "add d"]);

    // Diff main against the experiment branch.
    let out = ok(&dir, &["diff", "experiment", "main"]);
    assert!(out.contains("+ node: d"), "{out}");
    assert!(out.contains("+ interface: d/eth0"), "{out}");
    assert!(out.contains("+ link: c/eth0 <-> d/eth0"), "{out}");

    // Log shows two commits.
    let out = ok(&dir, &["log"]);
    assert!(out.contains("add d"), "{out}");
    assert!(out.contains("topology a-b-c"), "{out}");
    assert_eq!(out.matches("commit ").count(), 2);

    // Inspect HEAD: operations include interface and link additions.
    let out = ok(&dir, &["inspect", "main"]);
    assert!(out.contains("add d"), "{out}");
    assert!(out.contains("add_interface d/eth0"), "{out}");
    assert!(out.contains("add_link c/eth0 d/eth0"), "{out}");

    // Simulated apply (default backend): model-level plan, touches nothing.
    let out = ok(&dir, &["apply", "experiment"]);
    assert!(out.contains("backend: simulation"), "{out}");
    assert!(out.contains("Execution plan (2 step(s))"), "{out}");
    assert!(out.contains("remove link c/eth0 <-> d/eth0"), "{out}");
    assert!(out.contains("remove node d"), "{out}");
    // Interface removals subsumed by node removal must NOT appear.
    assert!(!out.contains("interface"), "{out}");
    assert!(out.contains("dry run"), "{out}");
    // linux-ns dry run shows the exact namespace commands (ADR 0012).
    let out = ok(&dir, &["apply", "--backend", "linux-ns", "experiment"]);
    assert!(out.contains("backend: linux-ns"), "{out}");
    assert!(out.contains("ip -n rahn-c link del"), "{out}");
    assert!(out.contains("ip netns del rahn-d"), "{out}");
    // Unknown backend fails explicitly.
    let err = fail(&dir, &["apply", "--backend", "ebpf", "experiment"]);
    assert!(err.contains("unknown execution backend"), "{err}");
    // Simulation cannot execute, even with the gate satisfied.
    let err = fail(
        &dir,
        &[
            "apply",
            "--backend",
            "simulation",
            "--execute",
            "--yes-i-know",
            "experiment",
        ],
    );
    assert!(err.contains("cannot execute"), "{err}");
    // Opt-in gating: --execute without --yes-i-know is a usage error.
    let argv: Vec<String> = ["apply", "experiment", "--execute"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert!(rahn_cli::parse(&argv).is_err());

    // Apply is purely observational: state unchanged.
    let out = ok(&dir, &["state"]);
    assert!(out.contains("4 node(s)"), "{out}");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn checkout_and_merge_of_diverged_branches() {
    let dir = temp_repo("diverged");
    ok(&dir, &["init"]);
    ok(&dir, &["node", "add", "a"]);
    ok(&dir, &["node", "add", "b"]);
    ok(&dir, &["interface", "add", "a", "eth0"]);
    ok(&dir, &["interface", "add", "b", "eth0"]);
    ok(&dir, &["link", "add", "a/eth0", "b/eth0"]);
    ok(&dir, &["commit", "-m", "base"]);

    // Branch at base, then diverge both branches.
    ok(&dir, &["branch", "feat-c"]);
    ok(&dir, &["checkout", "feat-c"]);
    let out = ok(&dir, &["state"]);
    assert!(out.contains("on branch feat-c"), "{out}");

    // On feat-c: add node c.
    ok(&dir, &["node", "add", "c"]);
    ok(&dir, &["interface", "add", "c", "eth0"]);
    ok(&dir, &["link", "add", "b/eth0", "c/eth0"]);
    ok(&dir, &["commit", "-m", "add c on feat-c"]);

    // Back to main; add node d.
    ok(&dir, &["checkout", "main"]);
    ok(&dir, &["node", "add", "d"]);
    ok(&dir, &["interface", "add", "d", "eth0"]);
    ok(&dir, &["link", "add", "b/eth0", "d/eth0"]);
    ok(&dir, &["commit", "-m", "add d on main"]);

    // The branches have diverged: feat-c has c, main has d.
    let out = ok(&dir, &["diff", "feat-c", "main"]);
    assert!(out.contains("- node: c"), "{out}");
    assert!(out.contains("+ node: d"), "{out}");

    // Merge feat-c into main: disjoint additions.
    let out = ok(&dir, &["merge", "feat-c"]);
    assert!(out.contains("merged feat-c into main"), "{out}");
    let out = ok(&dir, &["state"]);
    assert!(out.contains("4 node(s)"), "{out}");
    assert!(out.contains("node c"), "{out}");
    assert!(out.contains("node d"), "{out}");

    // Merge commit has two parents: inspect shows it.
    let out = ok(&dir, &["inspect", "main"]);
    assert!(out.contains("merge"), "{out}");
    assert!(out.contains("merge feat-c"), "{out}");

    // Log shows three commits.
    let out = ok(&dir, &["log"]);
    assert_eq!(out.matches("commit ").count(), 3, "{out}");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn checkout_refuses_dirty_working_state() {
    let dir = temp_repo("dirty");
    ok(&dir, &["init"]);
    ok(&dir, &["node", "add", "a"]);
    ok(&dir, &["commit", "-m", "first"]);
    ok(&dir, &["branch", "other"]);
    // Dirty the index.
    ok(&dir, &["node", "add", "b"]);
    let err = fail(&dir, &["checkout", "other"]);
    assert!(err.contains("uncommitted changes"), "{err}");
    // Clean up by committing, then checkout succeeds.
    ok(&dir, &["commit", "-m", "second"]);
    ok(&dir, &["checkout", "other"]);
    let out = ok(&dir, &["state"]);
    // "other" was created at the first commit: one node, no "b".
    assert!(out.contains("1 node(s)"), "{out}");
    assert!(out.contains("clean"), "{out}");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn commit_rejected_when_constitution_violated() {
    let dir = temp_repo("constitution");
    ok(&dir, &["init"]);

    // Set a constitution requiring a-b connectivity. The v0.2 CLI has no
    // constitution command; the constitution file is the documented
    // interface (docs/spec/constitution.md).
    std::fs::write(dir.join(".rahn/constitution"), "require-connectivity a b\n").unwrap();

    ok(&dir, &["node", "add", "a"]);
    ok(&dir, &["node", "add", "b"]);
    ok(&dir, &["interface", "add", "a", "eth0"]);
    ok(&dir, &["interface", "add", "b", "eth0"]);
    // No link a-b: connectivity violated.
    let err = fail(&dir, &["commit", "-m", "violates constitution"]);
    assert!(err.contains("named-connectivity"), "{err}");
    assert!(err.contains("no path"), "{err}");

    // Fix by adding the link; commit succeeds.
    ok(&dir, &["link", "add", "a/eth0", "b/eth0"]);
    let out = ok(&dir, &["commit", "-m", "fixed connectivity"]);
    assert!(out.contains("[main"), "{out}");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn commit_rejected_when_prohibited_connectivity_violated() {
    let dir = temp_repo("prohibit");
    ok(&dir, &["init"]);
    std::fs::write(
        dir.join(".rahn/constitution"),
        "prohibit-connectivity public db\n",
    )
    .unwrap();

    // Build public - relay - db: the path exists, so commit must be refused.
    ok(&dir, &["node", "add", "public"]);
    ok(&dir, &["node", "add", "relay"]);
    ok(&dir, &["node", "add", "db"]);
    for n in ["public", "relay", "db"] {
        ok(&dir, &["interface", "add", n, "eth0"]);
        ok(&dir, &["interface", "add", n, "eth1"]);
    }
    ok(&dir, &["link", "add", "public/eth0", "relay/eth0"]);
    ok(&dir, &["link", "add", "relay/eth1", "db/eth0"]);

    let err = fail(&dir, &["commit", "-m", "db is reachable"]);
    assert!(err.contains("prohibited-connectivity:public:db"), "{err}");
    assert!(err.contains("public -> relay -> db"), "{err}");

    // Remove the relay link; the isolation requirement passes.
    ok(&dir, &["link", "remove", "relay/eth1", "db/eth0"]);
    let out = ok(&dir, &["commit", "-m", "db isolated"]);
    assert!(out.contains("[main"), "{out}");
    // The evidence even reports the node-level path that was severed.
    let out = ok(&dir, &["path", "public", "db"]);
    assert!(out.contains("no path"), "{out}");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn invalid_transitions_are_explained() {
    let dir = temp_repo("invalid");
    ok(&dir, &["init"]);
    ok(&dir, &["node", "add", "a"]);
    let err = fail(&dir, &["node", "add", "a"]);
    assert!(err.contains("already exists"), "{err}");
    let err = fail(&dir, &["node", "remove", "ghost"]);
    assert!(err.contains("does not exist"), "{err}");
    ok(&dir, &["interface", "add", "a", "eth0"]);
    let err = fail(&dir, &["interface", "add", "a", "eth0"]);
    assert!(err.contains("already exists"), "{err}");
    ok(&dir, &["node", "add", "b"]);
    ok(&dir, &["interface", "add", "b", "eth0"]);
    let err = fail(&dir, &["link", "add", "a/eth0", "b/eth1"]);
    assert!(err.contains("does not exist on node"), "{err}");
    let err = fail(&dir, &["link", "add", "a/eth0", "a/eth0"]);
    assert!(err.contains("self-loop"), "{err}");
    let err = fail(&dir, &["link", "add", "a/eth0", "a/eth0"]);
    assert!(err.contains("self-loop"), "{err}");
    // Same-node loop via distinct interfaces.
    ok(&dir, &["interface", "add", "a", "eth1"]);
    let err = fail(&dir, &["link", "add", "a/eth0", "a/eth1"]);
    assert!(err.contains("two interfaces of node"), "{err}");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn commands_outside_a_repository_fail() {
    let dir = temp_repo("norepo");
    let err = fail(&dir, &["state"]);
    assert!(err.contains("not a rahn repository"), "{err}");
    let err = fail(&dir, &["commit", "-m", "x"]);
    assert!(err.contains("not a rahn repository"), "{err}");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn corrupted_object_is_detected_loudly() {
    let dir = temp_repo("corruption");
    ok(&dir, &["init"]);
    ok(&dir, &["node", "add", "a"]);
    ok(&dir, &["commit", "-m", "first"]);

    // Flip a byte inside the stored state object.
    let objects = dir.join(".rahn/objects");
    let mut object_file = None;
    for entry in std::fs::read_dir(&objects).unwrap() {
        let entry = entry.unwrap();
        if entry.path().is_dir() {
            for f in std::fs::read_dir(entry.path()).unwrap() {
                let f = f.unwrap().path();
                object_file = Some(f);
            }
        }
    }
    let path = object_file.expect("one stored object");
    let bytes = std::fs::read(&path).unwrap();
    let mut corrupted = bytes.clone();
    let last = corrupted.len() - 1;
    corrupted[last] ^= 0xFF;
    std::fs::write(&path, &corrupted).unwrap();

    // Reading the commit's state must fail loudly, naming corruption
    // (log alone only reads commit records, so use inspect, which loads
    // the state object too).
    let err = fail(&dir, &["inspect", "main"]);
    assert!(err.contains("CORRUPTED"), "{err}");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn determinism_same_commands_same_commit_ids() {
    let run_once = |tag: &str| {
        let dir = temp_repo(tag);
        ok(&dir, &["init"]);
        ok(&dir, &["node", "add", "a"]);
        ok(&dir, &["node", "add", "b"]);
        ok(&dir, &["interface", "add", "a", "eth0"]);
        ok(&dir, &["interface", "add", "b", "eth0"]);
        ok(&dir, &["link", "add", "a/eth0", "b/eth0"]);
        let out = ok(&dir, &["commit", "-m", "ab"]);
        std::fs::remove_dir_all(&dir).ok();
        out
    };
    let a = run_once("det-a");
    let b = run_once("det-b");
    // Commit output embeds the short commit id; identical inputs must
    // produce identical ids (timestamps/hosts must not leak in).
    assert_eq!(a, b, "identical histories must produce identical commits");
}

#[test]
fn observation_ingest_listing_and_determinism() {
    let run_once = |tag: &str| {
        let dir = temp_repo(tag);
        ok(&dir, &["init"]);
        ok(&dir, &["node", "add", "a"]);
        ok(&dir, &["interface", "add", "a", "eth0"]);
        ok(&dir, &["commit", "-m", "base"]);
        ok(
            &dir,
            &[
                "observe",
                "a",
                "latency_ns",
                "counter:1500",
                "--at",
                "1000000000",
            ],
        );
        ok(
            &dir,
            &[
                "observe",
                "a/eth0",
                "link_up",
                "event:link up",
                "--at",
                "1000000001",
            ],
        );
        // Provenance association: unknown subject rejected.
        let err = fail(&dir, &["observe", "ghost", "x", "counter:1", "--at", "1"]);
        assert!(err.contains("does not exist"), "{err}");
        // Malformed value and missing timestamp rejected.
        assert!(fail(&dir, &["observe", "a", "m", "counter:abc", "--at", "1"]).contains("u64"));
        // Missing --at is a usage error (caught at parse level).
        let argv: Vec<String> = ["observe", "a", "m", "counter:5"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(rahn_cli::parse(&argv).unwrap_err().contains("--at"));
        let out = ok(&dir, &["observations"]);
        assert!(out.contains("counter:1500"), "{out}");
        assert!(out.contains("event:link up"), "{out}");
        // Filtered by node.
        let out = ok(&dir, &["observations", "a"]);
        assert_eq!(out.lines().count(), 3, "{out}");
        let log = std::fs::read(dir.join(".rahn/observations.log")).unwrap();
        std::fs::remove_dir_all(&dir).ok();
        log
    };
    let a = run_once("obs-a");
    let b = run_once("obs-b");
    // Deterministic ingest: identical command sequences produce
    // byte-identical observation logs (caller-supplied time, positional seq).
    assert_eq!(a, b, "observation logs must be deterministic");
}

#[test]
fn causal_edges_cycles_status_rules_and_explain() {
    let run_once = |tag: &str| {
        let dir = temp_repo(tag);
        ok(&dir, &["init"]);
        ok(&dir, &["node", "add", "a"]);
        ok(&dir, &["commit", "-m", "base"]);
        ok(&dir, &["observe", "a", "link_up", "event:up", "--at", "10"]);
        ok(
            &dir,
            &["observe", "a", "latency_ns", "counter:99", "--at", "20"],
        );
        // Hypothesis edge between observations.
        ok(
            &dir,
            &[
                "relate",
                "obs:0",
                "obs:1",
                "hypothesis",
                "--note",
                "up before latency",
            ],
        );
        // Cycle rejected.
        let err = fail(
            &dir,
            &["relate", "obs:1", "obs:0", "hypothesis", "--note", "x"],
        );
        assert!(err.contains("cycle"), "{err}");
        // Verified requires commit anchors.
        let err = fail(
            &dir,
            &["relate", "obs:0", "obs:1", "verified", "--note", "x"],
        );
        assert!(err.contains("Commit anchors"), "{err}");
        // Dangling anchors rejected.
        let err = fail(
            &dir,
            &["relate", "obs:0", "obs:42", "hypothesis", "--note", "x"],
        );
        assert!(err.contains("does not exist"), "{err}");
        std::fs::remove_dir_all(&dir).ok();
        let dir = temp_repo(tag);
        ok(&dir, &["init"]);
        ok(&dir, &["node", "add", "a"]);
        ok(&dir, &["commit", "-m", "c1"]);
        ok(&dir, &["node", "add", "b"]);
        ok(&dir, &["commit", "-m", "c2"]);
        let hex_of = |line: &str| {
            line.split_whitespace()
                .find(|t| t.len() == 64)
                .unwrap()
                .to_string()
        };
        let commit_lines: Vec<String> = ok(&dir, &["log"])
            .lines()
            .filter(|l| l.contains("commit "))
            .map(|l| l.to_string())
            .collect();
        let c1 = hex_of(&commit_lines[1]); // oldest
        let c2 = hex_of(&commit_lines[0]); // newest
        ok(
            &dir,
            &[
                "relate",
                &format!("commit:{c1}"),
                &format!("commit:{c2}"),
                "verified",
                "--note",
                "c2 follows c1",
            ],
        );
        // Explain shows the asserted structure.
        let out = ok(&dir, &["explain", &format!("commit:{c1}")]);
        assert!(out.contains("[verified]"), "{out}");
        assert!(out.contains("c2 follows c1"), "{out}");
        let log = std::fs::read(dir.join(".rahn/causal.log")).unwrap();
        std::fs::remove_dir_all(&dir).ok();
        log
    };
    let a = run_once("causal-a");
    let b = run_once("causal-b");
    assert_eq!(a, b, "causal logs must be deterministic");
}
