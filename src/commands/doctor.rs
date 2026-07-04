use std::{path::PathBuf, process::ExitCode};

use anyhow::Result;
use serde::Serialize;

use crate::{
    bytes::{format_bytes, parse_size},
    cli::OutputArgs,
    config::PlusConfig,
    project::{dir_size, Project},
    tools,
};

const LARGE_TARGET_BYTES: u64 = 5 * 1024 * 1024 * 1024;

pub fn run(project: &Project, config: &PlusConfig, args: OutputArgs) -> Result<ExitCode> {
    if args.json {
        let report = build_report(project, config)?;
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(ExitCode::SUCCESS);
    }

    println!("Toolchain:");
    for name in [
        "cargo",
        "rustc",
        "rustup",
        "rustfmt",
        "clippy-driver",
        "cargo-watch",
        "bacon",
        "cargo-nextest",
        "sccache",
        "mold",
        "ld.lld",
    ] {
        let tool = tools::probe(name);
        match tool.version {
            Some(version) => println!("{name}: ok {version}"),
            None => println!("{name}: not found"),
        }
    }
    match tools::sccache_health() {
        tools::ToolHealth::Ok => println!("sccache health: ok"),
        tools::ToolHealth::Missing => println!("sccache health: missing"),
        tools::ToolHealth::Broken(reason) => println!("sccache health: broken ({reason})"),
    }

    println!();
    println!("Project:");
    println!("root: {}", project.root.display());
    println!("manifest: {}", project.manifest_path.display());
    println!("target: {}", project.target_dir.display());
    if project.cargo_config.exists() {
        println!("cargo config: ok {}", project.cargo_config.display());
    } else {
        println!("cargo config: missing .cargo/config.toml");
    }
    if project.plus_config.exists() {
        println!("plus config: ok {}", project.plus_config.display());
    } else {
        println!("plus config: missing plus.toml");
    }
    if !config.commands.is_empty() {
        println!("custom commands: {}", config.commands.len());
    }

    println!();
    println!("Preferences:");
    let prefer_sccache = config
        .tools
        .as_ref()
        .and_then(|tools| tools.prefer_sccache)
        .unwrap_or(true);
    let prefer_fast_linker = config
        .tools
        .as_ref()
        .and_then(|tools| tools.prefer_fast_linker)
        .unwrap_or(true);
    let prefer_nextest = config
        .tools
        .as_ref()
        .and_then(|tools| tools.prefer_nextest)
        .unwrap_or(true);
    println!("sccache: {}", yes_no(prefer_sccache));
    println!("fast linker: {}", yes_no(prefer_fast_linker));
    println!("nextest: {}", yes_no(prefer_nextest));

    println!();
    println!("Recommendations:");
    let recommendations = recommendations(project, config)?;
    let any = !recommendations.is_empty();
    if let Some(bytes) = target_size(project)? {
        println!("target size: {}", format_bytes(bytes));
    }
    for recommendation in &recommendations {
        println!("  - {recommendation}");
    }
    if !any {
        println!("  no obvious issues found.");
    }

    Ok(ExitCode::SUCCESS)
}

fn build_report(project: &Project, config: &PlusConfig) -> Result<DoctorReport> {
    let tools = [
        "cargo",
        "rustc",
        "rustup",
        "rustfmt",
        "clippy-driver",
        "cargo-watch",
        "bacon",
        "cargo-nextest",
        "sccache",
        "mold",
        "ld.lld",
    ]
    .into_iter()
    .map(|name| {
        let tool = tools::probe(name);
        ToolReport {
            name,
            found: tool.version.is_some(),
            version: tool.version,
        }
    })
    .collect();

    Ok(DoctorReport {
        project: ProjectReport {
            root: project.root.clone(),
            manifest: project.manifest_path.clone(),
            target: project.target_dir.clone(),
            cargo_config: project.cargo_config.exists(),
            plus_config: project.plus_config.exists(),
            custom_commands: config.commands.len(),
        },
        tools,
        sccache_health: match tools::sccache_health() {
            tools::ToolHealth::Ok => HealthReport {
                status: "ok",
                reason: None,
            },
            tools::ToolHealth::Missing => HealthReport {
                status: "missing",
                reason: None,
            },
            tools::ToolHealth::Broken(reason) => HealthReport {
                status: "broken",
                reason: Some(reason),
            },
        },
        preferences: preferences(config),
        target_size_bytes: target_size(project)?,
        recommendations: recommendations(project, config)?,
    })
}

fn recommendations(project: &Project, config: &PlusConfig) -> Result<Vec<String>> {
    let mut recommendations = Vec::new();
    let max_target_size = config
        .clean
        .as_ref()
        .and_then(|clean| clean.max_target_size.as_deref())
        .map(parse_size)
        .transpose()?
        .unwrap_or(LARGE_TARGET_BYTES);
    let prefs = preferences(config);

    if let Some(bytes) = target_size(project)? {
        if bytes > max_target_size {
            recommendations
                .push("target is large; run `plus clean` to preview safe cleanup.".to_owned());
        }
    }
    if prefs.prefer_sccache {
        match tools::sccache_health() {
            tools::ToolHealth::Ok => {}
            tools::ToolHealth::Missing => {
                recommendations.push(
                    "install sccache to reuse compiler outputs across clean builds.".to_owned(),
                );
            }
            tools::ToolHealth::Broken(reason) => {
                recommendations.push(format!("sccache is installed but unusable: {reason}"));
                recommendations
                    .push("fix sccache or disable rustc-wrapper in .cargo/config.toml.".to_owned());
            }
        }
    }
    if prefs.prefer_fast_linker && tools::fast_linker().is_none() {
        recommendations.push("install mold or lld for faster linking.".to_owned());
    }
    if !tools::exists("bacon") && !tools::exists("cargo-watch") {
        recommendations
            .push("install bacon or cargo-watch for a better `plus dev` loop.".to_owned());
    }
    if prefs.prefer_nextest && !tools::exists("cargo-nextest") {
        recommendations.push("install cargo-nextest for faster test runs.".to_owned());
    }
    if !project.cargo_config.exists() {
        recommendations
            .push("run `plus setup --write` to create local Cargo speed config.".to_owned());
    }

    Ok(recommendations)
}

fn target_size(project: &Project) -> Result<Option<u64>> {
    if project.target_dir.exists() {
        Ok(Some(dir_size(&project.target_dir)?))
    } else {
        Ok(None)
    }
}

fn preferences(config: &PlusConfig) -> PreferencesReport {
    PreferencesReport {
        prefer_sccache: config
            .tools
            .as_ref()
            .and_then(|tools| tools.prefer_sccache)
            .unwrap_or(true),
        prefer_fast_linker: config
            .tools
            .as_ref()
            .and_then(|tools| tools.prefer_fast_linker)
            .unwrap_or(true),
        prefer_nextest: config
            .tools
            .as_ref()
            .and_then(|tools| tools.prefer_nextest)
            .unwrap_or(true),
    }
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

#[derive(Serialize)]
struct DoctorReport {
    project: ProjectReport,
    tools: Vec<ToolReport>,
    sccache_health: HealthReport,
    preferences: PreferencesReport,
    target_size_bytes: Option<u64>,
    recommendations: Vec<String>,
}

#[derive(Serialize)]
struct ProjectReport {
    root: PathBuf,
    manifest: PathBuf,
    target: PathBuf,
    cargo_config: bool,
    plus_config: bool,
    custom_commands: usize,
}

#[derive(Serialize)]
struct ToolReport {
    name: &'static str,
    found: bool,
    version: Option<String>,
}

#[derive(Serialize)]
struct HealthReport {
    status: &'static str,
    reason: Option<String>,
}

#[derive(Serialize)]
struct PreferencesReport {
    prefer_sccache: bool,
    prefer_fast_linker: bool,
    prefer_nextest: bool,
}
