// SPDX-License-Identifier: Apache-2.0

//! Isolated Linux namespace execution backend (ADR 0012).
//!
//! This crate maps an [`ExecutionPlan`] to an ordered list of `ip`
//! (iproute2) argv vectors. The mapping is pure and deterministic — it
//! performs no I/O and is testable on any OS.
//!
//! Host-safety invariant (structurally tested): the only host-side
//! operations are `ip netns add/del rahn-*`; every link and interface
//! mutation runs inside `ip netns exec rahn-*`. Execution itself is
//! opt-in and refuses to run on non-Linux platforms.

use std::fmt;

use rahn_core::{Endpoint, Network};
use rahn_sim::Action;
use sha2::{Digest, Sha256};

/// Deterministic, IFNAMSIZ-safe interface name for a link endpoint:
/// `r` + 7 lowercase hex chars of SHA-256("node/iface").
pub fn veth_name(e: &Endpoint) -> String {
    let mut h = Sha256::new();
    h.update(e.to_string().as_bytes());
    let d = h.finalize();
    let hex: String = d.iter().map(|b| format!("{b:02x}")).collect();
    format!("r{}", &hex[..7])
}

/// Namespace name for a node: `rahn-<node-id>`.
pub fn ns_name(node: &str) -> String {
    format!("rahn-{node}")
}

/// Why execution is refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecError {
    /// Interface name exceeds IFNAMSIZ-1 (15 chars).
    InterfaceNameTooLong(String),
}

impl fmt::Display for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecError::InterfaceNameTooLong(name) => write!(
                f,
                "interface name {name:?} exceeds 15 characters (kernel IFNAMSIZ limit)"
            ),
        }
    }
}

impl std::error::Error for ExecError {}

/// One command to run: `argv[0]` is always `ip`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpCommand {
    pub args: Vec<String>,
}

impl IpCommand {
    /// `args` excludes the program name; `ip` is prepended so `argv()`
    /// always starts with it.
    fn new(args: &[&str]) -> Self {
        let mut argv = vec!["ip".to_string()];
        argv.extend(args.iter().map(|s| s.to_string()));
        IpCommand { args: argv }
    }

    pub fn program(&self) -> &str {
        "ip"
    }

    pub fn argv(&self) -> &[String] {
        &self.args
    }
}

impl fmt::Display for IpCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.args.join(" "))
    }
}

fn check_iface_name(node: &str, name: &str) -> Result<(), ExecError> {
    if name.len() > 15 {
        return Err(ExecError::InterfaceNameTooLong(format!("{node}/{name}")));
    }
    Ok(())
}

/// Map a plan (the diff `current -> target`) to namespace commands.
///
/// Removals first (links, interfaces, nodes), then additions (nodes,
/// interfaces, links) — the same dependency-safe ordering as the plan.
pub fn commands_for(current: &Network, target: &Network) -> Result<Vec<IpCommand>, ExecError> {
    // Validate interface-name lengths up front (fail before touching anything).
    for node in target.iter_nodes() {
        for ifc in node.iter_interfaces() {
            check_iface_name(&node.id, &ifc.name)?;
        }
    }
    let plan = rahn_sim::plan(current, target);
    let mut cmds = Vec::new();
    for action in &plan.actions {
        match action {
            Action::RemoveLink { a, b: _ } => {
                // Deleting either end destroys the pair.
                cmds.push(IpCommand::new(&[
                    "-n",
                    &ns_name(&a.node),
                    "link",
                    "del",
                    &veth_name(a),
                ]));
            }
            Action::RemoveInterface { node, name } => {
                cmds.push(IpCommand::new(&["-n", &ns_name(node), "link", "del", name]));
            }
            Action::RemoveNode { id } => {
                cmds.push(IpCommand::new(&["netns", "del", &ns_name(id)]));
            }
            Action::CreateNode { id } => {
                cmds.push(IpCommand::new(&["netns", "add", &ns_name(id)]));
                cmds.push(IpCommand::new(&[
                    "-n",
                    &ns_name(id),
                    "link",
                    "set",
                    "lo",
                    "up",
                ]));
            }
            Action::CreateInterface { node, name } => {
                cmds.push(IpCommand::new(&[
                    "-n",
                    &ns_name(node),
                    "link",
                    "add",
                    name,
                    "type",
                    "dummy",
                ]));
                cmds.push(IpCommand::new(&[
                    "-n",
                    &ns_name(node),
                    "link",
                    "set",
                    name,
                    "up",
                ]));
            }
            Action::CreateLink { a, b } => {
                let va = veth_name(a);
                let vb = veth_name(b);
                // Create the pair inside namespace A, move the peer to B.
                cmds.push(IpCommand::new(&[
                    "-n",
                    &ns_name(&a.node),
                    "link",
                    "add",
                    &va,
                    "type",
                    "veth",
                    "peer",
                    "name",
                    &vb,
                ]));
                cmds.push(IpCommand::new(&[
                    "-n",
                    &ns_name(&a.node),
                    "link",
                    "set",
                    &vb,
                    "netns",
                    &ns_name(&b.node),
                ]));
                cmds.push(IpCommand::new(&[
                    "-n",
                    &ns_name(&a.node),
                    "link",
                    "set",
                    &va,
                    "up",
                ]));
                cmds.push(IpCommand::new(&[
                    "-n",
                    &ns_name(&b.node),
                    "link",
                    "set",
                    &vb,
                    "up",
                ]));
            }
        }
    }
    Ok(cmds)
}

/// Destroy commands for every node of a state (veths die with namespaces).
pub fn destroy_commands(net: &Network) -> Vec<IpCommand> {
    net.iter_nodes()
        .map(|n| IpCommand::new(&["netns", "del", &ns_name(&n.id)]))
        .collect()
}

/// Structural host-safety check (ADR 0012, decision 5): every command must
/// be namespace-scoped. The only host-side argv allowed is
/// `ip netns add|del rahn-*`.
pub fn assert_host_safety(cmds: &[IpCommand]) {
    for c in cmds {
        let a = c.argv();
        assert_eq!(a[0], "ip");
        match (a.get(1).map(String::as_str), a.get(2).map(String::as_str)) {
            (Some("netns"), Some(op @ ("add" | "del"))) => {
                let name = a.get(3).expect("netns op needs a name");
                assert!(name.starts_with("rahn-"), "host netns op on {name:?}");
                let _ = op;
            }
            (Some("-n"), Some(ns)) => {
                assert!(ns.starts_with("rahn-"), "scoped op on non-rahn ns {ns:?}");
            }
            other => panic!("host-mutating command rejected: {other:?} — {:?}", c.argv()),
        }
    }
}

/// Execute commands sequentially via `ip`. Linux + root required.
/// Stops at the first failure, returning its 1-based position.
pub fn execute(cmds: &[IpCommand]) -> Result<usize, (usize, String)> {
    if !cfg!(target_os = "linux") {
        return Err((0, "execution requires Linux (ADR 0012)".to_owned()));
    }
    for (i, c) in cmds.iter().enumerate() {
        // argv[0] is the display program name; the real program is `ip`.
        let mut cmd = std::process::Command::new(c.program());
        cmd.args(&c.argv()[1..]);
        let out = cmd.output().map_err(|e| (i + 1, e.to_string()))?;
        if !out.status.success() {
            return Err((
                i + 1,
                format!(
                    "command {} failed: {}: {}{}",
                    i + 1,
                    c,
                    String::from_utf8_lossy(&out.stdout).trim(),
                    String::from_utf8_lossy(&out.stderr).trim()
                ),
            ));
        }
    }
    Ok(cmds.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rahn_core::Metadata;

    fn net(nodes: &[&str], links: &[(&str, &str, &str, &str)]) -> Network {
        let mut n = Network::empty();
        for id in nodes {
            n.add_node(id, Metadata::new()).unwrap();
            n.add_interface(id, "eth0").unwrap();
        }
        for (an, ai, bn, bi) in links {
            n.add_link(
                Endpoint::new(an, ai).unwrap(),
                Endpoint::new(bn, bi).unwrap(),
            )
            .unwrap();
        }
        n
    }

    #[test]
    fn veth_names_are_short_deterministic_and_distinct() {
        let a = Endpoint::new("web", "eth0").unwrap();
        let b = Endpoint::new("db", "eth0").unwrap();
        let (na, nb) = (veth_name(&a), veth_name(&b));
        assert_eq!(na, veth_name(&a), "deterministic");
        assert_ne!(na, nb);
        assert_eq!(na.len(), 8, "r + 7 hex, IFNAMSIZ-safe");
        assert!(na.starts_with('r') && na[1..].chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn mapping_is_pure_and_ordered() {
        let empty = Network::empty();
        let topo = net(&["a", "b"], &[("a", "eth0", "b", "eth0")]);
        let cmds = commands_for(&empty, &topo).unwrap();
        // 2 nodes (netns add + lo up) + 2 interfaces (dummy add + up)
        // + 1 link (veth add, move, up, up) = 12 commands.
        assert_eq!(cmds.len(), 12);
        assert_eq!(cmds[0].argv(), ["ip", "netns", "add", "rahn-a"]);
        assert_eq!(
            cmds[1].argv(),
            ["ip", "-n", "rahn-a", "link", "set", "lo", "up"]
        );
        assert_eq!(
            cmds[4].argv()[..6],
            ["ip", "-n", "rahn-a", "link", "add", "eth0"]
        );
        assert_eq!(&cmds[4].argv()[6..], ["type", "dummy"]);
        let rem = commands_for(&topo, &empty).unwrap();
        // link del first, then netns dels
        assert_eq!(rem.len(), 3);
        assert_eq!(rem[0].argv()[1..3], ["-n", "rahn-a"]);
        assert_eq!(rem[1].argv(), ["ip", "netns", "del", "rahn-a"]);
        assert_eq!(rem[2].argv(), ["ip", "netns", "del", "rahn-b"]);
        // Purity: same inputs, same commands.
        assert_eq!(commands_for(&empty, &topo).unwrap(), cmds);
    }

    #[test]
    fn every_command_is_namespace_scoped() {
        let empty = Network::empty();
        let topo = net(
            &["a", "b", "c"],
            &[("a", "eth0", "b", "eth0"), ("b", "eth0", "c", "eth0")],
        );
        for (from, to) in [(&empty, &topo), (&topo, &empty)] {
            let cmds = commands_for(from, to).unwrap();
            assert_host_safety(&cmds); // panics on any host mutation
        }
    }

    #[test]
    fn long_interface_names_are_refused_before_any_command() {
        let mut n = Network::empty();
        n.add_node("a", Metadata::new()).unwrap();
        let err = check_iface_name("a", "an-extremely-long-interface-name");
        assert!(matches!(err, Err(ExecError::InterfaceNameTooLong(_))));
    }

    #[test]
    fn execution_refuses_on_non_linux() {
        if cfg!(target_os = "linux") {
            return; // real execution is tested separately on Linux
        }
        let cmds = vec![IpCommand::new(&["netns", "add", "rahn-x"])];
        let err = execute(&cmds).unwrap_err();
        assert_eq!(err.0, 0);
        assert!(err.1.contains("Linux"));
    }

    #[test]
    fn destroy_targets_all_nodes() {
        let topo = net(&["a", "b"], &[]);
        let cmds = destroy_commands(&topo);
        assert_eq!(cmds.len(), 2);
        assert_host_safety(&cmds);
    }
}
