use std::{
    process::{Command, ExitCode},
    time::Instant,
};

use anyhow::Result;

use crate::{cargo, cli::DevArgs, config::PlusConfig, project::Project, tools};

pub fn run(_project: &Project, config: &PlusConfig, args: DevArgs) -> Result<ExitCode> {
    let started = Instant::now();
    let prefer = config.dev.as_ref().and_then(|dev| dev.prefer.as_deref());
    let code = if args.run {
        watch_or_cargo("run", &args.args)?
    } else if args.test {
        if tools::exists("bacon") {
            let mut command = Command::new("bacon");
            command.arg("test").args(args.args);
            cargo::run_command(command)?
        } else {
            watch_or_cargo("test", &args.args)?
        }
    } else if prefer == Some("cargo-watch") {
        watch_or_cargo("check", &args.args)?
    } else if tools::exists("bacon") {
        let mut command = Command::new("bacon");
        command.args(args.args);
        cargo::run_command(command)?
    } else {
        watch_or_cargo("check", &args.args)?
    };
    println!("duration: {:.2?}", started.elapsed());
    Ok(code)
}

fn watch_or_cargo(job: &str, args: &[String]) -> Result<ExitCode> {
    if tools::exists("cargo-watch") {
        let mut command = Command::new("cargo");
        command.arg("watch").arg("-x");
        if args.is_empty() {
            command.arg(job);
        } else {
            command.arg(format!("{job} {}", args.join(" ")));
        }
        cargo::run_command(command)
    } else {
        eprintln!("note: cargo-watch not found; falling back to `cargo {job}`.");
        cargo::run(&[job], args)
    }
}
