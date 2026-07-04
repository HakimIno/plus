use std::process::ExitCode;

use anyhow::Result;

use crate::cli::ExplainArgs;

pub fn run(args: ExplainArgs) -> Result<ExitCode> {
    match args.topic.as_str() {
        "size" => explain_size(),
        "build" => explain_build(),
        "tools" => explain_tools(),
        _ => unreachable!(),
    }
    Ok(ExitCode::SUCCESS)
}

fn explain_size() {
    println!("Why target/ gets large:");
    println!("  debug/deps        compiled dependency artifacts");
    println!("  debug/build       build.rs outputs and generated code");
    println!("  debug/incremental incremental compiler cache");
    println!();
    println!("What to do:");
    println!("  plus size --deep          inspect large buckets");
    println!("  plus clean                preview safe cleanup");
    println!("  plus clean --apply        remove low-risk artifacts");
    println!("  plus clean --nuclear      run cargo clean and rebuild everything later");
}

fn explain_build() {
    println!("Rust build speed depends on:");
    println!("  1. how much code changed");
    println!("  2. dependency graph size");
    println!("  3. proc macros and build scripts");
    println!("  4. linker speed");
    println!("  5. compiler cache availability");
    println!();
    println!("Useful commands:");
    println!("  plus dev                  fastest feedback loop available");
    println!("  plus profile check        time cargo check");
    println!("  cargo build --timings     detailed Cargo timing report");
}

fn explain_tools() {
    println!("Tool roles:");
    println!("  Cargo        build engine and package manager");
    println!("  plus         diagnosis, setup, cleanup, and workflow launcher");
    println!("  sccache      compiler output cache");
    println!("  mold/lld     faster linker");
    println!("  bacon        background checker");
    println!("  nextest      faster test runner");
}
