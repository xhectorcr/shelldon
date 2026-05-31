use crate::config::Project;
use crate::terminal::utils::{launch_extra, resolve};
#[cfg(target_os = "windows")]
use crate::terminal::utils::{apply_full_path, ps_path_fix};
use anyhow::Result;
use std::process::Command;

pub fn launch_powershell(project: &Project) -> Result<()> {
    for pane in &project.panes {
        // Forzar carga del perfil de PowerShell
        let script = format!(
            "& {{ {}; $profile_path = if (Test-Path $PROFILE.CurrentUserCurrentHost) {{ $PROFILE.CurrentUserCurrentHost }} else {{ $PROFILE.AllUsersCurrentHost }}; if (Test-Path $profile_path) {{ . $profile_path }}; Set-Location '{}'; {} }}",
            ps_path_fix(),
            resolve(&pane.path).replace('\\', "\\\\").replace('\'', "''"),
            pane.command
        );

        let mut ps = Command::new("powershell.exe");
        ps.args(["-NoExit", "-Command", &script]);

        // Apply full PATH from registry at the process level too
        #[cfg(target_os = "windows")]
        apply_full_path(&mut ps);

        ps.spawn()
            .map_err(|e| anyhow::anyhow!("PowerShell: {}", e))?;
    }
    for extra in &project.extras {
        launch_extra(extra)?;
    }
    Ok(())
}


