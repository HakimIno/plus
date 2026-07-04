use std::{fs, path::PathBuf, process::ExitCode};

use anyhow::Result;
use serde::Serialize;

use crate::{
    bytes::format_bytes,
    cargo,
    cli::CleanArgs,
    project::{dir_size, ensure_inside, named_dirs, Project},
};

pub fn run(project: &Project, args: CleanArgs) -> Result<ExitCode> {
    if args.nuclear {
        if !args.yes {
            println!("`plus clean --nuclear` runs `cargo clean` and removes all build artifacts.");
            println!("Run `plus clean --nuclear --yes` if you really want that.");
            return Ok(ExitCode::FAILURE);
        }
        return cargo::run(&["clean"], &[]);
    }

    let plan = cleanup_plan(project, args.deep)?;
    if args.json {
        let report = CleanReport {
            apply: args.apply,
            deep: args.deep,
            nuclear: false,
            total_bytes: plan.iter().map(|item| item.bytes).sum(),
            items: plan
                .iter()
                .map(|item| CleanItemReport {
                    kind: item.kind,
                    path: item.path.clone(),
                    bytes: item.bytes,
                })
                .collect(),
        };
        println!("{}", serde_json::to_string_pretty(&report)?);
    }

    if plan.is_empty() {
        if !args.json {
            println!("nothing to clean.");
        }
        return Ok(ExitCode::SUCCESS);
    }

    let total = plan.iter().map(|item| item.bytes).sum::<u64>();
    if !args.json {
        println!(
            "{} clean:",
            if args.apply { "applying" } else { "previewing" }
        );
        for item in &plan {
            println!(
                "  {:>10}  {:<12} {}",
                format_bytes(item.bytes),
                item.kind,
                item.path.display()
            );
        }
        println!("  ----------");
        println!("  {:>10}  possible freed space", format_bytes(total));
    }

    if !args.apply {
        if !args.json {
            println!();
            println!("Run `plus clean --apply` to remove safe items.");
            if !args.deep {
                println!("Run `plus clean --deep --apply` to remove more rebuildable artifacts.");
            }
        }
        return Ok(ExitCode::SUCCESS);
    }

    for item in plan {
        ensure_inside(&item.path, &project.target_dir)?;
        if item.path.exists() {
            fs::remove_dir_all(&item.path)?;
        }
    }
    if !args.json {
        println!("freed {}", format_bytes(total));
    }
    Ok(ExitCode::SUCCESS)
}

fn cleanup_plan(project: &Project, deep: bool) -> Result<Vec<CleanupItem>> {
    let mut plan = Vec::new();
    if !project.target_dir.exists() {
        return Ok(plan);
    }

    for path in named_dirs(&project.target_dir, "incremental") {
        plan.push(CleanupItem::new("incremental", path)?);
    }

    for profile in ["debug", "release"] {
        let examples = project.target_dir.join(profile).join("examples");
        if examples.exists() {
            plan.push(CleanupItem::new("examples", examples)?);
        }
    }

    if deep {
        for profile in ["debug", "release"] {
            for name in ["build"] {
                let path = project.target_dir.join(profile).join(name);
                if path.exists() {
                    plan.push(CleanupItem::new(name, path)?);
                }
            }
        }
    }

    plan.retain(|item| item.bytes > 0);
    plan.sort_by_key(|item| std::cmp::Reverse(item.bytes));
    Ok(plan)
}

struct CleanupItem {
    kind: &'static str,
    path: PathBuf,
    bytes: u64,
}

#[derive(Serialize)]
struct CleanReport {
    apply: bool,
    deep: bool,
    nuclear: bool,
    total_bytes: u64,
    items: Vec<CleanItemReport>,
}

#[derive(Serialize)]
struct CleanItemReport {
    kind: &'static str,
    path: PathBuf,
    bytes: u64,
}

impl CleanupItem {
    fn new(kind: &'static str, path: PathBuf) -> Result<Self> {
        let bytes = dir_size(&path)?;
        Ok(Self { kind, path, bytes })
    }
}
