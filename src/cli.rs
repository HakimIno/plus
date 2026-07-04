use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "plus")]
#[command(about = "Rust workflow doctor, smart cleaner, setup optimizer, and Cargo launcher.")]
pub struct Cli {
    #[arg(long, global = true)]
    pub manifest_path: Option<PathBuf>,

    #[arg(long, global = true)]
    pub target_dir: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Doctor(OutputArgs),
    Init(InitArgs),
    Setup(SetupArgs),
    Size(SizeArgs),
    Clean(CleanArgs),
    Dev(DevArgs),
    Test(TestArgs),
    Explain(ExplainArgs),
    Profile(ProfileArgs),
    Run(RunArgs),
    Check(PassthroughArgs),
    Build(PassthroughArgs),
    Release(PassthroughArgs),
}

#[derive(Debug, Args)]
pub struct InitArgs {
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct OutputArgs {
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct SetupArgs {
    #[arg(long)]
    pub write: bool,

    #[arg(long)]
    pub install: bool,
}

#[derive(Debug, Args)]
pub struct SizeArgs {
    #[arg(long)]
    pub deep: bool,

    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct CleanArgs {
    #[arg(long)]
    pub apply: bool,

    #[arg(long)]
    pub deep: bool,

    #[arg(long)]
    pub nuclear: bool,

    #[arg(long)]
    pub yes: bool,

    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct DevArgs {
    #[arg(long)]
    pub check: bool,

    #[arg(long)]
    pub run: bool,

    #[arg(long)]
    pub test: bool,

    #[arg(num_args = 0.., trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

#[derive(Debug, Args)]
pub struct TestArgs {
    #[arg(long)]
    pub fast: bool,

    #[arg(num_args = 0.., trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

#[derive(Debug, Args)]
pub struct ExplainArgs {
    #[arg(value_parser = ["size", "build", "tools"])]
    pub topic: String,
}

#[derive(Debug, Args)]
pub struct ProfileArgs {
    #[arg(value_parser = ["check", "build", "test"])]
    pub command: String,

    #[arg(num_args = 0.., trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

#[derive(Debug, Args)]
pub struct RunArgs {
    pub name: Option<String>,

    #[arg(num_args = 0.., trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

#[derive(Debug, Args)]
pub struct PassthroughArgs {
    #[arg(num_args = 0.., trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}
