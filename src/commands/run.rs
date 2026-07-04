use std::{
    process::{Command, ExitCode},
    str::SplitWhitespace,
};

use anyhow::Result;

use crate::{cargo, cli::RunArgs, config::PlusConfig};

pub fn run(config: &PlusConfig, args: RunArgs) -> Result<ExitCode> {
    if let Some(name) = args.name {
        if let Some(command) = config.commands.get(&name) {
            return run_shell_words(command.split_whitespace(), &args.args);
        }

        let mut cargo_args = vec!["run".to_owned(), "--bin".to_owned(), name];
        cargo_args.extend(args.args);
        return cargo::run(&[], &cargo_args);
    }

    cargo::run(&["run"], &args.args)
}

fn run_shell_words(mut words: SplitWhitespace<'_>, extra: &[String]) -> Result<ExitCode> {
    let Some(program) = words.next() else {
        anyhow::bail!("empty configured command");
    };
    let mut command = Command::new(program);
    command.args(words).args(extra);
    cargo::run_command(command)
}
