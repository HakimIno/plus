use std::{process::ExitCode, time::Instant};

use anyhow::Result;
use clap::Parser;

mod bytes;
mod cargo;
mod cli;
mod commands;
mod config;
mod project;
mod tools;

use cli::{Cli, Command};
use config::PlusConfig;
use project::Project;

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    let project = Project::discover(cli.manifest_path, cli.target_dir)?;

    if let Command::Init(args) = cli.command {
        return commands::init::run(&project, args);
    }

    let config = PlusConfig::load(&project.plus_config)?;

    match cli.command {
        Command::Doctor(args) => commands::doctor::run(&project, &config, args),
        Command::Init(_) => unreachable!(),
        Command::Setup(args) => commands::setup::run(&project, args),
        Command::Size(args) => commands::size::run(&project, args),
        Command::Clean(args) => commands::clean::run(&project, args),
        Command::Dev(args) => commands::dev::run(&project, &config, args),
        Command::Test(args) => commands::test::run(args),
        Command::Explain(args) => commands::explain::run(args),
        Command::Profile(args) => profile(args),
        Command::Run(args) => commands::run::run(&config, args),
        Command::Check(args) => cargo::run(&["check"], &args.args),
        Command::Build(args) => cargo::run(&["build"], &args.args),
        Command::Release(args) => cargo::run(&["build", "--release"], &args.args),
    }
}

fn profile(args: cli::ProfileArgs) -> Result<ExitCode> {
    let started = Instant::now();
    let code = match args.command.as_str() {
        "check" => cargo::run(&["check"], &args.args)?,
        "build" => cargo::run(&["build"], &args.args)?,
        "test" => commands::test::run(cli::TestArgs {
            fast: false,
            args: args.args,
        })?,
        _ => unreachable!(),
    };
    println!("duration: {:.2?}", started.elapsed());
    Ok(code)
}
