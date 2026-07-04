use std::{collections::BTreeMap, fs, path::Path};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct PlusConfig {
    #[serde(default)]
    pub commands: BTreeMap<String, String>,
    #[serde(default)]
    pub clean: Option<CleanConfig>,
    #[serde(default)]
    pub dev: Option<DevConfig>,
    #[serde(default)]
    pub tools: Option<ToolsConfig>,
}

#[derive(Debug, Deserialize)]
pub struct CleanConfig {
    pub max_target_size: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DevConfig {
    pub prefer: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ToolsConfig {
    pub prefer_sccache: Option<bool>,
    pub prefer_fast_linker: Option<bool>,
    pub prefer_nextest: Option<bool>,
}

impl PlusConfig {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let text = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        toml::from_str(&text).with_context(|| format!("failed to parse {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::init::default_plus_toml;

    #[test]
    fn parses_default_config() {
        let config: PlusConfig = toml::from_str(default_plus_toml()).unwrap();
        assert_eq!(
            config.commands.get("quick").map(String::as_str),
            Some("cargo check --workspace")
        );
        assert_eq!(
            config
                .clean
                .as_ref()
                .and_then(|clean| clean.max_target_size.as_deref()),
            Some("5GiB")
        );
        assert_eq!(
            config.dev.as_ref().and_then(|dev| dev.prefer.as_deref()),
            Some("bacon")
        );
        assert_eq!(
            config.tools.as_ref().and_then(|tools| tools.prefer_sccache),
            Some(true)
        );
    }
}
