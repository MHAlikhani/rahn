// SPDX-License-Identifier: Apache-2.0

//! Library facade for the RAHN CLI (testable without subprocesses).

pub mod args;
pub mod commands;

pub use args::{parse, Command, USAGE};
pub use commands::{run, CliError, REPO_DIR};
