use std::{ffi::CString, iter::once, path::Path, sync::Once};

use anyhow::{anyhow, Context};
use directories::BaseDirs;
use log::debug;
use windows::{
    core::{ComInterface, PCSTR, PCWSTR},
    Win32::{
        Foundation::TRUE,
        System::Com::{
            CoCreateInstance, CoInitializeEx, IPersistFile, CLSCTX_INPROC_SERVER,
            COINIT_MULTITHREADED,
        },
        UI::{Shell::*, WindowsAndMessaging::SW_HIDE},
    },
};

use crate::{Config, Project};
pub fn update_shortcuts(
    base: BaseDirs,
    projects: Vec<Project>,
    config: Config,
) -> anyhow::Result<()> {
    let start_menu = base
        .config_dir()
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs");
    if !start_menu.exists() {
        return Err(anyhow!("Start Menu does not exist"));
    }
    initialize_com();

    let programming_folder = start_menu.join("Programming Projects");
    if !programming_folder.exists() {
        std::fs::create_dir_all(&programming_folder)?;
    } else {
        std::fs::remove_dir_all(&programming_folder)?;
        std::fs::create_dir_all(&programming_folder)?;
    }
    debug!("Putting shortcuts in {}", programming_folder.display());

    let vs_code = path_to_c_string(&config.vs_code_path)?;
    for project in projects {
        debug!(
            "Creating Shortcut to {} at {}",
            project.name,
            project.path.display()
        );

        let project_dir = path_to_c_string(&project.path)?;
        let link_path = programming_folder.join(format!("{}.lnk", project.name));
        let description = CString::new(format!("Open {} in VS Code", project.name))
            .context("Unable to create description")?;

        unsafe {
            let shell_link: IShellLinkA = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
            shell_link.SetPath(PCSTR(vs_code.as_ptr().cast()))?;
            shell_link.SetArguments(PCSTR(project_dir.as_ptr().cast()))?;
            shell_link.SetDescription(PCSTR(description.as_ptr().cast()))?;
            shell_link.SetWorkingDirectory(PCSTR(project_dir.as_ptr().cast()))?;
            shell_link.SetShowCmd(SW_HIDE)?;
            if let Some(icon) = project.icon {
                let icon = path_to_c_string(icon)?;
                shell_link.SetIconLocation(PCSTR(icon.as_ptr().cast()), 0)?;
            }
            shell_link.cast::<IPersistFile>()?.Save(
                PCWSTR(string_to_os_utf16(link_path.to_str().unwrap()).as_ptr()),
                TRUE,
            )?;
        }
    }
    Ok(())
}

/// Converts a Path to a CString.
///
/// Path must be UTF-8
fn path_to_c_string(path: impl AsRef<Path>) -> anyhow::Result<CString> {
    let path = path.as_ref().to_str().context(format!(
        "Unable to convert path to CString. Path is not UTF-8, {:?}",
        path.as_ref()
    ))?;
    CString::new(path).context(format!(
        "Path was unable to be converted into a CString.  {:?}",
        path
    ))
}
pub fn string_to_os_utf16(string: &str) -> Vec<u16> {
    return string.encode_utf16().chain(once(0)).collect::<Vec<u16>>();
}
static CO_INITIALIZE_ONCE: Once = Once::new();

fn initialize_com() {
    CO_INITIALIZE_ONCE.call_once(|| unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok();
    })
}
