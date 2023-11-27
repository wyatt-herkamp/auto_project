use anyhow::Context;
use clap::Parser;
use config::Project;
use console::style;
use directories::ProjectDirs;
use human_panic::setup_panic;
use log::{debug, error, info};

use crate::{
    cli::{AutoProject, Command},
    config::Config,
    utils::GetConfig,
};
pub(crate) mod cli;
pub(crate) mod config;
pub(crate) mod icon;
pub(crate) mod utils;

#[cfg(not(any(target_os = "windows")))]
compile_error!("Your Platform is not Supported.");

#[cfg(target_os = "windows")]
mod windows_impl;
#[cfg(target_os = "windows")]
use windows_impl::update_shortcuts;

#[derive(Debug)]
pub struct AppState {
    pub config: Config,
    pub project_dirs: ProjectDirs,
}
fn main() -> anyhow::Result<()> {
    setup_panic!();
    pretty_env_logger::init();
    let project_dirs = ProjectDirs::from("dev", "wyatt-herkamp", "auto_project")
        .context("Unable to create project directory")?;
    debug!("Project Directory: {:?}", project_dirs);
    let config_file = project_dirs.get_config_path();
    if !config_file.exists() {
        return project_dirs.save_default_config();
    }
    let config = project_dirs.read_config()?;

    if !config.vs_code_path.exists() {
        info!(
            "{}",
            style(format!(
                "VS Code not found at {}",
                config.vs_code_path.display()
            ))
            .red()
        );
        return Ok(());
    }
    let mut app_state = AppState {
        config,
        project_dirs,
    };
    let cli = AutoProject::parse();
    match cli.command {
        Command::BuildShortcuts => build_shortcuts(app_state)?,
        Command::AddProjectsLocation(new_project) => new_project.execute(app_state)?,
        Command::AddProject(project) => {
            let project = project.execute(&mut app_state)?;
            info!("Added Project {}", style(&project.name).green());
            let base_dirs =
                directories::BaseDirs::new().context("Unable to Locate User Directories?")?;
            update_shortcuts(base_dirs, vec![project], app_state.config)?;
        }
        Command::AddDisabledProject { path } => {
            let AppState {
                mut config,
                project_dirs,
            } = app_state;
            let path = if let Some(path) = path {
                path
            } else {
                std::env::current_dir().context("Unable to get current directory")?
            };
            config.disabled_projects.push(path.clone());
            project_dirs.write_config(&config)?;
            info!("Added Disabled Project {}", style(path.display()).green());
        }
        Command::SetVSCodePath { path } => {
            let AppState {
                mut config,
                project_dirs,
            } = app_state;
            if !config.vs_code_path.exists() {
                error!(
                    "VS Code not found at {}",
                    style(config.vs_code_path.display()).red()
                );
            }
            config.vs_code_path = path;
            project_dirs.write_config(&config)?;
            info!(
                "Set VS Code Path to {}. Rebuilding Shortcuts",
                style(config.vs_code_path.display()).green()
            );
            build_shortcuts(AppState {
                config,
                project_dirs,
            })?;
        }
    }
    Ok(())
}
fn build_shortcuts(app_state: AppState) -> anyhow::Result<()> {
    let projects = get_projects(&app_state).context("Unable to get projects")?;
    for project in &projects {
        info!("{}", style(&project.name).green());
    }
    let base_dirs = directories::BaseDirs::new().context("Unable to Locate User Directories?")?;
    update_shortcuts(base_dirs, projects, app_state.config)?;
    Ok(())
}

fn get_projects(state: &AppState) -> anyhow::Result<Vec<Project>> {
    let mut projects = state.config.projects.clone();
    for project_location in &state.config.project_locations {
        if !project_location.path.exists() {
            error!(
                "Project Location {} does not exist",
                project_location.path.display()
            );
        }
        let mut directory = project_location.path.read_dir().context(format!(
            "Unable to read directory {}",
            project_location.path.display()
        ))?;
        projects.reserve(directory.size_hint().0);
        while let Some(entry) = directory.next() {
            let entry = entry.context("Unable to Read Project Folder")?;
            let path = entry.path();
            if path.is_dir() {
                let name = format!(
                    "{} - {}",
                    path.file_name().unwrap().to_string_lossy(),
                    project_location.name.as_deref().unwrap_or("Project")
                );
                let icon = icon::build_icon(project_location.icon_style, &name, &state)?;
                let project = Project {
                    path,
                    name,
                    icon: Some(icon),
                    ..Default::default()
                };
                projects.push(project);
            }
        }
    }
    Ok(projects)
}
