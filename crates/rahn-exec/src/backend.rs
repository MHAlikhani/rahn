// SPDX-License-Identifier: Apache-2.0

//! Linux namespace backend behind the ADR 0016 trait. Delegates to the
//! ADR 0012 mapping; enforces the Linux platform gate inside the backend.

use rahn_core::Network;
use rahn_sim::backend::{BackendError, BackendStep, Capabilities, ExecutionBackend};

use crate::{commands_for, IpCommand};

/// Isolated Linux network namespaces via iproute2 (ADR 0012).
pub struct LinuxNamespaceBackend;

/// Render an argv command as an `Invoke` step.
fn invoke(cmd: &IpCommand) -> BackendStep {
    let argv = cmd.argv();
    BackendStep::Invoke {
        program: argv[0].clone(),
        args: argv[1..].to_vec(),
    }
}

impl ExecutionBackend for LinuxNamespaceBackend {
    fn name(&self) -> &'static str {
        "linux-ns"
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            link_topology: true,
            addressing: false,
            traffic_control: false,
        }
    }

    fn plan(&self, current: &Network, target: &Network) -> Result<Vec<BackendStep>, BackendError> {
        // ADR 0012 mapping is pure and host-safety-asserted; the platform
        // gate applies to execution, and also surfaces here so a dry run on
        // a non-Linux host names the constraint honestly.
        let cmds = commands_for(current, target).map_err(|e| BackendError::Plan(e.to_string()))?;
        Ok(cmds.iter().map(invoke).collect())
    }
}

impl LinuxNamespaceBackend {
    /// Execute previously planned steps. Linux + root required (ADR 0012).
    pub fn execute(steps: &[BackendStep]) -> Result<usize, (usize, String)> {
        if !cfg!(target_os = "linux") {
            return Err((0, "execution requires Linux (ADR 0012)".to_owned()));
        }
        for (i, step) in steps.iter().enumerate() {
            let (program, args) = match step {
                BackendStep::Invoke { program, args } => (program, args),
                BackendStep::Describe(_) => {
                    return Err((i + 1, "simulation step reached during execution".into()))
                }
            };
            let out = std::process::Command::new(program)
                .args(args)
                .output()
                .map_err(|e| (i + 1, e.to_string()))?;
            if !out.status.success() {
                return Err((
                    i + 1,
                    format!(
                        "command {} failed ({} {}): {}{}",
                        i + 1,
                        program,
                        args.join(" "),
                        String::from_utf8_lossy(&out.stdout).trim(),
                        String::from_utf8_lossy(&out.stderr).trim()
                    ),
                ));
            }
        }
        Ok(steps.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rahn_core::{Endpoint, Metadata};

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

    #[test]
    fn linux_backend_yields_invoke_steps_deterministically() {
        let empty = Network::empty();
        let b = LinuxNamespaceBackend;
        assert_eq!(b.name(), "linux-ns");
        let caps = b.capabilities();
        assert!(caps.link_topology);
        assert!(!caps.addressing && !caps.traffic_control);
        let steps = b.plan(&empty, &topo()).unwrap();
        assert_eq!(steps.len(), 12); // same count as the ADR 0012 mapping
        assert!(steps
            .iter()
            .all(|s| matches!(s, BackendStep::Invoke { program, .. } if program == "ip")));
        assert_eq!(
            steps[0],
            BackendStep::Invoke {
                program: "ip".into(),
                args: vec!["netns".into(), "add".into(), "rahn-a".into()],
            }
        );
        assert_eq!(b.plan(&empty, &topo()).unwrap(), steps);
    }

    #[test]
    fn execution_refuses_on_non_linux() {
        if cfg!(target_os = "linux") {
            return;
        }
        let steps = LinuxNamespaceBackend
            .plan(&Network::empty(), &topo())
            .unwrap();
        let err = LinuxNamespaceBackend::execute(&steps).unwrap_err();
        assert_eq!(err.0, 0);
        assert!(err.1.contains("Linux"));
    }
}
