use std::{path::PathBuf, process::ExitCode};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::{
    bytes::format_bytes,
    cli::SizeArgs,
    project::{dir_size, largest_children, named_dirs, Project},
};

pub fn run(project: &Project, args: SizeArgs) -> Result<ExitCode> {
    if !project.target_dir.exists() {
        if args.json {
            let report = SizeReport {
                target_dir: project.target_dir.clone(),
                total_bytes: 0,
                top: Vec::new(),
                deep: Vec::new(),
                largest: Vec::new(),
            };
            println!("{}", serde_json::to_string_pretty(&report)?);
            return Ok(ExitCode::SUCCESS);
        }
        println!(
            "target directory does not exist: {}",
            project.target_dir.display()
        );
        return Ok(ExitCode::SUCCESS);
    }

    let total = dir_size(&project.target_dir).context("failed to size target directory")?;
    let mut rows = top_rows(project)?;
    rows.sort_by_key(|(_, bytes)| std::cmp::Reverse(*bytes));
    let deep = if args.deep {
        deep_rows(project)?
    } else {
        Vec::new()
    };
    let largest = largest_children(&project.target_dir, 8)?;

    if args.json {
        let report = SizeReport {
            target_dir: project.target_dir.clone(),
            total_bytes: total,
            top: rows
                .into_iter()
                .map(|(name, bytes)| NamedSize { name, bytes })
                .collect(),
            deep: deep
                .into_iter()
                .map(|(path, bytes)| PathSize { path, bytes })
                .collect(),
            largest: largest
                .into_iter()
                .map(|(path, bytes)| PathSize { path, bytes })
                .collect(),
        };
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(ExitCode::SUCCESS);
    }

    println!("target: {}", format_bytes(total));

    for (name, bytes) in rows {
        println!("  {name:<14} {}", format_bytes(bytes));
    }

    if args.deep {
        println!();
        println!("Deep buckets:");
        for (path, bytes) in deep {
            println!("  {:>10}  {}", format_bytes(bytes), path.display());
        }
    } else {
        println!();
        println!("Run `plus size --deep` to see which target buckets are largest.");
    }

    println!();
    println!("Largest target entries:");
    for (path, bytes) in largest {
        println!("  {:>10}  {}", format_bytes(bytes), path.display());
    }

    Ok(ExitCode::SUCCESS)
}

#[derive(Serialize)]
struct SizeReport {
    target_dir: PathBuf,
    total_bytes: u64,
    top: Vec<NamedSize>,
    deep: Vec<PathSize>,
    largest: Vec<PathSize>,
}

#[derive(Serialize)]
struct NamedSize {
    name: String,
    bytes: u64,
}

#[derive(Serialize)]
struct PathSize {
    path: PathBuf,
    bytes: u64,
}

pub fn top_rows(project: &Project) -> Result<Vec<(String, u64)>> {
    let mut rows = Vec::new();
    for name in ["debug", "release"] {
        let path = project.target_dir.join(name);
        if path.exists() {
            rows.push((name.to_owned(), dir_size(&path)?));
        }
    }

    let incremental = named_dirs(&project.target_dir, "incremental")
        .into_iter()
        .filter_map(|path| dir_size(&path).ok())
        .sum::<u64>();
    if incremental > 0 {
        rows.push(("incremental".to_owned(), incremental));
    }

    Ok(rows)
}

pub fn deep_rows(project: &Project) -> Result<Vec<(std::path::PathBuf, u64)>> {
    let mut rows = Vec::new();
    for path in [
        project.target_dir.join("debug").join("deps"),
        project.target_dir.join("debug").join("build"),
        project.target_dir.join("debug").join("incremental"),
        project.target_dir.join("release").join("deps"),
        project.target_dir.join("release").join("build"),
        project.target_dir.join("release").join("incremental"),
    ] {
        if path.exists() {
            rows.push((path.clone(), dir_size(&path)?));
        }
    }

    rows.sort_by_key(|(_, bytes)| std::cmp::Reverse(*bytes));
    Ok(rows)
}
