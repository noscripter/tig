use anyhow::{Context, Result};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub wrap_lines: bool,
    pub syntax_highlight: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { wrap_lines: false, syntax_highlight: true }
    }
}

impl Settings {
    pub fn load() -> Result<Self> {
        let mut s = Self::default();
        if let Some(path) = config_path() {
            if path.exists() {
                let data = fs::read_to_string(&path)
                    .with_context(|| format!("Reading config: {}", path.display()))?;
                let loaded: Self = toml::from_str(&data)
                    .with_context(|| format!("Parsing TOML config: {}", path.display()))?;
                s = loaded;
            }
        }
        Ok(s)
    }

    pub fn save(&self) -> Result<()> {
        if let Some(path) = config_path() {
            if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
            let data = toml::to_string_pretty(self)?;
            fs::write(&path, data)
                .with_context(|| format!("Writing config: {}", path.display()))?;
        }
        Ok(())
    }
}

fn config_path() -> Option<PathBuf> {
    let mut dir = config_dir()?;
    dir.push("tig-rs");
    dir.push("config.toml");
    Some(dir)
}
