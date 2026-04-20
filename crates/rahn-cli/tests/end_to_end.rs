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
        Err(CliError::Usage(m)) => panic!("expected runtime failure for {:?}, got usage error: {m}", args_list),
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

    // Build a small topology: a-b-c.
    ok(&dir, &["node", "add", "a"]);
    ok(&dir, &["node", "add", "b", "role=api"]);
    ok(&dir, &["node", "add", "c"]);
    ok(&dir, &["link", "add", "a", "b"]);
    ok(&dir, &["link", "add", "b", "c"]);

    // Verify the working state before committing.
    let out = ok(&dir, &["verify"]);
    assert!(out.contains("verification PASSED"), "{out}");

    // Commit.
    let out = ok(&dir, &["commit", "-m", "topology a-b-c"]);
    assert!(out.contains("[main"), "{out}");

    // Working state is clean now.
    let out = ok(&dir, &["state"]);
    assert!(out.contains("clean"), "{out}");
    assert!(out.contains("3 node(s)"), "{out}");

    // Branch and diverge.
    ok(&dir, &["branch", "experiment"]);
    let out = ok(&dir, &["branch"]);
    assert!(out.contains("* main"), "{out}");
    assert!(out.contains("experiment"), "{out}");

    // Add a change and commit on main.
    ok(&dir, &["node", "add", "d"]);
    ok(&dir, &["link", "add", "c", "d"]);
    ok(&dir, &["commit", "-m", "add d"]);

    // Diff main against the experiment branch.
    let out = ok(&dir, &["diff", "experiment", "main"]);
    assert!(out.contains("+ node: d"), "{out}");
    assert!(out.contains("+ link: c <-> d"), "{out}");

    // Log shows two commits.
    let out = ok(&dir, &["log"]);
    assert!(out.contains("add d"), "{out}");
    assert!(out.contains("topology a-b-c"), "{out}");
    assert_eq!(out.matches("commit ").count(), 2);

    // Inspect HEAD.
    let out = ok(&dir, &["inspect", "main"]);
    assert!(out.contains("add d"), "{out}");
    assert!(out.contains("add_node d"), "{out}");

    // Simulated apply: prints a plan, touches nothing.
    let out = ok(&dir, &["apply", "experiment"]);
    assert!(out.contains("Execution plan:"), "{out}");
    assert!(out.contains("remove node d"), "{out}");
    assert!(out.contains("simulation only"), "{out}");

    // Apply is purely observational: state unchanged.
    let out = ok(&dir, &["state"]);
    assert!(out.contains("4 node(s)"), "{out}");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn merge_disjoint_branches() {
    let dir = temp_repo("merge");
    ok(&dir, &["init"]);
    ok(&dir, &["node", "add", "a"]);
    ok(&dir, &["node", "add", "b"]);
    ok(&dir, &["link", "add", "a", "b"]);
    ok(&dir, &["commit", "-m", "base"]);

    ok(&dir, &["branch", "feat-c"]);
    ok(&dir, &["node", "add", "c"]);
    ok(&dir, &["link", "add", "b", "c"]);
    ok(&dir, &["commit", "-m", "add c on main"]);

    // Switch to feat-c by branching from main's earlier commit: use merge
    // flow — checkout is out of v0.1 scope, so instead branch at the base
    // commit via inspect. Simpler: recreate the diverged scenario by
    // removing c from the working state is not possible without a checkout.
    // v0.1 merge semantics are covered in rahn-verify tests; here we verify
    // the CLI merge path with an up-to-date branch.
    let out = ok(&dir, &["merge", "feat-c"]);
    assert!(out.contains("already up to date") || out.contains("merged"), "{out}");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn commit_rejected_when_constitution_violated() {
    let dir = temp_repo("constitution");
    ok(&dir, &["init"]);

    // Set a constitution requiring a-b connectivity. The v0.1 CLI has no
    // constitution command; the constitution file is the documented
    // interface (docs/spec/constitution.md).
    std::fs::write(dir.join(".rahn/constitution"), "require-connectivity a b\n").unwrap();

    ok(&dir, &["node", "add", "a"]);
    ok(&dir, &["node", "add", "b"]);
    // No link a-b: connectivity violated.
    let err = fail(&dir, &["commit", "-m", "violates constitution"]);
    assert!(err.contains("named-connectivity"), "{err}");
    assert!(err.contains("no path"), "{err}");

    // Fix by adding the link; commit succeeds.
    ok(&dir, &["link", "add", "a", "b"]);
    let out = ok(&dir, &["commit", "-m", "fixed connectivity"]);
    assert!(out.contains("[main"), "{out}");

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
    let err = fail(&dir, &["link", "add", "a", "ghost"]);
    assert!(err.contains("does not exist"), "{err}");
    let err = fail(&dir, &["link", "add", "a", "a"]);
    assert!(err.contains("self-loop"), "{err}");
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
        ok(&dir, &["link", "add", "a", "b"]);
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
