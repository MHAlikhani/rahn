// SPDX-License-Identifier: Apache-2.0

//! Command execution: wiring the CLI to the store, verification, and
//! simulation layers.
//!
//! Security posture (ADR 0008): no command performs any action against the
//! host or any network. `apply` prints an execution plan and nothing else.

use std::path::Path;

use rahn_core::{Metadata, Network, State, VerificationSummary};
use rahn_state::commit::CommitRecord;
use rahn_state::diff::diff;
use rahn_state::history::common_ancestor;
use rahn_state::transition::Operation;
use rahn_store::{Store, StoreError};
use rahn_verify::{verify, Constitution};use rahn_state::commit::CommitId;

use crate::args::Command;

#[derive(Debug)]
pub enum CliError {
    /// Usage problem: message is printed to stderr, exit code 2.
    Usage(String),
    /// Runtime problem: message is printed to stderr, exit code 1.
    Runtime(String),
}

impl From<StoreError> for CliError {
    fn from(e: StoreError) -> Self {
        CliError::Runtime(e.to_string())
    }
}

pub const REPO_DIR: &str = ".rahn";

/// Execute a command inside `dir`. Returns the text to print on success.
pub fn run(dir: &Path, command: Command) -> Result<String, CliError> {
    match command {
        Command::Init => init(dir),
        Command::NodeAdd { id, metadata } => modify(dir, |_| {
            let md: Metadata = metadata.into_iter().collect();
            Ok(Operation::AddNode { id, metadata: md })
        }),
        Command::NodeRemove { id } => modify(dir, |_| Ok(Operation::RemoveNode { id })),
        Command::LinkAdd { a, b } => modify(dir, |_| Ok(Operation::AddLink { a, b })),
        Command::LinkRemove { a, b } => modify(dir, |_| Ok(Operation::RemoveLink { a, b })),
        Command::Commit { message } => commit(dir, message),
        Command::State => state(dir),
        Command::BranchList => branch_list(dir),
        Command::BranchCreate { name } => branch_create(dir, &name),
        Command::Diff { from, to } => diff_cmd(dir, from, to),
        Command::Merge { branch } => merge_cmd(dir, branch),
        Command::Verify => verify_cmd(dir),
        Command::Log => log_cmd(dir),
        Command::Inspect { name } => inspect_cmd(dir, name),
        Command::Apply { name } => apply_cmd(dir, name),
    }
}

fn open_repo(dir: &Path) -> Result<Store, CliError> {
    Store::open(dir.join(REPO_DIR)).map_err(|_| {
        CliError::Runtime(format!(
            "not a rahn repository: {} missing or invalid (run `rahn init`)",
            dir.join(REPO_DIR).display()
        ))
    })
}

/// Load and parse the constitution.
fn constitution_of(store: &Store) -> Result<Constitution, CliError> {
    let text = store.load_constitution_text()?;
    Constitution::parse(&text)
        .map_err(|e| CliError::Runtime(format!("invalid constitution: {e}")))
}

fn init(dir: &Path) -> Result<String, CliError> {
    let root = dir.join(REPO_DIR);
    if root.exists() {
        return Err(CliError::Runtime(format!(
            "repository already exists at {}",
            root.display()
        )));
    }
    Store::init(&root)?;
    Ok(format!("initialized empty rahn repository in {}", root.display()))
}

/// Apply one operation to the working (index) state.
fn modify(dir: &Path, make_op: impl FnOnce(&State) -> Result<Operation, CliError>) -> Result<String, CliError> {
    let store = open_repo(dir)?;
    let index = store.load_index()?;
    let op = make_op(&index)?;
    let next = rahn_state::transition::apply(&index.network, &op)
        .map_err(|e| CliError::Runtime(e.to_string()))?;
    store.save_index(&State { network: next })?;
    Ok(format!("ok: {op} (working state, uncommitted — run `rahn commit -m \"...\"`)"))
}

fn head_commit(store: &Store) -> Result<Option<CommitId>, CliError> {
    let branch = store.head()?;
    store.get_branch(&branch).map_err(Into::into)
}

fn state_of_commit(store: &Store, commit: CommitId) -> Result<State, CliError> {
    let rec = store
        .get_commit(commit)?
        .ok_or_else(|| CliError::Runtime(format!("commit {} missing from store", commit.as_hex())))?;
    store
        .get_state(rec.state_id)?
        .ok_or_else(|| CliError::Runtime(format!("state {} missing from store", rec.state_id.as_hex())))
}

/// Derive the operation list implied by diffing `from` to `to`.
/// v0.1 has no metadata-edit operations, so only add/remove operations can
/// appear (documented in docs/spec/transitions.md).
fn operations_between(from: &Network, to: &Network) -> Result<Vec<Operation>, CliError> {
    let d = diff(from, to);
    if !d.changed_node_metadata.is_empty() {
        return Err(CliError::Runtime(
            "node metadata changed outside the v0.1 operation vocabulary; \
             commit rejected — recreate the node instead"
                .to_owned(),
        ));
    }
    let mut ops = Vec::new();
    for (a, b) in &d.removed_links {
        ops.push(Operation::RemoveLink { a: a.clone(), b: b.clone() });
    }
    for id in &d.removed_nodes {
        ops.push(Operation::RemoveNode { id: id.clone() });
    }
    for id in &d.added_nodes {
        let node = to.node(id).expect("diff added node exists in target");
        ops.push(Operation::AddNode { id: node.id.clone(), metadata: node.metadata.clone() });
    }
    for (a, b) in &d.added_links {
        ops.push(Operation::AddLink { a: a.clone(), b: b.clone() });
    }
    Ok(ops)
}

fn commit(dir: &Path, message: String) -> Result<String, CliError> {
    let store = open_repo(dir)?;
    let index = store.load_index()?;
    let parent = head_commit(&store)?;
    let parent_state = match parent {
        Some(id) => state_of_commit(&store, id)?,
        None => State::empty(),
    };
    if diff(&parent_state.network, &index.network).is_empty() {
        return Err(CliError::Runtime("nothing to commit (working state equals HEAD)".into()));
    }
    let operations = operations_between(&parent_state.network, &index.network)?;

    // Verify BEFORE any durable write (ADR 0006: verify before execute;
    // a failing candidate must not enter the store).
    let constitution = constitution_of(&store)?;
    let report = verify(&index, &constitution);
    if !report.passed() {
        return Err(CliError::Runtime(format!("commit rejected by verification:\n{report}")));
    }

    let state_id = store.put_state(&index)?;
    let record = CommitRecord {
        state_id,
        parents: parent.into_iter().collect(),
        operations,
        message,
        verification: VerificationSummary::passed(),
    };
    let commit_id = store.put_commit(&record)?;
    let branch = store.head()?;
    store.set_branch(&branch, commit_id)?;
    Ok(format!(
        "[{branch} {}] {}\n  state {} · {} node(s), {} link(s)",
        &commit_id.as_hex()[..12],
        record.message,
        &state_id.as_hex()[..12],
        index.network.node_count(),
        index.network.link_count(),
    ))
}

fn state(dir: &Path) -> Result<String, CliError> {
    let store = open_repo(dir)?;
    let index = store.load_index()?;
    let branch = store.head()?;
    let head = head_commit(&store)?;
    let parent_state = match head {
        Some(id) => state_of_commit(&store, id)?,
        None => State::empty(),
    };
    let dirty = !diff(&parent_state.network, &index.network).is_empty();
    let mut out = format!(
        "on branch {branch}\n\
         HEAD: {}\n\
         working state: {} node(s), {} link(s) [{}]\n",
        head.map(|c| c.as_hex().to_owned()).unwrap_or_else(|| "(no commits)".into()),
        index.network.node_count(),
        index.network.link_count(),
        if dirty { "uncommitted changes" } else { "clean" },
    );
    for node in index.network.iter_nodes() {
        out.push_str(&format!("  node {}\n", node.id));
    }
    for link in index.network.iter_links() {
        out.push_str(&format!("  link {link}\n"));
    }
    Ok(out)
}

fn branch_list(dir: &Path) -> Result<String, CliError> {
    let store = open_repo(dir)?;
    let head = store.head()?;
    let mut out = format!("* {head}\n");
    for name in store.list_branches()? {
        if name != head {
            out.push_str(&format!("  {name}\n"));
        }
    }
    Ok(out)
}

fn branch_create(dir: &Path, name: &str) -> Result<String, CliError> {
    let store = open_repo(dir)?;
    let head = head_commit(&store)?
        .ok_or_else(|| CliError::Runtime("cannot branch before the first commit".into()))?;
    if store.get_branch(name)?.is_some() {
        return Err(CliError::Runtime(format!("branch {name:?} already exists")));
    }
    store.set_branch(name, head)?;
    Ok(format!("branch {name:?} created at {}", head.as_hex()))
}

/// Resolve a ref (branch name or full commit id) to a commit.
fn resolve_commit(store: &Store, name: &str) -> Result<CommitId, CliError> {
    if let Some(commit) = store.get_branch(name)? {
        return Ok(commit);
    }
    if let Some(commit) = CommitId::from_hex(name) {
        if store.get_commit(commit)?.is_some() {
            return Ok(commit);
        }
        return Err(CliError::Runtime(format!("commit {name} exists nowhere in this repository")));
    }
    Err(CliError::Runtime(format!(
        "unknown ref {name:?} (use a branch name or full 64-character commit id)"
    )))
}

fn diff_cmd(dir: &Path, from: String, to: String) -> Result<String, CliError> {
    let store = open_repo(dir)?;
    let from_commit = resolve_commit(&store, &from)?;
    let to_commit = resolve_commit(&store, &to)?;
    let a = state_of_commit(&store, from_commit)?;
    let b = state_of_commit(&store, to_commit)?;
    let d = diff(&a.network, &b.network);
    if d.is_empty() {
        return Ok(format!("no differences between {from} and {to}"));
    }
    let mut out = String::new();
    for id in &d.added_nodes {
        out.push_str(&format!("+ node: {id}\n"));
    }
    for id in &d.removed_nodes {
        out.push_str(&format!("- node: {id}\n"));
    }
    for (a, b) in &d.added_links {
        out.push_str(&format!("+ link: {a} <-> {b}\n"));
    }
    for (a, b) in &d.removed_links {
        out.push_str(&format!("- link: {a} <-> {b}\n"));
    }
    for (id, key) in &d.changed_node_metadata {
        out.push_str(&format!("~ node {id}: metadata key {key:?} changed\n"));
    }
    Ok(out)
}

fn merge_cmd(dir: &Path, branch: String) -> Result<String, CliError> {
    let store = open_repo(dir)?;
    let head_branch = store.head()?;
    let ours_commit = head_commit(&store)?
        .ok_or_else(|| CliError::Runtime("cannot merge before the first commit".into()))?;
    let theirs_commit = resolve_commit(&store, &branch)?;
    if ours_commit == theirs_commit {
        return Err(CliError::Runtime("already up to date (refs point to the same commit)".into()));
    }

    let parents_of = |id: CommitId| -> Option<Vec<CommitId>> {
        store.get_commit(id).ok().flatten().map(|r| r.parents)
    };
    let ancestor = common_ancestor(ours_commit, theirs_commit, parents_of)
        .ok_or_else(|| CliError::Runtime("no common ancestor: histories are unrelated".into()))?;

    let base = state_of_commit(&store, ancestor)?;
    let ours = state_of_commit(&store, ours_commit)?;
    let theirs = state_of_commit(&store, theirs_commit)?;

    // Fail-closed semantic merge (ADR 0007).
    let merged_network = rahn_verify::merge(&base.network, &ours.network, &theirs.network)
        .map_err(|e| CliError::Runtime(e.to_string()))?;
    let merged = State { network: merged_network };

    // The merge result must pass the constitution like any transition.
    let constitution = constitution_of(&store)?;
    let report = verify(&merged, &constitution);
    if !report.passed() {
        return Err(CliError::Runtime(format!(
            "merge result rejected by verification:\n{report}"
        )));
    }

    let state_id = store.put_state(&merged)?;
    let record = CommitRecord {
        state_id,
        parents: vec![ours_commit, theirs_commit],
        operations: Vec::new(),
        message: format!("merge {branch}"),
        verification: VerificationSummary::passed(),
    };
    let commit_id = store.put_commit(&record)?;
    store.set_branch(&head_branch, commit_id)?;
    store.save_index(&merged)?;
    Ok(format!(
        "merged {branch} into {head_branch}: commit {}\n  merged state: {} node(s), {} link(s)",
        &commit_id.as_hex()[..12],
        merged.network.node_count(),
        merged.network.link_count(),
    ))
}

fn verify_cmd(dir: &Path) -> Result<String, CliError> {
    let store = open_repo(dir)?;
    let index = store.load_index()?;
    let constitution = constitution_of(&store)?;
    let report = verify(&index, &constitution);
    if report.passed() {
        Ok(report.to_string())
    } else {
        Err(CliError::Runtime(report.to_string()))
    }
}

fn log_cmd(dir: &Path) -> Result<String, CliError> {
    let store = open_repo(dir)?;
    let mut out = String::new();
    let mut current = head_commit(&store)?;
    let mut first = true;
    while let Some(id) = current {
        let rec = store
            .get_commit(id)?
            .ok_or_else(|| CliError::Runtime(format!("commit {} missing from store", id.as_hex())))?;
        let marker = if first { "-> " } else { "   " };
        out.push_str(&format!(
            "{marker}commit {}  state {}\n       {} [verification: {}]\n",
            id.as_hex(),
            rec.state_id.as_hex(),
            if rec.parents.len() == 2 { "merge · " } else { "" },
            if rec.verification.passed { "passed" } else { "FAILED" },
        ));
        out.push_str(&format!("       {}\n", rec.message));
        first = false;
        current = rec.parents.first().copied();
    }
    if first {
        out.push_str("no commits yet\n");
    }
    Ok(out)
}

fn inspect_cmd(dir: &Path, name: String) -> Result<String, CliError> {
    let store = open_repo(dir)?;
    let commit = resolve_commit(&store, &name)?;
    let rec = store
        .get_commit(commit)?
        .ok_or_else(|| CliError::Runtime(format!("commit {} missing from store", commit.as_hex())))?;
    let state = state_of_commit(&store, commit)?;
    let mut out = format!(
        "commit {}\n  state: {}\n  parents: {}\n  message: {}\n  verification: {}\n  operations:\n",
        commit.as_hex(),
        rec.state_id.as_hex(),
        if rec.parents.is_empty() { "(root)".to_owned() } else { rec.parents.iter().map(|p| p.as_hex()).collect::<Vec<_>>().join(", ") },
        rec.message,
        if rec.verification.passed { "passed" } else { "FAILED" },
    );
    if rec.operations.is_empty() {
        out.push_str("    (none)\n");
    }
    for op in &rec.operations {
        out.push_str(&format!("    {op}\n"));
    }
    out.push_str(&format!(
        "network: {} node(s), {} link(s)\n",
        state.network.node_count(),
        state.network.link_count()
    ));
    for node in state.network.iter_nodes() {
        out.push_str(&format!("  node {}\n", node.id));
    }
    for link in state.network.iter_links() {
        out.push_str(&format!("  link {link}\n"));
    }
    Ok(out)
}

fn apply_cmd(dir: &Path, name: String) -> Result<String, CliError> {
    let store = open_repo(dir)?;
    let target_commit = resolve_commit(&store, &name)?;
    let target = state_of_commit(&store, target_commit)?;
    let current = match head_commit(&store)? {
        Some(id) => state_of_commit(&store, id)?,
        None => State::empty(),
    };
    let plan = rahn_sim::plan(&current.network, &target.network);
    Ok(format!("target: {}\n{plan}", name))
}

/// Exposed for integration tests: run and get the exit code semantic.
pub fn run_or_exit_code(dir: &Path, command: Command) -> (Result<String, CliError>, i32) {
    match run(dir, command) {
        Ok(out) => (Ok(out), 0),
        Err(CliError::Usage(m)) => (Err(CliError::Usage(m)), 2),
        Err(e @ CliError::Runtime(_)) => (Err(e), 1),
    }
}
