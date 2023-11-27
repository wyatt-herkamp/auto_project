use std::{env::current_dir, path::PathBuf};

use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use log::info;

use crate::{
    config::{IconStyle, Project, ProjectLocation},
    utils::GetConfig,
    AppState,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct AutoProject {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Builds the shortcuts
    BuildShortcuts,
    /// Adds a new directory that contains projects
    AddProjectsLocation(AddProjectsDir),
    AddDisabledProject {
        path: Option<PathBuf>,
    },
    SetVSCodePath {
        path: PathBuf,
    },
    /// Adds a new project to the config
    AddProject(AddProject),
}
#[derive(Args, Debug)]
pub struct AddProject {
    #[arg(short, long)]
    name: Option<String>,
    #[arg(short, long)]
    icon_style: Option<IconStyle>,
    /// Currently Not Implemented
    ///
    /// Will use a specified icon instead of the default
    /// Must be SVG
    /// .ico files are allowed on Windows
    #[arg(long)]
    icon_path: Option<PathBuf>,
    #[arg(short, long)]
    description: Option<String>,
    /// If not provided, the current directory will be used
    #[arg(short, long)]
    path: Option<PathBuf>,
}
impl AddProject {
    pub fn execute(self, app_state: &mut AppState) -> anyhow::Result<Project> {
        let Self {
            name,
            description,
            path,
            ..
        } = self;

        let path = if let Some(path) = path {
            path
        } else {
            current_dir().context("Unable to get current directory")?
        };

        if !path.exists() {
            return Err(anyhow::anyhow!("Path does not exist: {}", path.display()));
        }
        let name = if let Some(name) = name {
            name
        } else {
            path.file_name().unwrap().to_string_lossy().to_string()
        };
        let AppState {
            config,
            project_dirs,
        } = app_state;

        let project = if let Some(value) = config.projects.iter_mut().find(|p| p.path == path) {
            info!("Updating Project");
            value.name = name.clone();
            if let Some(description) = description {
                value.description = Some(description);
            }
            value.clone()
        } else {
            let new_project = Project {
                path,
                name,
                icon: None,
                description,
            };

            config.projects.push(new_project.clone());
            new_project
        };
        project_dirs.write_config(&config)?;
        Ok(project)
    }
}
#[derive(Args, Debug)]
pub struct AddProjectsDir {
    #[arg(short, long)]
    name: Option<String>,
    #[arg(short, long)]
    icon_style: Option<IconStyle>,
    #[arg(short, long)]
    description: Option<String>,
    /// If not provided, the current directory will be used
    #[arg(short, long)]
    path: Option<PathBuf>,
}
impl AddProjectsDir {
    pub fn execute(self, app_state: AppState) -> anyhow::Result<()> {
        let Self {
            name,
            icon_style,
            description,
            path,
        } = self;

        let path = if let Some(path) = path {
            path
        } else {
            current_dir().context("Unable to get current directory")?
        };
        if !path.exists() {
            return Err(anyhow::anyhow!("Path does not exist: {}", path.display()));
        }
        let name = if let Some(name) = name {
            name
        } else {
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            info!("Using {} as the name", name);
            name
        };
        let AppState {
            mut config,
            project_dirs,
        } = app_state;

        if let Some(value) = config.project_locations.iter_mut().find(|p| p.path == path) {
            info!("Updating Project Location");
            value.name = Some(name);
            if let Some(icon_style) = icon_style {
                value.icon_style = icon_style;
            }
            if let Some(description) = description {
                value.description = Some(description);
            }
        } else {
            let new_project = ProjectLocation {
                path,
                name: Some(name),
                icon_style: icon_style.unwrap_or_default(),
                description,
            };
            config.project_locations.push(new_project);
        }
        project_dirs.write_config(&config)?;
        Ok(())
    }
}
