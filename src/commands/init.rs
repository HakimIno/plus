use std::{fs, process::ExitCode};

use anyhow::Result;

use crate::{cli::InitArgs, project::Project};

pub fn run(project: &Project, args: InitArgs) -> Result<ExitCode> {
    if project.plus_config.exists() && !args.force {
        println!(
            "{} already exists; leaving it untouched.",
            project.plus_config.display()
        );
        println!("Run `plus init --force` to overwrite it.");
        return Ok(ExitCode::SUCCESS);
    }

    fs::write(&project.plus_config, default_plus_toml())?;
    println!("created {}", project.plus_config.display());
    Ok(ExitCode::SUCCESS)
}

pub fn default_plus_toml() -> &'static str {
    "\
[commands]
quick = \"cargo check --workspace\"
app = \"cargo run\"

[clean]
max_target_size = \"5GiB\"

[dev]
prefer = \"bacon\"

[tools]
prefer_sccache = true
prefer_fast_linker = true
prefer_nextest = true
"
}
