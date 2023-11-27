use std::path::PathBuf;

use clap::ValueEnum;
use log::debug;
use serde::{Deserialize, Serialize};
use strum::AsRefStr;
#[cfg(target_os = "windows")]
fn default_vs_code_path() -> PathBuf {
    which::which("code").unwrap_or_else(|e| {
        debug!("Unable to find VS Code: {}", e);
        PathBuf::from("C:\\Program Files\\Microsoft VS Code\\Code.exe")
    })
}
#[cfg(not(target_os = "windows"))]
fn default_vs_code_path() -> PathBuf {
    which::which("code").unwrap_or_else(|e| {
        debug!("Unable to find VS Code: {}", e);
        PathBuf::from("/usr/bin/code")
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub vs_code_path: PathBuf,
    #[serde(default)]
    pub project_locations: Vec<ProjectLocation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub disabled_projects: Vec<PathBuf>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub projects: Vec<Project>,
}
impl Default for Config {
    fn default() -> Self {
        let code = default_vs_code_path();
        Self {
            vs_code_path: code,
            project_locations: vec![],
            disabled_projects: vec![],
            projects: Vec::new(),
        }
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, ValueEnum, AsRefStr)]
pub enum IconStyle {
    /// Use VS Code's Default Icon
    #[default]
    Default,
    Cargo,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectLocation {
    pub path: PathBuf,
    pub name: Option<String>,
    #[serde(default)]
    pub icon_style: IconStyle,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Project {
    pub path: PathBuf,
    pub name: String,
    pub icon: Option<PathBuf>,
    pub description: Option<String>,
}
