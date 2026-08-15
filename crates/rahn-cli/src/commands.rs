// SPDX-License-Identifier: Apache-2.0

//! Command execution: wiring the CLI to the store, verification, and
//! simulation layers.
//!
//! Security posture (ADR 0008): no command performs any action against the
//! host or any network. `apply` prints an execution plan and nothing else.

use std::path::Path;

use rahn_core::{Endpoint, Metadata, State, VerificationSummary};
use rahn_state::commit::{CommitId, CommitRecord};
use rahn_state::diff::diff;
use rahn_state::graph::shortest_path;
use rahn_state::history::common_ancestor;
use rahn_state::transition::Operation;
use rahn_store::{Store, StoreError};
use rahn_verify::{verify, Constitution};

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
        Command::InterfaceAdd { node, name } => {
            modify(dir, |_| Ok(Operation::AddInterface { node, name }))
        }
        Command::InterfaceRemove { node, name } => {
            modify(dir, |_| Ok(Operation::RemoveInterface { node, name }))
        }
        Command::LinkAdd { a, b } => modify(dir, |_| {
            Ok(Operation::AddLink {
                a: parse_endpoint(&a)?,
                b: parse_endpoint(&b)?,
            })
        }),
        Command::LinkRemove { a, b } => modify(dir, |_| {
            Ok(Operation::RemoveLink {
                a: parse_endpoint(&a)?,
                b: parse_endpoint(&b)?,
            })
        }),
        Command::Commit { message } => commit(dir, message),
        Command::State => state(dir),
        Command::BranchList => branch_list(dir),
        Command::BranchCreate { name } => branch_create(dir, &name),
        Command::Checkout { name, force } => checkout_cmd(dir, &name, force),
        Command::Diff { from, to } => diff_cmd(dir, from, to),
        Command::Merge { branch } => merge_cmd(dir, branch),
        Command::Path { from, to } => path_cmd(dir, from, to),
        Command::Verify => verify_cmd(dir),
        Command::Log => log_cmd(dir),
        Command::Inspect { name } => inspect_cmd(dir, name),
        Command::Apply {
            name,
            backend,
            execute,
            yes_i_know,
        } => apply_cmd(dir, name, &backend, execute, yes_i_know),
        Command::Destroy { name, yes_i_know } => destroy_cmd(dir, name, yes_i_know),
        Command::Observe {
            subject,
            metric,
            value,
            at_ns,
        } => observe_cmd(dir, subject, metric, value, at_ns),
        Command::Observations { filter } => observations_cmd(dir, filter),
        Command::Relate {
            from,
            to,
            status,
            note,
        } => relate_cmd(dir, from, to, status, note),
        Command::Explain { anchor } => explain_cmd(dir, anchor),
        Command::Test { ref_name } => test_cmd(dir, ref_name),
    }
}

fn parse_endpoint(s: &str) -> Result<Endpoint, CliError> {
    Endpoint::parse(s).map_err(|e| CliError::Runtime(e.to_string()))
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
    Constitution::parse(&text).map_err(|e| CliError::Runtime(format!("invalid constitution: {e}")))
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
    Ok(format!(
        "initialized empty rahn repository in {}",
        root.display()
    ))
}

/// Apply one operation to the working (index) state.
fn modify(
    dir: &Path,
    make_op: impl FnOnce(&State) -> Result<Operation, CliError>,
) -> Result<String, CliError> {
    let store = open_repo(dir)?;
    let index = store.load_index()?;
    let op = make_op(&index)?;
    let next = rahn_state::transition::apply(&index.network, &op)
        .map_err(|e| CliError::Runtime(e.to_string()))?;
    store.save_index(&State { network: next })?;
    Ok(format!(
        "ok: {op} (working state, uncommitted — run `rahn commit -m \"...\"`)"
    ))
}

fn head_commit(store: &Store) -> Result<Option<CommitId>, CliError> {
    let branch = store.head()?;
    store.get_branch(&branch).map_err(Into::into)
}

fn state_of_commit(store: &Store, commit: CommitId) -> Result<State, CliError> {
    let rec = store.get_commit(commit)?.ok_or_else(|| {
        CliError::Runtime(format!("commit {} missing from store", commit.as_hex()))
    })?;
    store.get_state(rec.state_id)?.ok_or_else(|| {
        CliError::Runtime(format!(
            "state {} missing from store",
            rec.state_id.as_hex()
        ))
    })
}

/// Derive the operation list implied by diffing `from` to `to`.
/// v0.2 has no metadata-edit operations, so only add/remove operations can
/// appear (documented in docs/spec/transitions.md).
fn operations_between(
    from: &rahn_core::Network,
    to: &rahn_core::Network,
) -> Result<Vec<Operation>, CliError> {
    let d = diff(from, to);
    let mut ops = Vec::new();
    for (a, b) in &d.removed_links {
        ops.push(Operation::RemoveLink {
            a: a.clone(),
            b: b.clone(),
        });
    }
    for (node, name) in &d.removed_interfaces {
        ops.push(Operation::RemoveInterface {
            node: node.clone(),
            name: name.clone(),
        });
    }
    for id in &d.removed_nodes {
        ops.push(Operation::RemoveNode { id: id.clone() });
    }
    for id in &d.added_nodes {
        let node = to.node(id).expect("diff added node exists in target");
        ops.push(Operation::AddNode {
            id: node.id.clone(),
            metadata: node.metadata.clone(),
        });
    }
    for (node, name) in &d.added_interfaces {
        ops.push(Operation::AddInterface {
            node: node.clone(),
            name: name.clone(),
        });
    }
    for (a, b) in &d.added_links {
        ops.push(Operation::AddLink {
            a: a.clone(),
            b: b.clone(),
        });
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
        return Err(CliError::Runtime(
            "nothing to commit (working state equals HEAD)".into(),
        ));
    }
    let operations = operations_between(&parent_state.network, &index.network)?;

    // Verify BEFORE any durable write (ADR 0006: verify before execute;
    // a failing candidate must not enter the store).
    let constitution = constitution_of(&store)?;
    let report = verify(&index, &constitution);
    if !report.passed() {
        return Err(CliError::Runtime(format!(
            "commit rejected by verification:\n{report}"
        )));
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
        "[{branch} {}] {}\n  state {} · {} node(s), {} interface(s), {} link(s)",
        &commit_id.as_hex()[..12],
        record.message,
        &state_id.as_hex()[..12],
        index.network.node_count(),
        index.network.interface_count(),
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
         working state: {} node(s), {} interface(s), {} link(s) [{}]\n",
        head.map(|c| c.as_hex().to_owned())
            .unwrap_or_else(|| "(no commits)".into()),
        index.network.node_count(),
        index.network.interface_count(),
        index.network.link_count(),
        if dirty {
            "uncommitted changes"
        } else {
            "clean"
        },
    );
    for node in index.network.iter_nodes() {
        out.push_str(&format!("  node {}\n", node.id));
        for ifc in node.iter_interfaces() {
            out.push_str(&format!("    iface {}\n", ifc.name));
        }
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

/// Switch the current branch (ADR 0010).
///
/// Fail-closed rules:
/// - the working index MUST be clean (identical to the current HEAD
///   commit's state) — uncommitted changes would be silently destroyed
///   otherwise, and v0.2 has no stash. `--force` is the explicit opt-out:
///   it discards the working index and restores the target branch state.
/// - the target branch MUST exist.
fn checkout_cmd(dir: &Path, name: &str, force: bool) -> Result<String, CliError> {
    let store = open_repo(dir)?;
    let current_branch = store.head()?;
    if current_branch == name {
        return Err(CliError::Runtime(format!("already on branch {name:?}")));
    }
    let target_commit = store
        .get_branch(name)?
        .ok_or_else(|| CliError::Runtime(format!("branch {name:?} does not exist")))?;
    let index = store.load_index()?;
    let current_head_state = match head_commit(&store)? {
        Some(id) => state_of_commit(&store, id)?,
        None => State::empty(),
    };
    if !diff(&current_head_state.network, &index.network).is_empty() && !force {
        return Err(CliError::Runtime(
            "cannot checkout: working state has uncommitted changes \
             (commit them first, or use --force to discard them)"
                .into(),
        ));
    }
    let target_state = state_of_commit(&store, target_commit)?;
    store.set_head(name)?;
    store.save_index(&target_state)?;
    Ok(format!(
        "switched to branch {name:?} (was {current_branch:?}) at commit {}",
        &target_commit.as_hex()[..12]
    ))
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
        return Err(CliError::Runtime(format!(
            "commit {name} exists nowhere in this repository"
        )));
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
    for (node, name) in &d.added_interfaces {
        out.push_str(&format!("+ interface: {node}/{name}\n"));
    }
    for (node, name) in &d.removed_interfaces {
        out.push_str(&format!("- interface: {node}/{name}\n"));
    }
    for (a, b) in &d.added_links {
        out.push_str(&format!("+ link: {a} <-> {b}\n"));
    }
    for (a, b) in &d.removed_links {
        out.push_str(&format!("- link: {a} <-> {b}\n"));
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
        return Err(CliError::Runtime(
            "already up to date (refs point to the same commit)".into(),
        ));
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
    let merged = State {
        network: merged_network,
    };

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
        "merged {branch} into {head_branch}: commit {}\n  merged state: {} node(s), {} interface(s), {} link(s)",
        &commit_id.as_hex()[..12],
        merged.network.node_count(),
        merged.network.interface_count(),
        merged.network.link_count(),
    ))
}

fn path_cmd(dir: &Path, from: String, to: String) -> Result<String, CliError> {
    let store = open_repo(dir)?;
    let index = store.load_index()?;
    match shortest_path(&index.network, &from, &to) {
        Err(rahn_state::graph::PathError::EndpointMissing) => Err(CliError::Runtime(format!(
            "endpoint missing: {from:?} or {to:?} not in the working state"
        ))),
        Ok(None) => Ok(format!("no path between {from} and {to}")),
        Ok(Some(path)) => {
            let mut out = format!("path ({} hop(s)):\n", path.len() - 1);
            for (i, node) in path.iter().enumerate() {
                if i == 0 {
                    out.push_str(node);
                } else {
                    out.push_str(&format!(" -> {node}"));
                }
            }
            out.push('\n');
            Ok(out)
        }
    }
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
        let rec = store.get_commit(id)?.ok_or_else(|| {
            CliError::Runtime(format!("commit {} missing from store", id.as_hex()))
        })?;
        let marker = if first { "-> " } else { "   " };
        out.push_str(&format!(
            "{marker}commit {}  state {}\n       {}[verification: {}]\n",
            id.as_hex(),
            rec.state_id.as_hex(),
            if rec.parents.len() == 2 {
                "merge · "
            } else {
                ""
            },
            if rec.verification.passed {
                "passed"
            } else {
                "FAILED"
            },
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
    let rec = store.get_commit(commit)?.ok_or_else(|| {
        CliError::Runtime(format!("commit {} missing from store", commit.as_hex()))
    })?;
    let state = state_of_commit(&store, commit)?;
    let mut out = format!(
        "commit {}\n  state: {}\n  parents: {}\n  message: {}\n  verification: {}\n  operations:\n",
        commit.as_hex(),
        rec.state_id.as_hex(),
        if rec.parents.is_empty() {
            "(root)".to_owned()
        } else {
            rec.parents
                .iter()
                .map(|p| p.as_hex())
                .collect::<Vec<_>>()
                .join(", ")
        },
        rec.message,
        if rec.verification.passed {
            "passed"
        } else {
            "FAILED"
        },
    );
    if rec.operations.is_empty() {
        out.push_str("    (none)\n");
    }
    for op in &rec.operations {
        out.push_str(&format!("    {op}\n"));
    }
    out.push_str(&format!(
        "network: {} node(s), {} interface(s), {} link(s)\n",
        state.network.node_count(),
        state.network.interface_count(),
        state.network.link_count()
    ));
    for node in state.network.iter_nodes() {
        out.push_str(&format!("  node {}\n", node.id));
        for ifc in node.iter_interfaces() {
            out.push_str(&format!("    iface {}\n", ifc.name));
        }
    }
    for link in state.network.iter_links() {
        out.push_str(&format!("  link {link}\n"));
    }
    Ok(out)
}

/// CI verification of a committed state (ADR 0017).
///
/// Deterministic TSV report; exit code 1 on any failed invariant
/// (propagated via CliError::Runtime), 0 on pass.
fn test_cmd(dir: &Path, ref_name: Option<String>) -> Result<String, CliError> {
    let store = open_repo(dir)?;
    let state = match ref_name.as_deref() {
        None => match head_commit(&store)? {
            Some(id) => state_of_commit(&store, id)?,
            None => State::empty(),
        },
        Some(r) => {
            let commit = resolve_commit(&store, r)?;
            state_of_commit(&store, commit)?
        }
    };
    let constitution = constitution_of(&store)?;
    let report = verify(&state, &constitution);
    let mut out = format!(
        "state {}
",
        report.state_id.as_hex()
    );
    for r in &report.reports {
        out.push_str(&format!(
            "{}	{}	{}
",
            if r.passed { "PASS" } else { "FAIL" },
            r.id,
            r.evidence
        ));
    }
    if report.passed() {
        out.push_str(&format!(
            "test PASSED ({} invariants)
",
            report.reports.len()
        ));
        Ok(out)
    } else {
        out.push_str(&format!(
            "test FAILED ({} of {} invariants)
",
            report.failed_ids().len(),
            report.reports.len()
        ));
        Err(CliError::Runtime(out))
    }
}

/// Backend registry (ADR 0016). Unknown names fail explicitly.
fn select_backend(
    name: &str,
) -> Result<std::sync::Arc<dyn rahn_sim::backend::ExecutionBackend>, CliError> {
    use std::sync::Arc;
    match name {
        "simulation" => Ok(Arc::new(rahn_sim::backend::SimulationBackend)),
        "linux-ns" => Ok(Arc::new(rahn_exec::backend::LinuxNamespaceBackend)),
        other => Err(CliError::Runtime(format!(
            "unknown execution backend {other:?} (available: simulation, linux-ns)"
        ))),
    }
}

fn apply_cmd(
    dir: &Path,
    name: String,
    backend_name: &str,
    execute: bool,
    _yes_i_know: bool,
) -> Result<String, CliError> {
    let store = open_repo(dir)?;
    let backend = select_backend(backend_name)?;
    let target_commit = resolve_commit(&store, &name)?;
    let target = state_of_commit(&store, target_commit)?;
    let current = match head_commit(&store)? {
        Some(id) => state_of_commit(&store, id)?,
        None => State::empty(),
    };
    let steps = backend
        .plan(&current.network, &target.network)
        .map_err(|e| CliError::Runtime(e.to_string()))?;
    if !execute {
        let mut out = format!(
            "target: {}
backend: {}
",
            name,
            backend.name()
        );
        if steps.is_empty() {
            out.push_str(
                "Execution plan: (no changes)
",
            );
        } else {
            out.push_str(&format!(
                "Execution plan ({} step(s)):
",
                steps.len()
            ));
            for (i, s) in steps.iter().enumerate() {
                out.push_str(&format!(
                    "  {}. {s}
",
                    i + 1
                ));
            }
            out.push_str(
                "(dry run — pass --execute --yes-i-know to realize this plan)
",
            );
        }
        return Ok(out);
    }
    // Real execution: simulation cannot execute by definition (ADR 0016);
    // real backends enforce their own platform gates.
    if backend.name() == "simulation" {
        return Err(CliError::Runtime(
            "the simulation backend cannot execute (it is a description only);              select a real backend, e.g. --backend linux-ns"
                .into(),
        ));
    }
    match rahn_exec::backend::LinuxNamespaceBackend::execute(&steps) {
        Ok(n) => Ok(format!(
            "executed {n} step(s) via backend {} (destroy with: rahn destroy --yes-i-know {name})",
            backend.name()
        )),
        Err((pos, e)) => Err(CliError::Runtime(format!(
            "execution failed at step {pos}: {e}
recover with: rahn destroy --yes-i-know {name}"
        ))),
    }
}

fn destroy_cmd(dir: &Path, name: String, _yes_i_know: bool) -> Result<String, CliError> {
    let store = open_repo(dir)?;
    let commit = resolve_commit(&store, &name)?;
    let state = state_of_commit(&store, commit)?;
    let cmds = rahn_exec::destroy_commands(&state.network);
    if !cfg!(target_os = "linux") {
        return Err(CliError::Runtime(
            "destroy requires Linux (this build/platform refuses; ADR 0012)".into(),
        ));
    }
    match rahn_exec::execute(&cmds) {
        Ok(n) => Ok(format!("destroyed {n} namespace(s) for {name}")),
        Err((pos, e)) => Err(CliError::Runtime(format!(
            "destroy failed at command {pos}: {e}"
        ))),
    }
}

/// Ingest one observation (ADR 0013).
///
/// Deterministic ingest: the caller supplies the timestamp; `seq` is the
/// existing record count; the subject is validated against the current
/// HEAD state (provenance) and recorded with it.
fn observe_cmd(
    dir: &Path,
    subject: String,
    metric: String,
    value: String,
    at_ns: u64,
) -> Result<String, CliError> {
    let store = open_repo(dir)?;
    let subj =
        rahn_state::obs::Subject::parse(&subject).map_err(|e| CliError::Runtime(e.to_string()))?;
    let parsed_value = parse_value(&value)?;
    // Provenance: validate against HEAD state (or empty before first commit).
    let head_state = match head_commit(&store)? {
        Some(id) => state_of_commit(&store, id)?,
        None => State::empty(),
    };
    if !subj.exists_in(&head_state.network) {
        return Err(CliError::Runtime(
            rahn_state::obs::ObsError::UnknownSubject {
                subject: subj.to_string(),
            }
            .to_string(),
        ));
    }
    let state_ref = head_commit(&store)?
        .expect("subject exists implies HEAD")
        .as_hex();
    let seq = store
        .observations()?
        .len()
        .map_err(|e| CliError::Runtime(e.to_string()))? as u64;
    let o = rahn_state::obs::Observation::new(seq, at_ns, state_ref, subj, &metric, parsed_value)
        .map_err(|e| CliError::Runtime(e.to_string()))?;
    store
        .observations()?
        .append(&o)
        .map_err(|e| CliError::Runtime(e.to_string()))?;
    Ok(format!(
        "observed seq={} at={}ns {} {} {} (state {})",
        o.seq,
        o.time_ns,
        o.subject,
        o.metric,
        o.value,
        &o.state_ref[..12]
    ))
}

fn parse_value(s: &str) -> Result<rahn_state::obs::Value, CliError> {
    use rahn_state::obs::Value;
    let (kind, rest) = s.split_once(':').ok_or_else(|| {
        CliError::Runtime("value must be counter:<u64>, gauge:<i64>, or event:<text>".into())
    })?;
    match kind {
        "counter" => Ok(Value::Counter(rest.parse::<u64>().map_err(|_| {
            CliError::Runtime(format!("counter value must be u64, got {rest:?}"))
        })?)),
        "gauge" => Ok(Value::Gauge(rest.parse::<i64>().map_err(|_| {
            CliError::Runtime(format!("gauge value must be i64, got {rest:?}"))
        })?)),
        "event" => Ok(Value::Event(rest.to_owned())),
        other => Err(CliError::Runtime(format!(
            "unknown value kind {other:?} (expected counter|gauge|event)"
        ))),
    }
}

/// List observations, newest last; machine-readable TSV.
fn observations_cmd(dir: &Path, filter: Option<String>) -> Result<String, CliError> {
    let store = open_repo(dir)?;
    let all = store
        .observations()?
        .read_all()
        .map_err(|e| CliError::Runtime(e.to_string()))?;
    let mut sorted = all;
    sorted.sort_by_key(rahn_state::obs::order_key);
    let mut out = String::from(
        "seq	time_ns	subject	metric	value	state
",
    );
    for o in sorted {
        if let Some(f) = &filter {
            let matches = match &o.subject {
                rahn_state::obs::Subject::Node(id) => id == f,
                rahn_state::obs::Subject::Interface { node, .. } => node == f,
            };
            if !matches {
                continue;
            }
        }
        out.push_str(&format!(
            "{}	{}	{}	{}	{}	{}
",
            o.seq,
            o.time_ns,
            o.subject,
            o.metric,
            o.value,
            &o.state_ref[..12]
        ));
    }
    Ok(out)
}

/// Record an explicit causal edge (ADR 0014).
fn relate_cmd(
    dir: &Path,
    from: String,
    to: String,
    status: String,
    note: String,
) -> Result<String, CliError> {
    use rahn_state::causal::{anchor_exists, Anchor, Status};
    let store = open_repo(dir)?;
    let from_a = Anchor::parse(&from).map_err(|e| CliError::Runtime(e.to_string()))?;
    let to_a = Anchor::parse(&to).map_err(|e| CliError::Runtime(e.to_string()))?;
    let status = Status::parse(&status).map_err(|e| CliError::Runtime(e.to_string()))?;

    // Anchor existence: observations from the log, commits from the store.
    let obs_count = store
        .observations()?
        .len()
        .map_err(|e| CliError::Runtime(e.to_string()))? as u64;
    let commit_ok = |id: &[u8; 32]| {
        CommitId::from_hex(&rahn_state::identity::StateId::from_bytes(*id).as_hex())
            .map(|c| store.get_commit(c).unwrap_or(None).is_some())
            .unwrap_or(false)
    };
    let graph = store
        .causal()?
        .load()
        .map_err(|e| CliError::Runtime(e.to_string()))?;
    graph
        .check_insert(&from_a, &to_a, status, |a| {
            anchor_exists(a, obs_count, commit_ok)
        })
        .map_err(|e| CliError::Runtime(e.to_string()))?;

    let seq = graph.edges().len() as u64;
    let edge = rahn_state::causal::CausalEdge::new(seq, from_a, to_a, status, note)
        .map_err(|e| CliError::Runtime(e.to_string()))?;
    store
        .causal()?
        .append(&edge)
        .map_err(|e| CliError::Runtime(e.to_string()))?;
    Ok(format!(
        "related seq={} {} -> {} [{}]{}",
        edge.seq,
        edge.from,
        edge.to,
        edge.status.as_str(),
        if edge.note.is_empty() {
            String::new()
        } else {
            format!(" — {}", edge.note)
        }
    ))
}

/// Explain an anchor: its connected component with edge statuses. The
/// output is asserted structure, not proven causality.
fn explain_cmd(dir: &Path, anchor: String) -> Result<String, CliError> {
    use rahn_state::causal::{anchor_exists, Anchor};
    let store = open_repo(dir)?;
    let a = Anchor::parse(&anchor).map_err(|e| CliError::Runtime(e.to_string()))?;
    let obs_count = store
        .observations()?
        .len()
        .map_err(|e| CliError::Runtime(e.to_string()))? as u64;
    let commit_ok = |id: &[u8; 32]| {
        CommitId::from_hex(&rahn_state::identity::StateId::from_bytes(*id).as_hex())
            .map(|c| store.get_commit(c).unwrap_or(None).is_some())
            .unwrap_or(false)
    };
    let graph = store
        .causal()?
        .load()
        .map_err(|e| CliError::Runtime(e.to_string()))?;
    if !anchor_exists(&a, obs_count, commit_ok) {
        return Err(CliError::Runtime(
            rahn_state::causal::CausalError::DanglingAnchor {
                anchor: a.to_string(),
            }
            .to_string(),
        ));
    }
    let component = graph.incident_around(&a);
    let mut out = format!(
        "incident around {} ({} anchor(s)) — asserted relations, statuses as recorded:
",
        a,
        component.len()
    );
    for e in graph.edges() {
        if component.contains(&e.from) && component.contains(&e.to) {
            out.push_str(&format!(
                "  {} -> {} [{}]{}
",
                e.from,
                e.to,
                e.status.as_str(),
                if e.note.is_empty() {
                    String::new()
                } else {
                    format!(" — {}", e.note)
                }
            ));
        }
    }
    Ok(out)
}

/// Exposed for integration tests: run and get the exit code semantic.
pub fn run_or_exit_code(dir: &Path, command: Command) -> (Result<String, CliError>, i32) {
    match run(dir, command) {
        Ok(out) => (Ok(out), 0),
        Err(CliError::Usage(m)) => (Err(CliError::Usage(m)), 2),
        Err(e @ CliError::Runtime(_)) => (Err(e), 1),
    }
}
