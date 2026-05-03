// SPDX-License-Identifier: Apache-2.0

//! Argument parsing for the `rahn` CLI. Hand-rolled and strict: unknown
//! flags, missing arguments, and malformed metadata are usage errors.
//! No external parser dependency — the v0.2 surface is deliberately small.

/// A parsed CLI command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Init,
    NodeAdd {
        id: String,
        metadata: Vec<(String, String)>,
    },
    NodeRemove {
        id: String,
    },
    InterfaceAdd {
        node: String,
        name: String,
    },
    InterfaceRemove {
        node: String,
        name: String,
    },
    LinkAdd {
        a: String,
        b: String,
    },
    LinkRemove {
        a: String,
        b: String,
    },
    Commit {
        message: String,
    },
    State,
    BranchList,
    BranchCreate {
        name: String,
    },
    Checkout {
        name: String,
        force: bool,
    },
    Diff {
        from: String,
        to: String,
    },
    Merge {
        branch: String,
    },
    Path {
        from: String,
        to: String,
    },
    Verify,
    Log,
    Inspect {
        name: String,
    },
    Apply {
        name: String,
    },
}

pub const USAGE: &str = r#"rahn — a stateful execution architecture for evolving networks (v0.2, simulation-only)

USAGE:
    rahn init
    rahn node add <id> [key=value ...]
    rahn node remove <id>
    rahn interface add <node> <name>
    rahn interface remove <node> <name>
    rahn link add <node/iface> <node/iface>
    rahn link remove <node/iface> <node/iface>
    rahn commit -m <message>
    rahn state
    rahn branch [<name>]
    rahn checkout [--force] <branch>
    rahn diff <from-ref> <to-ref>
    rahn merge <branch>
    rahn path <from-node> <to-node>
    rahn verify
    rahn log
    rahn inspect <branch-or-commit-id>
    rahn apply <branch-or-commit-id>

REFS: a branch name or a full 64-character commit id.
ENDPOINTS: node/interface pairs (e.g. web/eth0).

v0.2 performs NO real execution: `apply` prints an execution plan only."#;

/// Parse raw arguments (without the program name).
pub fn parse(args: &[String]) -> Result<Command, String> {
    let mut it = args.iter();
    let sub = it
        .next()
        .ok_or_else(|| format!("missing subcommand\n\n{USAGE}"))?;
    match sub.as_str() {
        "init" => {
            expect_end(&mut it, "init")?;
            Ok(Command::Init)
        }
        "node" => {
            let action = next(&mut it, "node <add|remove>")?;
            match action.as_str() {
                "add" => {
                    let id = next(&mut it, "node add <id>")?;
                    let mut metadata = Vec::new();
                    for arg in &mut it {
                        metadata.push(parse_kv(arg)?);
                    }
                    Ok(Command::NodeAdd { id, metadata })
                }
                "remove" => {
                    let id = next(&mut it, "node remove <id>")?;
                    expect_end(&mut it, "node remove")?;
                    Ok(Command::NodeRemove { id })
                }
                other => Err(format!("unknown node action {other:?}\n\n{USAGE}")),
            }
        }
        "interface" | "iface" => {
            let action = next(&mut it, "interface <add|remove>")?;
            match action.as_str() {
                "add" => {
                    let node = next(&mut it, "interface add <node> <name>")?;
                    let name = next(&mut it, "interface add <node> <name>")?;
                    expect_end(&mut it, "interface add")?;
                    Ok(Command::InterfaceAdd { node, name })
                }
                "remove" => {
                    let node = next(&mut it, "interface remove <node> <name>")?;
                    let name = next(&mut it, "interface remove <node> <name>")?;
                    expect_end(&mut it, "interface remove")?;
                    Ok(Command::InterfaceRemove { node, name })
                }
                other => Err(format!("unknown interface action {other:?}\n\n{USAGE}")),
            }
        }
        "link" => {
            let action = next(&mut it, "link <add|remove>")?;
            match action.as_str() {
                "add" => {
                    let a = next(&mut it, "link add <node/iface> <node/iface>")?;
                    let b = next(&mut it, "link add <node/iface> <node/iface>")?;
                    expect_end(&mut it, "link add")?;
                    Ok(Command::LinkAdd { a, b })
                }
                "remove" => {
                    let a = next(&mut it, "link remove <node/iface> <node/iface>")?;
                    let b = next(&mut it, "link remove <node/iface> <node/iface>")?;
                    expect_end(&mut it, "link remove")?;
                    Ok(Command::LinkRemove { a, b })
                }
                other => Err(format!("unknown link action {other:?}\n\n{USAGE}")),
            }
        }
        "commit" => {
            let mut message = None;
            while let Some(arg) = it.next() {
                if arg == "-m" {
                    if message.is_some() {
                        return Err(format!("-m given twice\n\n{USAGE}"));
                    }
                    message = Some(next(&mut it, "commit -m <message>")?);
                } else {
                    return Err(format!("unknown commit argument {arg:?}\n\n{USAGE}"));
                }
            }
            match message {
                Some(m) => Ok(Command::Commit { message: m }),
                None => Err(format!("commit requires -m <message>\n\n{USAGE}")),
            }
        }
        "state" => {
            expect_end(&mut it, "state")?;
            Ok(Command::State)
        }
        "branch" => match it.next() {
            None => Ok(Command::BranchList),
            Some(name) => {
                expect_end(&mut it, "branch")?;
                Ok(Command::BranchCreate { name: name.clone() })
            }
        },
        "checkout" => {
            let mut force = false;
            let mut name = None;
            for arg in &mut it {
                match arg.as_str() {
                    "--force" | "-f" => {
                        if force {
                            return Err(format!("--force given twice\n\n{USAGE}"));
                        }
                        force = true;
                    }
                    other if name.is_none() => name = Some(other.to_owned()),
                    _ => {
                        return Err(format!(
                            "unexpected argument {arg:?} for checkout\n\n{USAGE}"
                        ))
                    }
                }
            }
            let name =
                name.ok_or_else(|| format!("missing argument for checkout <branch>\n\n{USAGE}"))?;
            Ok(Command::Checkout { name, force })
        }
        "diff" => {
            let from = next(&mut it, "diff <from-ref> <to-ref>")?;
            let to = next(&mut it, "diff <from-ref> <to-ref>")?;
            expect_end(&mut it, "diff")?;
            Ok(Command::Diff { from, to })
        }
        "merge" => {
            let branch = next(&mut it, "merge <branch>")?;
            expect_end(&mut it, "merge")?;
            Ok(Command::Merge { branch })
        }
        "path" => {
            let from = next(&mut it, "path <from-node> <to-node>")?;
            let to = next(&mut it, "path <from-node> <to-node>")?;
            expect_end(&mut it, "path")?;
            Ok(Command::Path { from, to })
        }
        "verify" => {
            expect_end(&mut it, "verify")?;
            Ok(Command::Verify)
        }
        "log" => {
            expect_end(&mut it, "log")?;
            Ok(Command::Log)
        }
        "inspect" => {
            let name = next(&mut it, "inspect <branch-or-commit-id>")?;
            expect_end(&mut it, "inspect")?;
            Ok(Command::Inspect { name })
        }
        "apply" => {
            let name = next(&mut it, "apply <branch-or-commit-id>")?;
            expect_end(&mut it, "apply")?;
            Ok(Command::Apply { name })
        }
        other => Err(format!("unknown subcommand {other:?}\n\n{USAGE}")),
    }
}

fn next(it: &mut std::slice::Iter<'_, String>, what: &str) -> Result<String, String> {
    it.next()
        .map(|s| s.to_owned())
        .ok_or_else(|| format!("missing argument for {what}\n\n{USAGE}"))
}

fn expect_end(it: &mut std::slice::Iter<'_, String>, sub: &str) -> Result<(), String> {
    match it.next() {
        Some(extra) => Err(format!(
            "unexpected argument {extra:?} for subcommand {sub:?}\n\n{USAGE}"
        )),
        None => Ok(()),
    }
}

fn parse_kv(arg: &str) -> Result<(String, String), String> {
    let (k, v) = arg
        .split_once('=')
        .ok_or_else(|| format!("metadata must be key=value (got {arg:?})\n\n{USAGE}"))?;
    if k.is_empty() || v.is_empty() {
        return Err(format!(
            "metadata key and value must be non-empty (got {arg:?})\n\n{USAGE}"
        ));
    }
    Ok((k.to_owned(), v.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parses_all_subcommands() {
        assert_eq!(parse(&args(&["init"])).unwrap(), Command::Init);
        assert_eq!(
            parse(&args(&["node", "add", "web", "role=api", "zone=a"])).unwrap(),
            Command::NodeAdd {
                id: "web".into(),
                metadata: vec![("role".into(), "api".into()), ("zone".into(), "a".into())]
            }
        );
        assert_eq!(
            parse(&args(&["interface", "add", "web", "eth0"])).unwrap(),
            Command::InterfaceAdd {
                node: "web".into(),
                name: "eth0".into()
            }
        );
        assert_eq!(
            parse(&args(&["link", "add", "a/eth0", "b/eth0"])).unwrap(),
            Command::LinkAdd {
                a: "a/eth0".into(),
                b: "b/eth0".into()
            }
        );
        assert_eq!(
            parse(&args(&["commit", "-m", "msg"])).unwrap(),
            Command::Commit {
                message: "msg".into()
            }
        );
        assert_eq!(parse(&args(&["branch"])).unwrap(), Command::BranchList);
        assert_eq!(
            parse(&args(&["branch", "exp"])).unwrap(),
            Command::BranchCreate { name: "exp".into() }
        );
        assert_eq!(
            parse(&args(&["checkout", "exp"])).unwrap(),
            Command::Checkout {
                name: "exp".into(),
                force: false
            }
        );
        assert_eq!(
            parse(&args(&["checkout", "-f", "exp"])).unwrap(),
            Command::Checkout {
                name: "exp".into(),
                force: true
            }
        );
        assert_eq!(
            parse(&args(&["path", "a", "b"])).unwrap(),
            Command::Path {
                from: "a".into(),
                to: "b".into()
            }
        );
    }

    #[test]
    fn rejects_bad_input() {
        assert!(parse(&args(&[])).is_err());
        assert!(parse(&args(&["frobnicate"])).is_err());
        assert!(parse(&args(&["node", "add"])).is_err());
        assert!(parse(&args(&["node", "add", "web", "noequals"])).is_err());
        assert!(parse(&args(&["commit"])).is_err());
        assert!(parse(&args(&["link", "add", "a"])).is_err());
        assert!(parse(&args(&["init", "extra"])).is_err());
        assert!(parse(&args(&["commit", "-m", "a", "-m", "b"])).is_err());
        assert!(parse(&args(&["checkout", "--force"])).is_err());
    }
}
