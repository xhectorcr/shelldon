use crate::config::Project;
use crate::terminal::utils::{launch_extra, resolve};
use anyhow::Result;
use std::process::Command;

pub fn launch_gnome(project: &Project) -> Result<()> {
    let mut cmd = Command::new("gnome-terminal");
    
    for pane in &project.panes {
        let shell_cmd = format!("bash -ic 'cd \"{}\" && {}; exec bash'", resolve(&pane.path), pane.command);
        cmd.arg("--tab").arg("--").arg("bash").arg("-c").arg(&shell_cmd);
    }

    cmd.spawn().map_err(|e| anyhow::anyhow!("GNOME Terminal: {}", e))?;
    
    for extra in &project.extras {
        launch_extra(extra)?;
    }
    
    Ok(())
}
