// SPDX-License-Identifier: Apache-2.0

use rahn_cli::{parse, run};

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let command = match parse(&argv) {
        Ok(c) => c,
        Err(usage) => {
            eprintln!("{usage}");
            std::process::exit(2);
        }
    };
    let cwd = std::env::current_dir().expect("current directory");
    match run(&cwd, command) {
        Ok(out) => {
            println!("{out}");
        }
        Err(rahn_cli::CliError::Usage(msg)) => {
            eprintln!("{msg}");
            std::process::exit(2);
        }
        Err(rahn_cli::CliError::Runtime(msg)) => {
            eprintln!("error: {msg}");
            std::process::exit(1);
        }
    }
}
