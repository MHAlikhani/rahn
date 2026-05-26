// SPDX-License-Identifier: Apache-2.0

//! Real namespace execution. Requires Linux + root (CAP_SYS_ADMIN):
//! run with `sudo cargo test -p rahn-exec --test linux_real -- --ignored`
//! (CI runs this on ubuntu with sudo; skipped everywhere else).

#![cfg(target_os = "linux")]

use rahn_core::{Endpoint, Metadata, Network};
use rahn_exec::{commands_for, destroy_commands, execute};

fn topo() -> Network {
    let mut n = Network::empty();
    for id in ["a", "b"] {
        n.add_node(id, Metadata::new()).unwrap();
        n.add_interface(id, "eth0").unwrap();
    }
    n.add_link(
        Endpoint::new("a", "eth0").unwrap(),
        Endpoint::new("b", "eth0").unwrap(),
    )
    .unwrap();
    n
}

fn netns_list() -> String {
    let out = std::process::Command::new("ip")
        .args(["netns", "list"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
#[ignore]
fn instantiate_and_destroy_isolated_topology() {
    let empty = Network::empty();
    let t = topo();

    // Nothing pre-existing under our names.
    let _ = execute(&destroy_commands(&t));

    // Instantiate.
    let cmds = commands_for(&empty, &t).unwrap();
    let ran = execute(&cmds).unwrap_or_else(|(i, e)| panic!("cmd {i}: {e}"));
    assert_eq!(ran, cmds.len());
    let list = netns_list();
    assert!(list.contains("rahn-a"), "{list}");
    assert!(list.contains("rahn-b"), "{list}");

    // Host safety: both test networks were namespace-scoped by construction;
    // assert the structural invariant over what we ran.
    rahn_exec::assert_host_safety(&cmds);

    // Link exists inside namespace a.
    let out = std::process::Command::new("ip")
        .args(["-n", "rahn-a", "link", "show"])
        .output()
        .unwrap();
    let links = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(links.contains("lo"), "{links}");
    assert!(
        links.contains(&rahn_exec::veth_name(&Endpoint::new("a", "eth0").unwrap())),
        "{links}"
    );

    // Destroy; namespaces are gone.
    let dcmds = destroy_commands(&t);
    execute(&dcmds).unwrap();
    let list = netns_list();
    assert!(!list.contains("rahn-a"), "{list}");
    assert!(!list.contains("rahn-b"), "{list}");
}
