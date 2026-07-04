use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct Project {
    pub root: PathBuf,
    pub manifest_path: PathBuf,
    pub target_dir: PathBuf,
    pub cargo_config: PathBuf,
    pub plus_config: PathBuf,
}

impl Project {
    pub fn discover(manifest_path: Option<PathBuf>, target_dir: Option<PathBuf>) -> Result<Self> {
        let manifest_path = match manifest_path {
            Some(path) => path,
            None => find_manifest(env::current_dir()?)?,
        };
        let root = manifest_path
            .parent()
            .context("manifest path has no parent")?
            .to_path_buf();
        let target_dir = target_dir
            .or_else(|| env::var_os("CARGO_TARGET_DIR").map(PathBuf::from))
            .unwrap_or_else(|| root.join("target"));
        let cargo_config = root.join(".cargo").join("config.toml");
        let plus_config = root.join("plus.toml");

        Ok(Self {
            root,
            manifest_path,
            target_dir,
            cargo_config,
            plus_config,
        })
    }
}

pub fn dir_size(path: &Path) -> io::Result<u64> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_file() {
        return Ok(metadata.len());
    }
    if !metadata.is_dir() {
        return Ok(0);
    }

    let mut total = 0;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        total += dir_size(&entry.path())?;
    }
    Ok(total)
}

pub fn largest_children(root: &Path, limit: usize) -> Result<Vec<(PathBuf, u64)>> {
    let mut children = Vec::new();
    if !root.exists() {
        return Ok(children);
    }

    for entry in fs::read_dir(root).with_context(|| format!("failed to read {}", root.display()))? {
        let entry = entry?;
        let path = entry.path();
        let bytes =
            dir_size(&path).with_context(|| format!("failed to size {}", path.display()))?;
        children.push((path, bytes));
    }

    children.sort_by_key(|(_, bytes)| std::cmp::Reverse(*bytes));
    children.truncate(limit);
    Ok(children)
}

pub fn named_dirs(root: &Path, name: &str) -> Vec<PathBuf> {
    if !root.exists() {
        return Vec::new();
    }

    WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_dir() && entry.file_name() == name)
        .map(|entry| entry.path().to_path_buf())
        .collect()
}

pub fn ensure_inside(path: &Path, root: &Path) -> Result<()> {
    let path = path.canonicalize()?;
    let root = root.canonicalize()?;
    anyhow::ensure!(
        path.starts_with(&root),
        "{} is outside {}",
        path.display(),
        root.display()
    );
    Ok(())
}

fn find_manifest(start: PathBuf) -> Result<PathBuf> {
    for dir in start.ancestors() {
        let manifest = dir.join("Cargo.toml");
        if manifest.exists() {
            return Ok(manifest);
        }
    }
    anyhow::bail!("Cargo.toml not found; run plus inside a Rust project or pass --manifest-path")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_inside_accepts_child() {
        let root = std::env::temp_dir();
        let child = root.join("plus-test-child");
        fs::create_dir_all(&child).unwrap();
        ensure_inside(&child, &root).unwrap();
        fs::remove_dir_all(&child).unwrap();
    }

    #[test]
    fn ensure_inside_rejects_external_path() {
        let root = std::env::temp_dir().join("plus-test-root");
        let external = std::env::temp_dir().join("plus-test-external");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&external).unwrap();
        assert!(ensure_inside(&external, &root).is_err());
        fs::remove_dir_all(&root).unwrap();
        fs::remove_dir_all(&external).unwrap();
    }
}
