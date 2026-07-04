use std::process::{Command, ExitCode, ExitStatus};

use anyhow::{Context, Result};

pub fn run(base_args: &[&str], extra_args: &[String]) -> Result<ExitCode> {
    let mut command = Command::new("cargo");
    command.args(base_args).args(extra_args);
    run_command(command)
}

pub fn run_command(mut command: Command) -> Result<ExitCode> {
    let status = command
        .status()
        .with_context(|| format!("failed to start {:?}", command.get_program()))?;
    Ok(exit_code(status))
}

fn exit_code(status: ExitStatus) -> ExitCode {
    match status.code() {
        Some(0) => ExitCode::SUCCESS,
        Some(code) => ExitCode::from(code as u8),
        None => ExitCode::FAILURE,
    }
}
