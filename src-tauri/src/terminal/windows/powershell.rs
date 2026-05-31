use crate::config::Project;
use crate::terminal::utils::{launch_extra, resolve};
use anyhow::Result;
use std::process::Command;

pub fn launch_powershell(project: &Project) -> Result<()> {
    for pane in &project.panes {
        // Forzar carga del perfil de PowerShell
        let script = format!(
            "& {{ $profile_path = if (Test-Path $PROFILE.CurrentUserCurrentHost) {{ $PROFILE.CurrentUserCurrentHost }} else {{ $PROFILE.AllUsersCurrentHost }}; if (Test-Path $profile_path) {{ . $profile_path }}; Set-Location '{}'; {} }}",
            resolve(&pane.path).replace('\\', "\\\\").replace('\'', "''"),
            pane.command
        );

        Command::new("powershell.exe")
            .args(["-NoExit", "-Command", &script])
            .spawn()
            .map_err(|e| anyhow::anyhow!("PowerShell: {}", e))?;
    }
    for extra in &project.extras {
        launch_extra(extra)?;
    }
    Ok(())
}


