use std::process::ExitCode;

use anyhow::Result;

use crate::{cargo, cli::TestArgs, tools};

pub fn run(args: TestArgs) -> Result<ExitCode> {
    if args.fast || tools::exists("cargo-nextest") {
        if tools::exists("cargo-nextest") {
            return cargo::run(&["nextest", "run"], &args.args);
        }
        eprintln!("note: cargo-nextest not found; falling back to `cargo test`.");
    }
    cargo::run(&["test"], &args.args)
}
